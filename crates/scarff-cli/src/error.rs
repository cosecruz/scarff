//! Comprehensive error handling for Scarff CLI.
//!
//! Provides structured errors with:
//! - User-friendly messages
//! - Actionable suggestions
//! - Proper error chaining
//! - Exit code mapping

use std::path::PathBuf;
use std::{error::Error, fmt};

use anyhow::Context;
use owo_colors::OwoColorize;
use thiserror::Error;
use tracing::error;

use scarff_core::error::ScarffError;

// Re-export so callers only need `use crate::error::*`.
pub use scarff_core::error::ErrorCategory as CoreCategory;

/// Result type alias for CLI operations.
pub type CliResult<T> = Result<T, CliError>;

/// Comprehensive CLI error types.
#[derive(Debug, Error)]
pub enum CliError {
    /// Invalid user input (validation failed).
    #[error("Invalid input: {message}")]
    InvalidInput {
        message: String,
        #[source]
        source: Option<Box<dyn std::error::Error + Send + Sync>>,
    },

    /// Project already exists.
    #[error("Project already exists at {path}")]
    ProjectExists { path: PathBuf },

    /// Project name validation failed.
    #[error("Invalid project name '{name}': {reason}")]
    InvalidProjectName { name: String, reason: String },

    /// Unsupported language specified.
    /// The user specified a language Scarff does not support.
    ///
    /// This is surfaced as a `CliError` rather than a core error because
    /// language selection is validated at the CLI layer before the domain
    /// is ever reached (the `Language` enum in clap means this variant is
    /// currently unreachable via normal clap parsing — it exists for
    /// programmatic construction and future extensibility).
    #[error("Unsupported language '{language}'")]
    UnsupportedLanguage { language: String },

    /// Unsupported project type.
    #[error("Unsupported project type '{kind}'")]
    UnsupportedProjectKind { kind: String },

    /// Unsupported architecture.
    #[error("Unsupported architecture '{architecture}'")]
    UnsupportedArchitecture { architecture: String },

    /// Framework not available for language.
    #[error("framework '{framework}' is not available for {language}")]
    FrameworkNotAvailable {
        framework: String,
        language: String,
        // NEW: populated from FRAMEWORK_REGISTRY in parse_framework()
        available: Vec<&'static str>,
    },

    // ── Config errors ──────────────────────────────────────────────────────
    /// A configuration file could not be read, parsed, or written.
    #[error("Configuration error: {message}")]
    ConfigError {
        message: String,
        #[source]
        source: Option<Box<dyn std::error::Error + Send + Sync>>,
    },

    // ── Core errors ────────────────────────────────────────────────────────
    /// An error propagated from `scarff-core`.
    ///
    /// Wrapped here so that the CLI can attach suggestions drawn from the
    /// core error's category without touching core internals.
    #[error("Scaffolding failed: {0}")]
    Core(#[from] ScarffError),

    // ── System errors ──────────────────────────────────────────────────────
    /// An I/O operation failed.
    #[error("I/O error: {message}")]
    IoError {
        message: String,
        #[source]
        source: std::io::Error,
    },

    /// Operation cancelled by user.
    #[error("Operation cancelled")]
    Cancelled,

    /// Template not found.
    #[error("Template not found: {id}")]
    TemplateNotFound { id: String },

    /// Multiple templates matched (ambiguous).
    #[error("Multiple templates match: {matches}")]
    AmbiguousTemplate { matches: String },

    /// Feature not available (e.g., interactive mode without feature flag).
    #[error("Feature not available: {feature}")]
    FeatureNotAvailable { feature: &'static str },

    /// External command failed.
    #[error("External command failed: {command}")]
    ExternalCommandFailed {
        command: String,
        #[source]
        source: Option<Box<dyn std::error::Error + Send + Sync>>,
    },
}

impl From<std::io::Error> for CliError {
    fn from(err: std::io::Error) -> Self {
        CliError::IoError {
            message: err.to_string(),
            source: err,
        }
    }
}

impl CliError {
    /// Get user-actionable suggestions for fixing this error.
    pub fn suggestions(&self) -> Vec<String> {
        match self {
            Self::InvalidInput { message, .. } => vec![
                format!("Check your input: {}", message),
                "Use --help for usage information".into(),
            ],

            Self::ProjectExists { path } => vec![
                format!("The directory '{}' already exists", path.display()),
                "Use --force to overwrite (destructive)".into(),
                "Choose a different project name".into(),
                format!("Remove the existing directory: rm -rf {}", path.display()),
            ],

            Self::InvalidProjectName { name, reason } => vec![
                format!("Project name '{}' is invalid: {}", name, reason),
                "Use alphanumeric characters, hyphens, and underscores".into(),
                "Start with a letter or number".into(),
                "Examples: my-project, my_app, project123".into(),
            ],

            Self::UnsupportedLanguage { language } => vec![
                format!("'{}' is not a supported language", language),
                "Supported languages:".into(),
                "  • rust     - Rust programming language".into(),
                "  • python   - Python programming language".into(),
                "  • typescript (or ts) - TypeScript/JavaScript".into(),
                "Example: scarff new my-app --lang rust".into(),
            ],

            Self::UnsupportedProjectKind { kind } => vec![
                format!("'{}' is not a supported project type", kind),
                "Supported types:".into(),
                "  • cli        - Command-line application".into(),
                "  • backend    - Web backend/API service".into(),
                "  • frontend   - Web frontend application".into(),
                "  • fullstack  - Full-stack application".into(),
                "  • worker     - Background worker/job processor".into(),
            ],

            Self::UnsupportedArchitecture { architecture } => vec![
                format!("'{}' is not a supported architecture", architecture),
                "Supported architectures:".into(),
                "  • layered    - Traditional layered architecture".into(),
                "  • clean      - Clean/Hexagonal architecture".into(),
                "  • onion      - Onion architecture".into(),
                "  • modular    - Feature-based modular architecture".into(),
            ],

            Self::FrameworkNotAvailable {
                framework,
                language,
                available,
            } => {
                let available = match language.as_str() {
                    "rust" => vec!["axum", "actix-web", "rocket"],
                    "python" => vec!["fastapi", "django", "flask"],
                    "typescript" => vec!["express", "nestjs", "nextjs", "react", "vue"],
                    _ => vec![],
                };

                let mut suggestions = vec![
                    format!("'{}' is not available for {}", framework, language),
                    format!("Available frameworks for {}:", language),
                ];
                for fw in &available {
                    suggestions.push(format!("  • {}", fw));
                }
                suggestions.push(format!(
                    "Example: scarff new my-app --lang {} --framework {}",
                    language,
                    available.first().unwrap_or(&"axum")
                ));
                suggestions
            }

            Self::ConfigError { message, .. } => vec![
                format!("Configuration issue: {}", message),
                "Check your config file at ~/.config/scarff/config.toml".into(),
                "Use 'scarff config --init' to create a default config".into(),
            ],

            Self::Core(core_err) => core_err.suggestions(),

            Self::IoError { message, .. } => vec![
                format!("I/O operation failed: {}", message),
                "Check file permissions".into(),
                "Ensure the parent directory exists".into(),
                "Check available disk space".into(),
            ],

            Self::Cancelled => vec![
                "Operation was cancelled".into(),
                "No changes were made".into(),
            ],

            Self::TemplateNotFound { id } => vec![
                format!("No template found with ID: {}", id),
                "List available templates: scarff list".into(),
                "Use 'scarff list --all' to see all templates".into(),
            ],

            Self::AmbiguousTemplate { matches } => vec![
                format!("Multiple templates match your criteria: {}", matches),
                "Be more specific with --lang, --type, or --framework".into(),
                "Use 'scarff list' to see available templates".into(),
            ],

            Self::FeatureNotAvailable { feature } => vec![
                format!("The '{}' feature is not available in this build", feature),
                "Install with the feature enabled: cargo install scarff-cli --features {}".into(),
            ],

            Self::ExternalCommandFailed { command, .. } => vec![
                format!("External command failed: {}", command),
                "Ensure the command is installed and in your PATH".into(),
                "Check the command output above for details".into(),
            ],
        }
    }

    /// Get the error category for styling and exit codes.
    pub fn category(&self) -> ErrorCategory {
        match self {
            Self::InvalidInput { .. } => ErrorCategory::UserError,
            Self::ProjectExists { .. } => ErrorCategory::UserError,
            Self::InvalidProjectName { .. } => ErrorCategory::UserError,
            Self::UnsupportedLanguage { .. } => ErrorCategory::UserError,
            Self::UnsupportedProjectKind { .. } => ErrorCategory::UserError,
            Self::UnsupportedArchitecture { .. } => ErrorCategory::UserError,
            Self::FrameworkNotAvailable { .. } => ErrorCategory::UserError,
            Self::ConfigError { .. } => ErrorCategory::Configuration,
            Self::Core(core) => match core.category() {
                CoreCategory::Validation => ErrorCategory::UserError,
                CoreCategory::Compatibility => ErrorCategory::UserError,
                CoreCategory::NotFound => ErrorCategory::NotFound,
                CoreCategory::Internal => ErrorCategory::Internal,
                CoreCategory::Configuration => todo!(),
            },
            Self::IoError { .. } => ErrorCategory::Internal,
            Self::Cancelled => ErrorCategory::UserError,
            Self::TemplateNotFound { .. } => ErrorCategory::NotFound,
            Self::AmbiguousTemplate { .. } => ErrorCategory::UserError,
            Self::FeatureNotAvailable { .. } => ErrorCategory::Configuration,
            Self::ExternalCommandFailed { .. } => ErrorCategory::Internal,
        }
    }

    /// Get the appropriate exit code for this error.
    /// Exit code to pass to the OS.
    ///
    /// | Category      | Code |
    /// |---------------|------|
    /// | User error    |  2   |
    /// | Not found     |  3   |
    /// | Configuration |  4   |
    /// | Internal      |  1   |
    pub fn exit_code(&self) -> u8 {
        match self.category() {
            ErrorCategory::UserError => 2,
            ErrorCategory::NotFound => 3,
            ErrorCategory::Configuration => 4,
            ErrorCategory::Internal => 1,
        }
    }

    /// Format the error for display with colors and suggestions.
    pub fn format_colored(&self, verbose: bool) -> String {
        let mut output = String::new();

        // Error header
        output.push_str(&format!(
            "\n{} {}\n\n",
            "✗".red().bold(),
            "Error:".red().bold()
        ));

        // Main error message
        output.push_str(&format!("  {}\n", self.to_string().red()));

        // Error chain (if verbose)
        if verbose {
            let mut source = self.source();
            while let Some(err) = source {
                output.push_str(&format!(
                    "\n  {} {}\n",
                    "→".dimmed(),
                    err.to_string().dimmed()
                ));
                source = err.source();
            }
        }

        // Suggestions
        let suggestions = self.suggestions();
        if !suggestions.is_empty() {
            output.push_str(&format!("\n{}\n", "Suggestions:".yellow().bold()));
            for suggestion in suggestions {
                output.push_str(&format!("  {}\n", suggestion));
            }
        }

        // Hint to re-run with -v
        if !verbose {
            output.push('\n');
            output.push_str(&format!(
                "{} {}\n",
                "\u{2139}".blue(), // ℹ
                "Use -v / --verbose for more details.".dimmed(),
            ));
        }

        output
    }

    /// Plain-text version of [`Self::format_colored`] — no ANSI codes.
    pub fn format_plain(&self, verbose: bool) -> String {
        let mut out = String::new();
        out.push_str(&format!("\nError: {}\n", self));

        if verbose {
            let mut src = std::error::Error::source(self);
            while let Some(err) = src {
                out.push_str(&format!("  Caused by: {err}\n"));
                src = err.source();
            }
        }

        let suggestions = self.suggestions();
        if !suggestions.is_empty() {
            out.push_str("\nSuggestions:\n");
            for s in &suggestions {
                out.push_str(&format!("  {s}\n"));
            }
        }

        if !verbose {
            out.push_str("\nUse -v / --verbose for more details.\n");
        }

        out
    }

    /// Log the error using tracing.
    pub fn log(&self) {
        match self.category() {
            ErrorCategory::UserError => tracing::warn!("User error: {}", self),
            ErrorCategory::NotFound => tracing::warn!("Not found: {}", self),
            ErrorCategory::Configuration => tracing::error!("Configuration error: {}", self),
            ErrorCategory::Internal => tracing::error!("Internal error: {}", self),
        }

        if let Some(source) = self.source() {
            tracing::debug!("Caused by: {}", source);
        }
    }
}

/// Error categories for classification.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ErrorCategory {
    /// User input error (validation, invalid arguments).
    UserError,
    /// Resource not found.
    NotFound,
    /// Configuration error.
    Configuration,
    /// Internal/system error.
    Internal,
}

// ── IntoCli trait ─────────────────────────────────────────────────────────────

/// Extension trait to convert foreign error types into [`CliError`] at
/// call-sites with a descriptive context message.
///
/// Two concrete impls are provided:
/// - `Result<T, std::io::Error>` → `CliError::IoError`
/// - `Result<T, ScarffError>`    → `CliError::Core`
///
/// There is deliberately **no blanket impl** — it would conflict with both
/// concrete impls (rustc rejects overlapping trait implementations).
pub trait IntoCli<T> {
    /// Convert to CliResult with context.
    /// Convert to `CliResult` attaching a human-readable context message.
    fn with_cli_context<F, S>(self, f: F) -> CliResult<T>
    where
        F: FnOnce() -> S,
        S: Into<String>;
}

impl<T> IntoCli<T> for Result<T, std::io::Error> {
    fn with_cli_context<F, S>(self, f: F) -> CliResult<T>
    where
        F: FnOnce() -> S,
        S: Into<String>,
    {
        self.map_err(|e| CliError::IoError {
            message: f().into(),
            source: e,
        })
    }
}

impl<T> IntoCli<T> for Result<T, ScarffError> {
    /// The context message is ignored for core errors because the core error
    /// already carries sufficient context.  The method exists only to satisfy
    /// the trait contract at mixed call-sites.
    fn with_cli_context<F, S>(self, _f: F) -> CliResult<T>
    where
        F: FnOnce() -> S,
        S: Into<String>,
    {
        self.map_err(CliError::Core)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io;

    // ── suggestions ───────────────────────────────────────────────────────

    #[test]
    fn unsupported_language_suggestions_mention_rust() {
        let err = CliError::UnsupportedLanguage {
            language: "java".into(),
        };
        assert!(err.suggestions().iter().any(|s| s.contains("rust")));
    }

    #[test]
    fn framework_mismatch_lists_available() {
        let err = CliError::FrameworkNotAvailable {
            framework: "django".into(),
            language: "rust".into(),
            available: todo!(),
        };
        let suggestions = err.suggestions();
        assert!(suggestions.iter().any(|s| s.contains("axum")));
        assert!(suggestions.iter().any(|s| s.contains("actix")));
    }

    #[test]
    fn project_exists_suggests_force() {
        let err = CliError::ProjectExists {
            path: PathBuf::from("/tmp/test"),
        };
        assert!(err.suggestions().iter().any(|s| s.contains("--force")));
    }

    #[test]
    fn invalid_name_suggestions_non_empty() {
        let err = CliError::InvalidProjectName {
            name: ".hidden".into(),
            reason: "starts with '.'".into(),
        };
        assert!(!err.suggestions().is_empty());
    }

    // ── exit codes ────────────────────────────────────────────────────────

    #[test]
    fn exit_code_user_error() {
        assert_eq!(
            CliError::InvalidInput {
                message: "x".into(),
                source: None
            }
            .exit_code(),
            2
        );
    }

    #[test]
    fn exit_code_not_found() {
        assert_eq!(CliError::TemplateNotFound { id: "x".into() }.exit_code(), 3);
    }

    #[test]
    fn exit_code_configuration() {
        assert_eq!(
            CliError::ConfigError {
                message: "x".into(),
                source: None
            }
            .exit_code(),
            4
        );
    }

    #[test]
    fn exit_code_internal() {
        assert_eq!(
            CliError::IoError {
                message: "x".into(),
                source: io::Error::new(io::ErrorKind::Other, "e"),
            }
            .exit_code(),
            1
        );
    }

    // ── format ────────────────────────────────────────────────────────────

    #[test]
    fn format_plain_contains_error_header() {
        let err = CliError::ProjectExists {
            path: PathBuf::from("/tmp/x"),
        };
        let s = err.format_plain(false);
        assert!(s.contains("Error:"));
        assert!(s.contains("Suggestions:"));
    }

    #[test]
    fn format_plain_verbose_omits_hint() {
        let err = CliError::Cancelled;
        let s = err.format_plain(true);
        assert!(!s.contains("--verbose"));
    }

    // ── IntoCli ───────────────────────────────────────────────────────────

    #[test]
    fn into_cli_io_error() {
        let result: Result<(), io::Error> = Err(io::Error::new(io::ErrorKind::NotFound, "missing"));
        let cli: CliResult<()> = result.with_cli_context(|| "reading config");
        assert!(matches!(cli, Err(CliError::IoError { .. })));
    }
}
