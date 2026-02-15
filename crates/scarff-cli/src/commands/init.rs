//! `scarff init` — create a default configuration file.

use crate::{
    cli::{GlobalArgs, InitArgs},
    config::AppConfig,
    error::{CliError, CliResult},
    output::OutputManager,
};

/// Create a default Scarff configuration file.
pub fn execute(
    args: InitArgs,
    _global: GlobalArgs,
    _config: AppConfig,
    output: OutputManager,
) -> CliResult<()> {
    output.info("Initialising configuration...")?;

    let config_path = AppConfig::config_path();

    // Bail early if the file already exists and --force was not given.
    if config_path.exists() && !args.force {
        output.warning(&format!(
            "Config already exists at {}  (use --force to overwrite)",
            config_path.display(),
        ))?;
        return Ok(());
    }

    let default_config = AppConfig::default();
    let toml = toml::to_string_pretty(&default_config).map_err(|e| CliError::ConfigError {
        message: format!("Failed to serialise default config: {e}"),
        source: Some(Box::new(e)),
    })?;

    // Ensure parent directory exists.
    if let Some(parent) = config_path.parent() {
        std::fs::create_dir_all(parent).map_err(|e| CliError::IoError {
            message: format!("Failed to create config directory '{}'", parent.display()),
            source: e,
        })?;
    }

    std::fs::write(&config_path, &toml).map_err(|e| CliError::IoError {
        message: format!("Failed to write config to '{}'", config_path.display()),
        source: e,
    })?;

    output.success(&format!(
        "Configuration created at {}",
        config_path.display(),
    ))?;

    Ok(())
}
