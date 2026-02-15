//! Implementation of the `scarff list` command.

use crate::{
    cli::{ListArgs, ListFormat, global::GlobalArgs},
    error::CliResult,
    output::OutputManager,
};

pub fn execute(args: ListArgs, _global: GlobalArgs, output: OutputManager) -> CliResult<()> {
    use scarff_adapters::InMemoryStore;
    use scarff_core::application::TemplateService;

    let store =
        Box::new(InMemoryStore::with_builtin().map_err(|e| crate::error::CliError::Core(e))?);

    let service = TemplateService::new(store);
    let templates = service.list().map_err(crate::error::CliError::Core)?;

    match args.format {
        ListFormat::Table => {
            output.header("Available Templates:")?;
            for template in templates {
                output.print(&format!(
                    "  {} @ {} ({})",
                    template.metadata.name,
                    template.metadata.version,
                    template
                        .matcher
                        .language
                        .map(|l| l.to_string())
                        .unwrap_or_else(|| "any".into())
                ))?;
            }
        }
        ListFormat::Json => {
            // Serialise as a JSON array to stdout (bypasses OutputManager
            // because JSON output must be parseable even in non-TTY pipes).
            // todo: serialiaze template dto
            // let json = serde_json::to_string_pretty(&templates).unwrap_or_else(|_| "[]".into());
            // println!("{json}");
        }

        ListFormat::List => {
            for t in &templates {
                println!("{}", t.metadata.name);
            }
        }

        ListFormat::Csv => {
            println!("name,version,language");
            for t in &templates {
                let lang = t
                    .matcher
                    .language
                    .map(|l| l.to_string())
                    .unwrap_or_else(|| "any".into());
                println!("{},{},{}", t.metadata.name, t.metadata.version, lang);
            }
        }
    }

    Ok(())
}
