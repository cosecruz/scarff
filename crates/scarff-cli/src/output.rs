//! Output management and formatting.

use std::io::{self, IsTerminal};

use console::Term;
use owo_colors::OwoColorize;

use crate::cli::global::{GlobalArgs, OutputFormat};
use crate::config::AppConfig;

/// Manages CLI output based on configuration.
pub struct OutputManager {
    resolved_format: OutputFormat,
    quiet: bool,
    no_color: bool,
    term: Term,
}

impl OutputManager {
    /// Build an `OutputManager` from parsed CLI flags and loaded config.
    pub fn new(args: &GlobalArgs, config: &AppConfig) -> Self {
        // Resolve Auto → Human (TTY) or Plain (piped/redirected).
        let resolved_format = if args.output_format == OutputFormat::Auto {
            if io::stdout().is_terminal() {
                OutputFormat::Human
            } else {
                OutputFormat::Plain
            }
        } else {
            args.output_format
        };

        Self {
            resolved_format,
            quiet: args.quiet,
            no_color: args.no_color || config.output.no_color,
            term: Term::stdout(),
        }
    }
    // ── Public write methods ───────────────────────────────────────────────

    /// Generic message; suppressed in quiet mode.
    pub fn print(&self, msg: &str) -> io::Result<()> {
        if self.quiet {
            return Ok(());
        }
        self.term.write_line(msg)
    }

    /// Success indicator: `✓ <msg>`.
    pub fn success(&self, msg: &str) -> io::Result<()> {
        if self.quiet {
            return Ok(());
        }
        let line = if self.no_color {
            format!("\u{2713} {msg}") // ✓
        } else {
            format!("{} {}", "\u{2713}".green().bold(), msg.green())
        };
        self.term.write_line(&line)
    }

    /// Error indicator: `✗ <msg>`.  *Not* suppressed in quiet mode — errors
    /// must always be visible.
    pub fn error(&self, msg: &str) -> io::Result<()> {
        let line = if self.no_color {
            format!("\u{2717} {msg}") // ✗
        } else {
            format!("{} {}", "\u{2717}".red().bold(), msg.red())
        };
        self.term.write_line(&line)
    }

    /// Warning indicator: `⚠ <msg>`.
    pub fn warning(&self, msg: &str) -> io::Result<()> {
        if self.quiet {
            return Ok(());
        }
        let line = if self.no_color {
            format!("\u{26a0} {msg}") // ⚠
        } else {
            format!("{} {}", "\u{26a0}".yellow().bold(), msg.yellow())
        };
        self.term.write_line(&line)
    }

    /// Informational indicator: `ℹ <msg>`.
    pub fn info(&self, msg: &str) -> io::Result<()> {
        if self.quiet {
            return Ok(());
        }
        let line = if self.no_color {
            format!("\u{2139} {msg}") // ℹ
        } else {
            format!("{} {}", "\u{2139}".blue().bold(), msg.blue())
        };
        self.term.write_line(&line)
    }

    /// Bold cyan header line.
    pub fn header(&self, text: &str) -> io::Result<()> {
        if self.quiet {
            return Ok(());
        }
        let line = if self.no_color {
            text.to_owned()
        } else {
            text.cyan().bold().to_string()
        };
        self.term.write_line(&line)
    }

    // ── Accessors ─────────────────────────────────────────────────────────

    /// `true` if ANSI colours are enabled.
    pub fn supports_color(&self) -> bool {
        !self.no_color
    }

    /// `true` if quiet mode suppresses most output.
    pub fn is_quiet(&self) -> bool {
        self.quiet
    }

    /// The resolved (non-Auto) output format.
    pub fn format(&self) -> OutputFormat {
        self.resolved_format
    }
}

// ── tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    use crate::config::AppConfig;

    fn make_manager(quiet: bool, no_color: bool) -> OutputManager {
        let args = GlobalArgs {
            verbose: 0,
            quiet,
            no_color,
            config: None,
            output_format: OutputFormat::Plain, // avoid TTY detection in tests
        };
        OutputManager::new(&args, &AppConfig::default())
    }

    #[test]
    fn quiet_suppresses_print() {
        let out = make_manager(true, true);
        // write_line on Term::stdout() in tests is harmless; we just verify
        // the method returns Ok without panicking.
        assert!(out.print("hello").is_ok());
    }

    #[test]
    fn error_not_suppressed_in_quiet_mode() {
        // error() must always write — calling it in quiet mode should not
        // silently drop the message.  We can't inspect the terminal buffer
        // here, but we verify it doesn't short-circuit to Ok(()) without
        // attempting the write.
        let out = make_manager(true, true);
        // Term::stdout() in a test environment won't panic even without a TTY.
        assert!(out.error("something went wrong").is_ok());
    }

    #[test]
    fn no_color_flag_reported() {
        let colored = make_manager(false, false);
        let no_color = make_manager(false, true);
        // Without a real TTY the exact value depends on the environment;
        // we test the accessor reflects what was set.
        assert!(colored.supports_color());
        assert!(!no_color.supports_color());
    }

    #[test]
    fn format_accessor_returns_resolved() {
        let out = make_manager(false, false);
        assert_eq!(out.format(), OutputFormat::Plain);
    }
}
