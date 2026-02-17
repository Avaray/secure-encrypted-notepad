use crate::app_state::PendingAction;
use crate::crypto::{decrypt_file, encrypt_file, generate_keyfile};
use crate::history::DocumentWithHistory;
use crate::EditorApp;
use std::path::PathBuf;

impl EditorApp {
    /// Check for unsaved changes before action
    pub(crate) fn check_changes_before_action(&mut self, action: PendingAction) {
        if self.is_modified {
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
            .add_filter("SED Files", &["sed"])
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
        self.log_info(format!("Opening file: {}", path.display()));
        self.log_info(format!("Using keyfile: {}", keyfile.display()));

        match decrypt_file(&keyfile, &path) {
            Ok(content) => {
                self.document = DocumentWithHistory::from_file_content(&content);
                self.current_file_path = Some(path.clone());
                self.is_modified = false;
                self.loaded_history_index = None;

                let history_count = self.document.get_visible_history().len();
                self.status_message = format!(
                    "Opened: {} ({} history entries)",
                    path.display(),
                    history_count
                );
                self.log_info(format!(
                    "✓ File opened successfully with {} history entries",
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
            self.log_info(format!("Opening directory: {}", path.display()));
            self.file_tree_dir = Some(path.clone());
            self.sensitive_settings.last_directory = Some(path);
            self.show_file_tree = true;
            self.settings.show_file_tree = true;
            let _ = self.settings.save();
            self.refresh_file_tree();
        }
    }

    /// Change directory implementation
    pub(crate) fn perform_change_directory(&mut self, path: PathBuf) {
        self.log_info(format!("Changing to directory: {}", path.display()));
        self.file_tree_dir = Some(path.clone());
        self.sensitive_settings.last_directory = Some(path);
        let _ = self.settings.save();
        self.refresh_file_tree();
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
        if let Some(path) = rfd::FileDialog::new()
            .add_filter("SED Files", &["sed"])
            .set_file_name("document.sed")
            .save_file()
        {
            self.perform_save(path);
        } else {
            self.log_info("Save as dialog cancelled");
        }
    }

    /// Perform actual save
    pub(crate) fn perform_save(&mut self, path: PathBuf) {
        let keyfile = self.keyfile_path.clone().unwrap();
        self.log_info(format!("Saving file: {}", path.display()));

        // Auto-snapshot if enabled
        if self.settings.auto_snapshot_on_save && self.is_modified {
            self.document.add_snapshot(None);
            self.log_info("Snapshot created automatically");
        }

        let file_content = self.document.to_file_content();
        self.log_info(format!("Content size: {} bytes", file_content.len()));

        match encrypt_file(&file_content, &keyfile, &path) {
            Ok(_) => {
                self.current_file_path = Some(path.clone());
                self.is_modified = false;

                let history_count = self.document.get_visible_history().len();
                self.status_message = format!(
                    "Saved: {} ({} history entries)",
                    path.display(),
                    history_count
                );
                self.log_info("✓ File saved successfully");
                self.refresh_file_tree();

                // Cleanup autosave file
                let file_name = path.file_name().unwrap_or_default().to_string_lossy();
                let autosave_name = format!("{}.autosave.sed", file_name);
                let autosave_path = path.with_file_name(autosave_name);
                if autosave_path.exists() {
                     if let Err(e) = std::fs::remove_file(&autosave_path) {
                         self.log_error(format!("Failed to remove autosave file: {}", e));
                     } else {
                         self.log_info("Autosave file cleaned up");
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
            self.log_info(format!("Attempting to load keyfile: {}", path.display()));

            match std::fs::metadata(&path) {
                Ok(metadata) => {
                    let size = metadata.len();
                    self.log_info(format!("Keyfile size: {} bytes", size));

                    if size != 256 {
                        self.status_message =
                            format!("Error: Invalid keyfile (must be 256 bytes, got {})", size);
                        self.log_error(format!(
                            "Invalid keyfile size: {} bytes (expected 256)",
                            size
                        ));
                        return;
                    }

                    match std::fs::read(&path) {
                        Ok(content) => {
                            if content.len() != 256 {
                                self.status_message = "Error: Invalid keyfile content".to_string();
                                self.log_error("Keyfile content length mismatch");
                                return;
                            }

                            self.keyfile_path = Some(path.clone());
                            self.status_message =
                                format!("✓ Valid keyfile loaded: {}", path.display());
                            self.log_info(format!(
                                "✓ Valid keyfile loaded successfully: {}",
                                path.display()
                            ));
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
            self.log_info(format!("Generating new keyfile: {}", path.display()));

            match generate_keyfile(&path) {
                Ok(_) => {
                    self.keyfile_path = Some(path.clone());
                    self.status_message = format!("✓ Keyfile generated: {}", path.display());
                    self.log_info(format!(
                        "✓ Keyfile generated successfully (256 bytes): {}",
                        path.display()
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
            self.log_info(format!("Loaded history version #{}", index));
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
                    self.status_message =
                        format!("✓ Exported as plaintext: {}", path.display());
                    self.log_info(format!(
                        "✓ Exported {} bytes to {}",
                        content.len(),
                        path.display()
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
                    if metadata.len() != 256 {
                        self.status_message = format!(
                            "Error: New keyfile must be 256 bytes (got {})",
                            metadata.len()
                        );
                        self.log_error(format!(
                            "Invalid new keyfile size: {} bytes",
                            metadata.len()
                        ));
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
                        .and_then(|p| p.file_name())
                        .map(|n| n.to_string_lossy().to_string())
                        .unwrap_or_else(|| "unknown".to_string());

                    self.keyfile_path = Some(new_keyfile_path.clone());
                    self.is_modified = false;

                    self.status_message = format!(
                        "✓ Keyfile rotated: {} → {}",
                        old_name,
                        new_keyfile_path
                            .file_name()
                            .unwrap_or_default()
                            .to_string_lossy()
                    );
                    self.log_info(format!(
                        "✓ Keyfile rotated successfully for {}",
                        file_path.display()
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

    /// Perform auto-save if needed
    pub(crate) fn perform_autosave(&mut self) {
        if !self.settings.auto_save_enabled {
            return;
        }

        // Only save if modified, file is open, and keyfile exists
        if !self.is_modified || self.current_file_path.is_none() || self.keyfile_path.is_none() {
            return;
        }

        let now = std::time::Instant::now();
        if let Some(last_time) = self.last_autosave_time {
            if now.duration_since(last_time).as_secs() < self.settings.auto_save_interval_secs {
                return;
            }
        } else {
             // Initialize timer if not set (wait for first interval)
             self.last_autosave_time = Some(now);
             return;
        }

        let original_path = self.current_file_path.as_ref().unwrap();
        // Construct autosave path: filename.autosave.sed
        let file_name = original_path.file_name().unwrap_or_default().to_string_lossy();
        let autosave_name = format!("{}.autosave.sed", file_name);
        let autosave_path = original_path.with_file_name(autosave_name);
        
        let keyfile = self.keyfile_path.clone().unwrap();
        
        // Encrypt and save silently
        let file_content = self.document.to_file_content();
        
        match encrypt_file(&file_content, &keyfile, &autosave_path) {
            Ok(_) => {
                self.last_autosave_time = Some(now);
                self.log_info(format!("Auto-saved to {}", autosave_path.display()));
            }
            Err(e) => {
                self.log_error(format!("Auto-save failed: {}", e));
            }
        }
    }
}
