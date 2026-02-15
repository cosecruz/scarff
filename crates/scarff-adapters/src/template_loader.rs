//! Filesystem-based template loader.
//!
//! Discovers and parses `template.toml` manifests from a directory tree,
//! converting them into domain [`Template`] objects ready for use by the
//! scaffold orchestrator.
//!
//! # Directory layout expected
//!
//! ```text
//! templates/
//! ├── rust-cli-layered/
//! │   ├── template.toml        ← manifest (required)
//! │   ├── src/
//! │   │   └── main.rs          ← file content
//! │   └── Cargo.toml
//! └── python-backend/
//!     ├── template.toml
//!     └── src/
//!         └── main.py
//! ```
//!
//! # `template.toml` format
//!
//! ```toml
//! [template]
//! id      = "rust-cli-layered"   # unique identifier
//! version = "1.0.0"
//!
//! [matcher]
//! language     = "rust"          # rust | python | typescript | go
//! kind         = "cli"           # cli | webbackend | webfrontend | library
//! architecture = "layered"       # layered | clean | mvc | modular
//! framework    = "Rust:Axum"     # optional; format: "Language:Name"
//!
//! [metadata]
//! name        = "Rust CLI (Layered)"
//! description = "Production-ready Rust CLI."   # optional
//! author      = "Scarff"                        # optional
//! tags        = ["rust", "cli"]                 # optional
//!
//! # Optional: override per-file content type.
//! # If omitted, files containing {{ }} are auto-detected as parameterized.
//! [[files]]
//! path        = "LICENSE"
//! type        = "external"       # literal | parameterized | external
//! external_id = "builtin:mit"    # required when type = "external"
//! ```

use std::{
    collections::{HashMap, HashSet},
    fs,
    path::{Path, PathBuf},
    sync::OnceLock,
};

use serde::Deserialize;
use tracing::{debug, instrument, warn};
use walkdir::WalkDir;

use scarff_core::domain::{
    Architecture, ContentTemplateId, DirectorySpec, DomainError, FileSpec, Framework, Language,
    ProjectKind, RelativePath, TargetMatcher, Template, TemplateBuilder, TemplateContent,
    TemplateId, TemplateMetadata, TemplateNode, TemplateSource, TemplateTree,
};

// ── String interning ──────────────────────────────────────────────────────────

// `ContentTemplateId` requires a `&'static str`.  Since external IDs come from
// runtime TOML data we need to promote them to `'static` lifetime.
//
// We do this with a global string intern table (`OnceLock<Mutex<HashSet>>`) so
// that each unique string is leaked **at most once**, and repeated calls to
// `load_all()` (e.g. in tests or watch mode) re-use the same allocation.
//
// Trade-off: the leaked strings live for the process lifetime.  For a CLI tool
// that loads templates once at startup this is acceptable; for a long-running
// server the `ContentTemplateId` API should be changed to own a `String`.

use std::sync::Mutex;

fn intern(s: &str) -> &'static str {
    static TABLE: OnceLock<Mutex<HashSet<&'static str>>> = OnceLock::new();
    let table = TABLE.get_or_init(|| Mutex::new(HashSet::new()));
    let mut guard = table.lock().expect("string intern table poisoned");

    // Re-use an existing &'static str if we already interned this value.
    if let Some(&existing) = guard.get(s) {
        return existing;
    }

    // First time: leak exactly one allocation.
    let leaked: &'static str = Box::leak(s.to_owned().into_boxed_str());
    guard.insert(leaked);
    leaked
}

// ── Manifest types ────────────────────────────────────────────────────────────

/// Deserialised representation of a `template.toml` file.
///
/// All fields map 1-to-1 to TOML sections; see the module-level docs for the
/// full format.
#[derive(Debug, Deserialize, Clone)]
pub struct TemplateManifest {
    pub template: TemplateSection,
    pub matcher: MatcherSection,
    pub metadata: MetadataSection,
    /// Explicit per-file type overrides.  Files not listed here are
    /// auto-detected: content containing `{{` … `}}` is [`FileType::Parameterized`],
    /// everything else is [`FileType::Literal`].
    pub files: Option<Vec<FileEntry>>,
    /// Directories that must exist even if they contain no tracked files.
    pub directories: Option<Vec<DirectoryEntry>>,
}

/// `[template]` section — identity of the template.
#[derive(Debug, Deserialize, Clone)]
pub struct TemplateSection {
    /// Unique slug, e.g. `"rust-cli-layered"`.
    pub id: String,
    /// SemVer string, e.g. `"1.0.0"`.
    pub version: String,
}

/// `[matcher]` section — criteria used to select this template.
///
/// All fields are optional; omitting a field means "match any value".
#[derive(Debug, Deserialize, Clone)]
pub struct MatcherSection {
    /// Target language (e.g. `"rust"`, `"python"`, `"typescript"`, `"go"`).
    pub language: Option<String>,
    /// Framework in `"Language:Name"` format (e.g. `"Rust:Axum"`).
    pub framework: Option<String>,
    /// Project kind (e.g. `"cli"`, `"webbackend"`, `"webfrontend"`, `"library"`).
    pub kind: Option<String>,
    /// Architecture pattern (e.g. `"layered"`, `"clean"`, `"mvc"`, `"modular"`).
    pub architecture: Option<String>,
}

/// `[metadata]` section — human-facing information about the template.
#[derive(Debug, Deserialize, Clone)]
pub struct MetadataSection {
    /// Display name shown in `scarff list`.
    pub name: String,
    pub description: Option<String>,
    pub author: Option<String>,
    /// Free-form tags used for filtering and search.
    pub tags: Option<Vec<String>>,
}

/// One entry under `[[files]]`.
#[derive(Debug, Deserialize, Clone)]
pub struct FileEntry {
    /// Relative path from the template root (e.g. `"src/main.rs"`).
    pub path: String,
    /// Content handling strategy (see [`FileType`]).
    #[serde(rename = "type")]
    pub file_type: FileType,
    /// Required when `type = "external"` — identifies the built-in content to
    /// embed (e.g. `"builtin:mit"`).
    pub external_id: Option<String>,
}

/// Controls how a file's content is treated during scaffolding.
#[derive(Debug, Deserialize, Clone, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum FileType {
    /// Content is copied verbatim — no variable substitution.
    Literal,
    /// Content contains `{{VARIABLE}}` placeholders that are substituted at
    /// scaffold time.
    Parameterized,
    /// Content is not stored on disk; it is fetched from a named built-in
    /// content registry entry identified by `external_id`.
    External,
}

/// One entry under `[[directories]]`.
#[derive(Debug, Deserialize, Clone)]
pub struct DirectoryEntry {
    /// Relative path (e.g. `"src/generated"`).
    pub path: String,
}

// ── Loader ────────────────────────────────────────────────────────────────────

/// Loads [`Template`] objects from a directory tree of `template.toml` manifests.
///
/// Each immediate subdirectory of `templates_dir` that contains a valid
/// `template.toml` is treated as one template.  Subdirectories that are missing
/// `template.toml`, or whose manifest is invalid, emit a `WARN` log and are
/// skipped — they do not prevent other templates from loading.
///
/// # Example
///
/// ```no_run
/// use scarff_cli::template_loader::FilesystemTemplateLoader;
///
/// let loader = FilesystemTemplateLoader::new("./templates");
/// let templates = loader.load_all()?;
/// println!("Loaded {} templates", templates.len());
/// # Ok::<(), scarff_core::domain::DomainError>(())
/// ```
pub struct FilesystemTemplateLoader {
    templates_dir: PathBuf,
}

impl FilesystemTemplateLoader {
    /// Create a loader pointed at `templates_dir`.
    ///
    /// The directory does not need to exist yet; [`load_all`] will return an
    /// error if it is missing when called.
    pub fn new(templates_dir: impl Into<PathBuf>) -> Self {
        Self {
            templates_dir: templates_dir.into(),
        }
    }

    /// Load every valid template found under [`templates_dir`].
    ///
    /// # Errors
    ///
    /// Returns [`DomainError::InvalidTemplate`] if:
    /// - `templates_dir` does not exist.
    /// - `templates_dir` cannot be read (permissions, I/O failure).
    ///
    /// Individual template directories whose `template.toml` is missing or
    /// malformed are **skipped with a `WARN` log** rather than failing the
    /// whole batch.
    #[instrument(skip(self), fields(dir = %self.templates_dir.display()))]
    pub fn load_all(&self) -> Result<Vec<Template>, DomainError> {
        if !self.templates_dir.exists() {
            return Err(DomainError::InvalidTemplate(format!(
                "templates directory not found: {}",
                self.templates_dir.display()
            )));
        }

        let read_dir = fs::read_dir(&self.templates_dir).map_err(|e| {
            DomainError::InvalidTemplate(format!(
                "failed to read templates directory '{}': {e}",
                self.templates_dir.display()
            ))
        })?;

        let mut templates = Vec::new();

        for entry_result in read_dir {
            let entry = entry_result.map_err(|e| {
                DomainError::InvalidTemplate(format!("failed to read directory entry: {e}"))
            })?;

            let path = entry.path();
            if !path.is_dir() {
                continue; // Only process subdirectories.
            }

            match self.load_template_from_dir(&path) {
                Ok(template) => {
                    debug!(
                        id      = %template.id.name(),
                        version = %template.id.version(),
                        "loaded template"
                    );
                    templates.push(template);
                }
                Err(e) => {
                    // One bad template must not block all others.
                    warn!(
                        dir   = %path.display(),
                        error = %e,
                        "skipping template directory due to load error"
                    );
                }
            }
        }

        debug!(count = templates.len(), "finished loading templates");
        Ok(templates)
    }

    /// Load a single template from one subdirectory.
    ///
    /// # Errors
    ///
    /// Returns an error if `template.toml` is missing, unparseable, or if any
    /// file referenced in the manifest cannot be read.
    #[instrument(skip(self), fields(dir = %dir.display()))]
    fn load_template_from_dir(&self, dir: &Path) -> Result<Template, DomainError> {
        let manifest_path = dir.join("template.toml");
        if !manifest_path.exists() {
            return Err(DomainError::InvalidTemplate(format!(
                "missing template.toml in '{}'",
                dir.display()
            )));
        }

        let raw = fs::read_to_string(&manifest_path).map_err(|e| {
            DomainError::InvalidTemplate(format!(
                "failed to read '{}': {e}",
                manifest_path.display()
            ))
        })?;

        let manifest: TemplateManifest = toml::from_str(&raw).map_err(|e| {
            DomainError::InvalidTemplate(format!(
                "failed to parse '{}': {e}",
                manifest_path.display()
            ))
        })?;

        let id = TemplateId::new(&manifest.template.id, &manifest.template.version);
        let matcher = self.parse_matcher(&manifest.matcher)?;
        let tree = self.build_tree_from_dir(dir, &manifest)?;

        let metadata = TemplateMetadata::new(manifest.metadata.name)
            .description(manifest.metadata.description.unwrap_or_default())
            .version(manifest.template.version)
            .author(manifest.metadata.author.unwrap_or_else(|| "Scarff".into()))
            .tags(manifest.metadata.tags.unwrap_or_default());

        TemplateBuilder::default()
            .id(id)
            .matcher(matcher)
            .metadata(metadata)
            .tree(tree)
            .build()
    }

    /// Walk `dir` and build a [`TemplateTree`] according to the manifest.
    ///
    /// Resolution order:
    /// 1. Explicit `[[directories]]` entries from the manifest (guaranteed to
    ///    exist even if empty on disk).
    /// 2. `[[files]]` entries whose `type = "external"` (they have no disk
    ///    representation).
    /// 3. Every file and directory found by walking the directory tree,
    ///    deduplicating against anything already added.
    fn build_tree_from_dir(
        &self,
        dir: &Path,
        manifest: &TemplateManifest,
    ) -> Result<TemplateTree, DomainError> {
        let mut tree = TemplateTree::new();
        // Track paths we have already committed so we never push duplicates.
        let mut added_paths: HashSet<String> = HashSet::new();

        // ── 1. Manifest-declared directories ─────────────────────────────
        if let Some(dirs) = &manifest.directories {
            for entry in dirs {
                let path = normalize_path(&entry.path);
                if added_paths.insert(path.clone()) {
                    tree.push(TemplateNode::Directory(DirectorySpec::new(
                        RelativePath::new(path),
                    )));
                }
            }
        }

        // ── 2. Build fast-lookup maps for manifest [[files]] ──────────────
        //
        // Normalize all paths to forward-slashes so Windows and Unix paths
        // compare correctly.
        let manifest_files: HashMap<String, &FileEntry> = manifest
            .files
            .as_ref()
            .map(|files| files.iter().map(|f| (normalize_path(&f.path), f)).collect())
            .unwrap_or_default();

        // ── 3. Walk the directory tree ────────────────────────────────────
        for walk_entry in WalkDir::new(dir).min_depth(1) {
            let walk_entry = walk_entry
                .map_err(|e| DomainError::InvalidTemplate(format!("directory walk error: {e}")))?;
            let abs_path = walk_entry.path();
            let rel_raw = abs_path.strip_prefix(dir).map_err(|_| {
                DomainError::InvalidTemplate(format!(
                    "failed to relativise '{}' against '{}'",
                    abs_path.display(),
                    dir.display()
                ))
            })?;

            // template.toml is a loader artefact, not a project file.
            if rel_raw.file_name() == Some(std::ffi::OsStr::new("template.toml")) {
                continue;
            }

            let path_str = normalize_path(&rel_raw.to_string_lossy());

            if walk_entry.file_type().is_dir() {
                // Only emit directory nodes not already declared in the manifest.
                if added_paths.insert(path_str.clone()) {
                    tree.push(TemplateNode::Directory(DirectorySpec::new(
                        RelativePath::new(path_str),
                    )));
                }
                continue;
            }

            if !walk_entry.file_type().is_file() {
                continue; // Skip symlinks and other special types.
            }

            let content = fs::read_to_string(abs_path).map_err(|e| {
                DomainError::InvalidTemplate(format!("failed to read file '{path_str}': {e}"))
            })?;

            let template_content = self.resolve_file_content(
                &path_str,
                content,
                manifest_files.get(&path_str).copied(),
            )?;

            if added_paths.insert(path_str.clone()) {
                tree.push(TemplateNode::File(FileSpec::new(
                    RelativePath::new(path_str),
                    template_content,
                )));
            }
        }

        // ── 4. External-only files (no disk representation) ───────────────
        //
        // These exist only in the manifest `[[files]]` with `type = "external"`.
        // Since WalkDir never found them on disk they were not added above.
        if let Some(file_entries) = &manifest.files {
            for entry in file_entries
                .iter()
                .filter(|e| e.file_type == FileType::External)
            {
                let path_str = normalize_path(&entry.path);
                if added_paths.contains(&path_str) {
                    // Was found on disk (unusual but handled above).
                    continue;
                }
                let ext_id = entry.external_id.as_deref().ok_or_else(|| {
                    DomainError::InvalidTemplate(format!(
                        "external file '{path_str}' is missing required external_id"
                    ))
                })?;
                // Safe: intern() guarantees each unique string is leaked once.
                let static_id = intern(ext_id);
                tree.push(TemplateNode::File(FileSpec::new(
                    RelativePath::new(path_str.clone()),
                    TemplateContent::External(ContentTemplateId(static_id)),
                )));
                added_paths.insert(path_str);
            }
        }

        Ok(tree)
    }

    /// Determine the [`TemplateContent`] for one file.
    ///
    /// If the file appears in the manifest `[[files]]` section its explicit
    /// `type` field wins.  Otherwise content is auto-detected: files containing
    /// `{{` are [`TemplateContent::Parameterized`]; everything else is
    /// [`TemplateContent::Literal`].
    fn resolve_file_content(
        &self,
        path_str: &str,
        content: String,
        manifest_entry: Option<&FileEntry>,
    ) -> Result<TemplateContent, DomainError> {
        match manifest_entry {
            Some(entry) => match entry.file_type {
                FileType::Literal => Ok(TemplateContent::Literal(TemplateSource::from(content))),
                FileType::Parameterized => Ok(TemplateContent::Parameterized(
                    TemplateSource::from(content),
                )),
                FileType::External => {
                    // External files found on disk still use their manifest ID.
                    let ext_id = entry.external_id.as_deref().ok_or_else(|| {
                        DomainError::InvalidTemplate(format!(
                            "external file '{path_str}' is missing required external_id"
                        ))
                    })?;
                    Ok(TemplateContent::External(ContentTemplateId(intern(ext_id))))
                }
            },
            // Auto-detect: presence of {{ … }} marks the file as parameterized.
            None => {
                if content.contains("{{") {
                    Ok(TemplateContent::Parameterized(TemplateSource::from(
                        content,
                    )))
                } else {
                    Ok(TemplateContent::Literal(TemplateSource::from(content)))
                }
            }
        }
    }

    // ── Parsing helpers ───────────────────────────────────────────────────────

    /// Convert the `[matcher]` section into a [`TargetMatcher`].
    fn parse_matcher(&self, section: &MatcherSection) -> Result<TargetMatcher, DomainError> {
        let mut builder = TargetMatcher::builder();

        if let Some(s) = &section.language {
            builder = builder.language(parse_language(s)?);
        }
        if let Some(s) = &section.framework {
            builder = builder.framework(parse_framework(s)?);
        }
        if let Some(s) = &section.kind {
            builder = builder.kind(parse_project_kind(s)?);
        }
        if let Some(s) = &section.architecture {
            builder = builder.architecture(parse_architecture(s)?);
        }

        Ok(builder.build())
    }
}

// ── Free parsing functions ────────────────────────────────────────────────────
// These are `fn` rather than methods because they don't need `&self` and are
// easier to unit-test in isolation.

/// Parse a language string from a `template.toml` `[matcher]` section.
///
/// Valid values (case-insensitive): `rust`, `python`, `typescript`, `go`.
pub fn parse_language(s: &str) -> Result<Language, DomainError> {
    match s.to_lowercase().as_str() {
        "rust" => Ok(Language::Rust),
        "python" => Ok(Language::Python),
        "typescript" => Ok(Language::TypeScript),
        "go" => Ok(Language::Go),
        _ => Err(DomainError::InvalidTemplate(format!(
            "unknown language '{s}'; expected one of: rust, python, typescript, go"
        ))),
    }
}

/// Parse a framework string in `"Language:Name"` format.
///
/// # Examples
/// - `"Rust:Axum"` → [`Framework::Rust(RustFramework::Axum)`]
/// - `"Python:FastApi"` → [`Framework::Python(PythonFramework::FastApi)`]
pub fn parse_framework(s: &str) -> Result<Framework, DomainError> {
    let (lang_part, fw_part) = s.split_once(':').ok_or_else(|| {
        DomainError::InvalidTemplate(format!(
            "framework must be in 'Language:Name' format, got '{s}'"
        ))
    })?;

    use scarff_core::domain::value_objects::{PythonFramework, RustFramework, TypeScriptFramework};

    let lang = parse_language(lang_part)?;
    let fw_low = fw_part.to_lowercase();

    match (lang, fw_low.as_str()) {
        (Language::Rust, "axum") => Ok(Framework::Rust(RustFramework::Axum)),
        (Language::Rust, "actix") => Ok(Framework::Rust(RustFramework::Actix)),
        (Language::Python, "fastapi") => Ok(Framework::Python(PythonFramework::FastApi)),
        (Language::Python, "django") => Ok(Framework::Python(PythonFramework::Django)),
        (Language::TypeScript, "react") => Ok(Framework::TypeScript(TypeScriptFramework::React)),
        (Language::TypeScript, "vue") => Ok(Framework::TypeScript(TypeScriptFramework::Vue)),
        (Language::TypeScript, "express") => {
            Ok(Framework::TypeScript(TypeScriptFramework::Express))
        }
        (Language::TypeScript, "nestjs") => Ok(Framework::TypeScript(TypeScriptFramework::NestJs)),
        _ => Err(DomainError::InvalidTemplate(format!(
            "unknown framework '{s}'"
        ))),
    }
}

/// Parse a project-kind string.
///
/// Valid values (case-insensitive): `cli`, `webbackend`, `webfrontend`, `library`.
pub fn parse_project_kind(s: &str) -> Result<ProjectKind, DomainError> {
    match s.to_lowercase().as_str() {
        "cli" => Ok(ProjectKind::Cli),
        "webbackend" | "web_api" => Ok(ProjectKind::WebBackend),
        "webfrontend" | "web_fe" => Ok(ProjectKind::WebFrontend),
        "library" => Ok(ProjectKind::Library),
        "worker" => Ok(ProjectKind::Worker),
        _ => Err(DomainError::InvalidTemplate(format!(
            "unknown project kind '{s}'; expected one of: cli, webbackend/web_api, webfrontend/web_fe, library"
        ))),
    }
}

/// Parse an architecture string.
///
/// Valid values (case-insensitive): `layered`, `clean`, `mvc`, `modular`.
pub fn parse_architecture(s: &str) -> Result<Architecture, DomainError> {
    match s.to_lowercase().as_str() {
        "layered" => Ok(Architecture::Layered),
        "clean" => Ok(Architecture::Clean),
        "mvc" => Ok(Architecture::Mvc),
        "modular" => Ok(Architecture::FeatureModular),
        _ => Err(DomainError::InvalidTemplate(format!(
            "unknown architecture '{s}'; expected one of: layered, clean, mvc, modular"
        ))),
    }
}

/// Normalise a filesystem path to forward slashes so Windows and Unix paths
/// compare identically throughout the loader.
fn normalize_path(path: &str) -> String {
    path.replace('\\', "/")
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use std::{fs::File, io::Write};
    use tempfile::TempDir;

    // ── helpers ───────────────────────────────────────────────────────────

    /// Write a minimal template directory under a TempDir.
    fn make_template_dir(manifest: &str, files: &[(&str, &str)]) -> TempDir {
        let temp = TempDir::new().unwrap();
        let dir = temp.path();

        File::create(dir.join("template.toml"))
            .unwrap()
            .write_all(manifest.as_bytes())
            .unwrap();

        for (rel_path, content) in files {
            let full = dir.join(rel_path);
            if let Some(parent) = full.parent() {
                fs::create_dir_all(parent).unwrap();
            }
            File::create(full)
                .unwrap()
                .write_all(content.as_bytes())
                .unwrap();
        }

        temp
    }

    /// Minimal valid manifest shared across many tests.
    const MINIMAL_MANIFEST: &str = r#"
[template]
id      = "tpl"
version = "1.0"

[matcher]
language = "rust"

[metadata]
name = "Test Template"
"#;

    // ── load_all ──────────────────────────────────────────────────────────

    #[test]
    fn load_all_returns_error_for_missing_dir() {
        let loader = FilesystemTemplateLoader::new("/absolutely/does/not/exist");
        assert!(matches!(
            loader.load_all(),
            Err(DomainError::InvalidTemplate(_))
        ));
    }

    #[test]
    fn load_all_skips_files_at_top_level() {
        // A file in templates_dir (not a subdirectory) should be silently ignored.
        let temp = TempDir::new().unwrap();
        File::create(temp.path().join("README.md")).unwrap();

        // Add one valid template so load_all succeeds.
        let tmpl_dir = temp.path().join("rust-cli");
        fs::create_dir(&tmpl_dir).unwrap();
        fs::write(tmpl_dir.join("template.toml"), MINIMAL_MANIFEST).unwrap();

        let loader = FilesystemTemplateLoader::new(temp.path());
        let templates = loader.load_all().unwrap();
        assert_eq!(templates.len(), 1);
    }

    #[test]
    fn load_all_continues_when_one_template_is_invalid() {
        let temp = TempDir::new().unwrap();

        // Bad template — no template.toml
        fs::create_dir(temp.path().join("bad")).unwrap();

        // Good template
        let good = temp.path().join("good");
        fs::create_dir(&good).unwrap();
        fs::write(good.join("template.toml"), MINIMAL_MANIFEST).unwrap();

        let loader = FilesystemTemplateLoader::new(temp.path());
        let templates = loader.load_all().unwrap();
        assert_eq!(templates.len(), 1, "bad template should be skipped");
    }

    // ── template loading ──────────────────────────────────────────────────

    #[test]
    fn loads_template_id_and_version() {
        let manifest = r#"
[template]
id      = "my-template"
version = "2.3.4"

[matcher]
language = "rust"

[metadata]
name = "My Template"
"#;
        let temp_tmpl = make_template_dir(manifest, &[]);
        // load_all requires a parent directory; wrap in another temp dir.
        let root = TempDir::new().unwrap();
        let slot = root.path().join("my-template");
        fs_copy_dir(temp_tmpl.path(), &slot);

        let loader = FilesystemTemplateLoader::new(root.path());
        let templates = loader.load_all().unwrap();
        assert_eq!(templates.len(), 1);
        assert_eq!(templates[0].id.name(), "my-template");
        assert_eq!(templates[0].id.version(), "2.3.4");
    }

    #[test]
    fn loads_full_metadata() {
        let manifest = r#"
[template]
id      = "full"
version = "1.0.0"

[matcher]
language = "python"
kind     = "webbackend"

[metadata]
name        = "Full Template"
description = "Comprehensive test"
author      = "Alice"
tags        = ["a", "b"]
"#;
        let root = TempDir::new().unwrap();
        let slot = root.path().join("full");
        let temp_tmpl = make_template_dir(manifest, &[]);
        fs_copy_dir(temp_tmpl.path(), &slot);

        let templates = FilesystemTemplateLoader::new(root.path())
            .load_all()
            .unwrap();
        let t = &templates[0];
        assert_eq!(t.metadata.name, "Full Template");
        assert_eq!(t.metadata.description, "Comprehensive test");
        assert_eq!(t.metadata.author, "Alice");
        assert_eq!(t.metadata.tags, vec!["a", "b"]);
    }

    // ── file auto-detection ───────────────────────────────────────────────

    #[test]
    fn auto_detects_parameterized_file() {
        let root = TempDir::new().unwrap();
        let slot = root.path().join("t");
        let temp_tmpl = make_template_dir(MINIMAL_MANIFEST, &[("config.toml", "port = {{PORT}}")]);
        fs_copy_dir(temp_tmpl.path(), &slot);

        let templates = FilesystemTemplateLoader::new(root.path())
            .load_all()
            .unwrap();

        let node = find_file(&templates[0], "config.toml");
        assert!(
            matches!(node.content, TemplateContent::Parameterized(_)),
            "expected Parameterized, got {:?}",
            node.content
        );
    }

    #[test]
    fn auto_detects_literal_file() {
        let root = TempDir::new().unwrap();
        let slot = root.path().join("t");
        let temp_tmpl =
            make_template_dir(MINIMAL_MANIFEST, &[("README.md", "# No placeholders here")]);
        fs_copy_dir(temp_tmpl.path(), &slot);

        let templates = FilesystemTemplateLoader::new(root.path())
            .load_all()
            .unwrap();

        let node = find_file(&templates[0], "README.md");
        assert!(
            matches!(node.content, TemplateContent::Literal(_)),
            "expected Literal, got {:?}",
            node.content
        );
    }

    #[test]
    fn manifest_override_forces_literal_even_with_braces() {
        let manifest = r#"
[template]
id = "t"
version = "1.0"

[matcher]
language = "rust"

[metadata]
name = "T"

[[files]]
path = "raw.txt"
type = "literal"
"#;
        let root = TempDir::new().unwrap();
        let slot = root.path().join("t");
        let temp_tmpl = make_template_dir(
            manifest,
            &[("raw.txt", "this has {{BRACES}} but is literal")],
        );
        fs_copy_dir(temp_tmpl.path(), &slot);

        let templates = FilesystemTemplateLoader::new(root.path())
            .load_all()
            .unwrap();

        let node = find_file(&templates[0], "raw.txt");
        assert!(
            matches!(node.content, TemplateContent::Literal(_)),
            "manifest override should force Literal"
        );
    }

    // ── external files ────────────────────────────────────────────────────

    #[test]
    fn external_file_not_on_disk_is_added_from_manifest() {
        let manifest = r#"
[template]
id = "ext"
version = "1.0"

[matcher]
language = "rust"

[metadata]
name = "Ext"

[[files]]
path        = "LICENSE"
type        = "external"
external_id = "builtin:mit"
"#;
        let root = TempDir::new().unwrap();
        let slot = root.path().join("ext");
        let temp_tmpl = make_template_dir(manifest, &[]); // no LICENSE on disk
        fs_copy_dir(temp_tmpl.path(), &slot);

        let templates = FilesystemTemplateLoader::new(root.path())
            .load_all()
            .unwrap();

        let node = find_file(&templates[0], "LICENSE");
        match &node.content {
            TemplateContent::External(id) => assert_eq!(id.0, "builtin:mit"),
            other => panic!("expected External, got {other:?}"),
        }
    }

    #[test]
    fn missing_external_id_is_an_error() {
        let manifest = r#"
[template]
id = "bad"
version = "1.0"

[matcher]
language = "rust"

[metadata]
name = "Bad"

[[files]]
path = "LICENSE"
type = "external"
"#;
        let root = TempDir::new().unwrap();
        let slot = root.path().join("bad");
        let temp_tmpl = make_template_dir(manifest, &[]);
        fs_copy_dir(temp_tmpl.path(), &slot);

        let err = FilesystemTemplateLoader::new(root.path())
            .load_all()
            .unwrap_err();

        match err {
            DomainError::InvalidTemplate(msg) => {
                assert!(msg.contains("external_id"), "msg = {msg}");
            }
            other => panic!("expected InvalidTemplate, got {other:?}"),
        }
    }

    #[test]
    fn intern_deduplicates_identical_strings() {
        let a = intern("builtin:mit");
        let b = intern("builtin:mit");
        // Same pointer — only one allocation.
        assert!(std::ptr::eq(a, b));
    }

    // ── directory structure ───────────────────────────────────────────────

    #[test]
    fn nested_directories_are_discovered() {
        let root = TempDir::new().unwrap();
        let slot = root.path().join("t");
        let temp_tmpl = make_template_dir(
            MINIMAL_MANIFEST,
            &[
                ("src/main.rs", "fn main() {}"),
                ("src/lib.rs", ""),
                ("tests/test.rs", "#[test] fn t() {}"),
            ],
        );
        fs_copy_dir(temp_tmpl.path(), &slot);

        let templates = FilesystemTemplateLoader::new(root.path())
            .load_all()
            .unwrap();
        let t = &templates[0];

        let files = t
            .tree
            .nodes
            .iter()
            .filter(|n| matches!(n, TemplateNode::File(_)))
            .count();
        let dirs = t
            .tree
            .nodes
            .iter()
            .filter(|n| matches!(n, TemplateNode::Directory(_)))
            .count();

        assert!(files >= 3, "expected ≥3 files, got {files}");
        assert!(dirs >= 2, "expected ≥2 directories, got {dirs}");
    }

    #[test]
    fn manifest_directories_not_duplicated_by_walk() {
        let manifest = r#"
[template]
id = "t"
version = "1.0"

[matcher]
language = "rust"

[metadata]
name = "T"

[[directories]]
path = "src"
"#;
        let root = TempDir::new().unwrap();
        let slot = root.path().join("t");
        let temp_tmpl = make_template_dir(manifest, &[("src/main.rs", "fn main() {}")]);
        fs_copy_dir(temp_tmpl.path(), &slot);

        let templates = FilesystemTemplateLoader::new(root.path())
            .load_all()
            .unwrap();

        let src_dir_count = templates[0]
            .tree
            .nodes
            .iter()
            .filter(|n| matches!(n, TemplateNode::Directory(d) if d.path.as_str() == "src"))
            .count();

        assert_eq!(src_dir_count, 1, "src/ must appear exactly once");
    }

    // ── parse_* helpers ───────────────────────────────────────────────────

    #[test]
    fn parse_language_case_insensitive() {
        assert!(matches!(parse_language("Rust"), Ok(Language::Rust)));
        assert!(matches!(parse_language("RUST"), Ok(Language::Rust)));
        assert!(matches!(
            parse_language("typescript"),
            Ok(Language::TypeScript)
        ));
        assert!(matches!(parse_language("go"), Ok(Language::Go)));
    }

    #[test]
    fn parse_language_unknown_is_error() {
        let err = parse_language("java").unwrap_err();
        assert!(
            matches!(err, DomainError::InvalidTemplate(msg) if msg.contains("unknown language"))
        );
    }

    #[test]
    fn parse_framework_valid() {
        assert!(matches!(
            parse_framework("Rust:Axum"),
            Ok(Framework::Rust(_))
        ));
        assert!(matches!(
            parse_framework("Python:FastApi"),
            Ok(Framework::Python(_))
        ));
        assert!(matches!(
            parse_framework("TypeScript:React"),
            Ok(Framework::TypeScript(_))
        ));
    }

    #[test]
    fn parse_framework_missing_colon_is_error() {
        assert!(parse_framework("RustAxum").is_err());
    }

    #[test]
    fn parse_framework_unknown_name_is_error() {
        assert!(parse_framework("Rust:Rocket").is_err());
    }

    #[test]
    fn parse_project_kind_valid() {
        assert!(matches!(parse_project_kind("cli"), Ok(ProjectKind::Cli)));
        assert!(matches!(
            parse_project_kind("WebBackend"),
            Ok(ProjectKind::WebBackend)
        ));
        assert!(matches!(
            parse_project_kind("library"),
            Ok(ProjectKind::Library)
        ));
    }

    #[test]
    fn parse_architecture_valid() {
        assert!(matches!(
            parse_architecture("layered"),
            Ok(Architecture::Layered)
        ));
        assert!(matches!(
            parse_architecture("CLEAN"),
            Ok(Architecture::Clean)
        ));
        assert!(matches!(
            parse_architecture("modular"),
            Ok(Architecture::FeatureModular)
        ));
    }

    #[test]
    fn normalize_path_replaces_backslashes() {
        assert_eq!(normalize_path("src\\main.rs"), "src/main.rs");
        assert_eq!(normalize_path("src/main.rs"), "src/main.rs");
        assert_eq!(normalize_path(""), "");
    }

    // ── test utilities ────────────────────────────────────────────────────

    /// Find a `FileSpec` by relative path or panic.
    fn find_file<'a>(t: &'a Template, path: &str) -> &'a FileSpec {
        t.tree
            .nodes
            .iter()
            .find_map(|n| match n {
                TemplateNode::File(f) if f.path.as_str() == path => Some(f),
                _ => None,
            })
            .unwrap_or_else(|| panic!("file '{path}' not found in tree"))
    }

    /// Recursively copy a directory (poor-man's `cp -r` for tests).
    fn fs_copy_dir(src: &Path, dst: &Path) {
        fs::create_dir_all(dst).unwrap();
        for entry in fs::read_dir(src).unwrap() {
            let entry = entry.unwrap();
            let src_p = entry.path();
            let dst_p = dst.join(entry.file_name());
            if src_p.is_dir() {
                fs_copy_dir(&src_p, &dst_p);
            } else {
                fs::copy(&src_p, &dst_p).unwrap();
            }
        }
    }
}
