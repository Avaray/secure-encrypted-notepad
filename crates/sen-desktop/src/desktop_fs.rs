use sen_core::fs::FileSystem;
use std::io::Result;
use std::path::Path;

#[allow(dead_code)]
pub struct DesktopFs;

impl FileSystem for DesktopFs {
    fn read(&self, path: &Path) -> Result<Vec<u8>> {
        std::fs::read(path)
    }

    fn write(&self, path: &Path, content: &[u8]) -> Result<()> {
        sen_core::fs::atomic_write(path, content)
    }

    fn exists(&self, path: &Path) -> bool {
        path.exists()
    }
}
