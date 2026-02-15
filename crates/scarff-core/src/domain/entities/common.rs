use super::DomainError;
use std::fmt;
use std::path::{Path, PathBuf};

/// A filesystem path guaranteed to be relative.
///
/// Invariant: Never absolute. Enforced at construction.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct RelativePath(PathBuf);

impl RelativePath {
    /// Create a new relative path.
    ///
    /// # Panics
    /// Panics if path is absolute (use `try_new` for fallible).
    pub fn new(path: impl Into<PathBuf>) -> Self {
        let path = path.into();
        assert!(
            !path.is_absolute(),
            "RelativePath cannot be absolute: {:?}",
            path
        );
        Self(path)
    }

    /// Fallible constructor.
    pub fn try_new(path: impl Into<PathBuf>) -> Result<Self, DomainError> {
        let path = path.into();
        if path.is_absolute() {
            Err(DomainError::AbsolutePathNotAllowed {
                path: path.display().to_string(),
            })
        } else {
            Ok(Self(path))
        }
    }

    /// Join a segment, maintaining relative invariant.
    pub fn join(&self, segment: impl AsRef<Path>) -> Result<Self, DomainError> {
        let segment = segment.as_ref();
        if segment.is_absolute() {
            return Err(DomainError::AbsolutePathNotAllowed {
                path: segment.display().to_string(),
            });
        }
        Ok(Self(self.0.join(segment)))
    }

    pub fn as_path(&self) -> &Path {
        &self.0
    }

    pub fn as_str(&self) -> &str {
        self.0.to_str().unwrap_or("")
    }

    pub fn into_path_buf(self) -> PathBuf {
        self.0
    }
}

impl AsRef<Path> for RelativePath {
    fn as_ref(&self) -> &Path {
        &self.0
    }
}

impl From<&str> for RelativePath {
    fn from(s: &str) -> Self {
        Self::new(s)
    }
}

impl fmt::Display for RelativePath {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0.display())
    }
}

/// Capability-based permissions model.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Permissions {
    readable: bool,
    writable: bool,
    executable: bool,
}

impl Permissions {
    pub const fn read_only() -> Self {
        Self {
            readable: true,
            writable: false,
            executable: false,
        }
    }

    pub const fn read_write() -> Self {
        Self {
            readable: true,
            writable: true,
            executable: false,
        }
    }

    pub const fn executable() -> Self {
        Self {
            readable: true,
            writable: false,
            executable: true,
        }
    }

    pub const fn full() -> Self {
        Self {
            readable: true,
            writable: true,
            executable: true,
        }
    }

    pub const fn readable(&self) -> bool {
        self.readable
    }
    pub const fn writable(&self) -> bool {
        self.writable
    }
    pub const fn executable_flag(&self) -> bool {
        self.executable
    }
}

impl Default for Permissions {
    fn default() -> Self {
        Self::read_write()
    }
}
