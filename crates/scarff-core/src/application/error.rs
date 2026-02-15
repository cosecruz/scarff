//! Application layer errors.
//!
//! These errors represent failures in orchestration, not business logic.
//! Business logic errors are `DomainError` from `crate::domain`.

use std::path::PathBuf;
use thiserror::Error;

use crate::error::ErrorCategory;

/// Errors that occur during application orchestration.
#[derive(Debug, Error, Clone)]
pub enum ApplicationError {
    /// Template resolution failed (no match or ambiguous).
    #[error("Template resolution failed: {reason}")]
    TemplateResolution { reason: String },

    /// Template rendering failed.
    #[error("Template rendering failed: {reason}")]
    RenderingFailed { reason: String },

    /// Filesystem operation failed.
    #[error("Filesystem error at {path}: {reason}")]
    FilesystemError { path: PathBuf, reason: String },

    /// Store access failed (lock poisoned, etc.).
    #[error("Template store error")]
    StoreLockError,

    /// Port/Adapter not configured.
    #[error("Required adapter not configured: {name}")]
    AdapterNotConfigured { name: &'static str },

    /// Validation failed (application-level, not domain).
    #[error("Validation failed: {0}")]
    ValidationFailed(String),

    /// Project already exists at target location.
    #[error("Project already exists at {path}")]
    ProjectExists { path: PathBuf },

    /// Rollback failed (best-effort cleanup failed).
    #[error("Rollback failed for {path}: {reason}")]
    RollbackFailed { path: PathBuf, reason: String },
}

impl ApplicationError {
    /// Get user-actionable suggestions.
    pub fn suggestions(&self) -> Vec<String> {
        match self {
            Self::TemplateResolution { reason } => vec![
                format!("Resolution failed: {}", reason),
                "Try: scarff list-templates to see available templates".into(),
                "Or specify a template explicitly with --template".into(),
            ],
            Self::FilesystemError { path, .. } => vec![
                format!("Failed to access: {}", path.display()),
                "Check that you have write permissions".into(),
                "Ensure the parent directory exists".into(),
            ],
            Self::StoreLockError => vec![
                "The template store is locked".into(),
                "Try again in a moment".into(),
            ],
            Self::AdapterNotConfigured { name } => vec![
                format!("Required component not configured: {}", name),
                "This is likely a configuration error".into(),
            ],
            Self::ProjectExists { path } => vec![
                format!("Directory already exists: {}", path.display()),
                "Use --force to overwrite (destructive)".into(),
                "Choose a different project name".into(),
            ],
            _ => vec!["Check the error details above".into()],
        }
    }

    /// Get error category.
    pub fn category(&self) -> ErrorCategory {
        match self {
            Self::TemplateResolution { .. } => ErrorCategory::NotFound,
            Self::FilesystemError { .. } | Self::RollbackFailed { .. } => ErrorCategory::Internal,
            Self::StoreLockError => ErrorCategory::Internal,
            Self::AdapterNotConfigured { .. } => ErrorCategory::Configuration,
            Self::ValidationFailed(_) => ErrorCategory::Validation,
            Self::ProjectExists { .. } => ErrorCategory::Validation,
            Self::RenderingFailed { .. } => ErrorCategory::Internal,
        }
    }
}
