//! `scarff config` — read and write configuration values.

use crate::{
    cli::ConfigCommands,
    config::AppConfig,
    error::{CliError, CliResult},
    output::OutputManager,
};

/// Dispatch to the correct config subcommand.
pub fn execute(cmd: ConfigCommands, config: AppConfig, output: OutputManager) -> CliResult<()> {
    match cmd {
        ConfigCommands::Get { key } => {
            let value = get_config_value(&config, &key)?;
            output.print(&format!("{key} = {value:?}"))?;
        }

        ConfigCommands::Set { key, value } => {
            // Persisting config changes is not yet implemented.
            output.print(&format!("Setting {key} = {value}"))?;
            // TODO: read file, update key, write back.
        }

        ConfigCommands::List => {
            output.header("Current Configuration:")?;
            let serialised =
                toml::to_string_pretty(&config).map_err(|e| CliError::ConfigError {
                    message: format!("Failed to serialise config: {e}"),
                    source: Some(Box::new(e)),
                })?;
            output.print(&serialised)?;
        }

        ConfigCommands::Path => {
            output.print(&AppConfig::config_path().display().to_string())?;
        }
    }

    Ok(())
}

// ── helpers ───────────────────────────────────────────────────────────────────

fn get_config_value(config: &AppConfig, key: &str) -> CliResult<String> {
    match key {
        "defaults.lang" => Ok(config.defaults.language.clone().unwrap_or_default()),
        "defaults.type" => Ok(config.defaults.kind.clone().unwrap_or_default()),
        "defaults.arch" => Ok(config.defaults.architecture.clone().unwrap_or_default()),
        "output.no_color" => Ok(config.output.no_color.to_string()),
        "output.format" => Ok(config.output.format.clone()),
        _ => Err(CliError::ConfigError {
            message: format!("Unknown config key: '{key}'"),
            source: None,
        }),
    }
}

// ── tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::AppConfig;

    #[test]
    fn get_known_key() {
        let cfg = AppConfig::default();
        assert_eq!(get_config_value(&cfg, "defaults.lang").unwrap(), "rust");
    }

    #[test]
    fn get_unknown_key_is_error() {
        let cfg = AppConfig::default();
        assert!(matches!(
            get_config_value(&cfg, "does.not.exist"),
            Err(CliError::ConfigError { .. })
        ));
    }

    #[test]
    fn get_no_color_default() {
        let cfg = AppConfig::default();
        assert_eq!(get_config_value(&cfg, "output.no_color").unwrap(), "false");
    }
}
