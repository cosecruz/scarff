//! Local filesystem adapter using std::fs.

use std::io;
use std::path::Path;

use scarff_core::{application::ports::Filesystem, error::ScarffResult};

/// Production filesystem implementation using `std::fs`.
#[derive(Debug, Clone, Copy)]
pub struct LocalFilesystem;

impl LocalFilesystem {
    /// Create a new local filesystem adapter.
    pub fn new() -> Self {
        Self
    }
}

impl Default for LocalFilesystem {
    fn default() -> Self {
        Self::new()
    }
}

impl Filesystem for LocalFilesystem {
    fn create_dir_all(&self, path: &Path) -> ScarffResult<()> {
        std::fs::create_dir_all(path).map_err(|e| map_io_error(path, e, "create directory"))
    }

    fn write_file(&self, path: &Path, content: &str) -> ScarffResult<()> {
        std::fs::write(path, content).map_err(|e| map_io_error(path, e, "write file"))
    }

    fn set_permissions(&self, path: &Path, executable: bool) -> ScarffResult<()> {
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            if executable {
                let metadata =
                    std::fs::metadata(path).map_err(|e| map_io_error(path, e, "get metadata"))?;
                let mut perms = metadata.permissions();
                let mode = perms.mode();
                perms.set_mode(mode | 0o111);
                std::fs::set_permissions(path, perms)
                    .map_err(|e| map_io_error(path, e, "set permissions"))?;
            }
        }
        #[cfg(windows)]
        {
            // Windows doesn't have executable bit in the same way
            let _ = executable; // Silence unused warning
        }
        Ok(())
    }

    fn exists(&self, path: &Path) -> bool {
        path.exists()
    }

    fn remove_dir_all(&self, path: &Path) -> ScarffResult<()> {
        std::fs::remove_dir_all(path).map_err(|e| map_io_error(path, e, "remove directory"))
    }
}

fn map_io_error(path: &Path, e: io::Error, operation: &str) -> scarff_core::error::ScarffError {
    use scarff_core::application::ApplicationError;

    ApplicationError::FilesystemError {
        path: path.to_path_buf(),
        reason: format!("Failed to {}: {}", operation, e),
    }
    .into()
}
