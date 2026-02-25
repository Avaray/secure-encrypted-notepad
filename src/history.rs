use chrono::{DateTime, Local};
use serde::{Deserialize, Serialize};

/// Separator between content and history in file
const HISTORY_SEPARATOR: &str = "\n<>\n";

/// Single history entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HistoryEntry {
    /// Timestamp when this version was created
    pub timestamp: DateTime<Local>,
    /// Content snapshot
    pub content: String,
    /// Optional user comment
    pub comment: Option<String>,
    /// Soft delete flag (not saved until file save)
    #[serde(default)]
    pub deleted: bool,
}

impl HistoryEntry {
    pub fn display_timestamp(&self) -> String {
        self.timestamp.format("%Y-%m-%d %H:%M:%S").to_string()
    }

    pub fn display_size(&self) -> String {
        let size = self.content.len() as f64;
        if size < 1024.0 {
            format!("{} B", size)
        } else if size < 1024.0 * 1024.0 {
            format!("{:.1} KB", size / 1024.0)
        } else {
            format!("{:.1} MB", size / (1024.0 * 1024.0))
        }
    }
}

/// File content with embedded history
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentWithHistory {
    /// Current document content
    pub current_content: String,
    /// History entries (max configurable per file)
    pub history: Vec<HistoryEntry>,
    /// Maximum history length for this document
    #[serde(default = "default_max_history")]
    pub max_history_length: usize,
}

fn default_max_history() -> usize {
    100
}

impl Default for DocumentWithHistory {
    fn default() -> Self {
        Self {
            current_content: String::new(),
            history: Vec::new(),
            max_history_length: 100,
        }
    }
}

impl DocumentWithHistory {
    /// Create new document with specific history limit
    pub fn new_with_limit(max_history_length: usize) -> Self {
        Self {
            current_content: String::new(),
            history: Vec::new(),
            max_history_length,
        }
    }

    /// Parse file content (current + history + settings)
    pub fn from_file_content(file_content: &str) -> Self {
        if let Some(pos) = file_content.find(HISTORY_SEPARATOR) {
            let current_content = file_content[..pos].to_string();
            let history_json = &file_content[pos + HISTORY_SEPARATOR.len()..];

            // POPRAWKA: Deserializuj bezpośrednio jako obiekt z history i max_history_length
            #[derive(Deserialize)]
            struct HistoryData {
                history: Vec<HistoryEntry>,
                #[serde(default = "default_max_history")]
                max_history_length: usize,
            }

            if let Ok(data) = serde_json::from_str::<HistoryData>(history_json) {
                Self {
                    current_content,
                    history: data.history,
                    max_history_length: data.max_history_length,
                }
            } else {
                // Fallback: old format - tylko array historii
                let history: Vec<HistoryEntry> =
                    serde_json::from_str(history_json).unwrap_or_default();
                Self {
                    current_content,
                    history,
                    max_history_length: 100,
                }
            }
        } else {
            // No history, just content
            Self {
                current_content: file_content.to_string(),
                history: Vec::new(),
                max_history_length: 100,
            }
        }
    }

    /// Convert to file format (current + history + settings)
    pub fn to_file_content(&self) -> String {
        // Filter out deleted entries before saving
        let active_history: Vec<HistoryEntry> = self
            .history
            .iter()
            .filter(|entry| !entry.deleted)
            .cloned()
            .collect();

        if active_history.is_empty() {
            self.current_content.clone()
        } else {
            // Serialize as object with history and max_history_length
            let doc_data = serde_json::json!({
                "history": active_history,
                "max_history_length": self.max_history_length
            });

            let history_json = serde_json::to_string(&doc_data).unwrap_or_default();
            format!(
                "{}{}{}",
                self.current_content, HISTORY_SEPARATOR, history_json
            )
        }
    }

    /// Add new snapshot to history (with automatic trimming)
    /// Returns `true` if snapshot was added, `false` if content unchanged
    pub fn add_snapshot(&mut self, comment: Option<String>) -> bool {
        // ✅ NOWA LOGIKA: Sprawdź czy content się zmienił
        if let Some(last_entry) = self.history.iter().rev().find(|e| !e.deleted) {
            if last_entry.content == self.current_content {
                // Zawartość identyczna - nie dodawaj
                return false;
            }
        }

        let snapshot = HistoryEntry {
            content: self.current_content.clone(),
            timestamp: chrono::Local::now(),
            comment,
            deleted: false,
        };
        self.history.push(snapshot);

        // Automatically mark oldest as deleted if over limit
        self.trim_to_limit();

        true
    }

    /// Mark entries as deleted to fit max_history_length (soft delete)
    fn trim_to_limit(&mut self) {
        let visible_count = self.history.iter().filter(|e| !e.deleted).count();
        if visible_count > self.max_history_length {
            let to_delete = visible_count - self.max_history_length;
            let mut deleted_count = 0;

            for entry in &mut self.history {
                if deleted_count >= to_delete {
                    break;
                }
                if !entry.deleted {
                    entry.deleted = true;
                    deleted_count += 1;
                }
            }
        }
    }

    /// Set maximum history length and mark excess as deleted (soft delete)
    pub fn set_max_history_length(&mut self, max_length: usize) {
        self.max_history_length = max_length.max(1); // At least 1
        self.trim_to_limit();
    }

    /// Get maximum history length
    pub fn get_max_history_length(&self) -> usize {
        self.max_history_length
    }

    /// Get reference to ALL history (including deleted)
    pub fn get_history(&self) -> &Vec<HistoryEntry> {
        &self.history
    }

    /// Get only visible (non-deleted) history for UI display
    pub fn get_visible_history(&self) -> Vec<(usize, &HistoryEntry)> {
        self.history
            .iter()
            .enumerate()
            .filter(|(_, entry)| !entry.deleted)
            .collect()
    }

    /// Load version from history by index (original index, not filtered)
    pub fn load_version(&mut self, index: usize) -> bool {
        if index < self.history.len() && !self.history[index].deleted {
            self.current_content = self.history[index].content.clone();
            true
        } else {
            false
        }
    }

    /// Mark history entry as deleted by index (soft delete)
    pub fn mark_entry_deleted(&mut self, index: usize) -> bool {
        if index < self.history.len() {
            self.history[index].deleted = true;
            true
        } else {
            false
        }
    }

    /// Mark all history as deleted (soft delete)
    pub fn mark_all_deleted(&mut self) {
        for entry in &mut self.history {
            entry.deleted = true;
        }
    }

    /// Apply deletions - physically remove deleted entries
    /// NOTE: This is NOT used during save anymore - to_file_content() filters automatically
    #[allow(dead_code)]
    pub fn apply_deletions(&mut self) {
        self.history.retain(|entry| !entry.deleted);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_history_embedded() {
        let mut doc = DocumentWithHistory::default();
        doc.current_content = "Initial content".to_string();
        assert!(doc.add_snapshot(Some("First save".to_string())));

        doc.current_content = "Modified content".to_string();
        assert!(doc.add_snapshot(Some("Second save".to_string())));

        assert_eq!(doc.history.len(), 2);

        let file_content = doc.to_file_content();
        let loaded = DocumentWithHistory::from_file_content(&file_content);

        assert_eq!(loaded.current_content, "Modified content");
        assert_eq!(loaded.history.len(), 2);
    }

    #[test]
    fn test_no_duplicate_snapshots() {
        let mut doc = DocumentWithHistory::default();
        doc.current_content = "Same content".to_string();

        assert!(doc.add_snapshot(Some("First".to_string())));
        // Try to add same content again
        assert!(!doc.add_snapshot(Some("Second".to_string())));

        assert_eq!(doc.history.len(), 1); // Only one entry
    }

    #[test]
    fn test_soft_delete() {
        let mut doc = DocumentWithHistory::default();
        doc.current_content = "Test".to_string();
        doc.add_snapshot(Some("v1".to_string()));

        doc.current_content = "Test2".to_string();
        doc.add_snapshot(Some("v2".to_string()));

        assert_eq!(doc.history.len(), 2);

        // Soft delete first entry
        doc.mark_entry_deleted(0);
        assert_eq!(doc.history.len(), 2); // Still 2
        assert_eq!(doc.get_visible_history().len(), 1); // But only 1 visible

        // Save should remove deleted
        let content = doc.to_file_content();
        let reloaded = DocumentWithHistory::from_file_content(&content);
        assert_eq!(reloaded.history.len(), 1); // Only 1 after reload
    }
}
