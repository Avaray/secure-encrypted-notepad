use std::io::Result;
use std::path::Path;
use sen_core::fs::FileSystem;

pub struct DesktopFs;

impl FileSystem for DesktopFs {
    fn read(&self, path: &Path) -> Result<Vec<u8>> {
        std::fs::read(path)
    }

    fn write(&self, path: &Path, content: &[u8]) -> Result<()> {
        std::fs::write(path, content)
    }

    fn exists(&self, path: &Path) -> bool {
        path.exists()
    }
}
