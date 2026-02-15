use std::fmt;
use std::path::{Path, PathBuf};

/// A filesystem path guaranteed to be **relative**.
///
/// This type encodes an important invariant:
/// templates and project structures must never contain absolute paths.
///
/// Why?
/// - Absolute paths break portability
/// - They can overwrite arbitrary locations
/// - They are almost always a bug in scaffolding systems
///
/// `RelativePath` is a *semantic guardrail*, not a filesystem abstraction.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct RelativePath(PathBuf);

impl RelativePath {
    /// Create a new relative path.
    ///
    /// # Panics
    /// Panics if the provided path is absolute.
    pub fn new(path: impl Into<PathBuf>) -> Self {
        let path = path.into();
        assert!(
            !path.is_absolute(),
            "RelativePath cannot be absolute: {path:?}"
        );
        Self(path)
    }

    /// Try to create a relative path.
    ///
    /// This is the non-panicking variant.
    pub fn try_new(path: impl Into<PathBuf>) -> Result<Self, PathBuf> {
        let path = path.into();
        if path.is_absolute() {
            Err(path)
        } else {
            Ok(Self(path))
        }
    }

    /// Join a path segment onto this relative path.
    ///
    /// # Panics
    /// Panics if the joined path is absolute.
    pub fn join(&self, segment: impl AsRef<Path>) -> Self {
        let segment = segment.as_ref();
        assert!(
            !segment.is_absolute(),
            "cannot join absolute path to RelativePath"
        );
        Self(self.0.join(segment))
    }

    /// Borrow as a `Path`.
    pub fn as_path(&self) -> &Path {
        &self.0
    }

    /// Consume into a `PathBuf`.
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
        RelativePath::new(s)
    }
}

impl From<String> for RelativePath {
    fn from(s: String) -> Self {
        RelativePath::new(s)
    }
}

impl fmt::Display for RelativePath {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0.display())
    }
}

/// Simplified permission model for generated artifacts.
///
/// This is a **capability model**, not a Unix permission model.
///
/// It answers:
/// - Can this file be read?
/// - Can it be modified?
/// - Can it be executed or entered?
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Permissions {
    readable: bool,
    writable: bool,
    executable: bool,
}

impl Permissions {
    /// Read-only permissions.
    pub const fn read_only() -> Self {
        Self {
            readable: true,
            writable: false,
            executable: false,
        }
    }

    /// Read and write permissions.
    pub const fn read_write() -> Self {
        Self {
            readable: true,
            writable: true,
            executable: false,
        }
    }

    /// Read and execute permissions.
    pub const fn executable() -> Self {
        Self {
            readable: true,
            writable: false,
            executable: true,
        }
    }

    /// Full permissions.
    pub const fn full() -> Self {
        Self {
            readable: true,
            writable: true,
            executable: true,
        }
    }

    // Getters

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

#[cfg(test)]
mod tests {
    use super::*;

    // ---------------------------------------------------------------------
    // RelativePath
    // ---------------------------------------------------------------------

    #[test]
    fn relative_path_accepts_relative() {
        let p = RelativePath::new("src/main.rs");
        assert_eq!(p.as_path(), Path::new("src/main.rs"));
    }

    #[test]
    #[should_panic]
    fn relative_path_rejects_absolute() {
        RelativePath::new("/etc/passwd");
    }

    #[test]
    fn try_new_rejects_absolute() {
        let result = RelativePath::try_new("/etc/passwd");
        assert!(result.is_err());
    }

    #[test]
    fn join_relative_path() {
        let base = RelativePath::new("src");
        let joined = base.join("main.rs");
        assert_eq!(joined.as_path(), Path::new("src/main.rs"));
    }

    #[test]
    #[should_panic]
    fn join_rejects_absolute_segment() {
        let base = RelativePath::new("src");
        base.join("/etc/passwd");
    }

    // ---------------------------------------------------------------------
    // Permissions
    // ---------------------------------------------------------------------

    #[test]
    fn permissions_defaults() {
        let p = Permissions::default();
        assert!(p.readable());
        assert!(p.writable());
        assert!(!p.executable_flag());
    }

    #[test]
    fn permissions_read_only() {
        let p = Permissions::read_only();
        assert!(p.readable());
        assert!(!p.writable());
        assert!(!p.executable_flag());
    }

    #[test]
    fn permissions_executable() {
        let p = Permissions::executable();
        assert!(p.readable());
        assert!(!p.writable());
        assert!(p.executable_flag());
    }

    #[test]
    fn permissions_full() {
        let p = Permissions::full();
        assert!(p.readable());
        assert!(p.writable());
        assert!(p.executable_flag());
    }
}
