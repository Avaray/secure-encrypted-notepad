use serde::{Deserialize, Serialize};

/// File tree entry type - shared between desktop and mobile.
///
/// Uses `String` for `uri` to support both absolute paths (Desktop)
/// and Content URIs (Android Storage Access Framework).
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct FileTreeEntry {
    /// Absolute path or Content URI
    pub uri: String,
    /// Display name (usually just the filename)
    pub name: String,
    /// Whether this is a directory
    pub is_dir: bool,
    /// UI state: whether this folder is expanded in the tree view
    pub is_expanded: bool,
    /// Indentation level in the UI
    pub depth: usize,
}

impl FileTreeEntry {
    pub fn new(uri: String, name: String, is_dir: bool, depth: usize) -> Self {
        Self {
            uri,
            name,
            is_dir,
            is_expanded: false,
            depth,
        }
    }
}
