//! Template domain aggregate and rendering infrastructure.
//!
//! This module defines the core template domain model following **Domain-Driven Design**
//! and **Hexagonal Architecture** principles. Templates are the central concept in Scarff:
//! they define how projects are generated from declarative descriptions.
//!
//! ## Architecture Overview
//!
//! ```text
//! ┌─────────────────────────────────────────────────────────────┐
//! │                    Template Domain                          │
//! ├─────────────────────────────────────────────────────────────┤
//! │  Template (Aggregate Root)                                  │
//! │  ├── TemplateId (Entity)                                    │
//! │  ├── TargetMatcher (Value Object) - When to apply          │
//! │  ├── TemplateMetadata (Value Object) - Human-readable info  │
//! │  └── TemplateTree (Value Object) - What to create           │
//! │       └── Vec<TemplateNode>                                 │
//! │            ├── FileSpec (path, content, permissions)      │
//! │            └── DirectorySpec (path, permissions)             │
//! ├─────────────────────────────────────────────────────────────┤
//! │  TemplateEngine (Driven Port)                               │
//! │  ├── resolve(target) -> TemplateRecord                     │
//! │  └── render(record, ctx) -> ProjectStructure               │
//! ├─────────────────────────────────────────────────────────────┤
//! │  RenderContext (Value Object)                               │
//! │  └── Variable substitution: {{PROJECT_NAME}} -> "MyApp"     │
//! └─────────────────────────────────────────────────────────────┘
//! ```
//!
//! ## Design Decisions
//!
//! ### 1. Why `&'static str` for Metadata?
//!
//! **Current (MVP):** Metadata uses `&'static str` because templates are either:
//! - Hardcoded in the binary (compile-time strings)
//! - Loaded once at startup and leaked for simplicity (acceptable for CLI tool)
//!
//! **Future:** When dynamic templates arrive (remote registry, user-defined), these will
//! become `String` or `Arc<str>`. The builder pattern allows this change without breaking
//! the public API.
//!
//! ### 2. Why Two Content Types: `Literal` vs `Parameterized`?
//!
//! **Performance:** Literal content skips the rendering engine entirely - no string scanning,
//! no replacement overhead. For a 1000-line LICENSE file, this matters.
//!
//! **Clarity:** Explicit intent. A `Cargo.toml` with `{{PROJECT_NAME}}` is obviously
//! parameterized; a README without placeholders is obviously literal.
//!
//! ### 3. Why `TemplateSource` with `Static` vs `Owned`?
//!
//! **Zero-copy for hardcoded:** `TemplateSource::Static` references compile-time strings
//! without allocation or cloning.
//!
//! **Flexibility for loaded:** `TemplateSource::Owned` allows filesystem-loaded or
//! remotely-fetched templates to own their content.
//!
//! **Future-proof:** When we add `TemplateSource::Cached(Arc<str>)` for the template hub,
//! the enum variant approach makes this a non-breaking addition.
//!
//! ### 4. Why `TargetMatcher` with `Option<T>` fields?
//!
//! **Gradual specificity:** `None` means "wildcard" (matches anything). This allows:
//! - Broad: `language=Rust, rest=None` → matches any Rust project
//! - Specific: `language=Rust, kind=Cli, architecture=Layered` → exact match
//!
//! **Conflict resolution:** The `specificity()` method counts non-None fields. When multiple
//! templates match, the most specific wins (e.g., "Rust CLI Layered" beats "Rust CLI").
//!
//! ### 5. Why `TemplateRecord` wrapper?
//!
//! **Identity vs equality:** Two templates with the same `TemplateId` (rust-cli@1.0.0) are
//! semantically equal, but we need unique instance IDs for:
//! - Caching (did we already render this template?)
//! - Provenance (which exact template instance created this project?)
//! - Analytics (usage tracking per instance, not per template type)
//!
//! ## Extension Points for Post-MVP
//!
//! The current design supports these future features without breaking changes:
//!
//! ### Template Hub / Remote Registry
//! ```rust
//! // Add to TemplateSource:
//! Remote { url: Url, etag: String, cached_at: DateTime }
//! ```
//!
//! ### Conditional Rendering
//! ```rust
//! // Add to TemplateContent:
//! Conditional {
//!     condition: String, // "{{FEATURE_X}}"
//!     then_branch: Box<TemplateContent>,
//!     else_branch: Option<Box<TemplateContent>>,
//! }
//! ```
//!
//! ### Template Composition (Inheritance)
//! ```rust
//! // Add to TemplateTree:
//! Extend { base: TemplateId, overrides: Vec<TemplateNode> }
//! ```
//!
//! ### Live Reloading
//! ```rust
//! // Add to TemplateSource:
//! Watched { path: PathBuf, last_modified: SystemTime }
//! ```

use std::collections::HashSet;
use std::fmt;
use uuid::Uuid;

use super::{project_structure::ProjectStructure, target::Target};

use crate::domain::{
    entities::common::{Permissions, RelativePath},
    error::DomainError,
    value_objects::{Architecture, Framework, Language, ProjectKind},
};

/// Template engine port (trait).
///
/// This is a **driven port** in Hexagonal Architecture terms. The domain defines
/// the interface, and infrastructure provides implementations (filesystem loader,
/// remote registry client, cached resolver, etc.).
///
/// ## Port vs Adapter
///
/// - **Port (this trait):** Defines the capability the domain needs
/// - **Adapters (infra):** `FilesystemTemplateEngine`, `RemoteTemplateEngine`, `CachedTemplateEngine`
///
/// ## Lifecycle
///
/// 1. **Resolve:** Given a target (Rust CLI), find the best matching template
/// 2. **Render:** Given a template and context (project name), generate concrete files
pub trait TemplateEngine: Send + Sync {
    /// Resolve the best matching template for a target.
    ///
    /// # Resolution Algorithm
    /// 1. Find all templates where `template.matcher.matches(target)` is true
    /// 2. Sort by `specificity()` descending (most specific first)
    /// 3. Return the first match, or `TemplateNotFound` error
    ///
    /// # Errors
    /// - `TemplateNotFound`: No template matches the target
    /// - `AmbiguousTemplate`: Multiple templates with equal specificity match (rare)
    fn resolve(&self, target: &Target) -> Result<TemplateRecord, DomainError>;

    /// Render a template into a concrete project structure.
    ///
    /// This transforms the declarative `TemplateTree` into actual file content
    /// by applying the `RenderContext` (variable substitution).
    ///
    /// # Rendering Process
    /// 1. Walk `TemplateTree` nodes in order
    /// 2. For each `FileSpec`:
    ///    - `Literal`: Copy content as-is
    ///    - `Parameterized`: Apply `ctx.render()` to substitute variables
    ///    - `External`: Fetch from remote/external source, then render
    /// 3. Return `ProjectStructure` with fully resolved paths and content
    ///
    /// # Errors
    /// - `RenderError`: Variable substitution failed (missing variable)
    /// - `ExternalFetchError`: Failed to load external template content
    fn render(
        &self,
        record: &TemplateRecord,
        ctx: &RenderContext,
    ) -> Result<ProjectStructure, DomainError>;
}

/// Context for template rendering.
///
/// A **Value Object** containing all data needed to render a parameterized template.
/// Immutable after creation - transformations create new instances (see `with_variable`).
///
/// ## Variable Naming Convention
///
/// All built-in variables are `SCREAMING_SNAKE_CASE` to avoid collision with
/// user-defined variables (which should use `snake_case` or `camelCase`).
///
/// ## Built-in Variables
///
/// | Variable | Example | Source |
/// |----------|---------|--------|
/// | `PROJECT_NAME` | "My Awesome App" | User input |
/// | `PROJECT_NAME_SNAKE` | "my_awesome_app" | Computed |
/// | `PROJECT_NAME_KEBAB` | "my-awesome-app" | Computed |
/// | `PROJECT_NAME_PASCAL` | "MyAwesomeApp" | Computed |
/// | `YEAR` | "2026" | System time |
///
/// ## Future Extensions
///
/// - `SCARFF_VERSION`: Version of tool generating the project
/// - `GENERATED_AT`: ISO 8601 timestamp
/// - `GIT_USER_NAME`, `GIT_USER_EMAIL`: From git config
/// - `RANDOM_ID`: UUID fragment for unique identifiers
#[derive(Debug, Clone)]
pub struct RenderContext {
    /// Original project name as provided by user.
    /// Kept separate from variables for debugging and display purposes.
    project_name: String,

    /// Variable map for substitution.
    /// Using `HashMap` (not `BTreeMap`) because:
    /// - Order doesn't matter for simple replacement
    /// - O(1) lookup for variable resolution
    /// - No need for sorted iteration in this use case
    variables: std::collections::HashMap<String, String>,
}

impl RenderContext {
    /// Create a new render context with automatic variable derivation.
    ///
    /// # Automatic Derivations
    ///
    /// The project name is transformed into multiple casing variants to cover
    /// common coding conventions:
    ///
    /// - **Original:** "My Awesome App" (display, docs)
    /// - **snake_case:** "my_awesome_app" (Rust modules, Python files)
    /// - **kebab-case:** "my-awesome-app" (package names, directories)
    /// - **PascalCase:** "MyAwesomeApp" (Rust structs, TypeScript classes)
    ///
    /// # Performance Note
    ///
    /// All transformations happen once at construction. Rendering is then
    /// O(n*m) where n=template length, m=variable count (typically < 20).
    pub fn new(project_name: impl Into<String>) -> Self {
        let name = project_name.into();
        let mut vars = std::collections::HashMap::new();

        // Standard variables - these are the "contract" between Scarff and templates.
        // Any template using {{PROJECT_NAME}} can expect this to exist.
        vars.insert("PROJECT_NAME".to_string(), name.clone());
        vars.insert("PROJECT_NAME_SNAKE".to_string(), to_snake_case(&name));
        vars.insert("PROJECT_NAME_KEBAB".to_string(), to_kebab_case(&name));
        vars.insert("PROJECT_NAME_PASCAL".to_string(), to_pascal_case(&name));

        // Static for now; should use `chrono` for actual current year in production.
        // Using literal "2026" avoids chrono dependency in MVP.
        vars.insert("YEAR".to_string(), "2026".to_string());

        Self {
            project_name: name,
            variables: vars,
        }
    }

    /// Add a custom variable, consuming self and returning a new context.
    ///
    /// # Builder Pattern
    ///
    /// This enables fluent construction:
    /// ```rust,ignore
    /// let ctx = RenderContext::new("MyApp")
    ///     .with_variable("FEATURE_X", "enabled")
    ///     .with_variable("DATABASE", "postgres");
    /// ```
    ///
    /// # Variable Precedence
    ///
    /// User-defined variables can **override** built-ins if needed (though this
    /// is generally discouraged to avoid confusion).
    pub fn with_variable(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.variables.insert(key.into(), value.into());
        self
    }

    /// Get a variable value if it exists.
    ///
    /// Returns `None` for undefined variables (rendering will leave placeholder as-is
    /// or fail based on strict mode - TBD).
    pub fn get(&self, key: &str) -> Option<&str> {
        self.variables.get(key).map(|s| s.as_str())
    }

    /// Render a template string by replacing `{{VARIABLE}}` placeholders.
    ///
    /// # Algorithm
    ///
    /// Simple linear scan and replace. Not the most efficient for large templates
    /// with many variables (would benefit from Aho-Corasick or similar), but
    /// adequate for MVP file sizes (< 10KB typical).
    ///
    /// # Future: Proper Template Engine
    ///
    /// This is intentionally simple. When we need conditionals (`{{#if}}`), loops,
    /// or filters, we'll replace this with a real engine (Handlebars, Tera, or Minijinja)
    /// without changing the `RenderContext` API.
    ///
    /// # Edge Cases
    ///
    /// - `{{UNKNOWN}}` → remains as literal `{{UNKNOWN}}` (no error)
    /// - `{{PROJECT_NAME}}{{PROJECT_NAME}}` → both replaced correctly
    /// - Nested braces `{{{PROJECT_NAME}}}` → outer braces preserved, inner replaced
    pub fn render(&self, template: &str) -> String {
        let mut result = template.to_string();

        // Single-pass replacement. Order doesn't matter for independent variables.
        for (key, value) in &self.variables {
            let placeholder = format!("{{{{{key}}}}}");
            result = result.replace(&placeholder, value);
        }

        result
    }
}

// ============================================================================
// String Case Conversion Helpers
// ============================================================================

/// Convert a string to snake_case.
///
/// ## Rules
///
/// 1. Split on word boundaries (see `split_words`)
/// 2. Join with `_`
/// 3. Lowercase everything
///
/// ## Examples
///
/// | Input | Output |
/// |-------|--------|
/// | "MyApp" | "my_app" |
/// | "my-app" | "my_app" |
/// | "HTTPRequest" | "http_request" |
/// | "XMLHttpRequest" | "xml_http_request" |
fn to_snake_case(s: &str) -> String {
    split_words(s).join("_")
}

/// Convert a string to kebab-case.
///
/// ## Rules
///
/// Same as `to_snake_case` but joins with `-` instead of `_`.
/// Used for package names, directory names, and CLI tools.
fn to_kebab_case(s: &str) -> String {
    split_words(s).join("-")
}

/// Convert a string to PascalCase.
///
/// ## Rules
///
/// 1. Split on word boundaries
/// 2. Capitalize first letter of each word
/// 3. Join without separator
///
/// ## Examples
///
/// | Input | Output |
/// |-------|--------|
/// | "my-app" | "MyApp" |
/// | "HTTPRequest" | "HttpRequest" |
fn to_pascal_case(s: &str) -> String {
    split_words(s)
        .into_iter()
        .map(|w| {
            let mut chars = w.chars();
            match chars.next() {
                Some(first) => {
                    let mut out = String::new();
                    // to_uppercase handles Unicode correctly (e.g., "ß" -> "SS")
                    out.extend(first.to_uppercase());
                    out.push_str(chars.as_str());
                    out
                }
                None => String::new(),
            }
        })
        .collect()
}

/// Split a string into words based on casing and separators.
///
/// ## Word Boundary Detection
///
/// 1. **Explicit separators:** `_`, `-`, whitespace → always split
/// 2. **Case transition (camelCase):** `aB` → split between `a` and `B`
/// 3. **Acronym boundary:** `HTTPRequest` → split between `P` and `R`
///    (detected by `Upper Upper Lower` pattern)
///
/// ## Rationale
///
/// This handles the "identifier hell" of programming:
/// - `my_awesome_app` (snake_case)
/// - `my-awesome-app` (kebab-case)
/// - `myAwesomeApp` (camelCase)
/// - `MyAwesomeApp` (PascalCase)
/// - `XMLHttpRequest` (acronyms)
/// - `my HTTP request` (natural language)
fn split_words(input: &str) -> Vec<String> {
    let mut words = Vec::new();
    let mut current = String::new();

    // Peekable allows looking ahead for boundary detection without consuming
    let mut chars = input.chars().peekable();

    while let Some(c) = chars.next() {
        // Rule 1: Explicit separators always end the current word
        if c == '_' || c == '-' || c.is_whitespace() {
            if !current.is_empty() {
                words.push(current.to_lowercase());
                current.clear();
            }
            continue;
        }

        // Rule 2: camelCase transition (lowercase -> uppercase)
        // "myApp" → "my" + "App"
        if let Some(next) = chars.peek() {
            if c.is_lowercase() && next.is_uppercase() {
                current.push(c);
                words.push(current.to_lowercase());
                current.clear();
                continue;
            }

            // Rule 3: Acronym boundary detection
            // "HTTPServer" → "HTTP" + "Server"
            // Detected by: Uppercase, Next is Uppercase, Next+1 is Lowercase
            if c.is_uppercase()
                && next.is_uppercase()
                && chars.clone().nth(1).map_or(false, |n| n.is_lowercase())
            {
                current.push(c);
                words.push(current.to_lowercase());
                current.clear();
                continue;
            }
        }

        current.push(c);
    }

    // Don't forget the last word
    if !current.is_empty() {
        words.push(current.to_lowercase());
    }

    words
}

// ============================================================================
// Template Identity and Storage
// ============================================================================

/// Unique identifier for a template type.
///
/// ## Format
///
/// Human-readable: `name@version` (e.g., `rust-cli-default@1.0.0`)
///
/// ## Constraints
///
/// - Name cannot contain `@` (enforced by `assert!` in constructor)
/// - Version follows SemVer in practice, but stored as opaque string
/// - Case-sensitive (treat `Rust` and `rust` as different)
///
/// ## Future: Version Ranges
///
/// May add `matches_version(req: &VersionReq)` method for dependency-style
/// version matching (e.g., `rust-cli-default@^1.0` matches `1.0.0`, `1.2.3`).
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct TemplateId {
    /// Template type name (e.g., "rust-cli-default")
    name: String,
    /// SemVer version string (e.g., "1.0.0")
    version: String,
}

impl TemplateId {
    /// Create a new template ID.
    ///
    /// # Panics
    ///
    /// Panics in debug mode if name contains `@`. This is a programming error
    /// (invalid template name), not a runtime error.
    pub fn new(name: impl Into<String>, version: impl Into<String>) -> Self {
        let name = name.into();
        let version = version.into();
        assert!(
            !name.contains('@'),
            "Template name cannot contain @: {}",
            name
        );
        Self { name, version }
    }

    /// Parse from string format `name@version`.
    ///
    /// # Errors
    ///
    /// Returns `InvalidTemplate` if format is wrong (missing `@` or multiple `@`).
    pub fn parse(s: &str) -> Result<Self, DomainError> {
        let parts: Vec<&str> = s.split('@').collect();
        if parts.len() != 2 {
            return Err(DomainError::InvalidTemplate(format!(
                "Invalid template ID format: {}. Expected 'name@version'",
                s
            )));
        }
        Ok(Self::new(parts[0], parts[1]))
    }

    pub fn name(&self) -> &str {
        &self.name
    }
    pub fn version(&self) -> &str {
        &self.version
    }
}

impl fmt::Display for TemplateId {
    /// Display as `name@version` format.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}@{}", self.name, self.version)
    }
}

/// Storage wrapper providing unique identity for template instances.
///
/// ## Why This Exists
///
/// `TemplateId` identifies the *type* (rust-cli-default@1.0.0). `TemplateRecord`
/// identifies the *instance* (the specific copy loaded at 2026-02-13 10:00:00).
///
/// This distinction enables:
/// - **Caching:** "Have we already processed this exact instance?"
/// - **Provenance:** "Which template instance created this project file?"
/// - **Analytics:** "How many times was this specific version used?"
/// - **Hot-reloading:** Compare UUIDs to detect if template changed on disk
///
/// ## UUID Generation
///
/// Uses `Uuid::new_v4()` (random) by default. `with_uuid` allows restoring
/// from persistence or distributed systems.
#[derive(Debug, Clone)]
pub struct TemplateRecord {
    /// Unique instance identifier (never nil except in error states)
    pub uuid: Uuid,
    /// The actual template aggregate
    pub template: Template,
}

impl TemplateRecord {
    /// Create a new record with random UUID.
    pub fn new(template: Template) -> Self {
        Self {
            uuid: Uuid::new_v4(),
            template,
        }
    }

    /// Create with specific UUID (for reconstruction from persistence).
    pub fn with_uuid(uuid: Uuid, template: Template) -> Self {
        Self { uuid, template }
    }

    /// Validate the record integrity.
    ///
    /// Checks:
    /// 1. UUID is not nil (all zeros)
    /// 2. Inner template passes its own validation
    pub fn validate(&self) -> Result<(), DomainError> {
        if self.uuid.is_nil() {
            return Err(DomainError::InvalidTemplate("UUID cannot be nil".into()));
        }
        self.template.validate()
    }
}

// ============================================================================
// Core Template Aggregate
// ============================================================================

/// The central domain aggregate: a reusable project blueprint.
///
/// ## Aggregate Boundaries
///
/// A `Template` is a consistency boundary. All operations on a template
/// (validation, matching, rendering) treat it as a unit.
///
/// ## Invariants (enforced by `validate()`)
///
/// 1. `id.name` is non-empty
/// 2. `metadata.name` is non-empty (human-readable display name)
/// 3. `tree` is non-empty (templates must create at least one file/dir)
/// 4. All paths in `tree` are unique (no duplicate files or dirs)
///
/// ## Lifecycle
///
/// 1. **Definition:** Created via `TemplateBuilder` or loaded from manifest
/// 2. **Validation:** `validate()` ensures invariants before use
/// 3. **Matching:** `matches(target)` checks applicability
/// 4. **Resolution:** `specificity()` helps pick best match among candidates
/// 5. **Rendering:** (via `TemplateEngine`) produces `ProjectStructure`
#[derive(Debug, Clone)]
pub struct Template {
    /// Unique type identifier (e.g., "rust-cli-default@1.0.0")
    pub id: TemplateId,

    /// Matching rules: when does this template apply?
    pub matcher: TargetMatcher,

    /// Human-readable metadata for UI/CLI display
    pub metadata: TemplateMetadata,

    /// The actual content: files and directories to create
    pub tree: TemplateTree,
}

impl Template {
    /// Start the builder pattern for fluent construction.
    ///
    /// # Example
    /// ```rust,ignore
    /// let template = Template::builder()
    ///     .id(TemplateId::new("my-template", "1.0.0"))
    ///     .matcher(TargetMatcher::builder().language(Language::Rust).build())
    ///     .metadata(TemplateMetadata::new("My Template"))
    ///     .tree(tree)
    ///     .build()?;
    /// ```
    pub fn builder() -> TemplateBuilder {
        TemplateBuilder::default()
    }

    /// Validate all invariants.
    ///
    /// Should be called before persisting or using a template. The `TemplateEngine`
    /// port implementations should validate templates at load time.
    pub fn validate(&self) -> Result<(), DomainError> {
        // Invariant 1: ID must have name
        if self.id.name().is_empty() {
            return Err(DomainError::InvalidTemplate(
                "Template name cannot be empty".into(),
            ));
        }

        // Invariant 2: Metadata must have display name
        if self.metadata.name.is_empty() {
            return Err(DomainError::InvalidTemplate(
                "Metadata name cannot be empty".into(),
            ));
        }

        // Invariant 3: Must have content to create
        if self.tree.is_empty() {
            return Err(DomainError::EmptyTemplate {
                template_id: self.id.to_string(),
            });
        }

        // Invariant 4: No duplicate paths (would cause filesystem conflicts)
        let mut seen = HashSet::new();
        for node in &self.tree.nodes {
            let path = match node {
                TemplateNode::File(f) => f.path.as_str(),
                TemplateNode::Directory(d) => d.path.as_str(),
            };

            if !seen.insert(path.to_string()) {
                return Err(DomainError::DuplicatePath {
                    path: path.to_string(),
                });
            }
        }

        Ok(())
    }

    /// Check if this template matches a target.
    ///
    /// Delegates to `TargetMatcher::matches`. Convenience method for
    /// filtering template lists.
    pub fn matches(&self, target: &Target) -> bool {
        self.matcher.matches(target)
    }

    /// Calculate specificity for conflict resolution.
    ///
    /// Higher score = more specific = preferred when multiple templates match.
    /// See `TargetMatcher::specificity()` for calculation details.
    pub fn specificity(&self) -> u8 {
        self.matcher.specificity()
    }
}

/// Builder for constructing templates with validation.
///
/// ## Design Rationale
///
/// Using a builder instead of a giant `new()` method:
/// - Optional fields can be added without breaking changes
/// - Validation happens at `build()`, not scattered across setters
/// - Fluent API is more readable than positional parameters
///
/// ## Required Fields
///
/// All fields are optional during construction, but `build()` enforces:
/// - `id` (must be set)
/// - `matcher` (must be set)
/// - `metadata` (must be set)
/// - `tree` (must be non-empty)
#[derive(Default)]
pub struct TemplateBuilder {
    id: Option<TemplateId>,
    matcher: Option<TargetMatcher>,
    metadata: Option<TemplateMetadata>,
    tree: TemplateTree,
}

impl TemplateBuilder {
    pub fn id(mut self, id: TemplateId) -> Self {
        self.id = Some(id);
        self
    }

    pub fn matcher(mut self, matcher: TargetMatcher) -> Self {
        self.matcher = Some(matcher);
        self
    }

    pub fn metadata(mut self, metadata: TemplateMetadata) -> Self {
        self.metadata = Some(metadata);
        self
    }

    /// Set the entire tree at once (replaces any previous nodes).
    pub fn tree(mut self, tree: TemplateTree) -> Self {
        self.tree = tree;
        self
    }

    /// Add a single node to the tree (accumulates).
    pub fn add_node(mut self, node: TemplateNode) -> Self {
        self.tree.push(node);
        self
    }

    /// Consume builder and construct `Template`.
    ///
    /// # Errors
    ///
    /// - `MissingRequiredField` if id/matcher/metadata not set
    /// - `InvalidTemplate` if tree is empty
    pub fn build(self) -> Result<Template, DomainError> {
        // Early validation: empty tree is always wrong
        if self.tree.is_empty() {
            return Err(DomainError::InvalidTemplate(
                "Template tree cannot be empty".into(),
            ));
        }

        Ok(Template {
            id: self
                .id
                .ok_or_else(|| DomainError::MissingRequiredField { field: "id" })?,
            matcher: self
                .matcher
                .ok_or_else(|| DomainError::MissingRequiredField { field: "matcher" })?,
            metadata: self
                .metadata
                .ok_or_else(|| DomainError::MissingRequiredField { field: "metadata" })?,
            tree: self.tree,
        })
    }
}

// ============================================================================
// TargetMatcher - When to Apply a Template
// ============================================================================

/// Declarative rules for when a template applies to a target.
///
/// ## Matching Semantics
///
/// Uses **open-world assumption**: `None` means "don't care" (wildcard).
/// All specified constraints must match (AND logic).
///
/// ## Specificity Scoring
///
/// Used to resolve conflicts when multiple templates match:
/// - `language=Rust, rest=None` → score 1 (broad)
/// - `language=Rust, kind=Cli, architecture=Layered` → score 3 (specific)
///
/// This enables "default + override" patterns:
/// 1. Define broad default template
/// 2. Define specific templates for special cases
/// 3. System picks best match automatically
///
/// ## Example Matrix
///
/// | Template | Language | Framework | Kind | Architecture | Specificity |
/// |----------|----------|-----------|------|--------------|-------------|
/// | Default Rust | Rust | * | * | * | 1 |
/// | Rust CLI | Rust | * | Cli | * | 2 |
/// | Rust CLI Layered | Rust | * | Cli | Layered | 3 |
/// | Axum Backend | Rust | Axum | WebBackend | * | 3 |
///
/// ## Future: Partial Matching
///
/// Could add `matches_score()` returning `Option<u8>` where `None` = no match,
/// `Some(n)` = match with specificity `n`. This would allow "closest match"
/// suggestions when no template matches exactly.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct TargetMatcher {
    /// Required language (e.g., Rust, Python)
    pub language: Option<Language>,

    /// Required framework (e.g., Axum, FastAPI)
    ///
    /// Note: `Framework` enum includes language info (e.g., `Framework::Rust(RustFramework::Axum)`),
    /// so this implicitly constrains language if set. The `matches()` method handles this
    /// by checking consistency between `language` and `framework` fields.
    pub framework: Option<Framework>,

    /// Required project kind (e.g., Cli, WebBackend)
    pub kind: Option<ProjectKind>,

    /// Required architecture pattern (e.g., Layered, Hexagonal)
    pub architecture: Option<Architecture>,
}

impl TargetMatcher {
    /// Start the builder pattern for fluent construction.
    pub fn builder() -> TargetMatcherBuilder {
        TargetMatcherBuilder::default()
    }

    /// Check whether this matcher applies to a target.
    ///
    /// ## Logic
    ///
    /// For each field:
    /// - `None` → matches (wildcard)
    /// - `Some(x)` → matches if `x == target.field()`
    ///
    /// All fields must match (AND logic).
    ///
    /// ## Framework/Language Consistency
    ///
    /// If both `language` and `framework` are set, they must be consistent
    /// (e.g., `language=Rust` and `framework=Axum` is valid, but
    /// `language=Python` and `framework=Axum` would not match any real target
    /// because Axum implies Rust).
    pub fn matches(&self, target: &Target) -> bool {
        self.language.map_or(true, |l| l == target.language())
            && self
                .framework
                .map_or(true, |f| Some(f) == target.framework())
            && self.kind.map_or(true, |k| k == target.kind())
            && self
                .architecture
                .map_or(true, |a| a == target.architecture())
    }

    /// Calculate specificity score (higher = more specific).
    ///
    /// Count of non-None fields. Used for conflict resolution.
    ///
    /// # Panics
    ///
    /// Never - the `expect` is defensive since we have exactly 4 boolean fields.
    pub fn specificity(&self) -> u8 {
        u8::try_from(
            [
                self.language.is_some(),
                self.framework.is_some(),
                self.kind.is_some(),
                self.architecture.is_some(),
            ]
            .into_iter()
            .filter(|b| *b)
            .count(),
        )
        .expect("specificity count should fit in u8")
    }
}

/// Builder for `TargetMatcher`.
///
/// Allows incremental construction:
/// ```rust,ignore
/// let matcher = TargetMatcher::builder()
///     .language(Language::Rust)
///     .kind(ProjectKind::Cli)
///     .build();
/// ```
#[derive(Default)]
pub struct TargetMatcherBuilder {
    language: Option<Language>,
    framework: Option<Framework>,
    kind: Option<ProjectKind>,
    architecture: Option<Architecture>,
}

impl TargetMatcherBuilder {
    pub fn language(mut self, lang: Language) -> Self {
        self.language = Some(lang);
        self
    }

    pub fn framework(mut self, fw: Framework) -> Self {
        self.framework = Some(fw);
        self
    }

    pub fn kind(mut self, kind: ProjectKind) -> Self {
        self.kind = Some(kind);
        self
    }

    pub fn architecture(mut self, arch: Architecture) -> Self {
        self.architecture = Some(arch);
        self
    }

    pub fn build(self) -> TargetMatcher {
        TargetMatcher {
            language: self.language,
            framework: self.framework,
            kind: self.kind,
            architecture: self.architecture,
        }
    }
}

// ============================================================================
// Template Metadata
// ============================================================================

/// Human-readable information about a template.
///
/// ## MVP Trade-off: `&'static str`
///
/// Currently uses `&'static str` for zero-cost storage of hardcoded templates.
/// This means:
/// - No allocation for built-in templates
/// - Cannot be constructed from runtime strings (e.g., user input)
/// - Requires `Box::leak` or similar if loading dynamic strings
///
/// ## Post-MVP Migration Path
///
/// Change to `String` or `Arc<str>`:
/// ```rust,ignore
/// // Current:
/// pub name: String,
///
/// // Future:
/// pub name: Arc<str>,  // Or String
/// ```
///
/// The builder pattern (`TemplateMetadata::new().description(...)`) means this
/// change won't break call sites - just the internal storage.
///
/// ## Display vs Search
///
/// - `name`: Short display name (e.g., "Rust CLI")
/// - `description`: Longer explanation for `scarff list-templates --verbose`
/// - `tags`: Search keywords for `scarff search --tag web`
#[derive(Debug, Clone)]
pub struct TemplateMetadata {
    /// Short display name (e.g., "Rust CLI (Default)")
    pub name: String,

    /// Longer description for help text and documentation
    pub description: String,

    /// Version string (often mirrors `TemplateId.version` but can differ for metadata updates)
    pub version: String,

    /// Author or organization that created the template
    pub author: String,

    /// Searchable tags for discovery
    pub tags: Vec<String>,
    // Future: Architecture pattern auto-detected from directory structure
    // pub architecture_tag: Option<ArchitecturePattern>,
}

impl TemplateMetadata {
    /// Create new metadata with required name.
    ///
    /// Other fields get sensible defaults:
    /// - `description`: empty string
    /// - `version`: "0.1.0"
    /// - `author`: "Scarff"
    /// - `tags`: empty vector
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            description: "".into(),
            version: "0.1.0".to_string(),
            author: "Scarff".to_string(),
            tags: Vec::new(),
            // architecture_tag: None,
        }
    }

    pub fn description(mut self, desc: impl Into<String>) -> Self {
        self.description = desc.into();
        self
    }

    pub fn version(mut self, ver: impl Into<String>) -> Self {
        self.version = ver.into();
        self
    }

    pub fn author(mut self, author: impl Into<String>) -> Self {
        self.author = author.into();
        self
    }

    pub fn tags(mut self, tags: Vec<String>) -> Self {
        self.tags = tags;
        self
    }

    // Future: Auto-detect architecture from file structure
    // pub fn architecture_tag(mut self, pattern: ArchitecturePattern) -> Self {
    //     self.architecture_tag = Some(pattern);
    //     self
    // }
}

// ============================================================================
// Template Tree - The "What"
// ============================================================================

/// Declarative description of a project's filesystem structure.
///
/// ## Ordering
///
/// Nodes are processed in order. This matters for:
/// - Directory creation before file creation
/// - Dependencies (e.g., create `src/` before `src/main.rs`)
///
/// ## Future: Lazy Evaluation
///
/// May add `TemplateNode::Conditional` or `TemplateNode::Generated` for
/// content that isn't known until render time (e.g., "create file for each module
/// in user's existing project").
#[derive(Debug, Clone, Default)]
pub struct TemplateTree {
    /// Ordered list of filesystem nodes to create.
    pub nodes: Vec<TemplateNode>,
}

impl TemplateTree {
    pub fn new() -> Self {
        Self::default()
    }

    /// Add a node to the tree (maintains insertion order).
    pub fn push(&mut self, node: TemplateNode) {
        self.nodes.push(node);
    }

    /// Fluent variant of `push` for builder chains.
    pub fn with_node(mut self, node: TemplateNode) -> Self {
        self.push(node);
        self
    }

    pub fn is_empty(&self) -> bool {
        self.nodes.is_empty()
    }

    pub fn len(&self) -> usize {
        self.nodes.len()
    }
}

/// A single node in the template tree: either a file or directory.
///
/// ## Future Variants
///
/// - `Symlink(RelativePath, RelativePath)` - for shared configs
/// - `Command(String)` - run shell command during generation
/// - `Conditional { condition: String, node: Box<TemplateNode> }` - feature-gated
#[derive(Debug, Clone)]
pub enum TemplateNode {
    File(FileSpec),
    Directory(DirectorySpec),
}

/// Specification for a file to create.
///
/// ## Content Types
///
/// - `TemplateContent::Literal`: Copy as-is
/// - `TemplateContent::Parameterized`: Run through `RenderContext::render()`
/// - `TemplateContent::External`: Fetch from remote, then render
///
/// ## Permissions
///
/// Defaults to read-write (0o644). Use `executable()` for scripts (0o755).
///
/// ## Future: Content Addressing
///
/// May add `content_hash: Option<Sha256>` for integrity verification of
/// external templates.
#[derive(Debug, Clone)]
pub struct FileSpec {
    /// Relative path from project root (e.g., "src/main.rs")
    pub path: RelativePath,

    /// Content specification (literal, parameterized, or external reference)
    pub content: TemplateContent,

    /// Unix-style permissions (e.g., 0o644 for files, 0o755 for executables)
    pub permissions: Permissions,
}

impl FileSpec {
    /// Create a new file spec with default read-write permissions.
    pub fn new(path: impl Into<RelativePath>, content: TemplateContent) -> Self {
        Self {
            path: path.into(),
            content,
            permissions: Permissions::read_write(),
        }
    }

    /// Mark this file as executable (e.g., for CLI entry points or shell scripts).
    pub fn executable(mut self) -> Self {
        self.permissions = Permissions::executable();
        self
    }
}

/// Specification for a directory to create.
///
/// Currently simple (just path and permissions), but reserved for future
/// attributes like:
/// - `git_init: bool` - run `git init` here
/// - `package_root: bool` - mark as npm/cargo package boundary
#[derive(Debug, Clone)]
pub struct DirectorySpec {
    pub path: RelativePath,
    pub permissions: Permissions,
}

impl DirectorySpec {
    pub fn new(path: impl Into<RelativePath>) -> Self {
        Self {
            path: path.into(),
            permissions: Permissions::read_write(),
        }
    }
}

// ============================================================================
// Content Types
// ============================================================================

/// Content specification for a file.
///
/// ## Variant Rationale
///
/// **Literal:** No processing needed. Fast, safe, deterministic.
///
/// **Parameterized:** Requires rendering. Slower but flexible.
///
/// **External:** Indirection allows:
/// - Shared content across templates (e.g., standard LICENSE)
/// - Remote content (fetch latest CoC from GitHub)
/// - Large content without bloating the template definition
///
/// ## Future: Caching for External
///
/// External content should be cached with TTL to avoid network calls on every
/// project generation. Cache key = `ContentTemplateId + version`.
#[derive(Debug, Clone)]
pub enum TemplateContent {
    /// Content used exactly as provided.
    Literal(TemplateSource),

    /// Content with `{{VARIABLE}}` placeholders to be substituted.
    Parameterized(TemplateSource),

    /// Reference to external content fetched at render time.
    ///
    /// The [`ContentTemplateId`] is resolved by the [`TemplateEngine`] implementation
    /// (e.g., looking up in a registry or cache).
    External(ContentTemplateId),
}

/// Source of template content: either compile-time or runtime.
///
/// ## Memory Efficiency
///
/// `Static` references binary data (zero-cost). `Owned` allocates for dynamic
/// content (filesystem-loaded, network-fetched, user-provided).
///
/// ## Future: Shared Ownership
///
/// May add `Cached(Arc<str>)` for content shared across multiple templates
/// (e.g., a standard README used by 50 templates, stored once in memory).
#[derive(Debug, Clone)]
pub enum TemplateSource {
    /// Compile-time string literal (e.g., `include_str!("template.rs")`)
    Static(&'static str),

    /// Runtime-owned string (heap-allocated)
    Owned(String),
}

impl From<&'static str> for TemplateSource {
    fn from(s: &'static str) -> Self {
        Self::Static(s)
    }
}

impl From<String> for TemplateSource {
    fn from(s: String) -> Self {
        Self::Owned(s)
    }
}

impl TemplateSource {
    /// Get string slice regardless of storage type.
    pub fn as_str(&self) -> &str {
        match self {
            Self::Static(s) => s,
            Self::Owned(s) => s,
        }
    }

    /// Check if content contains placeholder syntax (quick heuristic).
    ///
    /// Used by loaders to auto-detect if a file should be `Parameterized`
    /// vs `Literal` when the manifest doesn't specify.
    pub fn contains_placeholder(&self) -> bool {
        self.as_str().contains("{{") && self.as_str().contains("}}")
    }
}

/// Identifier for external content templates.
///
/// ## Format
///
/// Currently a simple string. Future versions may use URIs:
/// - `builtin:license-mit` - built into scarff binary
/// - `remote:github.com/scarff/templates/license-mit` - fetch from GitHub
/// - `workspace:shared-readme` - from workspace shared templates
///
/// ## Resolution
///
/// The [`TemplateEngine`] implementation maintains a registry of resolvers:
/// ```rust,ignore
/// trait ContentResolver {
///     fn resolve(&self, id: &ContentTemplateId) -> Result<String, Error>;
/// }
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ContentTemplateId(pub &'static str);
