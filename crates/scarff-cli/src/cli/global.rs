//! Global arguments that apply to every subcommand.
//!
//! Declared here and flattened into [`super::Cli`] so that `-v`, `-q`, etc.
//! are available on any invocation without repetition

use clap::Args;
use std::path::PathBuf;

/// Global arguments for all commands.
#[derive(Debug, Args)]
pub struct GlobalArgs {
    /// Enable verbose output.
    ///
    /// Use -v for info level, -vv for debug level, -vvv for trace level.
    /// Increase logging verbosity.
    ///
    /// Pass once for INFO (`-v`), twice for DEBUG (`-vv`), three times for
    /// TRACE (`-vvv`).  Conflicts with `--quiet`.
    #[arg(
        short = 'v',
        long = "verbose",
        action = clap::ArgAction::Count,
        global = true,
        help = "Increase verbosity (-v, -vv, -vvv)",
        long_help = "Increase logging verbosity:
    (none)  - Only errors
    -v      - Info level (progress messages)
    -vv     - Debug level (detailed diagnostics)
    -vvv    - Trace level (very verbose)"
    )]
    pub verbose: u8,

    /// Suppress all non-error output.
    #[arg(
        short = 'q',
        long = "quiet",
        global = true,
        conflicts_with = "verbose",
        help = "Suppress non-error output"
    )]
    pub quiet: bool,

    /// Disable ANSI colour codes.
    ///
    /// Automatically honoured when `NO_COLOR` is set in the environment
    /// (see <https://no-color.org>).
    #[arg(
        long = "no-color",
        global = true,
        env = "NO_COLOR",
        help = "Disable colored output"
    )]
    pub no_color: bool,

    /// Configuration file path.
    #[arg(
        short = 'c',
        long = "config",
        global = true,
        value_name = "FILE",
        help = "Configuration file path"
    )]
    pub config: Option<PathBuf>,

    /// Output format.
    /// Machine-readable output format.
    #[arg(
        long = "output-format",
        global = true,
        value_enum,
        default_value = "auto",
        help = "Output format"
    )]
    pub output_format: OutputFormat,
}

/// How the CLI should render its output.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, clap::ValueEnum)]
pub enum OutputFormat {
    /// Auto-detect based on terminal.
    #[default]
    Auto,
    /// Human-readable with colors.
    Human,
    /// Plain text without colors.
    Plain,
    /// JSON output.
    Json,
}
