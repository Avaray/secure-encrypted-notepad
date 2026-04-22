use crate::app_state::PendingAction;
use crate::crypto::{encrypt_file, generate_keyfile};
use crate::history::DocumentWithHistory;
use crate::EditorApp;

use std::path::PathBuf;

impl EditorApp {
    /// Check for unsaved changes before action
    pub(crate) fn check_changes_before_action(&mut self, action: PendingAction) {
        // Skip check for certain actions that don't close the current file
        let skip_check = matches!(
            action,
            PendingAction::OpenDirectory | PendingAction::ChangeDirectory(_)
        );

        let is_empty = self.document.current_content.is_empty();
        if self.is_modified && !skip_check && !is_empty {
            self.pending_action = action;
            self.show_close_confirmation = true;
        } else {
            self.execute_pending_action(action);
        }
    }

    /// Execute pending action
    pub(crate) fn execute_pending_action(&mut self, action: PendingAction) {
        match action {
            PendingAction::None => {}
            PendingAction::NewDocument => self.perform_new_document(),
            PendingAction::OpenFile => self.perform_open_file_dialog(),
            PendingAction::OpenDirectory => self.perform_open_directory(),
            PendingAction::Exit => {
                // Exit is handled in update loop
            }
            PendingAction::OpenFileFromTree(path) => self.perform_open_file(path, false),
            PendingAction::OpenFileFromIPC(path) => self.perform_open_file(path, false),
            PendingAction::CloseFile => self.perform_close_file(),
            PendingAction::ChangeDirectory(path) => self.perform_change_directory(path),
        }
    }

    pub(crate) fn perform_close_file(&mut self) {
        self.document = DocumentWithHistory::new_with_limit(self.settings.max_history_length);
        self.current_file_path = None;
        self.opened_as_plaintext = false;
        self.is_modified = false;
        self.loaded_history_index = None;
        self.show_autosave_restore = false;
        self.status_message = t!("actions.status_closed").to_string();
        self.log_info(t!("actions.log_closed"));
        if self.show_search_panel {
            self.perform_search();
        }
        self.replace_undo_stack.clear();
        self.commit_history_state();
    }

    pub(crate) fn perform_new_document(&mut self) {
        self.document = DocumentWithHistory::new_with_limit(self.settings.max_history_length);
        self.current_file_path = None;
        self.opened_as_plaintext = false;
        self.is_modified = false;
        self.loaded_history_index = None;
        self.show_autosave_restore = false;
        self.status_message = t!("actions.status_new").to_string();
        self.log_info(t!("actions.log_new"));
        if self.show_search_panel {
            self.perform_search();
        }
        self.replace_undo_stack.clear();
        self.commit_history_state();
    }

    /// Open file dialog implementation
    pub(crate) fn perform_open_file_dialog(&mut self) {
        if let Some(path) = rfd::FileDialog::new()
            .add_filter(t!("actions.filter_sen"), &["sen"])
            .add_filter(t!("actions.filter_all"), &["*"])
            .pick_file()
        {
            self.perform_open_file(path, false);
        }
    }

    /// Open file implementation
    pub(crate) fn perform_open_file(&mut self, path: PathBuf, exit_on_cancel: bool) {
        // Enforce 1GB file size limit
        if let Ok(metadata) = std::fs::metadata(&path) {
            let max_size: u64 = 1024 * 1024 * 1024; // 1 GB
            if metadata.len() > max_size {
                let msg = t!("actions.log_key_large", size = metadata.len()).to_string();
                self.status_message = msg.clone();
                self.log_error(msg);
                if exit_on_cancel {
                    std::process::exit(0);
                }
                return;
            }
        }

        let buffer = match std::fs::read(&path) {
            Ok(b) => b,
            Err(e) => {
                let msg = format!("Failed to read file: {}", e);
                self.status_message = msg.clone();
                self.log_error(msg);
                if exit_on_cancel {
                    std::process::exit(0);
                }
                return;
            }
        };

        let mut is_stealth = false;
        let mut is_sen = crate::crypto::is_sen_buffer(&buffer);

        // If not a standard SEN file, check if it's stealth-compatible or just binary
        if !is_sen {
            // Check for stealth compatibility first if keyfile is already loaded
            if let Some(keyfile) = &self.keyfile_path {
                if let Ok(key_hash) = crate::crypto::get_keyfile_hash(keyfile) {
                    // We need a buffer-based stealth check or just use the existing logic
                    // since we already have the buffer.
                    // For simplicity, we'll check if it "looks" binary first if no key matches.
                    if let Ok(true) = crate::crypto::check_stealth_compatibility(&key_hash, &path) {
                        is_stealth = true;
                    }
                }
            }

            // If still not identified, use heuristics
            if !is_stealth {
                if crate::crypto::is_buffer_text(&buffer) {
                    // It looks like a normal text file — open as plaintext
                    let content_str = String::from_utf8_lossy(&buffer).to_string();
                    self.document =
                        DocumentWithHistory::new_with_limit(self.settings.max_history_length);
                    self.document.current_content = content_str;
                    self.current_file_path = Some(path.clone());
                    self.opened_as_plaintext = true;
                    self.is_modified = false;

                    if self.show_search_panel {
                        self.perform_search();
                    }
                    self.replace_undo_stack.clear();
                    self.loaded_history_index = None;
                    self.show_autosave_restore = false;

                    let masked_path = self.mask_directory_path(&path);
                    self.status_message = t!("actions.status_opened_plaintext").to_string();
                    self.log_info(t!("actions.log_opened_plaintext", file = masked_path));

                    self.commit_history_state();
                    return;
                } else {
                    // It looks binary but we don't have a key or it's not stealth.
                    // We treat it as a SEN file (missing key) to trigger the key selection loop.
                    is_sen = true;
                }
            }
        }

        if self.keyfile_path.is_none() {
            let filename = path.file_name().unwrap_or_default().to_string_lossy();
            self.status_message = t!("actions.status_open_key_needed", file = filename).to_string();
            self.log_warning(t!("actions.log_open_key_needed", file = filename));

            if let Some(kf_path) = rfd::FileDialog::new()
                .set_title(t!("actions.title_select_key", file = filename))
                .pick_file()
            {
                self.keyfile_path = Some(kf_path);
                // Also update settings if we want to remember this?
                // For now just for this session is safest.
            } else {
                if exit_on_cancel {
                    std::process::exit(0);
                }
                self.log_info(t!("actions.log_open_cancel"));
                return;
            }
        }

        let keyfile = self.keyfile_path.clone().unwrap();
        self.log_info(t!(
            "actions.log_opening",
            file = self.mask_directory_path(&path)
        ));
        self.log_info(t!(
            "actions.log_using_key",
            file = self.mask_keyfile_path(&keyfile)
        ));

        let mut current_keyfile = keyfile;
        loop {
            let decrypt_result = if is_stealth || (!is_sen && self.settings.stealth_mode) {
                // If it was detected as stealth, or if it failed standard headers and stealth mode is aggressively on
                crate::crypto::decrypt_stealth_buffer(&current_keyfile, &buffer)
            } else {
                crate::crypto::decrypt_bytes_buffer(&current_keyfile, &buffer)
            };

            match decrypt_result {
                Ok(content_bytes) => {
                    let content = String::from_utf8_lossy(&content_bytes).to_string();
                    self.keyfile_path = Some(current_keyfile);
                    self.document = DocumentWithHistory::from_file_content(&content);
                    self.current_file_path = Some(path.clone());
                    self.opened_as_plaintext = false;
                    self.is_modified = false;
                    if self.show_search_panel {
                        self.perform_search();
                    }
                    self.replace_undo_stack.clear();
                    self.loaded_history_index = None;
                    self.show_autosave_restore = self.document.autosave.is_some();

                    let history_count = self.document.get_visible_history().len();

                    let masked_path = self.mask_directory_path(&path);

                    self.status_message = if masked_path == "Secured" {
                        t!("actions.status_opened_history", count = history_count).to_string()
                    } else {
                        t!(
                            "actions.status_opened_file_history",
                            file = masked_path,
                            count = history_count
                        )
                        .to_string()
                    };

                    self.log_info(if masked_path == "Secured" {
                        t!("actions.log_opened_history", count = history_count).to_string()
                    } else {
                        t!(
                            "actions.log_opened_file_history",
                            file = masked_path,
                            count = history_count
                        )
                        .to_string()
                    });
                    self.commit_history_state();
                    break;
                }
                Err(e) => {
                    let filename = path.file_name().unwrap_or_default().to_string_lossy();
                    self.status_message =
                        t!("actions.status_wrong_key", file = filename).to_string();
                    self.log_warning(t!("actions.log_dec_failed", e = e));

                    if let Some(new_kf) = rfd::FileDialog::new()
                        .set_title(t!("actions.title_select_key", file = filename))
                        .pick_file()
                    {
                        current_keyfile = new_kf;
                        self.log_info(t!(
                            "actions.log_retry_key",
                            file = self.mask_keyfile_path(&current_keyfile)
                        ));
                    } else {
                        if exit_on_cancel {
                            std::process::exit(0);
                        }
                        self.log_info(t!("actions.log_open_cancel"));
                        break;
                    }
                }
            }
        }
    }

    /// Open directory implementation
    pub(crate) fn perform_open_directory(&mut self) {
        if let Some(path) = rfd::FileDialog::new().pick_folder() {
            self.log_info(t!(
                "actions.log_open_dir",
                file = self.mask_directory_path(&path)
            ));
            self.file_tree_dir = Some(path.clone());

            self.show_file_tree = true;
            self.settings.show_file_tree = true;
            let _ = self.settings.save(None);
            self.refresh_file_tree();
            self.setup_watcher();
        }
    }

    /// Change directory implementation
    pub(crate) fn perform_change_directory(&mut self, path: PathBuf) {
        self.log_info(t!(
            "actions.log_change_dir",
            file = self.mask_directory_path(&path)
        ));
        self.file_tree_dir = Some(path.clone());

        let _ = self.settings.save(None);
        self.refresh_file_tree();
        self.setup_watcher();
    }

    pub(crate) fn save_file(&mut self) {
        if self.keyfile_path.is_none() {
            self.status_message = t!("actions.status_no_key").to_string();
            self.log_error(t!("actions.log_save_no_key"));
            return;
        }

        if self.opened_as_plaintext {
            self.save_file_as();
            return;
        }

        if let Some(path) = self.current_file_path.clone() {
            if self.settings.stealth_mode
                || path.extension().and_then(|e| e.to_str()).unwrap_or("") == "sen"
            {
                self.perform_save(path);
            } else {
                self.save_file_as();
            }
        } else {
            self.save_file_as();
        }
    }

    /// Save file as
    pub(crate) fn save_file_as(&mut self) {
        if self.keyfile_path.is_none() {
            self.status_message = t!("actions.status_no_key").to_string();
            self.log_error(t!("actions.log_save_as_no_key"));
            return;
        }

        let default_name = if let Some(path) = &self.current_file_path {
            if self.settings.stealth_mode {
                // In stealth mode, always strip the extension to produce a raw binary file.
                // Do not propose the original .txt extension to avoid accidental overwrites.
                path.file_stem()
                    .unwrap_or_default()
                    .to_string_lossy()
                    .to_string()
            } else {
                if path.extension().and_then(|e| e.to_str()).unwrap_or("") == "sen" {
                    path.file_name()
                        .unwrap_or_default()
                        .to_string_lossy()
                        .into_owned()
                } else {
                    let file_stem = path.file_stem().unwrap_or_default().to_string_lossy();
                    format!("{}.sen", file_stem)
                }
            }
        } else {
            if self.settings.stealth_mode {
                "document".to_string()
            } else {
                "document.sen".to_string()
            }
        };

        let mut dialog = rfd::FileDialog::new().set_file_name(&default_name);

        if !self.settings.stealth_mode {
            dialog = dialog.add_filter(t!("actions.filter_sen"), &["sen"]);
        }

        // If a directory is open in the file tree, use it as default
        if let Some(dir) = &self.file_tree_dir {
            dialog = dialog.set_directory(dir);
        }

        if let Some(path) = dialog.save_file() {
            self.perform_save(path);
        }
    }

    /// Perform actual save
    pub(crate) fn perform_save(&mut self, path: PathBuf) {
        let keyfile = self.keyfile_path.clone().unwrap();
        self.log_info(t!(
            "actions.log_saving",
            file = self.mask_directory_path(&path)
        ));

        // Save current state to history (snapshot) if modified
        if self.is_modified {
            self.document.add_snapshot(None);
            self.log_info(t!("actions.log_snapshot_auto"));
        }

        // Clear autosave slot on proper save
        self.document.clear_autosave();

        let file_content = self.document.to_file_content();

        let save_result = if self.settings.stealth_mode {
            crate::crypto::encrypt_stealth(file_content.as_bytes(), &keyfile, &path)
        } else {
            encrypt_file(&file_content, &keyfile, &path)
        };

        match save_result {
            Ok(_) => {
                self.current_file_path = Some(path.clone());
                self.opened_as_plaintext = false;
                self.is_modified = false;

                // Commit trimmed history state after successful save
                self.document.trim_to_limit();
                // Filter out entries marked as deleted to fully synchronize
                self.document.history.retain(|e| !e.deleted);

                let history_count = self.document.get_visible_history().len();
                let masked_path = self.mask_directory_path(&path);

                self.status_message = if masked_path == "Secured" {
                    t!("actions.status_saved_history", count = history_count).to_string()
                } else {
                    t!(
                        "actions.status_saved_file_history",
                        file = masked_path,
                        count = history_count
                    )
                    .to_string()
                };

                self.log_success(if masked_path == "Secured" {
                    t!("actions.log_save_success").to_string()
                } else {
                    t!("actions.log_save_file_success", file = masked_path).to_string()
                });
                self.commit_history_state();
                self.refresh_file_tree();

                // Auto-Backup Logic
                if self.settings.auto_backup_enabled {
                    if let Some(backup_dir) = &self.settings.auto_backup_dir {
                        if let Some(file_name) = path.file_name() {
                            let backup_path = backup_dir.join(file_name);
                            match std::fs::copy(&path, &backup_path) {
                                Ok(_) => self.log_info(t!(
                                    "actions.log_backup_success",
                                    file = self.mask_directory_path(&backup_path)
                                )),
                                Err(e) => self.log_error(t!("actions.log_backup_failed", e = e)),
                            }
                        }
                    }
                }
            }
            Err(e) => {
                self.status_message = t!("actions.log_export_err", e = e).to_string(); // Re-use generic err? Actuallyactions.log_save_failed
                self.status_message = format!("Error: {}", e);
                self.log_error(t!("actions.log_save_failed", e = e));
            }
        }
    }

    /// Load keyfile
    pub(crate) fn load_keyfile(&mut self) {
        if let Some(path) = rfd::FileDialog::new().pick_file() {
            self.log_info(t!(
                "actions.log_load_key",
                file = self.mask_keyfile_path(&path)
            ));

            match std::fs::metadata(&path) {
                Ok(metadata) => {
                    let size = metadata.len();

                    if size == 0 {
                        self.status_message = t!("actions.status_key_empty").to_string();
                        self.log_error(t!("actions.log_key_empty"));
                        return;
                    }
                    const MAX_KEYFILE_SIZE: u64 = 100 * 1024 * 1024; // 100 MB
                    if size > MAX_KEYFILE_SIZE {
                        self.status_message = t!(
                            "actions.status_key_large",
                            size = format!("{:.1}", size as f64 / (1024.0 * 1024.0))
                        )
                        .to_string();
                        self.log_error(t!("actions.log_key_large", size = size));
                        return;
                    }

                    match std::fs::read(&path) {
                        Ok(_content) => {
                            self.keyfile_path = Some(path.clone());
                            self.refresh_file_access_status();
                            let masked = self.mask_keyfile_path(&path);
                            self.status_message = if masked == "Secured" {
                                t!("actions.status_key_valid").to_string()
                            } else {
                                t!("actions.status_key_file_valid", file = masked).to_string()
                            };
                            self.log_info(if masked == "Secured" {
                                t!("actions.log_key_valid").to_string()
                            } else {
                                t!("actions.log_key_file_valid", file = masked).to_string()
                            });
                        }
                        Err(e) => {
                            self.status_message =
                                t!("actions.status_key_read_err", e = e).to_string();
                            self.log_error(t!("actions.log_key_read_err", e = e));
                        }
                    }
                }
                Err(e) => {
                    self.status_message = t!("actions.status_key_access_err", e = e).to_string();
                    self.log_error(t!("actions.log_key_access_err", e = e));
                }
            }
        }
    }

    /// Generate new keyfile
    pub(crate) fn generate_new_keyfile(&mut self) {
        if let Some(path) = rfd::FileDialog::new().set_file_name("keyfile").save_file() {
            self.log_info(t!(
                "actions.log_gen_key",
                file = self.mask_keyfile_path(&path)
            ));

            match generate_keyfile(&path) {
                Ok(_) => {
                    self.keyfile_path = Some(path.clone());
                    self.refresh_file_access_status();
                    let masked = self.mask_keyfile_path(&path);
                    self.status_message = if masked == "Secured" {
                        t!("actions.status_gen_success").to_string()
                    } else {
                        t!("actions.status_gen_file_success", file = masked).to_string()
                    };
                    self.log_info(if masked == "Secured" {
                        t!("actions.log_gen_success").to_string()
                    } else {
                        t!("actions.log_gen_file_success", file = masked).to_string()
                    });
                }
                Err(e) => {
                    self.status_message = format!("Error: {}", e); // generic err?
                    self.log_error(format!("Keyfile generation failed: {}", e));
                    // generic err?
                }
            }
        }
    }

    /// Load version from history
    pub(crate) fn load_history_version(&mut self, index: usize) {
        if self.document.load_version(index) {
            self.is_modified = true;
            self.status_message = t!("actions.status_ver_loaded").to_string();
            self.log_success(t!("actions.log_ver_loaded", index = index));
            if self.show_search_panel {
                self.perform_search();
            }
            self.replace_undo_stack.clear();
        }
    }

    /// Delete history entry (soft delete - mark as deleted)
    pub(crate) fn delete_history_entry(&mut self, index: usize) {
        if self.document.mark_entry_deleted(index) {
            self.is_modified = true;
            self.status_message = t!("actions.status_hist_del").to_string();
            self.log_info(t!("actions.log_hist_del", index = index));
        }
    }

    /// Revert to history version (delete newer entries)
    pub(crate) fn revert_to_history_version(&mut self, index: usize) {
        if self.document.revert_to_version(index) {
            self.is_modified = true;
            self.loaded_history_index = Some(index);
            self.status_message = t!("actions.status_revert_success").to_string();
            self.log_success(t!("actions.log_revert_success", index = index));
            if self.show_search_panel {
                self.perform_search();
            }
            self.replace_undo_stack.clear();
        }
    }

    /// Clear all history (soft delete - mark all as deleted)
    pub(crate) fn clear_all_history(&mut self) {
        let count = self.document.get_visible_history().len();
        self.document.mark_all_deleted();
        self.is_modified = true;
        self.loaded_history_index = None;
        self.status_message = t!("actions.status_hist_clear", count = count).to_string();
        self.log_info(t!("actions.log_hist_clear", count = count));
    }

    /// Wrapper functions for UI
    pub(crate) fn new_document(&mut self) {
        self.check_changes_before_action(PendingAction::NewDocument);
    }

    pub(crate) fn open_file_dialog(&mut self) {
        self.check_changes_before_action(PendingAction::OpenFile);
    }

    pub(crate) fn open_file(&mut self, path: PathBuf) {
        // Files check for unsaved changes
        self.check_changes_before_action(PendingAction::OpenFileFromTree(path));
    }

    pub(crate) fn close_file(&mut self) {
        // Closing a file checks for unsaved changes
        self.check_changes_before_action(PendingAction::CloseFile);
    }

    pub(crate) fn open_directory(&mut self) {
        self.check_changes_before_action(PendingAction::OpenDirectory);
    }

    pub(crate) fn change_directory(&mut self, path: PathBuf) {
        // FIX: Folders do NOT check for changes - directory navigation is always allowed
        self.perform_change_directory(path);
    }

    /// Export current document as plaintext .txt file
    pub(crate) fn export_plaintext(&mut self) {
        let content = &self.document.current_content;
        if content.is_empty() {
            self.status_message = t!("actions.status_export_empty").to_string();
            self.log_warning(t!("actions.log_export_empty"));
            return;
        }

        // Suggest filename based on current file
        let suggested_name = if let Some(path) = &self.current_file_path {
            let stem = path.file_stem().unwrap_or_default().to_string_lossy();
            format!("{}.txt", stem)
        } else {
            "document.txt".to_string()
        };

        if let Some(path) = rfd::FileDialog::new()
            .add_filter(t!("actions.filter_all"), &["*"])
            .set_file_name(&suggested_name)
            .save_file()
        {
            match sen_core::fs::atomic_write(&path, content) {
                Ok(_) => {
                    let masked = self.mask_directory_path(&path);
                    self.status_message = t!("actions.status_exported", file = masked).to_string();
                    self.log_info(t!(
                        "actions.log_exported",
                        size = content.len(),
                        file = masked
                    ));
                }
                Err(e) => {
                    self.status_message = t!("actions.status_export_err", e = e).to_string();
                    self.log_error(t!("actions.log_export_err", e = e));
                }
            }
        } else {
            self.log_info(t!("actions.log_export_cancel"));
        }
    }

    /// Set file association for .sen files (Windows/Linux)
    pub(crate) fn associate_sen_files(&mut self) {
        #[cfg(not(any(target_os = "windows", target_os = "linux")))]
        {
            self.log_warning(t!("actions.log_assoc_not_supp"));
            self.status_message = t!("actions.status_assoc_not_supp").to_string();
            return;
        }

        #[cfg(any(target_os = "windows", target_os = "linux"))]
        match self.perform_association() {
            Ok(_) => {
                self.log_success(t!("actions.log_assoc_success"));
                self.status_message = t!("actions.status_assoc_success").to_string();
            }
            Err(e) => {
                self.log_error(t!("actions.log_assoc_failed", e = e));
                self.status_message = t!("actions.status_assoc_failed").to_string();
                crate::sen_debug!("Association error: {}", e);
            }
        }
    }

    #[cfg(any(target_os = "windows", target_os = "linux"))]
    fn perform_association(&self) -> Result<(), Box<dyn std::error::Error>> {
        let exe_path = std::env::current_exe()?;

        #[cfg(target_os = "windows")]
        {
            use winreg::enums::*;
            use winreg::RegKey;

            let hkcu = RegKey::predef(HKEY_CURRENT_USER);
            let classes = hkcu.open_subkey_with_flags("Software\\Classes", KEY_ALL_ACCESS)?;

            // 1. .sen -> sen_file
            let (dot_sen, _) = classes.create_subkey(".sen")?;
            dot_sen.set_value("", &"sen_file")?;

            // 2. sen_file -> commands
            let (sen_file, _) = classes.create_subkey("sen_file")?;
            sen_file.set_value("", &"Secure Encrypted Notepad Document")?;

            let (default_icon, _) = classes.create_subkey("sen_file\\DefaultIcon")?;
            let icon_str = format!("\"{}\",0", exe_path.display());
            default_icon.set_value("", &icon_str)?;

            let (shell_open_command, _) =
                classes.create_subkey("sen_file\\shell\\open\\command")?;
            let command_str = format!("\"{}\" \"%1\"", exe_path.display());
            shell_open_command.set_value("", &command_str)?;

            // Force Explorer to refresh icon cache immediately
            unsafe {
                windows_sys::Win32::UI::Shell::SHChangeNotify(
                    0x08000000, // SHCNE_ASSOCCHANGED
                    0x0000,     // SHCNF_IDLIST
                    std::ptr::null(),
                    std::ptr::null(),
                );
            }
        }

        #[cfg(target_os = "linux")]
        {
            use std::fs;
            let home = dirs::home_dir().ok_or("Could not find home directory")?;

            // 1. MIME Type
            let mime_dir = home.join(".local/share/mime/packages");
            fs::create_dir_all(&mime_dir)?;
            let mime_content = r#"<?xml version="1.0" encoding="UTF-8"?>
<mime-info xmlns="http://www.freedesktop.org/standards/shared-mime-info">
  <mime-type type="application/x-sen">
    <comment>Secure Encrypted Notepad Document</comment>
    <glob pattern="*.sen"/>
  </mime-type>
</mime-info>"#;
            sen_core::fs::atomic_write(mime_dir.join("sen.xml"), mime_content)?;

            // 2. Desktop Entry
            let app_dir = home.join(".local/share/applications");
            fs::create_dir_all(&app_dir)?;
            let desktop_content = format!(
                r#"[Desktop Entry]
Type=Application
Name=Secure Encrypted Notepad
Exec={} %f
MimeType=application/x-sen;
Icon=text-x-generic
Categories=Utility;TextEditor;
Terminal=false"#,
                exe_path.display()
            );
            sen_core::fs::atomic_write(app_dir.join("sen.desktop"), desktop_content)?;

            // 3. Update databases
            let _ = std::process::Command::new("update-mime-database")
                .arg(home.join(".local/share/mime"))
                .status();
            let _ = std::process::Command::new("update-desktop-database")
                .arg(&app_dir)
                .status();
        }

        Ok(())
    }

    /// Rotate keyfile — re-encrypt current file with a new keyfile
    pub(crate) fn rotate_keyfile(&mut self) {
        // Must have an open, decrypted file
        if self.current_file_path.is_none() {
            self.status_message = t!("actions.status_rotate_no_file").to_string();
            self.log_error(t!("actions.log_rotate_no_file"));
            return;
        }
        if self.keyfile_path.is_none() {
            self.status_message = t!("actions.status_rotate_no_key").to_string();
            self.log_error(t!("actions.log_rotate_no_key"));
            return;
        }

        // Ask user to select or generate a new keyfile
        self.log_info(t!("actions.log_rotate_select"));

        if let Some(new_keyfile_path) = rfd::FileDialog::new()
            .set_title(t!("dialog.rotate_title"))
            .pick_file()
        {
            // Validate the new keyfile
            match std::fs::metadata(&new_keyfile_path) {
                Ok(metadata) => {
                    if metadata.len() == 0 {
                        self.status_message = t!("actions.log_key_empty").to_string(); // Re-use log?
                        self.log_error(t!("actions.log_key_empty"));
                        return;
                    }
                    const MAX_KEYFILE_SIZE: u64 = 100 * 1024 * 1024;
                    if metadata.len() > MAX_KEYFILE_SIZE {
                        self.status_message = t!(
                            "actions.status_key_large",
                            size = format!("{:.1}", metadata.len() as f64 / (1024.0 * 1024.0))
                        )
                        .to_string();
                        self.log_error(t!("actions.log_key_large", size = metadata.len()));
                        return;
                    }
                }
                Err(e) => {
                    self.status_message = t!("actions.status_key_access_err", e = e).to_string();
                    self.log_error(t!("actions.log_key_access_err", e = e));
                    return;
                }
            }

            // Re-encrypt with the new keyfile
            let file_path = self.current_file_path.clone().unwrap();
            let file_content = self.document.to_file_content();

            match encrypt_file(&file_content, &new_keyfile_path, &file_path) {
                Ok(_) => {
                    let old_name = self
                        .keyfile_path
                        .as_ref()
                        .map(|p| self.mask_keyfile_path(p))
                        .unwrap_or_else(|| "unknown".to_string());

                    self.keyfile_path = Some(new_keyfile_path.clone());
                    self.refresh_file_access_status();
                    self.is_modified = false;

                    let new_masked = self.mask_keyfile_path(&new_keyfile_path);

                    self.status_message = if new_masked == "Secured" && old_name == "Secured" {
                        t!("actions.status_rotate_success").to_string()
                    } else {
                        t!(
                            "actions.status_rotate_file_success",
                            old = old_name,
                            new = new_masked
                        )
                        .to_string()
                    };

                    self.log_success(if new_masked == "Secured" {
                        t!("actions.status_rotate_success").to_string()
                    } else {
                        t!(
                            "actions.log_rotate_success_file",
                            file = self.mask_directory_path(&file_path)
                        )
                        .to_string()
                    });
                }
                Err(e) => {
                    self.status_message = t!("actions.status_rotate_failed", e = e).to_string();
                    self.log_error(t!("actions.log_rotate_failed", e = e));
                }
            }
        } else {
            self.log_info(t!("actions.log_rotate_cancel"));
        }
    }

    /// Perform auto-save if needed. If `immediate` is true, bypass interval check.
    pub(crate) fn perform_autosave(&mut self, immediate: bool) {
        if (!self.settings.auto_save_enabled && !immediate) || !self.is_modified {
            return;
        }

        // Only save if file is open, and keyfile exists
        if self.current_file_path.is_none() || self.keyfile_path.is_none() {
            return;
        }

        // Do not auto-save plaintext files in-place! They must be explicitly saved as .sen first.
        if let Some(path) = &self.current_file_path {
            if path.extension().and_then(|e| e.to_str()).unwrap_or("") != "sen" {
                return;
            }
        }

        let now = std::time::Instant::now();
        if !immediate {
            // DEBOUNCE LOGIC: Wait for inactivity
            let elapsed = now.duration_since(self.last_modification_time).as_secs();
            if elapsed < self.settings.auto_save_debounce_secs {
                return;
            }

            // Don't spam saves if we already auto-saved the current changes
            if let Some(last_time) = self.last_autosave_time {
                if last_time > self.last_modification_time {
                    return;
                }
            }
        }

        let original_path = self.current_file_path.as_ref().unwrap().clone();
        let keyfile = self.keyfile_path.clone().unwrap();

        // Store autosave content inside the document's autosave slot
        self.document
            .set_autosave(self.document.current_content.clone());

        // Re-encrypt the entire file in-place with the autosave slot included
        let file_content = self.document.to_file_content();

        match encrypt_file(&file_content, &keyfile, &original_path) {
            Ok(_) => {
                self.last_autosave_time = Some(now);
                self.log_info(t!(
                    "actions.log_autosave_success",
                    file = self.mask_directory_path(&original_path)
                ));
            }
            Err(e) => {
                // Revert autosave slot on failure to avoid stale data
                self.document.clear_autosave();
                self.log_error(t!("actions.log_autosave_failed", e = e));
            }
        }
    }

    /// Commit the current history state as the "last saved" state
    pub(crate) fn commit_history_state(&mut self) {
        self.initial_history_len = self.document.history.len();
        self.initial_max_history_length = self.document.max_history_length;
    }

    /// Revert history to its last saved state
    pub(crate) fn revert_history_changes(&mut self) {
        // 1. Revert max history length
        self.document.max_history_length = self.initial_max_history_length;

        // 2. Remove any snapshots added since last save/open
        if self.document.history.len() > self.initial_history_len {
            self.document.history.truncate(self.initial_history_len);
        }

        // 3. Clear soft deletions (mark deleted = false)
        for entry in &mut self.document.history {
            entry.deleted = false;
        }

        self.log_info(t!("history.log_reverted"));
    }
}
