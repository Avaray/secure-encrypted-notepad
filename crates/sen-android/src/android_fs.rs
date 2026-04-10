use std::io::{Result, Error, ErrorKind};
use std::path::Path;
use sen_core::fs::FileSystem;

pub struct AndroidFs;

impl FileSystem for AndroidFs {
    fn read(&self, path: &Path) -> Result<Vec<u8>> {
        // Here we will eventually bind to JNI/SAF.
        // For now, return an error reminding us to implement SAF.
        Err(Error::new(ErrorKind::Unsupported, "SAF read over JNI is pending implementation"))
    }

    fn write(&self, path: &Path, _content: &[u8]) -> Result<()> {
        Err(Error::new(ErrorKind::Unsupported, "SAF write over JNI is pending implementation"))
    }

    fn exists(&self, path: &Path) -> bool {
        // Placeholder
        path.exists()
    }
}
