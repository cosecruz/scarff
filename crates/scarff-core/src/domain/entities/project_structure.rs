use std::collections::HashSet;
use std::path::PathBuf;

use crate::domain::{entities::common::Permissions, error::DomainError};

/// Final project structure ready for materialization.
///
/// This is the output of the template rendering process.
/// It contains no business logic, only data.
#[derive(Debug, Clone)]
pub struct ProjectStructure {
    pub(crate) root: PathBuf,
    pub(crate) entries: Vec<FsEntry>,
}

impl ProjectStructure {
    pub fn new(root: impl Into<PathBuf>) -> Self {
        Self {
            root: root.into(),
            entries: Vec::new(),
        }
    }

    pub fn add_file(&mut self, path: impl Into<PathBuf>, content: String, perms: Permissions) {
        self.entries.push(FsEntry::File(FileToWrite {
            path: path.into(),
            content,
            permissions: perms,
        }));
    }

    pub fn add_directory(&mut self, path: impl Into<PathBuf>, perms: Permissions) {
        self.entries.push(FsEntry::Directory(DirectoryToCreate {
            path: path.into(),
            permissions: perms,
        }));
    }

    pub fn with_file(
        mut self,
        path: impl Into<PathBuf>,
        content: String,
        perms: Permissions,
    ) -> Self {
        self.add_file(path, content, perms);
        self
    }

    pub fn with_directory(mut self, path: impl Into<PathBuf>, perms: Permissions) -> Self {
        self.add_directory(path, perms);
        self
    }

    pub fn validate(&self) -> Result<(), DomainError> {
        if self.entries.is_empty() {
            return Err(DomainError::InvalidTemplate(
                "Project structure is empty".into(),
            ));
        }

        let mut seen = HashSet::new();
        for entry in &self.entries {
            let path = match entry {
                FsEntry::File(f) => &f.path,
                FsEntry::Directory(d) => &d.path,
            };

            let path_str = path.display().to_string();
            if !seen.insert(path_str.clone()) {
                return Err(DomainError::DuplicatePath { path: path_str });
            }

            if path.is_absolute() {
                return Err(DomainError::AbsolutePathNotAllowed {
                    path: path.display().to_string(),
                });
            }
        }

        Ok(())
    }

    pub fn files(&self) -> impl Iterator<Item = &FileToWrite> {
        self.entries.iter().filter_map(|e| match e {
            FsEntry::File(f) => Some(f),
            _ => None,
        })
    }

    pub fn directories(&self) -> impl Iterator<Item = &DirectoryToCreate> {
        self.entries.iter().filter_map(|e| match e {
            FsEntry::Directory(d) => Some(d),
            _ => None,
        })
    }

    pub fn entry_count(&self) -> usize {
        self.entries.len()
    }
}

#[derive(Debug, Clone)]
pub enum FsEntry {
    File(FileToWrite),
    Directory(DirectoryToCreate),
}

#[derive(Debug, Clone)]
pub struct FileToWrite {
    pub path: PathBuf,
    pub content: String,
    pub permissions: Permissions,
}

impl FileToWrite {
    pub fn is_empty(&self) -> bool {
        self.content.is_empty()
    }

    pub fn size(&self) -> usize {
        self.content.len()
    }
}

#[derive(Debug, Clone)]
pub struct DirectoryToCreate {
    pub path: PathBuf,
    pub permissions: Permissions,
}
