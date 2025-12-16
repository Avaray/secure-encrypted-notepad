use chrono::{DateTime, Local};
use serde::{Deserialize, Serialize};

/// Maximum number of history entries per file
const MAX_HISTORY_ENTRIES: usize = 100;

/// Separator between content and history in file
const HISTORY_SEPARATOR: &str = "\n<<HISTORY>>\n";

/// Single history entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HistoryEntry {
    /// Timestamp when this version was created
    pub timestamp: DateTime<Local>,
    /// Content snapshot
    pub content: String,
    /// Optional user comment
    pub comment: Option<String>,
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
    /// History entries (max 100)
    pub history: Vec<HistoryEntry>,
}

impl Default for DocumentWithHistory {
    fn default() -> Self {
        Self {
            current_content: String::new(),
            history: Vec::new(),
        }
    }
}

impl DocumentWithHistory {
    /// Parse file content (current + history)
    pub fn from_file_content(file_content: &str) -> Self {
        if let Some(pos) = file_content.find(HISTORY_SEPARATOR) {
            let current_content = file_content[..pos].to_string();
            let history_json = &file_content[pos + HISTORY_SEPARATOR.len()..];
            let history: Vec<HistoryEntry> = serde_json::from_str(history_json).unwrap_or_default();
            Self {
                current_content,
                history,
            }
        } else {
            // No history, just content
            Self {
                current_content: file_content.to_string(),
                history: Vec::new(),
            }
        }
    }

    /// Convert to file format (current + history)
    pub fn to_file_content(&self) -> String {
        if self.history.is_empty() {
            self.current_content.clone()
        } else {
            let history_json = serde_json::to_string(&self.history).unwrap_or_default();
            format!(
                "{}{}{}",
                self.current_content, HISTORY_SEPARATOR, history_json
            )
        }
    }

    /// Add new snapshot to history
    pub fn add_snapshot(&mut self, description: Option<String>) {
        let snapshot = HistoryEntry {
            content: self.current_content.clone(),
            timestamp: chrono::Local::now(),
            comment: description,
        };
        self.history.push(snapshot);
    }

    /// Add snapshot with max length check
    pub fn add_snapshot_with_limit(&mut self, description: Option<String>, max_length: usize) {
        let snapshot = HistoryEntry {
            content: self.current_content.clone(),
            timestamp: chrono::Local::now(),
            comment: description,
        };
        self.history.push(snapshot);

        // Trim oldest entries if exceeded
        while self.history.len() > max_length {
            self.history.remove(0);
        }
    }

    /// Get history entries
    pub fn get_history(&self) -> &[HistoryEntry] {
        &self.history
    }

    /// Load specific version into current content
    pub fn load_version(&mut self, index: usize) -> bool {
        if let Some(entry) = self.history.get(index) {
            self.current_content = entry.content.clone();
            true
        } else {
            false
        }
    }

    /// Delete specific history entry
    pub fn delete_entry(&mut self, index: usize) -> bool {
        if index < self.history.len() {
            self.history.remove(index);
            true
        } else {
            false
        }
    }

    /// Clear all history
    pub fn clear_history(&mut self) {
        self.history.clear();
    }

    /// Set maximum history length
    pub fn set_max_history_length(&mut self, max_length: usize) {
        // Trim existing history if needed
        while self.history.len() > max_length {
            self.history.remove(0);
        }
    }

    /// Get maximum history length (currently hardcoded but can be made configurable)
    pub fn get_max_history_length(&self) -> usize {
        100 // This will be replaced with settings value
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_history_embedded() {
        let mut doc = DocumentWithHistory::default();
        doc.current_content = "Initial content".to_string();
        doc.add_snapshot(Some("First save".to_string()));
        doc.current_content = "Modified content".to_string();
        doc.add_snapshot(Some("Second save".to_string()));

        assert_eq!(doc.history.len(), 2);

        let file_content = doc.to_file_content();
        let loaded = DocumentWithHistory::from_file_content(&file_content);

        assert_eq!(loaded.current_content, "Modified content");
        assert_eq!(loaded.history.len(), 2);
    }
}
