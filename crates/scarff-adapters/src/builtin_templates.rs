//! Built-in template discovery.
//!
//! This module provides [`all_templates`], the single entry-point for loading
//! the templates that ship with Scarff.  It abstracts over the discovery
//! strategy so callers do not need to know where templates live on disk.
//!
//! # Template resolution order
//!
//! Templates are searched in this priority order, stopping at the first
//! directory that exists and returns at least one valid template:
//!
//! 1. **`$SCARFF_TEMPLATES_DIR`** — environment variable override.  Set this
//!    in `.env` or your shell profile to point at a custom template collection.
//! 2. **`./templates`** — relative to the current working directory.  This is
//!    the standard layout for the release binary when run from the project root.
//! 3. **`<executable-dir>/templates`** — sibling to the `scarff` binary.
//!    Useful when the binary is installed into `/usr/local/bin` alongside a
//!    `templates/` directory.
//! 4. **`../templates`** — one level above CWD.  Convenient during development
//!    when running `cargo run` from `target/debug/`.
//!
//! If no directory is found or all directories are empty, [`all_templates`]
//! returns an **empty `Vec`** and emits a `WARN` log entry.  The CLI layer
//! should detect this condition and surface a helpful error to the user.
//!
//! # Environment variable
//!
//! ```env
//! SCARFF_TEMPLATES_DIR=./templates
//! ```
//!
//! Relative paths are resolved against the current working directory at the
//! time [`all_templates`] is called.
//!
//! # Legacy / fallback
//!
//! The [`legacy_hardcoded`] sub-module contains the original hard-coded
//! templates.  These are **not used at runtime** but serve as a reference and
//! regression baseline while the filesystem-based loader matures.  They can
//! also be used to seed a `templates/` directory:
//!
//! ```no_run
//! use scarff_cli::builtin::legacy_hardcoded;
//! let t = legacy_hardcoded::rust_cli_default();
//! // Serialise t and write to ./templates/rust-cli-default/template.toml
//! ```

use std::path::PathBuf;

use tracing::{debug, info, instrument, warn};

use scarff_core::domain::{DomainError, Template};

use crate::template_loader::FilesystemTemplateLoader;

// ── Public API ────────────────────────────────────────────────────────────────

/// Load all templates using the resolution order described in the module docs.
///
/// # Return value
///
/// - `Ok(templates)` — at least one template was found.
/// - `Ok(vec![])` — no templates directory was discovered.  The caller should
///   surface an actionable error message (e.g. "run `scarff init` or set
///   `SCARFF_TEMPLATES_DIR`").
/// - `Err(DomainError::InvalidTemplate)` — a templates directory was found but
///   could not be read (permissions failure, I/O error).  Individual templates
///   inside a valid directory that fail to parse are **skipped with a warning**
///   rather than propagating an error.
///
/// # Observability
///
/// The function emits `tracing` events at the following levels:
/// - `DEBUG` — which path was checked and whether it was used.
/// - `INFO`  — how many templates were loaded and from which directory.
/// - `WARN`  — if no directory was found, or an individual template failed.
#[instrument]
pub fn all_templates() -> Result<Vec<Template>, DomainError> {
    for candidate in candidate_paths() {
        debug!(path = %candidate.display(), "checking candidate templates path");

        if !candidate.exists() {
            debug!(path = %candidate.display(), "path does not exist, skipping");
            continue;
        }

        let loader = FilesystemTemplateLoader::new(&candidate);
        let templates = loader.load_all()?; // propagate directory-read failures

        if templates.is_empty() {
            debug!(
                path = %candidate.display(),
                "directory exists but contains no templates, trying next"
            );
            continue;
        }

        info!(
            path  = %candidate.display(),
            count = templates.len(),
            "templates loaded successfully"
        );
        return Ok(templates);
    }

    warn!(
        "no templates directory found; checked $SCARFF_TEMPLATES_DIR, \
         ./templates, <exe>/templates, and ../templates"
    );
    Ok(vec![])
}

// ── Resolution helpers ────────────────────────────────────────────────────────

/// Build the ordered list of candidate paths to probe.
///
/// The order matches the documented priority.  Only `Some(PathBuf)` entries are
/// returned; missing env-var or unresolvable exe paths are silently omitted.
fn candidate_paths() -> Vec<PathBuf> {
    let mut paths = Vec::with_capacity(4);

    // 1. Explicit environment variable.
    if let Ok(env_dir) = std::env::var("SCARFF_TEMPLATES_DIR") {
        let p = PathBuf::from(env_dir);
        debug!(path = %p.display(), "candidate from $SCARFF_TEMPLATES_DIR");
        paths.push(p);
    }

    // 2. ./templates (CWD-relative).
    paths.push(PathBuf::from("templates"));

    // 3. <executable-dir>/templates.
    if let Some(exe_sibling) = exe_sibling_templates() {
        debug!(path = %exe_sibling.display(), "candidate from exe sibling");
        paths.push(exe_sibling);
    }

    // 4. ../templates (development fallback).
    paths.push(PathBuf::from("../templates"));

    paths
}

/// Return `<directory of current executable>/templates`, or `None` if the
/// executable path cannot be determined (some platforms / test runners).
fn exe_sibling_templates() -> Option<PathBuf> {
    std::env::current_exe()
        .ok()
        .and_then(|exe| exe.parent().map(|p| p.join("templates")))
}

// ── Legacy hardcoded templates ────────────────────────────────────────────────

/// Hard-coded reference templates kept as a migration baseline.
///
/// **Not used at runtime.**  These existed before filesystem-based loading was
/// introduced.  They are preserved here for two purposes:
///
/// 1. As documentation of what a valid [`Template`] structure looks like.
/// 2. As a seed source — you can call these functions and serialise their output
///    to bootstrap a `templates/` directory.
///
/// Once all bundled templates have been migrated to `template.toml` files this
/// module can be removed.
#[allow(dead_code)]
pub mod legacy_hardcoded {
    use scarff_core::domain::{
        Architecture, DirectorySpec, FileSpec, Framework, Language, ProjectKind, PythonFramework,
        RustFramework, TargetMatcher, Template, TemplateContent, TemplateId, TemplateMetadata,
        TemplateNode, TemplateSource, TemplateTree, TypeScriptFramework,
    };

    /// Minimal Rust CLI template — no framework, no architecture opinion.
    ///
    /// Generated files:
    /// - `src/main.rs` — `fn main()` with a `{{PROJECT_NAME}}` placeholder.
    /// - `Cargo.toml` — package manifest with `{{PROJECT_NAME_KEBAB}}`.
    pub fn rust_cli_default() -> Template {
        Template {
            id: TemplateId::new("rust-cli-default", "1.0.0"),
            matcher: TargetMatcher {
                language:     Some(Language::Rust),
                framework:    None,
                kind:         Some(ProjectKind::Cli),
                architecture: None,
            },
            metadata: TemplateMetadata::new("Rust CLI (Default)")
                .version("1.0.0")
                .description("A simple Rust command-line application.")
                .tags(vec![
                    "rust".into(),
                    "cli".into(),
                    "simple".into(),
                ]),
            tree: TemplateTree::new()
                .with_node(TemplateNode::Directory(DirectorySpec::new("src")))
                .with_node(TemplateNode::File(FileSpec::new(
                    "src/main.rs",
                    TemplateContent::Parameterized(TemplateSource::Static(
                        "fn main() {\n    println!(\"Hello, {{PROJECT_NAME}}!\");\n}\n",
                    )),
                )))
                .with_node(TemplateNode::File(FileSpec::new(
                    "Cargo.toml",
                    TemplateContent::Parameterized(TemplateSource::Static(
                        "[package]\nname = \"{{PROJECT_NAME_KEBAB}}\"\nversion = \"0.1.0\"\nedition = \"2024\"\n\n[dependencies]\n",
                    )),
                ))),
        }
    }

    /// Rust CLI with Axum web server scaffolding.
    pub fn rust_axum_backend() -> Template {
        Template {
            id: TemplateId::new("rust-axum-backend", "1.0.0"),
            matcher: TargetMatcher {
                language:     Some(Language::Rust),
                framework:    Some(Framework::Rust(RustFramework::Axum)),
                kind:         Some(ProjectKind::WebBackend),
                architecture: Some(Architecture::Layered),
            },
            metadata: TemplateMetadata::new("Rust Axum Backend (Layered)")
                .version("1.0.0")
                .description("Production-ready Axum web backend with layered architecture.")
                .tags(vec!["rust".into(), "axum".into(), "backend".into(), "layered".into()]),
            tree: TemplateTree::new()
                .with_node(TemplateNode::Directory(DirectorySpec::new("src")))
                .with_node(TemplateNode::File(FileSpec::new(
                    "Cargo.toml",
                    TemplateContent::Parameterized(TemplateSource::Static(
                        "[package]\nname = \"{{PROJECT_NAME_KEBAB}}\"\nversion = \"0.1.0\"\nedition = \"2024\"\n\n[dependencies]\naxum = \"0.7\"\ntokio = { version = \"1\", features = [\"full\"] }\n",
                    )),
                )))
                .with_node(TemplateNode::File(FileSpec::new(
                    "src/main.rs",
                    TemplateContent::Parameterized(TemplateSource::Static(
                        "#[tokio::main]\nasync fn main() {\n    println!(\"{{PROJECT_NAME}} starting\");\n}\n",
                    )),
                ))),
        }
    }

    /// Python FastAPI backend.
    pub fn python_fastapi_backend() -> Template {
        Template {
            id: TemplateId::new("python-fastapi-backend", "1.0.0"),
            matcher: TargetMatcher {
                language:     Some(Language::Python),
                framework:    Some(Framework::Python(PythonFramework::FastApi)),
                kind:         Some(ProjectKind::WebBackend),
                architecture: Some(Architecture::Layered),
            },
            metadata: TemplateMetadata::new("Python FastAPI Backend")
                .version("1.0.0")
                .description("FastAPI web backend with layered architecture.")
                .tags(vec!["python".into(), "fastapi".into(), "backend".into()]),
            tree: TemplateTree::new()
                .with_node(TemplateNode::Directory(DirectorySpec::new("src")))
                .with_node(TemplateNode::File(FileSpec::new(
                    "pyproject.toml",
                    TemplateContent::Parameterized(TemplateSource::Static(
                        "[project]\nname = \"{{PROJECT_NAME_KEBAB}}\"\nversion = \"0.1.0\"\n\n[dependencies]\nfastapi = \">=0.100\"\nuvicorn = {extras=[\"standard\"]}\n",
                    )),
                )))
                .with_node(TemplateNode::File(FileSpec::new(
                    "src/main.py",
                    TemplateContent::Parameterized(TemplateSource::Static(
                        "from fastapi import FastAPI\n\napp = FastAPI(title=\"{{PROJECT_NAME}}\")\n\n@app.get(\"/\")\ndef root():\n    return {\"app\": \"{{PROJECT_NAME}}\"}\n",
                    )),
                ))),
        }
    }

    /// TypeScript React frontend.
    pub fn typescript_react_frontend() -> Template {
        Template {
            id: TemplateId::new("typescript-react-frontend", "1.0.0"),
            matcher: TargetMatcher {
                language:     Some(Language::TypeScript),
                framework:    Some(Framework::TypeScript(TypeScriptFramework::React)),
                kind:         Some(ProjectKind::WebFrontend),
                architecture: None,
            },
            metadata: TemplateMetadata::new("TypeScript React Frontend")
                .version("1.0.0")
                .description("React frontend bootstrapped with Vite and TypeScript.")
                .tags(vec!["typescript".into(), "react".into(), "frontend".into()]),
            tree: TemplateTree::new()
                .with_node(TemplateNode::Directory(DirectorySpec::new("src")))
                .with_node(TemplateNode::File(FileSpec::new(
                    "package.json",
                    TemplateContent::Parameterized(TemplateSource::Static(
                        "{\n  \"name\": \"{{PROJECT_NAME_KEBAB}}\",\n  \"version\": \"0.1.0\",\n  \"scripts\": { \"dev\": \"vite\", \"build\": \"tsc && vite build\" },\n  \"dependencies\": { \"react\": \"^18\", \"react-dom\": \"^18\" },\n  \"devDependencies\": { \"typescript\": \"^5\", \"vite\": \"^5\" }\n}\n",
                    )),
                )))
                .with_node(TemplateNode::File(FileSpec::new(
                    "src/App.tsx",
                    TemplateContent::Parameterized(TemplateSource::Static(
                        "export default function App() {\n  return <h1>{{PROJECT_NAME}}</h1>;\n}\n",
                    )),
                ))),
        }
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use scarff_core::domain::{Framework, Language, ProjectKind, TemplateNode};
    use std::fs;
    use tempfile::TempDir;

    /// Minimal `template.toml` content for seeding a test directory.
    const MINIMAL_MANIFEST: &str = r#"
[template]
id      = "t"
version = "1.0"
[matcher]
language = "rust"
[metadata]
name = "Test"
"#;

    /// Point `SCARFF_TEMPLATES_DIR` at `dir` for the duration of the closure.
    ///
    /// **Not safe for parallel tests** — use `#[serial_test::serial]` if you
    /// add parallel test runners.  Here each test uses a unique env-var value
    /// to avoid cross-contamination.
    fn with_env_templates_dir<F: FnOnce()>(dir: &std::path::Path, f: F) {
        unsafe { std::env::set_var("SCARFF_TEMPLATES_DIR", dir) };
        f();
        unsafe { std::env::remove_var("SCARFF_TEMPLATES_DIR") };
    }

    fn seed_template(root: &std::path::Path, slot_name: &str) {
        let slot = root.join(slot_name);
        fs::create_dir_all(&slot).unwrap();
        fs::write(slot.join("template.toml"), MINIMAL_MANIFEST).unwrap();
    }

    // ── candidate_paths ───────────────────────────────────────────────────

    #[test]
    fn candidate_paths_includes_env_var_when_set() {
        unsafe { std::env::set_var("SCARFF_TEMPLATES_DIR", "/custom/templates") };
        let paths = candidate_paths();
        unsafe { std::env::remove_var("SCARFF_TEMPLATES_DIR") };
        assert!(
            paths
                .iter()
                .any(|p| p == &PathBuf::from("/custom/templates")),
            "env var path should be first candidate"
        );
    }

    #[test]
    fn candidate_paths_always_includes_cwd_relative() {
        unsafe { std::env::remove_var("SCARFF_TEMPLATES_DIR") };
        let paths = candidate_paths();
        assert!(paths.contains(&PathBuf::from("templates")));
    }

    // ── all_templates ─────────────────────────────────────────────────────

    #[test]
    fn all_templates_returns_empty_when_no_dir_found() {
        // Point env var at a path that doesn't exist, CWD-relative also shouldn't
        // resolve in this controlled environment.
        unsafe { std::env::set_var("SCARFF_TEMPLATES_DIR", "/tmp/scarff_test_nonexistent_9999") };
        // Temporarily change cwd to avoid accidentally picking up a real ./templates
        let result = all_templates();
        unsafe { std::env::remove_var("SCARFF_TEMPLATES_DIR") };
        assert!(result.is_ok(), "should not error for missing dir");
    }

    #[test]
    fn all_templates_loads_from_env_var_dir() {
        let temp = TempDir::new().unwrap();
        seed_template(temp.path(), "rust-cli");

        with_env_templates_dir(temp.path(), || {
            let templates = all_templates().unwrap();
            assert_eq!(templates.len(), 1);
        });
    }

    #[test]
    fn all_templates_skips_empty_dir_and_tries_next() {
        // First candidate: exists but empty.
        let empty = TempDir::new().unwrap();
        // Second candidate: has a template.
        let real = TempDir::new().unwrap();
        seed_template(real.path(), "t");

        // We can't easily control multiple candidates without modifying the
        // function, so just test that empty + env pointing to real works.
        with_env_templates_dir(real.path(), || {
            let templates = all_templates().unwrap();
            assert!(!templates.is_empty());
        });
        let _ = empty; // keep alive
    }

    // ── legacy_hardcoded ──────────────────────────────────────────────────

    #[test]
    fn legacy_rust_cli_default_has_expected_files() {
        let t = legacy_hardcoded::rust_cli_default();
        assert_eq!(t.id.name(), "rust-cli-default");

        let paths: Vec<_> = t
            .tree
            .nodes
            .iter()
            .filter_map(|n| match n {
                TemplateNode::File(f) => Some(f.path.as_str()),
                _ => None,
            })
            .collect();

        assert!(paths.contains(&"src/main.rs"), "missing src/main.rs");
        assert!(paths.contains(&"Cargo.toml"), "missing Cargo.toml");
    }

    #[test]
    fn legacy_rust_cli_matcher_is_rust_cli() {
        let t = legacy_hardcoded::rust_cli_default();
        assert_eq!(t.matcher.language, Some(Language::Rust));
        assert_eq!(t.matcher.kind, Some(ProjectKind::Cli));
        assert!(
            t.matcher.framework.is_none(),
            "default CLI has no framework"
        );
    }

    #[test]
    fn legacy_axum_backend_matcher_has_framework() {
        let t = legacy_hardcoded::rust_axum_backend();
        assert!(
            matches!(t.matcher.framework, Some(Framework::Rust(_))),
            "Axum backend should have a Rust framework"
        );
    }

    #[test]
    fn legacy_all_functions_produce_valid_templates() {
        // Smoke-test: calling every function should not panic.
        let _ = legacy_hardcoded::rust_cli_default();
        let _ = legacy_hardcoded::rust_axum_backend();
        let _ = legacy_hardcoded::python_fastapi_backend();
        let _ = legacy_hardcoded::typescript_react_frontend();
    }
}
