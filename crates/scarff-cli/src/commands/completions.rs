//! Shell completion generation.

use clap::CommandFactory;
use clap_complete::{generate, shells};

use crate::cli::{Cli, CompletionsArgs, Shell};

pub fn execute(args: CompletionsArgs) -> crate::error::CliResult<()> {
    let mut cmd = Cli::command();

    match args.shell {
        Shell::Bash => generate(shells::Bash, &mut cmd, "scarff", &mut std::io::stdout()),
        Shell::Zsh => generate(shells::Zsh, &mut cmd, "scarff", &mut std::io::stdout()),
        Shell::Fish => generate(shells::Fish, &mut cmd, "scarff", &mut std::io::stdout()),
        Shell::PowerShell => generate(
            shells::PowerShell,
            &mut cmd,
            "scarff",
            &mut std::io::stdout(),
        ),
        Shell::Elvish => generate(shells::Elvish, &mut cmd, "scarff", &mut std::io::stdout()),
    };

    Ok(())
}
