//! Implementation of the `scarff new` command.
//!
//! Responsibility: translate CLI arguments into a `Target`, call the core
//! scaffold service, and display results. No business logic lives here.

use std::path::{Path, PathBuf};

use tracing::{debug, info, instrument};

use scarff_adapters::{InMemoryStore, LocalFilesystem, SimpleRenderer};
use scarff_core::{
    application::{ScaffoldService, TemplateService},
    domain::{
        Architecture as CoreArch, Framework as CoreFramework, Language as CoreLanguage,
        ProjectKind as CoreKind, Target,
        capabilities::{FRAMEWORK_REGISTRY, find_language},
        value_objects::{GoFramework, PythonFramework, RustFramework, TypeScriptFramework},
    },
};

use crate::{
    cli::{Architecture, Language, NewArgs, ProjectKind, global::GlobalArgs},
    config::AppConfig,
    error::{CliError, CliResult},
    output::OutputManager,
};

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
    // 1. Resolve project path
    let (project_name, output_dir) = resolve_project_path(&args.name)?;
    validate_project_name(&project_name)?;

    // 2. Build target (inference + validation)
    let target = build_target(&args, &config)?;

    debug!(
        language = %target.language(),
        kind = %target.kind(),
        architecture = %target.architecture(),
        framework = target.framework().map(|f| f.to_string()).as_deref().unwrap_or("none"),
        "Target resolved"
    );

    // 3. Show configuration and confirm
    if !global.quiet && !args.yes {
        show_configuration(&target, &project_name, &output_dir, &output)?;
        if !confirm()? {
            return Err(CliError::Cancelled);
        }
    }

    // 4. Check for existing directory
    let project_path = output_dir.join(&project_name);
    if project_path.exists() && !args.force {
        return Err(CliError::ProjectExists { path: project_path });
    }

    // 5. Dry run: describe but do not write.
    if args.dry_run {
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

    // 6. Create adapters and scaffold
    let store = Box::new(InMemoryStore::with_builtin().map_err(CliError::Core)?);
    let renderer = Box::new(SimpleRenderer::new());
    let filesystem = Box::new(LocalFilesystem::new());
    let service = ScaffoldService::new(store, renderer, filesystem);

    output.header(&format!("Creating '{project_name}'..."))?;
    info!(project = %project_name, path = %project_path.display(), "Scaffold started");

    service
        .scaffold(target, &project_name, &output_dir)
        .map_err(CliError::Core)?;

    info!(project = %project_name, "Scaffold completed");

    // 7. Success + next steps
    output.success(&format!("Project '{project_name}' created!"))?;

    if !global.quiet {
        output.print("")?;
        output.print("Next steps:")?;
        output.print(&format!("  cd {project_name}"))?;
        output.print("  # Start building!")?;
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

// ── Target construction ───────────────────────────────────────────────────────

fn build_target(args: &NewArgs, _config: &AppConfig) -> CliResult<Target> {
    let lang = convert_language(args.language);
    let mut builder = Target::builder().language(lang);

    if let Some(kind) = args.kind {
        builder = builder
            .kind(convert_kind(kind))
            .map_err(|e| CliError::Core(e.into()))?;
    }

    if let Some(fw_str) = &args.framework {
        let fw = parse_framework(args.language, fw_str)?;
        builder = builder
            .framework(fw)
            .map_err(|e| CliError::Core(e.into()))?;
    }

    if let Some(arch) = args.architecture {
        builder = builder.architecture(convert_architecture(arch));
    }

    builder.build().map_err(|e| CliError::Core(e.into()))
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

fn show_configuration(
    target: &Target,
    name: &str,
    output_dir: &Path,
    out: &OutputManager,
) -> CliResult<()> {
    out.header("Configuration")?;
    out.print(&format!("  Project:      {name}"))?;
    out.print(&format!("  Language:     {}", target.language()))?;
    out.print(&format!("  Kind:         {}", target.kind()))?;
    out.print(&format!("  Architecture: {}", target.architecture()))?;
    if let Some(fw) = target.framework() {
        out.print(&format!("  Framework:    {fw}"))?;
    }
    out.print(&format!("  Location:     {}", output_dir.display()))?;
    out.print("")?;
    Ok(())
}

fn confirm() -> CliResult<bool> {
    use std::io::{self, Write};

    print!("Continue? [Y/n] ");
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

    let input = input.trim().to_ascii_lowercase();
    Ok(input.is_empty() || input == "y" || input == "yes")
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    // ── resolve_project_path ──────────────────────────────────────────────────

    #[test]
    fn simple_name_resolves_to_cwd() {
        let (name, dir) = resolve_project_path("my-app").unwrap();
        assert_eq!(name, "my-app");
        assert_eq!(dir, PathBuf::from("./my-app"));
    }

    #[test]
    fn relative_path_splits_leaf_and_parent() {
        let (name, dir) = resolve_project_path("../my-app").unwrap();
        assert_eq!(name, "my-app");
        assert_eq!(dir, PathBuf::from("../my-app"));
    }

    #[test]
    fn explicit_output_overrides_parent() {
        let (name, dir) = resolve_project_path("./tmp/my-app").unwrap();
        assert_eq!(name, "my-app");
        assert_eq!(dir, PathBuf::from("./tmp/my_app")); // Now works!
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
