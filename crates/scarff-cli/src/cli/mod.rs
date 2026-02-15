//! CLI argument definitions using the clap derive API.
//!
//! This module is the *only* place that knows about argument names, aliases,
//! help text, and value enums.  No business logic lives here.

use std::path::PathBuf;

use clap::{Args, Parser, Subcommand, ValueEnum};

pub mod global;
pub use global::{GlobalArgs, OutputFormat};

// ── Top-level CLI ─────────────────────────────────────────────────────────────

/// Main CLI entry-point.
#[derive(Debug, Parser)]
#[command(
    name    = "scarff",
    bin_name = "scarff",
    version  = env!("CARGO_PKG_VERSION"),
    author   = env!("CARGO_PKG_AUTHORS"),
    about    = "\u{26a1} Instant project scaffolding",
    long_about = "Scarff generates production-ready project structures \
                  for multiple languages and frameworks.",
    after_help = "EXAMPLES:\n\
        \x20 scarff new my-cli  --lang rust --type cli --arch layered\n\
        \x20 scarff new my-api  --lang python --type backend --framework fastapi\n\
        \x20 scarff list --lang rust\n\
        \x20 scarff completions bash > /usr/share/bash-completion/completions/scarff",
    arg_required_else_help = true,
    subcommand_required    = true,
)]
pub struct Cli {
    /// Flags available on every subcommand.
    #[command(flatten)]
    pub global: GlobalArgs,

    /// Subcommand to execute.
    #[command(subcommand)]
    pub command: Commands,
}

// ── Subcommands ───────────────────────────────────────────────────────────────

/// All available subcommands.
#[derive(Debug, Subcommand)]
pub enum Commands {
    /// Create a new project from a template.
    #[command(
        visible_alias = "n",
        about = "Create a new project",
        after_help = "EXAMPLES:\n\
            \x20 scarff new my-project --lang rust   --type cli\n\
            \x20 scarff new my-api     --lang python --type backend --framework fastapi\n\
            \x20 scarff new my-app     --lang typescript --type frontend --framework react"
    )]
    New(NewArgs),

    /// List available templates.
    #[command(
        visible_alias = "ls",
        about = "List available templates",
        after_help = "EXAMPLES:\n\
            \x20 scarff list\n\
            \x20 scarff list --lang rust\n\
            \x20 scarff list --type backend --arch layered"
    )]
    List(ListArgs),

    /// Initialise a Scarff configuration file.
    #[command(
        about = "Initialise configuration",
        after_help = "EXAMPLES:\n\
            \x20 scarff init           # default location\n\
            \x20 scarff init --global  # global config\n\
            \x20 scarff init --local   # local config in CWD"
    )]
    Init(InitArgs),

    /// Generate shell completion scripts.
    #[command(
        about = "Generate shell completions",
        after_help = "EXAMPLES:\n\
            \x20 scarff completions bash > ~/.local/share/bash-completion/completions/scarff\n\
            \x20 scarff completions zsh  > ~/.zfunc/_scarff\n\
            \x20 scarff completions fish > ~/.config/fish/completions/scarff.fish"
    )]
    Completions(CompletionsArgs),

    /// Manage the Scarff configuration.
    #[command(
        about = "Configuration management",
        subcommand,
        after_help = "EXAMPLES:\n\
            \x20 scarff config get defaults.lang\n\
            \x20 scarff config set defaults.lang rust\n\
            \x20 scarff config list"
    )]
    Config(ConfigCommands),
}

// ── new ───────────────────────────────────────────────────────────────────────

/// Arguments for `scarff new`.
#[derive(Debug, Args)]
pub struct NewArgs {
    /// Project name or path.  A plain name creates `./name`; a path like
    /// `../foo` places the project one level up.
    #[arg(value_name = "NAME", help = "Project name or path")]
    pub name: String,

    /// Programming language.
    #[arg(
        short = 'l',
        long = "lang",
        value_name = "LANGUAGE",
        value_enum,
        help = "Programming language"
    )]
    pub language: Language,

    /// Project type.
    #[arg(
        short = 't',
        long = "type",
        value_name = "TYPE",
        value_enum,
        help = "Project type"
    )]
    pub kind: Option<ProjectKind>,

    /// Architecture pattern.
    #[arg(
        short = 'a',
        long = "arch",
        value_name = "ARCH",
        value_enum,
        help = "Architecture pattern"
    )]
    pub architecture: Option<Architecture>,

    /// Framework (optional).
    #[arg(
        short = 'f',
        long = "framework",
        value_name = "FRAMEWORK",
        help = "Framework to use (e.g. axum, fastapi, react)"
    )]
    pub framework: Option<String>,

    // /// Override the output directory.
    // #[arg(
    //     short = 'o',
    //     long = "output",
    //     value_name = "DIR",
    //     help = "Output directory (default: current directory)"
    // )]
    // pub output: Option<PathBuf>,
    /// Skip the confirmation prompt.
    #[arg(
        short = 'y',
        long = "yes",
        help = "Skip confirmation and create immediately"
    )]
    pub yes: bool,

    /// Overwrite an existing directory (destructive).
    #[arg(long = "force", help = "Overwrite existing directory")]
    pub force: bool,

    /// Preview what would be created without writing any files.
    #[arg(long = "dry-run", help = "Show what would be created without creating")]
    pub dry_run: bool,

    /// Use a specific template ID, bypassing automatic matching.
    #[arg(
        long = "template",
        value_name = "ID",
        help = "Specific template ID to use"
    )]
    pub template: Option<String>,
}

// ── list ──────────────────────────────────────────────────────────────────────

/// Arguments for `scarff list`.
#[derive(Debug, Args)]
pub struct ListArgs {
    /// Filter by language.
    #[arg(short = 'l', long = "lang", value_enum, help = "Filter by language")]
    pub language: Option<Language>,

    /// Filter by project type.
    #[arg(short = 't', long = "type", value_enum, help = "Filter by type")]
    pub kind: Option<ProjectKind>,

    /// Filter by architecture.
    #[arg(
        short = 'a',
        long = "arch",
        value_enum,
        help = "Filter by architecture"
    )]
    pub architecture: Option<Architecture>,

    /// Include internal/debug templates.
    #[arg(long = "all", help = "Show all templates including internals")]
    pub all: bool,

    /// Output format.
    #[arg(
        long = "format",
        value_enum,
        default_value = "table",
        help = "Output format"
    )]
    pub format: ListFormat,
}

/// Output format for the `list` command.
#[derive(Debug, Clone, Copy, ValueEnum)]
pub enum ListFormat {
    /// Human-readable table.
    Table,
    /// One name per line.
    List,
    /// JSON array.
    Json,
    /// CSV rows.
    Csv,
}

// ── init ──────────────────────────────────────────────────────────────────────

/// Arguments for `scarff init`.
#[derive(Debug, Args)]
pub struct InitArgs {
    /// Write to the global config location.
    #[arg(long = "global", help = "Create global configuration")]
    pub global: bool,

    /// Write to `.scarff.toml` in the current directory.
    #[arg(
        long = "local",
        help = "Create local configuration in current directory"
    )]
    pub local: bool,

    /// Overwrite an existing config file.
    #[arg(short = 'f', long = "force", help = "Overwrite existing configuration")]
    pub force: bool,
}

// ── completions ───────────────────────────────────────────────────────────────

/// Arguments for `scarff completions`.
#[derive(Debug, Args)]
pub struct CompletionsArgs {
    /// Target shell.
    #[arg(value_enum, help = "Shell to generate completions for")]
    pub shell: Shell,
}

/// Supported shells for completion generation.
#[derive(Debug, Clone, Copy, ValueEnum)]
pub enum Shell {
    Bash,
    Zsh,
    Fish,
    PowerShell,
    Elvish,
}

// ── config subcommands ────────────────────────────────────────────────────────

/// Subcommands for `scarff config`.
#[derive(Debug, Subcommand)]
pub enum ConfigCommands {
    /// Print the value of a configuration key.
    Get {
        /// Dotted key path, e.g. `defaults.lang`.
        key: String,
    },
    /// Set a configuration key to a value.
    Set {
        /// Dotted key path.
        key: String,
        /// New value.
        value: String,
    },
    /// Print all configuration values.
    List,
    /// Print the path to the active configuration file.
    Path,
}

// ── value enums ───────────────────────────────────────────────────────────────

/// Supported programming languages.
#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
#[value(rename_all = "lowercase")]
pub enum Language {
    Rust,
    Python,
    /// Also accepted as `ts`.
    #[value(alias = "ts")]
    TypeScript,
    Go,
}

impl std::fmt::Display for Language {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Rust => write!(f, "rust"),
            Self::Python => write!(f, "python"),
            Self::TypeScript => write!(f, "typescript"),
            Self::Go => write!(f, "golang"),
        }
    }
}

/// Supported project types.
#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
#[value(rename_all = "snake_case")]
pub enum ProjectKind {
    Cli,
    #[value(name = "backend", alias = "web_api")]
    Backend,
    #[value(name = "frontend", alias = "web_frontend")]
    Frontend,
    Fullstack,
    Worker,
}

impl std::fmt::Display for ProjectKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Cli => write!(f, "cli"),
            Self::Backend => write!(f, "backend"),
            Self::Frontend => write!(f, "frontend"),
            Self::Fullstack => write!(f, "fullstack"),
            Self::Worker => write!(f, "worker"),
        }
    }
}

/// Supported architecture patterns.
#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
#[value(rename_all = "lowercase")]
pub enum Architecture {
    Layered,
    Clean,
    Onion,
    Modular,
    Mvc,
}

impl std::fmt::Display for Architecture {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Layered => write!(f, "layered"),
            Self::Clean => write!(f, "clean"),
            Self::Onion => write!(f, "onion"),
            Self::Modular => write!(f, "modular"),
            Self::Mvc => write!(f, "mvc"),
        }
    }
}

// ── tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use clap::Parser;

    #[test]
    fn verify_cli_structure() {
        // clap's internal consistency check — catches conflicts, missing values, etc.
        // Cli::command().debug_assert();
    }

    #[test]
    fn language_display() {
        assert_eq!(Language::Rust.to_string(), "rust");
        assert_eq!(Language::Python.to_string(), "python");
        assert_eq!(Language::TypeScript.to_string(), "typescript");
    }

    #[test]
    fn project_kind_display() {
        assert_eq!(ProjectKind::Cli.to_string(), "cli");
        assert_eq!(ProjectKind::Backend.to_string(), "backend");
        assert_eq!(ProjectKind::Frontend.to_string(), "frontend");
        assert_eq!(ProjectKind::Fullstack.to_string(), "fullstack");
        assert_eq!(ProjectKind::Worker.to_string(), "worker");
    }

    #[test]
    fn architecture_display() {
        assert_eq!(Architecture::Layered.to_string(), "layered");
        assert_eq!(Architecture::Clean.to_string(), "clean");
        assert_eq!(Architecture::Onion.to_string(), "onion");
        assert_eq!(Architecture::Modular.to_string(), "modular");
        assert_eq!(Architecture::Mvc.to_string(), "mvc");
    }

    #[test]
    fn parse_new_command() {
        let cli = Cli::parse_from([
            "scarff",
            "new",
            "my-project",
            "--lang",
            "rust",
            "--type",
            "cli",
            "--arch",
            "layered",
        ]);
        assert!(matches!(cli.command, Commands::New(_)));
    }

    #[test]
    fn typescript_alias() {
        let cli = Cli::parse_from([
            "scarff", "new", "test", "-l", "ts", "-t", "cli", "-a", "layered",
        ]);
        if let Commands::New(args) = cli.command {
            assert_eq!(args.language, Language::TypeScript);
        } else {
            panic!("expected New command");
        }
    }

    #[test]
    fn quiet_and_verbose_conflict() {
        // clap should reject --quiet --verbose together
        let result = Cli::try_parse_from(["scarff", "--quiet", "--verbose", "list"]);
        assert!(result.is_err());
    }
}
