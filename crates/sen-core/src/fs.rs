use std::fs::OpenOptions;
use std::io::{Result, Write};
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

/// Atomically writes content to a file on the local filesystem.
/// Writes to a temporary file, syncs memory buffers to persistent storage, 
/// and renames over the target path.
pub fn atomic_write<P: AsRef<Path>, C: AsRef<[u8]>>(path: P, content: C) -> Result<()> {
    let path = path.as_ref();
    let parent = path.parent().unwrap_or_else(|| Path::new(""));
    let parent_str = if parent.as_os_str().is_empty() { Path::new(".") } else { parent };
    
    use rand::{distr::Alphanumeric, Rng};
    let rand_suffix: String = rand::rng()
        .sample_iter(&Alphanumeric)
        .take(8)
        .map(char::from)
        .collect();
    
    let file_name = path.file_name().unwrap_or_default().to_string_lossy();
    let temp_name = format!("{}.tmp.{}", file_name, rand_suffix);
    let temp_path = parent_str.join(temp_name);
    
    {
        let mut file = OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .open(&temp_path)?;
            
        file.write_all(content.as_ref())?;
        file.sync_all()?;
    }
    
    match std::fs::rename(&temp_path, path) {
        Ok(_) => Ok(()),
        Err(e) => {
            let _ = std::fs::remove_file(temp_path);
            Err(e)
        }
    }
}
