//! Unified error handling for Scarff Core.
//!
//! This module provides a unified error type that wraps domain and application
//! errors, with rich context and user-actionable suggestions.

use std::path::PathBuf;
use thiserror::Error;

use crate::application::ApplicationError;
use crate::domain::DomainError;

/// Root error type for Scarff Core operations.
///
/// This enum wraps all possible errors that can occur when using scarff-core,
/// providing a unified interface for error handling.
#[derive(Debug, Error, Clone)]
pub enum ScarffError {
    /// Errors from the domain layer (business logic violations).
    #[error("Domain error: {0}")]
    Domain(#[from] DomainError),

    /// Errors from the application layer (orchestration failures).
    #[error("Application error: {0}")]
    Application(#[from] ApplicationError),

    /// Configuration or setup errors.
    #[error("Configuration error: {message}")]
    Configuration { message: String },

    /// Unexpected internal errors (bugs).
    #[error("Internal error: {message}. This is a bug, please report it.")]
    Internal { message: String },
}

impl ScarffError {
    /// Get user-actionable suggestions for fixing this error.
    pub fn suggestions(&self) -> Vec<String> {
        match self {
            Self::Domain(e) => e.suggestions(),
            Self::Application(e) => e.suggestions(),
            Self::Configuration { message } => vec![
                format!("Configuration issue: {}", message),
                "Check your setup and try again".into(),
            ],
            Self::Internal { .. } => vec![
                "This appears to be a bug in Scarff".into(),
                "Please report this issue at: https://github.com/yourusername/scarff/issues".into(),
            ],
        }
    }

    /// Get error category for display/styling purposes.
    pub fn category(&self) -> ErrorCategory {
        match self {
            Self::Domain(e) => match e.category() {
                crate::domain::ErrorCategory::Validation => ErrorCategory::Validation,
                crate::domain::ErrorCategory::Compatibility => ErrorCategory::Compatibility,
                crate::domain::ErrorCategory::NotFound => ErrorCategory::NotFound,
                crate::domain::ErrorCategory::Internal => ErrorCategory::Internal,
            },
            Self::Application(e) => e.category(),
            Self::Configuration { .. } => ErrorCategory::Configuration,
            Self::Internal { .. } => ErrorCategory::Internal,
        }
    }

    /// Check if this error is retryable.
    pub fn is_retryable(&self) -> bool {
        matches!(
            self,
            Self::Application(ApplicationError::StoreLockError)
                | Self::Domain(DomainError::InferenceFailed { .. })
        )
    }
}

/// Error categories for UI display.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ErrorCategory {
    Validation,
    Compatibility,
    NotFound,
    Configuration,
    Internal,
}

/// Convenient result type alias.
pub type ScarffResult<T> = Result<T, ScarffError>;

/// Extension trait for adding context to errors.
pub trait Context<T> {
    /// Add context to an error.
    fn context(self, msg: impl Into<String>) -> ScarffResult<T>;
}

impl<T, E> Context<T> for Result<T, E>
where
    E: std::error::Error + Send + Sync + 'static,
{
    fn context(self, msg: impl Into<String>) -> ScarffResult<T> {
        self.map_err(|e| ScarffError::Internal {
            message: format!("{}: {}", msg.into(), e),
        })
    }
}
