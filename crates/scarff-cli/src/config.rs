//! Application configuration.
//!
//! [`AppConfig`] is loaded once at startup and passed down by value.  The
//! CLI layer owns config; the core crate never sees it.
//!
//! # Resolution order (highest priority first)
//!
//! 1. CLI flags (handled at the call-site, not here)
//! 2. Environment variables (TODO: implement)
//! 3. Config file (TODO: implement file reading)
//! 4. Built-in defaults (always present)

use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use crate::cli::global::GlobalArgs;

/// Application configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    /// Default values for new projects.
    pub defaults: Defaults,
    /// Output settings.
    pub output: OutputConfig,
    /// Template settings.
    pub templates: TemplateConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Defaults {
    pub language: Option<String>,
    pub kind: Option<String>,
    pub architecture: Option<String>,
    pub framework: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OutputConfig {
    pub no_color: bool,
    pub format: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TemplateConfig {
    pub registry_url: Option<String>,
    pub local_path: Option<PathBuf>,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            defaults: Defaults {
                language: Some("rust".into()),
                kind: Some("cli".into()),
                architecture: Some("layered".into()),
                framework: None,
            },
            output: OutputConfig {
                no_color: false,
                format: "human".into(),
            },
            templates: TemplateConfig {
                registry_url: None,
                local_path: None,
            },
        }
    }
}

impl AppConfig {
    /// Load configuration, starting from defaults.
    ///
    /// The `config_file` parameter is the path the user passed via `--config`
    /// (or `None` to use the default location).  File reading is not yet
    /// implemented; this always returns the built-in defaults.
    pub fn load(config_file: Option<&PathBuf>) -> anyhow::Result<Self> {
        // The `config_file` parameter is intentionally unused until file-
        // reading is implemented.  Prefix with `_` to avoid an unused-variable
        // warning while making the intent clear.
        let _config_file = config_file;
        // TODO: read from TOML file, merge env vars, merge CLI overrides.
        Ok(Self::default())
    }

    /// Path to the default configuration file.
    ///
    /// Uses `directories::ProjectDirs` for cross-platform correctness,
    /// falling back to `.scarff.toml` in the current directory.
    pub fn config_path() -> PathBuf {
        directories::ProjectDirs::from("com", "scarff", "scarff")
            .map(|d| d.config_dir().join("config.toml"))
            .unwrap_or_else(|| PathBuf::from(".scarff.toml"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_language_is_rust() {
        let cfg = AppConfig::default();
        assert_eq!(cfg.defaults.language.as_deref(), Some("rust"));
    }

    #[test]
    fn default_no_color_is_false() {
        assert!(!AppConfig::default().output.no_color);
    }

    #[test]
    fn load_without_file_returns_defaults() {
        let cfg = AppConfig::load(None).unwrap();
        assert_eq!(cfg.defaults.kind.as_deref(), Some("cli"));
    }

    #[test]
    fn config_path_is_absolute_or_relative() {
        // Just assert it doesn't panic and returns a non-empty path.
        let p = AppConfig::config_path();
        assert!(!p.as_os_str().is_empty());
    }
}
