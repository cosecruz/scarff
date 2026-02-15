//! In-memory filesystem adapter for testing.

use std::{
    collections::{HashMap, HashSet},
    path::{Path, PathBuf},
    sync::{Arc, RwLock},
};

use scarff_core::application::ports::Filesystem;

/// In-memory filesystem for testing.
#[derive(Debug, Clone)]
pub struct MemoryFilesystem {
    inner: Arc<RwLock<MemoryFilesystemInner>>,
}

#[derive(Debug, Default)]
struct MemoryFilesystemInner {
    files: HashMap<PathBuf, String>,
    directories: HashSet<PathBuf>,
    executables: HashSet<PathBuf>,
}

impl MemoryFilesystem {
    /// Create a new empty memory filesystem.
    pub fn new() -> Self {
        Self {
            inner: Arc::new(RwLock::new(MemoryFilesystemInner::default())),
        }
    }

    /// Read a file's content (testing helper).
    pub fn read_file(&self, path: &Path) -> Option<String> {
        let inner = self.inner.read().ok()?;
        inner.files.get(path).cloned()
    }

    /// Check if a file is marked executable.
    pub fn is_executable(&self, path: &Path) -> bool {
        let inner = self.inner.read().unwrap();
        inner.executables.contains(path)
    }

    /// List all files.
    pub fn list_files(&self) -> Vec<PathBuf> {
        let inner = self.inner.read().unwrap();
        inner.files.keys().cloned().collect()
    }

    /// Clear all contents.
    pub fn clear(&self) {
        let mut inner = self.inner.write().unwrap();
        inner.files.clear();
        inner.directories.clear();
        inner.executables.clear();
    }
}

impl Default for MemoryFilesystem {
    fn default() -> Self {
        Self::new()
    }
}

impl Filesystem for MemoryFilesystem {
    fn create_dir_all(&self, path: &Path) -> scarff_core::error::ScarffResult<()> {
        let mut inner = self
            .inner
            .write()
            .map_err(|_| scarff_core::application::ApplicationError::StoreLockError)?;

        let mut current = PathBuf::new();
        for component in path.components() {
            current.push(component);
            inner.directories.insert(current.clone());
        }

        Ok(())
    }

    fn write_file(&self, path: &Path, content: &str) -> scarff_core::error::ScarffResult<()> {
        let mut inner = self
            .inner
            .write()
            .map_err(|_| scarff_core::application::ApplicationError::StoreLockError)?;

        // Ensure parent exists
        if let Some(parent) = path.parent() {
            if !parent.as_os_str().is_empty() && !inner.directories.contains(parent) {
                return Err(
                    scarff_core::application::ApplicationError::FilesystemError {
                        path: path.to_path_buf(),
                        reason: "Parent directory does not exist".into(),
                    }
                    .into(),
                );
            }
        }

        inner.files.insert(path.to_path_buf(), content.to_string());
        Ok(())
    }

    fn set_permissions(
        &self,
        path: &Path,
        executable: bool,
    ) -> scarff_core::error::ScarffResult<()> {
        let mut inner = self
            .inner
            .write()
            .map_err(|_| scarff_core::application::ApplicationError::StoreLockError)?;

        if executable {
            inner.executables.insert(path.to_path_buf());
        } else {
            inner.executables.remove(path);
        }

        Ok(())
    }

    fn exists(&self, path: &Path) -> bool {
        let inner = self.inner.read().unwrap();
        inner.files.contains_key(path) || inner.directories.contains(path)
    }

    fn remove_dir_all(&self, path: &Path) -> scarff_core::error::ScarffResult<()> {
        let mut inner = self
            .inner
            .write()
            .map_err(|_| scarff_core::application::ApplicationError::StoreLockError)?;

        inner.directories.remove(path);
        inner.files.retain(|p, _| !p.starts_with(path));
        inner.executables.retain(|p| !p.starts_with(path));

        Ok(())
    }
}
