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

            // Try to deserialize with max_history_length field
            if let Ok(doc) = serde_json::from_str::<DocumentWithHistory>(&format!(
                r#"{{"current_content":"","history":{},"max_history_length":100}}"#,
                history_json
            )) {
                Self {
                    current_content,
                    history: doc.history,
                    max_history_length: doc.max_history_length,
                }
            } else {
                // Fallback: old format without max_history_length
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
        if self.history.is_empty() {
            self.current_content.clone()
        } else {
            // Serialize entire document structure
            let doc_data = serde_json::json!({
                "history": self.history,
                "max_history_length": self.max_history_length
            });
            let history_json = serde_json::to_string(&doc_data).unwrap_or_default();
            format!(
                "{}\n{}\n{}",
                self.current_content, HISTORY_SEPARATOR, history_json
            )
        }
    }

    /// Add new snapshot to history (with automatic trimming)
    pub fn add_snapshot(&mut self, comment: Option<String>) {
        let snapshot = HistoryEntry {
            content: self.current_content.clone(),
            timestamp: chrono::Local::now(),
            comment,
        };
        self.history.push(snapshot);

        // Automatically trim to max_history_length
        while self.history.len() > self.max_history_length {
            self.history.remove(0);
        }
    }

    /// Set maximum history length and trim if necessary
    pub fn set_max_history_length(&mut self, max_length: usize) {
        self.max_history_length = max_length.max(1); // At least 1

        // Trim existing history if needed
        while self.history.len() > self.max_history_length {
            self.history.remove(0);
        }
    }

    /// Get maximum history length
    pub fn get_max_history_length(&self) -> usize {
        self.max_history_length
    }

    /// Get reference to history
    pub fn get_history(&self) -> &Vec<HistoryEntry> {
        &self.history
    }

    /// Load version from history by index
    pub fn load_version(&mut self, index: usize) -> bool {
        if index < self.history.len() {
            self.current_content = self.history[index].content.clone();
            true
        } else {
            false
        }
    }

    /// Delete history entry by index
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
