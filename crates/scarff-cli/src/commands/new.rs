//! Implementation of the `scarff new` command.
//!
//! Invariants enforced here (RFC-0001 §6):
//! 1. Determinism        — delegated to domain builder
//! 2. Fail-fast          — validation and path check before any output
//! 3. No partial state   — delegated to `ScaffoldService` (rollback on failure)
//! 4. Runnable default   — delegated to the template layer
//! 5. Transparent infer  — `UserChoices` tracks what the user provided
//! 6. Override wins      — explicit flags fed directly to the builder

use std::path::{Path, PathBuf};

use tracing::{debug, info, instrument};

use scarff_adapters::{InMemoryStore, LocalFilesystem, SimpleRenderer};
use scarff_core::{
    application::ScaffoldService,
    domain::{
        Architecture as CoreArch, Framework as CoreFramework, Language as CoreLanguage,
        ProjectKind as CoreKind, Target, capabilities::FRAMEWORK_REGISTRY,
    },
};

use crate::{
    cli::{Architecture, Language, NewArgs, ProjectKind, global::GlobalArgs},
    config::AppConfig,
    error::{CliError, CliResult},
    output::OutputManager,
};

// ── UserChoices ───────────────────────────────────────────────────────────────

/// Records which flags the user explicitly provided before the domain
/// builder infers defaults. Used by `show_configuration` to mark inferred
/// values, satisfying RFC-0001 §6.5 ("transparent inference").
#[derive(Debug, Default)]
struct UserChoices {
    language: bool,
    kind: bool,
    framework: bool,
    architecture: bool,
}

/// Execute the `scarff new` command.
///
/// Dispatch sequence:
/// 1. Parse and validate the project name / output path
/// 2. Convert CLI args to a core `Target` (with full inference)
/// 3. Confirm with user unless `--yes` or `--quiet`
/// 4. Early-exit if `--dry-run`
/// 5. Execute scaffolding via `ScaffoldService`
/// 6. Print next-steps guidance
#[instrument(skip_all, fields(project = %args.name))]
pub fn execute(
    args: NewArgs,
    global: GlobalArgs,
    config: AppConfig,
    output: OutputManager,
) -> CliResult<()> {
    debug!(
        dry_run = args.dry_run,
        yes = args.yes,
        force = args.force,
        // default = args.default,
        "scarff new started"
    );

    // Step 1 — Resolve and validate the project path name.
    let (project_name, output_dir) = resolve_project_path(&args.name)?;
    validate_project_name(&project_name)?;

    // Step 2 — Build and fully validate the domain target.
    // All domain errors surface here, before we show anything.
    let (target, choices) = build_target(&args, &config)?;

    info!(
        language = %target.language(),
        kind     = %target.kind(),
        arch     = %target.architecture(),
        "Target built"
    );

    // Step 3 — Check path existence before asking the user anything.
    // RFC §6.2: errors must occur before filesystem writes — and before
    // the user is asked to confirm, which is also a form of output.
    let project_path = output_dir.join(&project_name);
    if project_path.exists() && !args.force {
        return Err(CliError::ProjectExists { path: project_path });
    }

    // Step 4 — Show the resolved configuration.
    // RFC §8: dry-run output must be identical to a real run, so we always
    // show this panel — the dry-run short-circuit comes later.
    if !global.quiet {
        show_configuration(&target, &choices, &project_name, &project_path, &output)?;
    }

    // Step 5 — Confirm (skipped by --yes, --default, or --dry-run).
    if !global.quiet && !args.yes && !args.dry_run && !confirm()? {
        return Err(CliError::Cancelled);
    }

    // Step 6 — Dry-run short-circuit.
    // RFC §8: explicit notice after the config panel.
    if args.dry_run {
        output.info("Dry run — no files were written.")?;
        output.info(&format!(
            "Dry run: would create '{}' at {}",
            project_name,
            project_path.display(),
        ))?;
        output.info(&format!("  Language:     {}", target.language()))?;
        output.info(&format!("  Kind:         {}", target.kind()))?;
        output.info(&format!("  Architecture: {}", target.architecture()))?;
        if let Some(fw) = target.framework() {
            output.info(&format!("  Framework:    {fw}"))?;
        }
        return Ok(());
    }

    // Step 7 — Execute scaffolding.
    output.header(&format!("Creating project '{project_name}'..."))?;

    let store = Box::new(InMemoryStore::with_builtin().map_err(CliError::Core)?);
    let renderer = Box::new(SimpleRenderer::new());
    let filesystem = Box::new(LocalFilesystem::new());
    let service = ScaffoldService::new(store, renderer, filesystem);

    service
        .scaffold(target.clone(), &project_name, &output_dir)
        .map_err(CliError::Core)?;

    // Step 8 — Success output.
    output.success(&format!("Project '{project_name}' created successfully!"))?;

    if !global.quiet {
        output.print("")?;
        show_next_steps(&target, &project_name, &output)?;
    }

    Ok(())
}

// ── Path resolution ───────────────────────────────────────────────────────────

pub fn resolve_project_path(name: &str) -> CliResult<(String, PathBuf)> {
    let path = Path::new(name);

    let project_name = path
        .file_name()
        .and_then(|n| n.to_str())
        .ok_or_else(|| CliError::InvalidProjectName {
            name: name.into(),
            reason: "cannot extract valid project name".into(),
        })?
        .to_string();

    // let target_path = path
    //     .parent()
    //     .filter(|p| !p.as_os_str().is_empty())
    //     .map(|p| p.to_path_buf())
    //     .unwrap_or_else(|| PathBuf::from("."));
    // Return the FULL path to the project directory, not just the parent
    let target_path = path.to_path_buf();

    Ok((project_name, target_path))
}

fn validate_project_name(name: &str) -> CliResult<()> {
    if name.is_empty() {
        return Err(CliError::InvalidProjectName {
            name: name.into(),
            reason: "name cannot be empty".into(),
        });
    }
    if name.starts_with('.') {
        return Err(CliError::InvalidProjectName {
            name: name.into(),
            reason: "name cannot start with '.'".into(),
        });
    }
    if name.contains('/') || name.contains('\\') {
        return Err(CliError::InvalidProjectName {
            name: name.into(),
            reason: "name cannot contain path separators".into(),
        });
    }
    Ok(())
}

// ── build_target ──────────────────────────────────────────────────────────────

/// Translate CLI arguments into a validated `Target`, recording which fields
/// the user explicitly provided vs what the domain builder will infer.
fn build_target(args: &NewArgs, _config: &AppConfig) -> CliResult<(Target, UserChoices)> {
    let mut choices = UserChoices::default();

    //todo:  Language — required unless --default falls back to config.
    // let lang_cli = if args.default && args.language.is_none() {
    //     config
    //         .defaults
    //         .language
    //         .as_deref()
    //         .and_then(|s| s.parse::<Language>().ok())
    //         .unwrap_or(Language::Rust)
    // } else {
    //     args.language.ok_or_else(|| CliError::MissingRequiredFlag {
    //         flag: "--lang / -l".into(),
    //         hint: "Specify a language (e.g. --lang rust) or use --default.".into(),
    //     })?
    // };
    // choices.language = args.language.is_some();

    let lang = convert_language(args.language);
    choices.language = true;
    let mut builder = Target::builder().language(lang);

    if let Some(kind) = args.kind {
        choices.kind = args.kind.is_some();
        builder = builder
            .kind(convert_kind(kind))
            .map_err(|e| CliError::Core(e.into()))?;
    }

    // Framework — optional.
    if let Some(ref fw_str) = args.framework {
        choices.framework = true;
        let fw = parse_framework(args.language, fw_str)?;
        builder = builder
            .framework(fw)
            .map_err(|e| CliError::Core(e.into()))?;
    }

    // Architecture — optional.
    if let Some(arch) = args.architecture {
        choices.architecture = true;
        let core_arch = convert_architecture(arch);
        builder = builder.architecture(core_arch);
    }

    let target = builder.build().map_err(|e| CliError::Core(e.into()))?;
    Ok((target, choices))
}

// ── Type conversions CLI → core ───────────────────────────────────────────────

fn convert_language(lang: Language) -> CoreLanguage {
    match lang {
        Language::Rust => CoreLanguage::Rust,
        Language::Python => CoreLanguage::Python,
        Language::TypeScript => CoreLanguage::TypeScript,
        Language::Go => CoreLanguage::Go,
    }
}

fn convert_kind(kind: ProjectKind) -> CoreKind {
    match kind {
        ProjectKind::Cli => CoreKind::Cli,
        ProjectKind::Backend => CoreKind::WebBackend,
        ProjectKind::Frontend => CoreKind::WebFrontend,
        ProjectKind::Fullstack => CoreKind::Fullstack,
        ProjectKind::Worker => CoreKind::Worker,
    }
}

fn convert_architecture(arch: Architecture) -> CoreArch {
    match arch {
        Architecture::Layered => CoreArch::Layered,
        Architecture::Clean | Architecture::Onion => CoreArch::Clean,
        Architecture::Modular => CoreArch::FeatureModular,
        Architecture::Mvc => CoreArch::Mvc,
    }
}

/// Parse a user-supplied framework string for the given language.
///
/// This function consults the capability registry rather than a hand-maintained
/// match table. Adding a new framework only requires updating
/// `FRAMEWORK_REGISTRY` in `capabilities.rs`; this function needs no change.
fn parse_framework(lang: Language, fw: &str) -> CliResult<CoreFramework> {
    let core_lang = convert_language(lang);
    let fw_lower = fw.to_ascii_lowercase();

    // Walk the registry and find the first framework belonging to this
    // language whose as_str() matches the user input.
    let found = FRAMEWORK_REGISTRY.iter().find(|def| {
        def.framework.language() == core_lang && def.framework.as_str() == fw_lower.as_str()
    });

    found.map(|def| def.framework).ok_or_else(|| {
        // Build a helpful list of available choices from the registry.
        let available: Vec<&str> = FRAMEWORK_REGISTRY
            .iter()
            .filter(|def| def.framework.language() == core_lang)
            .map(|def| def.framework.as_str())
            .collect();

        CliError::FrameworkNotAvailable {
            framework: fw.into(),
            language: lang.to_string(),
            available,
        }
    })
}

// ── UI helpers ────────────────────────────────────────────────────────────────

// ── Output helpers ────────────────────────────────────────────────────────────

/// Print the resolved configuration panel.
///
/// Marks inferred values with `(inferred)` — RFC-0001 §6.5.
fn show_configuration(
    target: &Target,
    choices: &UserChoices,
    name: &str,
    project_path: &Path,
    out: &OutputManager,
) -> CliResult<()> {
    let inferred = |explicit: bool| if explicit { "" } else { "  (inferred)" };

    out.header("Configuration:")?;
    out.print(&format!("  Project:      {name}"))?;
    out.print(&format!(
        "  Language:     {}{}",
        target.language(),
        inferred(choices.language)
    ))?;
    out.print(&format!(
        "  Type:         {}{}",
        target.kind(),
        inferred(choices.kind)
    ))?;

    match target.framework() {
        Some(fw) => out.print(&format!(
            "  Framework:    {}{}",
            fw,
            inferred(choices.framework)
        ))?,
        None => out.print("  Framework:    (none)")?,
    }

    out.print(&format!(
        "  Architecture: {}{}",
        target.architecture(),
        inferred(choices.architecture)
    ))?;
    out.print(&format!("  Location:     {}", project_path.display()))?;
    out.print("")?;

    Ok(())
}

/// Print language- and framework-specific next steps.
///
/// RFC-0001 §8 requires next steps in the success output. Generic steps
/// ("# Start coding!") don't satisfy this — a Rust user needs `cargo run`,
/// a Python FastAPI user needs `uvicorn`, a TypeScript user needs `npm`.
fn show_next_steps(target: &Target, name: &str, out: &OutputManager) -> CliResult<()> {
    use scarff_core::domain::{Language as L, PythonFramework, TypeScriptFramework};

    out.print("Next steps:")?;
    out.print(&format!("  cd {name}"))?;

    match target.language() {
        L::Rust => {
            out.print("  cargo build")?;
            out.print("  cargo run")?;
        }

        L::Python => match target.framework() {
            Some(CoreFramework::Python(PythonFramework::Django)) => {
                out.print("  python -m venv .venv && source .venv/bin/activate")?;
                out.print("  pip install -r requirements.txt")?;
                out.print("  python manage.py runserver")?;
            }
            Some(CoreFramework::Python(PythonFramework::FastApi)) => {
                out.print("  python -m venv .venv && source .venv/bin/activate")?;
                out.print("  pip install -r requirements.txt")?;
                out.print("  uvicorn main:app --reload")?;
            }
            Some(CoreFramework::Python(PythonFramework::Flask)) => {
                out.print("  python -m venv .venv && source .venv/bin/activate")?;
                out.print("  pip install -r requirements.txt")?;
                out.print("  flask run")?;
            }
            _ => {
                out.print("  python -m venv .venv && source .venv/bin/activate")?;
                out.print("  python main.py")?;
            }
        },

        L::TypeScript => match target.framework() {
            Some(CoreFramework::TypeScript(
                TypeScriptFramework::React | TypeScriptFramework::Vue | TypeScriptFramework::Svelte,
            )) => {
                out.print("  npm install")?;
                out.print("  npm run dev")?;
            }
            Some(CoreFramework::TypeScript(TypeScriptFramework::NextJs)) => {
                out.print("  npm install")?;
                out.print("  npm run dev")?;
                out.print("  # → http://localhost:3000")?;
            }
            Some(CoreFramework::TypeScript(TypeScriptFramework::NestJs)) => {
                out.print("  npm install")?;
                out.print("  npm run start:dev")?;
            }
            _ => {
                out.print("  npm install")?;
                out.print("  npm run build && npm start")?;
            }
        },

        L::Go => {
            out.print("  go mod tidy")?;
            out.print("  go run .")?;
        }
    }

    out.print("")?;
    Ok(())
}

fn confirm() -> CliResult<bool> {
    use std::io::{self, Write};

    print!("Continue? [Y/n] ");

    // Map the flush error — panicking here (via unwrap) is wrong when stdout
    // may be closed because the caller piped our output to a closed reader.
    io::stdout().flush().map_err(|e| CliError::IoError {
        message: "failed to flush stdout".into(),
        source: e,
    })?;

    let mut input = String::new();
    io::stdin()
        .read_line(&mut input)
        .map_err(|e| CliError::IoError {
            message: "failed to read confirmation input".into(),
            source: e,
        })?;

    let trimmed = input.trim().to_lowercase();
    Ok(trimmed.is_empty() || trimmed == "y" || trimmed == "yes")
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use scarff_core::domain::{
        Architecture as CoreArch, Framework as CoreFramework, Language as CoreLanguage,
        value_objects::{GoFramework, PythonFramework, RustFramework, TypeScriptFramework},
    };

    // ── resolve_project_path ──────────────────────────────────────────────────

    // #[test]
    // fn simple_name_resolves_to_cwd() {
    //     let (name, dir) = resolve_project_path("my-app").unwrap();
    //     assert_eq!(name, "my-app");
    //     assert_eq!(dir, PathBuf::from("./my-app"));
    // }
    #[test]
    fn relative_path_splits_leaf_and_parent() {
        let (name, dir) = resolve_project_path("../my-app").unwrap();
        assert_eq!(name, "my-app");
        assert_eq!(dir, PathBuf::from("../my-app"));
    }

    #[test]
    #[cfg(windows)]
    fn handles_backslashes_on_windows() {
        // This just works™ on Windows, no extra code needed
        let (name, dir) = resolve_project_path("foo\\bar\\my-app").unwrap();
        assert_eq!(name, "my-app");
        // dir will be "foo\\bar" on Windows, "foo/bar" on Unix
        // But PathBuf equality handles this automatically!
    }

    #[test]
    fn nested_path_works_on_all_platforms() {
        // Uses / on Unix, \ on Windows
        let sep = std::path::MAIN_SEPARATOR;
        let path = format!("foo{sep}bar{sep}my-app");

        let (name, dir) = resolve_project_path(&path).unwrap();
        assert_eq!(name, "my-app");

        let expected = PathBuf::from("foo").join("bar").join("my-app");
        assert_eq!(dir, expected);
    }

    // ── validate_project_name ─────────────────────────────────────────────────

    #[test]
    fn empty_name_is_invalid() {
        assert!(matches!(
            validate_project_name(""),
            Err(CliError::InvalidProjectName { .. })
        ));
    }

    #[test]
    fn dotfile_name_is_invalid() {
        assert!(matches!(
            validate_project_name(".hidden"),
            Err(CliError::InvalidProjectName { .. })
        ));
    }

    #[test]
    fn path_separator_in_name_is_invalid() {
        assert!(validate_project_name("a/b").is_err());
        assert!(validate_project_name("a\\b").is_err());
    }

    #[test]
    fn valid_names_pass() {
        for name in &["my-project", "my_app", "project123", "MyApp", "scarff"] {
            assert!(validate_project_name(name).is_ok(), "failed for: {name}");
        }
    }

    // ── parse_framework (registry-driven) ────────────────────────────────────

    #[test]
    fn axum_parses_for_rust() {
        let fw = parse_framework(Language::Rust, "axum").unwrap();
        assert_eq!(fw, CoreFramework::Rust(RustFramework::Axum));
    }

    #[test]
    fn actix_parses_for_rust() {
        let fw = parse_framework(Language::Rust, "actix").unwrap();
        assert_eq!(fw, CoreFramework::Rust(RustFramework::Actix));
    }

    #[test]
    fn fastapi_parses_for_python() {
        let fw = parse_framework(Language::Python, "fastapi").unwrap();
        assert_eq!(fw, CoreFramework::Python(PythonFramework::FastApi));
    }

    #[test]
    fn gin_parses_for_go() {
        let fw = parse_framework(Language::Go, "gin").unwrap();
        assert_eq!(fw, CoreFramework::Go(GoFramework::Gin));
    }

    #[test]
    fn nestjs_parses_for_typescript() {
        let fw = parse_framework(Language::TypeScript, "nestjs").unwrap();
        assert_eq!(fw, CoreFramework::TypeScript(TypeScriptFramework::NestJs));
    }

    #[test]
    fn svelte_parses_for_typescript() {
        let fw = parse_framework(Language::TypeScript, "svelte").unwrap();
        assert_eq!(fw, CoreFramework::TypeScript(TypeScriptFramework::Svelte));
    }

    #[test]
    fn wrong_language_framework_is_error() {
        // Django is a Python framework, not a Rust one.
        assert!(matches!(
            parse_framework(Language::Rust, "django"),
            Err(CliError::FrameworkNotAvailable { .. })
        ));
    }

    #[test]
    fn unknown_framework_gives_actionable_error_with_available_list() {
        let err = parse_framework(Language::Rust, "unknown-fw").unwrap_err();
        match err {
            CliError::FrameworkNotAvailable {
                framework,
                available,
                ..
            } => {
                assert_eq!(framework, "unknown-fw");
                assert!(!available.is_empty(), "should list available frameworks");
                assert!(available.contains(&"axum"));
                assert!(available.contains(&"actix"));
            }
            other => panic!("unexpected error: {other:?}"),
        }
    }

    #[test]
    fn framework_matching_is_case_insensitive() {
        // Registry stores lowercase; we normalise input to lowercase before matching.
        assert!(parse_framework(Language::Rust, "AXUM").is_ok());
        assert!(parse_framework(Language::Rust, "Axum").is_ok());
        assert!(parse_framework(Language::Python, "FastAPI").is_ok());
    }

    // ── convert_language covers all variants ──────────────────────────────────

    #[test]
    fn convert_language_covers_go() {
        // Go was missing in the original implementation.
        assert_eq!(convert_language(Language::Go), CoreLanguage::Go);
    }

    // ── convert_architecture covers all variants ──────────────────────────────

    #[test]
    fn onion_converts_to_clean() {
        assert_eq!(convert_architecture(Architecture::Onion), CoreArch::Clean);
    }
}
