// ============================================================================
// domain/errors.rs - COMPREHENSIVE ERROR DOMAIN
// ============================================================================

use std::path::PathBuf;
use thiserror::Error;

/// Root domain error type.
///
/// All errors are:
/// - Cloneable (for retry logic)
/// - Categorizable (for CLI display)
/// - Actionable (provides suggestions)
#[derive(Debug, Error, Clone, PartialEq)]
pub enum DomainError {
    // ========================================================================
    // Validation Errors (400-level equivalent)
    // ========================================================================
    #[error("Invalid target configuration: {0}")]
    InvalidTarget(String),

    #[error("Invalid template: {0}")]
    InvalidTemplate(String),

    #[error("Template '{template_id}' has no content")]
    EmptyTemplate { template_id: String },

    #[error("Duplicate path in template: {path}")]
    DuplicatePath { path: String },

    #[error("Absolute paths not allowed: {path}")]
    AbsolutePathNotAllowed { path: String },

    // ========================================================================
    // Compatibility Errors (409-level equivalent)
    // ========================================================================
    #[error("language '{language}' does not support kind '{kind}': {reason}")]
    IncompatibleLanguageKind {
        language: String,
        kind: String,
        reason: String, // populated from capabilities::validate_language_kind
    },

    #[error("framework '{framework}' incompatible with '{context}': {reason}")]
    IncompatibleFramework {
        framework: String,
        context: String,
        reason: String, // populated from capabilities::validate_framework_compatibility
    },

    #[error("architecture '{architecture}' invalid: {reason}")]
    InvalidArchitecture {
        architecture: String,
        reason: String,
    },

    // ========================================================================
    // Not Found Errors (404-level equivalent)
    // ========================================================================
    #[error("No template matches target: {0}")]
    NoMatchingTemplate(String),

    #[error("Ambiguous template match: {0}")]
    AmbiguousTemplateMatch(String),

    // ========================================================================
    // Constraint Violations
    // ========================================================================
    #[error("Required field missing: {field}")]
    MissingRequiredField { field: &'static str },

    #[error("Inference failed for '{field}': {reason}")]
    InferenceFailed { field: String, reason: String },
}

impl DomainError {
    /// Get user-actionable suggestions for fixing this error.
    pub fn suggestions(&self) -> Vec<String> {
        match self {
            Self::InvalidTarget(msg) => vec![
                "Check your target configuration".into(),
                format!("Details: {}", msg),
            ],
            Self::IncompatibleLanguageKind { language, kind, reason } => vec![
                format!("{} projects typically use:", kind),
                match kind.as_str() {
                    "cli" => "  • Rust, Python, Go".into(),
                    "web-backend" => "  • Rust (Axum/Actix), Python (FastAPI/Django), TypeScript (Express/NestJS)".into(),
                    "web-frontend" => "  • TypeScript (React/Vue)".into(),
                    _ => "  • Check documentation for supported combinations".into(),
                },
            ],
            Self::NoMatchingTemplate(target) => vec![
                "No template found for your target configuration".into(),
                format!("Target: {}", target),
                "Try: scarff list-templates".into(),
            ],
            Self::EmptyTemplate { template_id } => vec![
                format!("Template '{}' is corrupted", template_id),
                "Please report this issue or use a different template".into(),
            ],
            _ => vec!["See documentation for more details".into()],
        }
    }

    /// Error category for CLI display styling.
    pub fn category(&self) -> ErrorCategory {
        match self {
            Self::InvalidTarget(_) | Self::InvalidTemplate(_) => ErrorCategory::Validation,
            Self::IncompatibleLanguageKind { .. } | Self::IncompatibleFramework { .. } => {
                ErrorCategory::Compatibility
            }
            Self::NoMatchingTemplate(_) | Self::AmbiguousTemplateMatch(_) => {
                ErrorCategory::NotFound
            }
            _ => ErrorCategory::Internal,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ErrorCategory {
    Validation,
    Compatibility,
    NotFound,
    Internal,
}
