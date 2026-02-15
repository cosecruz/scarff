//! # Scarff CLI
//!
//! Production-grade project scaffolding tool.
//!
//! ## Startup sequence
//!
//! 1. Parse CLI arguments (clap handles `--help` / `--version` early-exit).
//! 2. Initialise the tracing subscriber (logging).
//! 3. Load configuration (file + env + defaults).
//! 4. Build the [`OutputManager`].
//! 5. Dispatch to the appropriate command handler.
//! 6. Translate any [`CliError`] into a user-facing message and exit code.
//!
//! ## Exit codes
//!
//! | Code | Meaning                 |
//! |------|-------------------------|
//! |  0   | Success                 |
//! |  1   | Internal / system error |
//! |  2   | User / input error      |
//! |  3   | Resource not found      |
//! |  4   | Configuration error     |

use std::process::ExitCode;

use clap::Parser;
use tracing::{debug, info, instrument};

use crate::{
    cli::{Cli, Commands, GlobalArgs},
    config::AppConfig,
    error::{CliError, CliResult},
    logging::init_logging,
    output::OutputManager,
};

mod cli;
mod commands;
mod config;
mod error;
mod logging;
mod output;

fn main() -> ExitCode {
    // Load .env before anything else — including tracing init.
    // Silently ignored if .env doesn't exist (production deployments
    // use real environment variables, not .env files).
    let _ = dotenvy::dotenv();

    // ── 1. Parse arguments ────────────────────────────────────────────────
    // clap handles --help / --version and exits automatically; errors here
    // are argument-parse failures (exit 2).
    let cli = match Cli::try_parse() {
        Ok(cli) => cli,
        Err(e) => {
            // Render clap's own error (already user-friendly) and exit 2.
            eprintln!("{}", e.render().ansi());
            return ExitCode::from(2);
        }
    };

    // ── 2. Initialise tracing ─────────────────────────────────────────────
    if let Err(e) = init_logging(&cli.global) {
        eprintln!("Failed to initialise logging: {e}");
        return ExitCode::from(1);
    }

    debug!(
        verbose = cli.global.verbose,
        quiet = cli.global.quiet,
        no_color = cli.global.no_color,
        "CLI started"
    );

    // ── 3. Load configuration ─────────────────────────────────────────────
    let config = match AppConfig::load(cli.global.config.as_ref()) {
        Ok(cfg) => cfg,
        Err(e) => {
            tracing::error!("Failed to load configuration: {e:#}");
            return ExitCode::from(4);
        }
    };

    // ── 4. Build output manager ───────────────────────────────────────────
    let output = OutputManager::new(&cli.global, &config);

    // ── 5. Dispatch + 6. Error handling ──────────────────────────────────
    match run(cli, config, output) {
        Ok(()) => {
            info!("Scarff completed successfully");
            ExitCode::SUCCESS
        }
        Err(e) => handle_error(e),
    }
}

/// Dispatch to the correct command handler.
#[instrument(skip_all)]
fn run(cli: Cli, config: AppConfig, output: OutputManager) -> CliResult<()> {
    match cli.command {
        Commands::New(cmd) => commands::new::execute(cmd, cli.global, config, output),
        Commands::List(cmd) => commands::list::execute(cmd, cli.global, output),
        Commands::Init(cmd) => commands::init::execute(cmd, cli.global, config, output),
        Commands::Completions(cmd) => commands::completions::execute(cmd),
        Commands::Config(cmd) => commands::config::execute(cmd, config, output),
    }
}

/// Translate a `CliError` into a user message and an appropriate exit code.
///
/// This is the single place where structured errors become human-readable
/// output and OS exit codes — the format/suggestion machinery in `CliError`
/// is all exercised here.
fn handle_error(err: CliError) -> ExitCode {
    // 1. Emit a structured log event at the right severity.
    err.log();

    // 2. Print a user-friendly message.  We write directly to stderr so the
    //    message appears even when stdout is redirected.
    //
    //    Colour is disabled when stderr is not a TTY (same logic as logging.rs).
    let verbose = false; // TODO: thread global.verbose down into handle_error
    let msg = if std::io::IsTerminal::is_terminal(&std::io::stderr()) {
        err.format_colored(verbose)
    } else {
        err.format_plain(verbose)
    };
    eprint!("{msg}");

    ExitCode::from(err.exit_code())
}

// ── tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use clap::CommandFactory;

    #[test]
    fn cli_structure_is_valid() {
        // Clap's internal consistency check — catches missing values, conflicts, etc.
        Cli::command().debug_assert();
    }

    #[test]
    fn cli_version_matches_cargo() {
        let cmd = Cli::command();
        assert_eq!(cmd.get_version(), Some(env!("CARGO_PKG_VERSION")));
    }

    #[test]
    fn cli_has_author() {
        let cmd = Cli::command();
        assert!(cmd.get_author().is_some());
    }
}
