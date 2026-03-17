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
    Success,
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
            LogLevel::Success => "SUCCESS",
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
pub struct FileTreeEntry {
    pub path: PathBuf,
    pub is_dir: bool,
    pub is_expanded: bool,
    pub depth: usize,
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
    #[allow(dead_code)]
    ChangeDirectory(PathBuf),
}

/// Status of file access relative to currently loaded key
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KeyStatus {
    Unknown,     // Not checked yet
    Decryptable, // Matches current keyfile (Green)
    WrongKey,    // Keyfile doesn't match (Red)
    NotSen,      // Not a SEN file (Default)
}

impl Default for KeyStatus {
    fn default() -> Self {
        Self::Unknown
    }
}

/// Batch converter operation mode
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BatchMode {
    Encrypt,
    Decrypt,
    Rotate,
}

impl Default for BatchMode {
    fn default() -> Self {
        Self::Encrypt
    }
}
