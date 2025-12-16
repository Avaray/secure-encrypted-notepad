use chrono;
use std::path::PathBuf;

/// Debug log entry
#[derive(Debug, Clone)]
pub struct LogEntry {
    pub timestamp: chrono::DateTime<chrono::Local>,
    pub level: LogLevel,
    pub message: String,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum LogLevel {
    Info,
    Warning,
    Error,
}

impl LogEntry {
    pub fn new(level: LogLevel, message: String) -> Self {
        Self {
            timestamp: chrono::Local::now(),
            level,
            message,
        }
    }

    pub fn display(&self) -> String {
        let level_str = match self.level {
            LogLevel::Info => "INFO",
            LogLevel::Warning => "WARN",
            LogLevel::Error => "ERROR",
        };
        format!(
            "[{}] {}: {}",
            self.timestamp.format("%H:%M:%S"),
            level_str,
            self.message
        )
    }
}

/// File tree entry type
#[derive(Debug, Clone)]
pub enum FileTreeEntry {
    File(PathBuf),
    Directory(PathBuf),
}

/// Pending action for unsaved changes confirmation
#[derive(Debug, Clone)]
pub enum PendingAction {
    None,
    NewDocument,
    OpenFile,
    OpenDirectory,
    Exit,
    OpenFileFromTree(PathBuf),
    ChangeDirectory(PathBuf),
}
