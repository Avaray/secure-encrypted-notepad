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
            self.settings.last_directory = Some(path);
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
        self.settings.last_directory = Some(path);
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
        self.check_changes_before_action(PendingAction::OpenFileFromTree(path));
    }

    pub(crate) fn open_directory(&mut self) {
        self.check_changes_before_action(PendingAction::OpenDirectory);
    }

    pub(crate) fn change_directory(&mut self, path: PathBuf) {
        self.check_changes_before_action(PendingAction::ChangeDirectory(path));
    }
}
