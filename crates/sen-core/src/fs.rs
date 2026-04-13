use std::io::Result;
use std::path::Path;

/// Abstraction over the underlying platform file system.
/// This allows the core logic to seamlessly operate on standard desktop
/// file paths or Android's Storage Access Framework (SAF) document URIs.
pub trait FileSystem {
    /// Reads the entire contents of a file into a byte vector.
    fn read(&self, path: &Path) -> Result<Vec<u8>>;

    /// Writes a byte slice as the entire contents of a file.
    fn write(&self, path: &Path, content: &[u8]) -> Result<()>;

    /// Check if a file exists.
    fn exists(&self, path: &Path) -> bool;

    // Future additions may include opening file choosers, handling stream readers for huge files, etc.
}
