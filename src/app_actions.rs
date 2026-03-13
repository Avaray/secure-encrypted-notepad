use crate::app_state::PendingAction;
use crate::crypto::{decrypt_file, encrypt_file, generate_keyfile};
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

        if self.is_modified && !skip_check {
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
            PendingAction::OpenFileFromTree(path) => self.perform_open_file(path),
            PendingAction::ChangeDirectory(path) => self.perform_change_directory(path),
        }
    }

    /// New document implementation
    pub(crate) fn perform_new_document(&mut self) {
        self.document = DocumentWithHistory::new_with_limit(self.settings.max_history_length);
        self.current_file_path = None;
        self.is_modified = false;
        self.loaded_history_index = None;
        self.status_message = "New document created".to_string();
        self.log_info("New document created");
    }

    /// Open file dialog implementation
    pub(crate) fn perform_open_file_dialog(&mut self) {
        self.log_info("Opening file dialog");
        if let Some(path) = rfd::FileDialog::new()
            .add_filter("SEN Files", &["sen"])
            .add_filter("All Files", &["*"])
            .pick_file()
        {
            self.perform_open_file(path);
        } else {
            self.log_info("File dialog cancelled");
        }
    }

    /// Open file implementation
    pub(crate) fn perform_open_file(&mut self, path: PathBuf) {
        if self.keyfile_path.is_none() {
            self.status_message = "Error: No keyfile loaded".to_string();
            self.log_error("Attempted to open file without keyfile");
            return;
        }

        let keyfile = self.keyfile_path.clone().unwrap();
        self.log_info(format!("Opening file: {}", self.mask_directory_path(&path)));
        self.log_info(format!(
            "Using keyfile: {}",
            self.mask_keyfile_path(&keyfile)
        ));

        match decrypt_file(&keyfile, &path) {
            Ok(content) => {
                self.document = DocumentWithHistory::from_file_content(&content);
                self.current_file_path = Some(path.clone());
                self.is_modified = false;
                self.loaded_history_index = None;

                let history_count = self.document.get_visible_history().len();

                // Check for internal auto-save slot
                if self.document.autosave.is_some() {
                    self.show_autosave_restore = true;
                }

                self.status_message = format!(
                    "Opened: {} ({} history entries)",
                    self.mask_directory_path(&path),
                    history_count
                );
                self.log_info(format!(
                    "OK: File opened successfully with {} history entries",
                    history_count
                ));
            }
            Err(e) => {
                self.status_message = format!("Error: {}", e);
                self.log_error(format!("Failed to open file: {}", e));
            }
        }
    }

    /// Open directory implementation
    pub(crate) fn perform_open_directory(&mut self) {
        if let Some(path) = rfd::FileDialog::new().pick_folder() {
            self.log_info(format!(
                "Opening directory: {}",
                self.mask_directory_path(&path)
            ));
            self.file_tree_dir = Some(path.clone());

            self.show_file_tree = true;
            self.settings.show_file_tree = true;
            let _ = self.settings.save();
            self.refresh_file_tree();
            self.setup_watcher();
        }
    }

    /// Change directory implementation
    pub(crate) fn perform_change_directory(&mut self, path: PathBuf) {
        self.log_info(format!(
            "Changing to directory: {}",
            self.mask_directory_path(&path)
        ));
        self.file_tree_dir = Some(path.clone());

        let _ = self.settings.save();
        self.refresh_file_tree();
        self.setup_watcher();
    }

    /// Save file
    pub(crate) fn save_file(&mut self) {
        if self.keyfile_path.is_none() {
            self.status_message = "Error: No keyfile loaded".to_string();
            self.log_error("Attempted to save without keyfile");
            return;
        }

        if let Some(path) = self.current_file_path.clone() {
            self.log_info("Saving to existing file path");
            self.perform_save(path);
        } else {
            self.log_info("No file path set, opening save dialog");
            self.save_file_as();
        }
    }

    /// Save file as
    pub(crate) fn save_file_as(&mut self) {
        if self.keyfile_path.is_none() {
            self.status_message = "Error: No keyfile loaded".to_string();
            self.log_error("Attempted to save as without keyfile");
            return;
        }

        self.log_info("Opening save as dialog");
        let mut dialog = rfd::FileDialog::new()
            .add_filter("SEN Files", &["sen"])
            .set_file_name("document.sen");

        // If a directory is open in the file tree, use it as default
        if let Some(dir) = &self.file_tree_dir {
            dialog = dialog.set_directory(dir);
        }

        if let Some(path) = dialog.save_file() {
            self.perform_save(path);
        } else {
            self.log_info("Save as dialog cancelled");
        }
    }

    /// Perform actual save
    pub(crate) fn perform_save(&mut self, path: PathBuf) {
        let keyfile = self.keyfile_path.clone().unwrap();
        self.log_info(format!("Saving file: {}", self.mask_directory_path(&path)));

        // Save current state to history (snapshot) if modified
        if self.is_modified {
            self.document.add_snapshot(None);
            self.log_info("Snapshot created automatically");
        }

        // Clear autosave slot on proper save
        self.document.clear_autosave();

        let file_content = self.document.to_file_content();
        self.log_info(format!("Content size: {} bytes", file_content.len()));

        match encrypt_file(&file_content, &keyfile, &path) {
            Ok(_) => {
                self.current_file_path = Some(path.clone());
                self.is_modified = false;
                
                // Commit trimmed history state after successful save
                self.document.trim_to_limit();
                // Filter out entries marked as deleted to fully synchronize
                self.document.history.retain(|e| !e.deleted);

                let history_count = self.document.get_visible_history().len();
                self.status_message = format!(
                    "Saved: {} ({} history entries)",
                    self.mask_directory_path(&path),
                    history_count
                );
                self.log_success("OK: File saved successfully");
                self.refresh_file_tree();

                // Auto-Backup Logic
                if self.settings.auto_backup_enabled {
                    if let Some(backup_dir) = &self.settings.auto_backup_dir {
                        if let Some(file_name) = path.file_name() {
                            let backup_path = backup_dir.join(file_name);
                            match std::fs::copy(&path, &backup_path) {
                                Ok(_) => self.log_info(format!("Auto-backed up to {}", self.mask_directory_path(&backup_path))),
                                Err(e) => self.log_error(format!("Auto-backup failed: {}", e)),
                            }
                        }
                    }
                }
            }
            Err(e) => {
                self.status_message = format!("Error: {}", e);
                self.log_error(format!("Save failed: {}", e));
            }
        }
    }

    /// Load keyfile
    pub(crate) fn load_keyfile(&mut self) {
        if let Some(path) = rfd::FileDialog::new().pick_file() {
            self.log_info(format!(
                "Attempting to load keyfile: {}",
                self.mask_keyfile_path(&path)
            ));

            match std::fs::metadata(&path) {
                Ok(metadata) => {
                    let size = metadata.len();
                    self.log_info(format!("Keyfile size: {} bytes", size));

                    if size == 0 {
                        self.status_message = "Error: Keyfile is empty (0 bytes)".to_string();
                        self.log_error("Keyfile is empty");
                        return;
                    }
                    const MAX_KEYFILE_SIZE: u64 = 100 * 1024 * 1024; // 100 MB
                    if size > MAX_KEYFILE_SIZE {
                        self.status_message = format!(
                            "Error: Keyfile too large ({:.1} MB, max 100 MB)",
                            size as f64 / (1024.0 * 1024.0)
                        );
                        self.log_error(format!("Keyfile too large: {} bytes", size));
                        return;
                    }

                    match std::fs::read(&path) {
                        Ok(_content) => {
                            self.keyfile_path = Some(path.clone());
                            self.refresh_file_access_status();
                            let masked = self.mask_keyfile_path(&path);
                            self.status_message = format!("Valid keyfile loaded: {}", masked);
                            self.log_info(format!("Valid keyfile loaded successfully: {}", masked));
                        }
                        Err(e) => {
                            self.status_message = format!("Error: Cannot read keyfile: {}", e);
                            self.log_error(format!("Cannot read keyfile: {}", e));
                        }
                    }
                }
                Err(e) => {
                    self.status_message = format!("Error: Cannot access keyfile: {}", e);
                    self.log_error(format!("Cannot access keyfile: {}", e));
                }
            }
        }
    }

    /// Generate new keyfile
    pub(crate) fn generate_new_keyfile(&mut self) {
        if let Some(path) = rfd::FileDialog::new().set_file_name("keyfile").save_file() {
            self.log_info(format!(
                "Generating new keyfile: {}",
                self.mask_keyfile_path(&path)
            ));

            match generate_keyfile(&path) {
                Ok(_) => {
                    self.keyfile_path = Some(path.clone());
                    self.refresh_file_access_status();
                    let masked = self.mask_keyfile_path(&path);
                    self.status_message = format!("OK: Keyfile generated: {}", masked);
                    self.log_info(format!(
                        "OK: Keyfile generated successfully (256 bytes): {}",
                        masked
                    ));
                }
                Err(e) => {
                    self.status_message = format!("Error: {}", e);
                    self.log_error(format!("Keyfile generation failed: {}", e));
                }
            }
        }
    }

    /// Load version from history
    pub(crate) fn load_history_version(&mut self, index: usize) {
        if self.document.load_version(index) {
            self.is_modified = true;
            self.status_message = "Version loaded from history".to_string();
            self.log_success(format!("Loaded history version #{}", index));
        }
    }

    /// Delete history entry (soft delete - mark as deleted)
    pub(crate) fn delete_history_entry(&mut self, index: usize) {
        if self.document.mark_entry_deleted(index) {
            self.is_modified = true;
            self.status_message = "History entry marked for deletion (save to apply)".to_string();
            self.log_info(format!("Marked history entry #{} for deletion", index));
        }
    }

    /// Clear all history (soft delete - mark all as deleted)
    pub(crate) fn clear_all_history(&mut self) {
        let count = self.document.get_visible_history().len();
        self.document.mark_all_deleted();
        self.is_modified = true;
        self.loaded_history_index = None;
        self.status_message = format!("Marked {} entries for deletion (save to apply)", count);
        self.log_info(format!(
            "Marked all history for deletion ({} entries)",
            count
        ));
    }

    /// Wrapper functions for UI
    pub(crate) fn new_document(&mut self) {
        self.check_changes_before_action(PendingAction::NewDocument);
    }

    pub(crate) fn open_file_dialog(&mut self) {
        self.check_changes_before_action(PendingAction::OpenFile);
    }

    pub(crate) fn open_file(&mut self, path: PathBuf) {
        // Pliki sprawdzają zmiany
        self.check_changes_before_action(PendingAction::OpenFileFromTree(path));
    }

    pub(crate) fn open_directory(&mut self) {
        self.check_changes_before_action(PendingAction::OpenDirectory);
    }

    pub(crate) fn change_directory(&mut self, path: PathBuf) {
        // POPRAWKA: Foldery NIE sprawdzają zmian - nawigacja jest zawsze dozwolona
        self.perform_change_directory(path);
    }

    /// Export current document as plaintext .txt file
    pub(crate) fn export_plaintext(&mut self) {
        let content = &self.document.current_content;
        if content.is_empty() {
            self.status_message = "Nothing to export — document is empty".to_string();
            self.log_warning("Export cancelled: empty document");
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
            .add_filter("Text Files", &["txt"])
            .add_filter("All Files", &["*"])
            .set_file_name(&suggested_name)
            .save_file()
        {
            match std::fs::write(&path, &content) {
                Ok(_) => {
                    self.status_message = format!(
                        "OK: Exported as plaintext: {}",
                        self.mask_directory_path(&path)
                    );
                    self.log_info(format!(
                        "OK: Exported {} bytes to {}",
                        content.len(),
                        self.mask_directory_path(&path)
                    ));
                }
                Err(e) => {
                    self.status_message = format!("Error exporting: {}", e);
                    self.log_error(format!("Export failed: {}", e));
                }
            }
        } else {
            self.log_info("Export dialog cancelled");
        }
    }

    /// Rotate keyfile — re-encrypt current file with a new keyfile
    pub(crate) fn rotate_keyfile(&mut self) {
        // Must have an open, decrypted file
        if self.current_file_path.is_none() {
            self.status_message = "Error: No file is open to rotate keyfile for".to_string();
            self.log_error("Keyfile rotation requires an open file");
            return;
        }
        if self.keyfile_path.is_none() {
            self.status_message = "Error: No current keyfile loaded".to_string();
            self.log_error("Keyfile rotation requires a current keyfile");
            return;
        }

        // Ask user to select or generate a new keyfile
        self.log_info("Selecting new keyfile for rotation...");

        if let Some(new_keyfile_path) = rfd::FileDialog::new()
            .set_title("Select New Keyfile")
            .pick_file()
        {
            // Validate the new keyfile
            match std::fs::metadata(&new_keyfile_path) {
                Ok(metadata) => {
                    if metadata.len() == 0 {
                        self.status_message = "Error: New keyfile is empty".to_string();
                        self.log_error("New keyfile is empty");
                        return;
                    }
                    const MAX_KEYFILE_SIZE: u64 = 100 * 1024 * 1024;
                    if metadata.len() > MAX_KEYFILE_SIZE {
                        self.status_message = format!(
                            "Error: New keyfile too large ({:.1} MB, max 100 MB)",
                            metadata.len() as f64 / (1024.0 * 1024.0)
                        );
                        self.log_error(format!("New keyfile too large: {} bytes", metadata.len()));
                        return;
                    }
                }
                Err(e) => {
                    self.status_message = format!("Error: Cannot read new keyfile: {}", e);
                    self.log_error(format!("Cannot access new keyfile: {}", e));
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
                    self.status_message =
                        format!("OK: Keyfile rotated: {} → {}", old_name, new_masked);
                    self.log_success(format!(
                        "OK: Keyfile rotated successfully for {}",
                        self.mask_directory_path(&file_path)
                    ));
                }
                Err(e) => {
                    self.status_message = format!("Error during keyfile rotation: {}", e);
                    self.log_error(format!("Keyfile rotation failed: {}", e));
                }
            }
        } else {
            self.log_info("Keyfile rotation cancelled");
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
        self.document.set_autosave(self.document.current_content.clone());

        // Re-encrypt the entire file in-place with the autosave slot included
        let file_content = self.document.to_file_content();

        match encrypt_file(&file_content, &keyfile, &original_path) {
            Ok(_) => {
                self.last_autosave_time = Some(now);
                self.log_info(format!(
                    "Auto-saved inside {}",
                    self.mask_directory_path(&original_path)
                ));
            }
            Err(e) => {
                // Revert autosave slot on failure to avoid stale data
                self.document.clear_autosave();
                self.log_error(format!("Auto-save failed: {}", e));
            }
        }
    }
}
