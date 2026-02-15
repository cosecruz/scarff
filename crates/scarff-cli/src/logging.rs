//! Tracing subscriber initialisation.
//!
//! Only the CLI crate is allowed to call [`init_logging`]; `scarff-core`
//! only *emits* spans and events — it never touches subscribers.
//!
//! # Verbosity mapping
//!
//! | Flag(s)  | Filter level |
//! |----------|--------------|
//! | (none)   | WARN         |
//! | `-v`     | INFO         |
//! | `-vv`    | DEBUG        |
//! | `-vvv`   | TRACE        |
//! | `--quiet`| ERROR        |
//!
//! `RUST_LOG` overrides all of the above if set.

use std::io::IsTerminal as _;

use tracing_subscriber::{EnvFilter, layer::SubscriberExt, util::SubscriberInitExt};

use crate::cli::{GlobalArgs, OutputFormat};

/// Initialise the global tracing subscriber.
///
/// Must be called exactly once, before any tracing macros fire.
/// Returns an error if the subscriber could not be registered (e.g. it was
/// already set by a previous call in the same process — harmless in tests if
/// you use `std::sync::Once`).
pub fn init_logging(args: &GlobalArgs) -> anyhow::Result<()> {
    let level = derive_level(args);

    // RUST_LOG wins; otherwise build our own filter string so each crate gets
    // the same level as the top-level filter.
    let filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new(format!("scarff={level},scarff_core={level}",)));

    // Detect colour support via the stdlib (stable since 1.70).
    // `std::io::IsTerminal` supersedes the deprecated `atty` crate.
    let use_ansi = !args.no_color && std::io::stderr().is_terminal();

    let fmt_layer = tracing_subscriber::fmt::layer()
        .with_target(false)
        .with_thread_ids(false)
        .with_file(false)
        .with_line_number(false)
        .with_ansi(use_ansi)
        .with_writer(std::io::stderr);

    // `try_init` returns an error instead of panicking if a subscriber is
    // already set.  In integration tests multiple test binaries may run in the
    // same process; we silently ignore the "already initialised" error.
    tracing_subscriber::registry()
        .with(filter)
        .with(fmt_layer)
        .try_init()
        .map_err(|e| anyhow::anyhow!("Failed to initialise tracing: {e}"))?;

    Ok(())
}

/// Translate the verbosity counter + quiet flag to a level string.
fn derive_level(args: &GlobalArgs) -> &'static str {
    if args.quiet {
        return "error";
    }
    match args.verbose {
        0 => "warn",
        1 => "info",
        2 => "debug",
        _ => "trace",
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cli::GlobalArgs;

    fn args_with(verbose: u8, quiet: bool) -> GlobalArgs {
        GlobalArgs {
            verbose,
            quiet,
            no_color: true,
            config: None,
            output_format: OutputFormat::Auto,
        }
    }

    #[test]
    fn level_quiet() {
        assert_eq!(derive_level(&args_with(0, true)), "error");
    }

    #[test]
    fn level_default() {
        assert_eq!(derive_level(&args_with(0, false)), "warn");
    }

    #[test]
    fn level_verbose_one() {
        assert_eq!(derive_level(&args_with(1, false)), "info");
    }

    #[test]
    fn level_verbose_two() {
        assert_eq!(derive_level(&args_with(2, false)), "debug");
    }

    #[test]
    fn level_verbose_three_plus() {
        assert_eq!(derive_level(&args_with(3, false)), "trace");
        assert_eq!(derive_level(&args_with(10, false)), "trace");
    }

    // quiet takes precedence over verbose
    #[test]
    fn quiet_overrides_verbose() {
        assert_eq!(derive_level(&args_with(3, true)), "error");
    }
}
