use chrono::{DateTime, Local};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};

use crate::crypto::{encrypt_file, decrypt_file, CryptoError};

/// Information about a file version
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VersionInfo {
    /// Timestamp when version was created
    pub timestamp: DateTime<Local>,
    
    /// Path to the snapshot file
    pub file_path: PathBuf,
    
    /// File size in bytes
    pub size_bytes: u64,
    
    /// Optional user comment
    pub comment: Option<String>,
}

impl VersionInfo {
    /// Format timestamp as string (for filename)
    // FIX: Dodano #[allow(dead_code)] - metoda jest publiczna, przeznaczona do użytku zewnętrznego.
    #[allow(dead_code)]
    pub fn format_timestamp(&self) -> String {
        self.timestamp.format("%Y-%m-%d_%H-%M-%S").to_string()
    }
    
    /// Format timestamp as human-readable string
    pub fn display_timestamp(&self) -> String {
        self.timestamp.format("%Y-%m-%d %H:%M:%S").to_string()
    }
    
    /// Format size as human-readable string
    pub fn display_size(&self) -> String {
        let size = self.size_bytes as f64;
        if size < 1024.0 {
            format!("{} B", size)
        } else if size < 1024.0 * 1024.0 {
            format!("{:.1} KB", size / 1024.0)
        } else {
            format!("{:.1} MB", size / (1024.0 * 1024.0))
        }
    }
}

/// Errors related to version control
#[derive(Debug)]
pub enum HistoryError {
    IoError(std::io::Error),
    CryptoError(CryptoError),
    // FIX: Usunięto InvalidHistoryFolder - wariant nigdy nie był używany
    VersionNotFound,
}

impl From<std::io::Error> for HistoryError {
    fn from(err: std::io::Error) -> Self {
        HistoryError::IoError(err)
    }
}

impl From<CryptoError> for HistoryError {
    fn from(err: CryptoError) -> Self {
        HistoryError::CryptoError(err)
    }
}

impl std::fmt::Display for HistoryError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            HistoryError::IoError(e) => write!(f, "IO Error: {}", e),
            HistoryError::CryptoError(e) => write!(f, "Crypto Error: {}", e),
            // FIX: Usunięto obsługę InvalidHistoryFolder
            HistoryError::VersionNotFound => write!(f, "Version not found"),
        }
    }
}

impl std::error::Error for HistoryError {}

/// Returns path to history folder for given file
/// 
/// Example:
/// - document.sed -> document.sed.history/
fn get_history_folder(base_file_path: &Path) -> PathBuf {
    let mut history_folder = base_file_path.to_path_buf();
    let new_name = format!(
        "{}.history",
        base_file_path.file_name().unwrap_or_default().to_string_lossy()
    );
    history_folder.set_file_name(new_name);
    history_folder
}

/// Creates history folder if it doesn't exist
fn ensure_history_folder(base_file_path: &Path) -> Result<PathBuf, HistoryError> {
    let history_folder = get_history_folder(base_file_path);
    
    if !history_folder.exists() {
        fs::create_dir_all(&history_folder)?;
    }
    
    Ok(history_folder)
}

/// CREATES NEW SNAPSHOT IN HISTORY
/// 
/// SECURITY:
/// - Snapshot is encrypted with same password + keyfile as main file
/// - Timestamp in filename prevents collisions
/// - Each snapshot has its own salt and nonce
/// 
/// Workflow:
/// 1. Create .history folder if doesn't exist
/// 2. Generate filename with timestamp
/// 3. Encrypt content and save as snapshot
/// 4. Return VersionInfo with metadata
pub fn create_snapshot(
    content: &str,
    password: &str,
    keyfile_path: &Path,
    base_file_path: &Path,
    comment: Option<String>,
) -> Result<VersionInfo, HistoryError> {
    // 1. Ensure history folder exists
    let history_folder = ensure_history_folder(base_file_path)?;
    
    // 2. Generate timestamp and filename
    let timestamp = Local::now();
    let formatted_timestamp = timestamp.format("%Y-%m-%d_%H-%M-%S").to_string();
    let snapshot_filename = format!("version_{}.sed", formatted_timestamp);
    let snapshot_path = history_folder.join(&snapshot_filename);
    
    // 3. Encrypt and save snapshot
    encrypt_file(content, password, keyfile_path, &snapshot_path)?;
    
    // 4. Get file size
    let metadata = fs::metadata(&snapshot_path)?;
    let size_bytes = metadata.len();
    
    // 5. Return VersionInfo
    Ok(VersionInfo {
        timestamp,
        file_path: snapshot_path,
        size_bytes,
        comment,
    })
}

/// LISTS ALL AVAILABLE VERSIONS
/// 
/// Workflow:
/// 1. Check if history folder exists
/// 2. Scan folder for version_*.sed files
/// 3. Parse timestamp from filename
/// 4. Return sorted list (newest first)
pub fn list_versions(base_file_path: &Path) -> Result<Vec<VersionInfo>, HistoryError> {
    let history_folder = get_history_folder(base_file_path);
    
    // If folder doesn't exist, return empty list (not an error)
    if !history_folder.exists() {
        return Ok(Vec::new());
    }
    
    let mut versions = Vec::new();
    
    // Scan folder for snapshot files
    for entry in fs::read_dir(&history_folder)? {
        let entry = entry?;
        let path = entry.path();
        
        // Check if it's a .sed file
        if path.is_file() && path.extension().and_then(|s| s.to_str()) == Some("sed") {
            // Try to parse timestamp from name
            if let Some(filename) = path.file_stem().and_then(|s| s.to_str()) {
                // Format: version_2025-11-30_09-58-23
                if filename.starts_with("version_") {
                    let timestamp_str = &filename[8..]; // Remove "version_"
                    
                    // Parse timestamp (format: YYYY-MM-DD_HH-MM-SS)
                    if let Ok(timestamp) = DateTime::parse_from_str(
                        &format!("{} +0000", timestamp_str.replace('_', " ").replace('-', ":")),
                        "%Y:%m:%d %H:%M:%S %z"
                    ) {
                        let metadata = fs::metadata(&path)?;
                        
                        versions.push(VersionInfo {
                            timestamp: timestamp.with_timezone(&Local),
                            file_path: path,
                            size_bytes: metadata.len(),
                            comment: None, // TODO: Load from metadata.json
                        });
                    }
                }
            }
        }
    }
    
    // Sort by timestamp (newest first)
    versions.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));
    
    Ok(versions)
}

/// LOADS SELECTED VERSION (READ-ONLY)
/// 
/// SECURITY:
/// - Uses same password + keyfile as for encryption
/// - If password or keyfile is wrong, decryption fails
pub fn load_version(
    version_info: &VersionInfo,
    password: &str,
    keyfile_path: &Path,
) -> Result<String, HistoryError> {
    let content = decrypt_file(password, keyfile_path, &version_info.file_path)?;
    Ok(content)
}

/// RESTORES SELECTED VERSION AS CURRENT
/// 
/// SECURITY:
/// - Creates backup of current version before overwriting
/// - Backup is also a snapshot in history
/// 
/// Workflow:
/// 1. Load selected version (decrypt)
/// 2. Encrypt and save as main file
/// 3. Optionally: Create snapshot of current version before overwriting
pub fn restore_version(
    version_info: &VersionInfo,
    password: &str,
    keyfile_path: &Path,
    base_file_path: &Path,
    create_backup: bool,
) -> Result<(), HistoryError> {
    // 1. Load content from selected version
    let content = load_version(version_info, password, keyfile_path)?;
    
    // 2. Optionally: Create backup of current version
    if create_backup && base_file_path.exists() {
        // Load current content
        let current_content = decrypt_file(password, keyfile_path, base_file_path)?;
        
        // Create snapshot with comment "Backup before restore"
        create_snapshot(
            &current_content,
            password,
            keyfile_path,
            base_file_path,
            Some("Backup before restore".to_string()),
        )?;
    }
    
    // 3. Save selected version as main file
    encrypt_file(&content, password, keyfile_path, base_file_path)?;
    
    Ok(())
}

/// DELETES SELECTED VERSION FROM HISTORY
/// 
/// WARNING: This is a destructive operation - file is permanently deleted
pub fn delete_version(version_info: &VersionInfo) -> Result<(), HistoryError> {
    if !version_info.file_path.exists() {
        return Err(HistoryError::VersionNotFound);
    }
    
    fs::remove_file(&version_info.file_path)?;
    
    Ok(())
}

/// DELETES OLD VERSIONS (older than X days)
/// 
/// Usage: Cleanup old versions automatically or on user request
pub fn cleanup_old_versions(
    base_file_path: &Path,
    retention_days: i64,
) -> Result<usize, HistoryError> {
    let versions = list_versions(base_file_path)?;
    let cutoff_date = Local::now() - chrono::Duration::days(retention_days);
    
    let mut deleted_count = 0;
    
    for version in versions {
        if version.timestamp < cutoff_date {
            delete_version(&version)?;
            deleted_count += 1;
        }
    }
    
    Ok(deleted_count)
}

/// HISTORY STATISTICS
#[derive(Debug)]
pub struct HistoryStats {
    pub total_versions: usize,
    pub total_size_bytes: u64,
    // FIX: Dodano #[allow(dead_code)] - pola są publiczne, przeznaczone do użytku zewnętrznego.
    #[allow(dead_code)]
    pub oldest_version: Option<DateTime<Local>>,
    // FIX: Dodano #[allow(dead_code)] - pola są publiczne, przeznaczone do użytku zewnętrznego.
    #[allow(dead_code)]
    pub newest_version: Option<DateTime<Local>>,
}

/// Get history statistics
pub fn get_history_stats(base_file_path: &Path) -> Result<HistoryStats, HistoryError> {
    let versions = list_versions(base_file_path)?;
    
    let total_versions = versions.len();
    let total_size_bytes = versions.iter().map(|v| v.size_bytes).sum();
    
    let oldest_version = versions.last().map(|v| v.timestamp);
    let newest_version = versions.first().map(|v| v.timestamp);
    
    Ok(HistoryStats {
        total_versions,
        total_size_bytes,
        oldest_version,
        newest_version,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::crypto::generate_keyfile;
    use std::thread;
    use std::time::Duration;
    
    #[test]
    fn test_create_and_list_snapshots() {
        let keyfile_path = Path::new("test_history.key");
        generate_keyfile(keyfile_path).unwrap();
        
        let base_file = Path::new("test_doc.sed");
        let password = "test_password";
        
        // Utwórz 3 snapshoty z opóźnieniem
        create_snapshot("Version 1", password, keyfile_path, base_file, Some("First".to_string())).unwrap();
        thread::sleep(Duration::from_secs(1));
        
        create_snapshot("Version 2", password, keyfile_path, base_file, None).unwrap();
        thread::sleep(Duration::from_secs(1));
        
        create_snapshot("Version 3", password, keyfile_path, base_file, Some("Latest".to_string())).unwrap();
        
        // Listuj wersje
        let versions = list_versions(base_file).unwrap();
        assert_eq!(versions.len(), 3);
        
        // Sprawdź sortowanie (najnowsze najpierw)
        assert!(versions[0].timestamp > versions[1].timestamp);
        assert!(versions[1].timestamp > versions[2].timestamp);
        
        // Cleanup
        let history_folder = get_history_folder(base_file);
        fs::remove_dir_all(&history_folder).unwrap();
        fs::remove_file(keyfile_path).unwrap();
    }
    
    #[test]
    fn test_restore_version() {
        let keyfile_path = Path::new("test_restore.key");
        generate_keyfile(keyfile_path).unwrap();
        
        let base_file = Path::new("test_restore.sed");
        let password = "test_password";
        
        // Utwórz snapshot
        let version = create_snapshot("Original content", password, keyfile_path, base_file, None).unwrap();
        
        // Wczytaj
        let content = load_version(&version, password, keyfile_path).unwrap();
        assert_eq!(content, "Original content");
        
        // Cleanup
        let history_folder = get_history_folder(base_file);
        fs::remove_dir_all(&history_folder).unwrap();
        fs::remove_file(keyfile_path).unwrap();
    }
}
