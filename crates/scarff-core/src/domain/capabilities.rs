//! Framework capability registry.
//!
//! # Design Rationale
//!
//! The previous design scattered compatibility information across multiple
//! `impl` blocks in `value_objects.rs`, requiring edits in 5+ places per
//! new framework. This module replaces that with a single static registry:
//! each framework is described exactly once by its [`FrameworkDef`]. All
//! inference and compatibility checks are O(n) table lookups.
//!
//! # Adding a New Framework
//!
//! 1. Add a variant to the appropriate `*Framework` enum in `value_objects.rs`
//! 2. Add one [`FrameworkDef`] entry to [`FRAMEWORK_REGISTRY`]
//! 3. That's it — no other files change
//!
//! # Adding a New Language
//!
//! 1. Add a variant to `Language` in `value_objects.rs`
//! 2. Add a [`LanguageDef`] entry to [`LANGUAGE_REGISTRY`]
//! 3. Add [`FrameworkDef`] entries for its frameworks
//! 4. That's it — all inference derives from the registries

use crate::domain::value_objects::{Architecture, Framework, Language, ProjectKind};
use crate::domain::value_objects::{
    GoFramework, PythonFramework, RustFramework, TypeScriptFramework,
};

// ── Language definitions ─────────────────────────────────────────────────────

/// Describes what a language can produce and its default behaviour.
///
/// Replaces the scattered `Language::supports()` and `ProjectKind::default_for`
/// logic. Adding a new language means adding one entry here.
#[derive(Debug, Clone, Copy)]
pub struct LanguageDef {
    /// The language this definition describes.
    pub language: Language,

    /// All project kinds this language can produce.
    ///
    /// Validation rejects any kind not in this list.
    pub supported_kinds: &'static [ProjectKind],

    /// The kind to infer when the user omits `--kind`.
    pub default_kind: ProjectKind,
}

/// Single source of truth for language capabilities.
///
/// To add a new language: add one entry here. No `match` arms elsewhere.
pub static LANGUAGE_REGISTRY: &[LanguageDef] = &[
    LanguageDef {
        language: Language::Rust,
        supported_kinds: &[
            ProjectKind::Cli,
            ProjectKind::WebBackend,
            ProjectKind::Library,
            ProjectKind::Worker,
        ],
        default_kind: ProjectKind::Cli,
    },
    LanguageDef {
        language: Language::Python,
        supported_kinds: &[
            ProjectKind::Cli,
            ProjectKind::WebBackend,
            ProjectKind::Fullstack,
            ProjectKind::Worker,
        ],
        default_kind: ProjectKind::WebBackend,
    },
    LanguageDef {
        language: Language::TypeScript,
        supported_kinds: &[
            ProjectKind::WebFrontend,
            ProjectKind::WebBackend,
            ProjectKind::Fullstack,
            ProjectKind::Worker,
        ],
        default_kind: ProjectKind::WebFrontend,
    },
    LanguageDef {
        language: Language::Go,
        supported_kinds: &[
            ProjectKind::Cli,
            ProjectKind::WebBackend,
            ProjectKind::Worker,
        ],
        default_kind: ProjectKind::Cli,
    },
];

// ── Framework definitions ────────────────────────────────────────────────────

/// Describes everything the domain needs to know about one framework.
///
/// This is the single source of truth for a framework's capabilities.
/// All inference, validation, and compatibility checks derive from here.
#[derive(Debug, Clone, Copy)]
pub struct FrameworkDef {
    /// The framework variant this entry describes.
    pub framework: Framework,

    /// All project kinds this framework can scaffold.
    ///
    /// A framework may support multiple kinds. For example, Django supports
    /// both `WebBackend` and `Fullstack`. Svelte supports both `WebFrontend`
    /// and `Fullstack` (via SvelteKit).
    pub supported_kinds: &'static [ProjectKind],

    /// The kind to infer when the user names this framework but omits `--kind`.
    ///
    /// Example: `--framework axum` → `default_kind = WebBackend`
    /// This MUST be a member of `supported_kinds`.
    pub default_kind: ProjectKind,

    /// The architecture pattern that fits this framework best.
    ///
    /// Used to infer `--architecture` when neither the user nor a
    /// higher-priority rule has set one.
    pub default_architecture: Architecture,

    /// Whether this framework is the automatic choice when the user specifies
    /// `(language, kind)` without naming a framework.
    ///
    /// At most one entry per `(language, kind)` pair should be `true`.
    /// The `assert_registry_integrity` test enforces this invariant.
    pub is_default: bool,
}

/// Single source of truth for all framework capabilities.
///
/// To add a new framework: add one entry here. No `match` arms elsewhere.
///
/// Ordering: within each language block, `is_default = true` entries appear
/// first as a convention, but lookup is exhaustive so order is not semantic.
pub static FRAMEWORK_REGISTRY: &[FrameworkDef] = &[
    // ── Rust ─────────────────────────────────────────────────────────────────
    FrameworkDef {
        framework: Framework::Rust(RustFramework::Axum),
        supported_kinds: &[ProjectKind::WebBackend],
        default_kind: ProjectKind::WebBackend,
        default_architecture: Architecture::Layered,
        is_default: true, // Default Rust web-backend framework
    },
    FrameworkDef {
        framework: Framework::Rust(RustFramework::Actix),
        supported_kinds: &[ProjectKind::WebBackend],
        default_kind: ProjectKind::WebBackend,
        default_architecture: Architecture::Layered,
        is_default: false,
    },
    FrameworkDef {
        framework: Framework::Rust(RustFramework::Rocket),
        // Rocket supports both; its default is backend (Fullstack is niche).
        supported_kinds: &[ProjectKind::WebBackend, ProjectKind::Fullstack],
        default_kind: ProjectKind::WebBackend,
        default_architecture: Architecture::Layered,
        is_default: false,
    },
    // ── Python ────────────────────────────────────────────────────────────────
    FrameworkDef {
        framework: Framework::Python(PythonFramework::FastApi),
        supported_kinds: &[ProjectKind::WebBackend, ProjectKind::Worker],
        default_kind: ProjectKind::WebBackend,
        default_architecture: Architecture::Layered,
        is_default: true, // Default Python web-backend
    },
    FrameworkDef {
        framework: Framework::Python(PythonFramework::Django),
        supported_kinds: &[ProjectKind::WebBackend, ProjectKind::Fullstack],
        default_kind: ProjectKind::Fullstack,
        // Django's MVT is conceptually MVC; we map it to Architecture::Mvc.
        default_architecture: Architecture::Mvc,
        is_default: true, // Default Python fullstack
    },
    FrameworkDef {
        framework: Framework::Python(PythonFramework::Flask),
        supported_kinds: &[ProjectKind::WebBackend, ProjectKind::Worker],
        default_kind: ProjectKind::WebBackend,
        default_architecture: Architecture::Layered,
        is_default: false,
    },
    // ── TypeScript ────────────────────────────────────────────────────────────
    FrameworkDef {
        framework: Framework::TypeScript(TypeScriptFramework::Express),
        supported_kinds: &[ProjectKind::WebBackend, ProjectKind::Worker],
        default_kind: ProjectKind::WebBackend,
        default_architecture: Architecture::Layered,
        is_default: true, // Default TypeScript backend
    },
    FrameworkDef {
        framework: Framework::TypeScript(TypeScriptFramework::NestJs),
        supported_kinds: &[ProjectKind::WebBackend, ProjectKind::Worker],
        default_kind: ProjectKind::WebBackend,
        // NestJs has strong opinions toward feature-modular organization.
        default_architecture: Architecture::FeatureModular,
        is_default: false,
    },
    FrameworkDef {
        framework: Framework::TypeScript(TypeScriptFramework::React),
        supported_kinds: &[ProjectKind::WebFrontend],
        default_kind: ProjectKind::WebFrontend,
        default_architecture: Architecture::FeatureModular,
        is_default: true, // Default TypeScript frontend
    },
    FrameworkDef {
        framework: Framework::TypeScript(TypeScriptFramework::Vue),
        supported_kinds: &[ProjectKind::WebFrontend],
        default_kind: ProjectKind::WebFrontend,
        default_architecture: Architecture::FeatureModular,
        is_default: false,
    },
    FrameworkDef {
        framework: Framework::TypeScript(TypeScriptFramework::NextJs),
        // Next.js is fullstack-first, but can produce a frontend-only app.
        supported_kinds: &[ProjectKind::Fullstack, ProjectKind::WebFrontend],
        default_kind: ProjectKind::Fullstack,
        default_architecture: Architecture::FeatureModular,
        is_default: true, // Default TypeScript fullstack
    },
    FrameworkDef {
        framework: Framework::TypeScript(TypeScriptFramework::Svelte),
        // Svelte = frontend; SvelteKit = fullstack. Same variant, both kinds.
        supported_kinds: &[ProjectKind::WebFrontend, ProjectKind::Fullstack],
        default_kind: ProjectKind::WebFrontend,
        default_architecture: Architecture::FeatureModular,
        is_default: false,
    },
    // ── Go ────────────────────────────────────────────────────────────────────
    FrameworkDef {
        framework: Framework::Go(GoFramework::Gin),
        supported_kinds: &[ProjectKind::WebBackend, ProjectKind::Worker],
        default_kind: ProjectKind::WebBackend,
        default_architecture: Architecture::Layered,
        is_default: true, // Default Go web-backend
    },
    FrameworkDef {
        framework: Framework::Go(GoFramework::Echo),
        supported_kinds: &[ProjectKind::WebBackend, ProjectKind::Worker],
        default_kind: ProjectKind::WebBackend,
        default_architecture: Architecture::Layered,
        is_default: false,
    },
    FrameworkDef {
        framework: Framework::Go(GoFramework::Stdlib),
        // The standard library is valid for CLI, backend, and workers.
        supported_kinds: &[
            ProjectKind::Cli,
            ProjectKind::WebBackend,
            ProjectKind::Worker,
        ],
        default_kind: ProjectKind::Cli,
        default_architecture: Architecture::Layered,
        is_default: false,
    },
];

// ── Registry lookup API ───────────────────────────────────────────────────────
//
// These functions are the ONLY entry points for capability queries.
// Do not write `match` arms on frameworks or languages elsewhere.

/// Find the capability definition for a specific framework.
///
/// Returns `None` only if the framework is not registered — a programming
/// error, not a user error. The `assert_registry_integrity` test catches it.
pub fn find_framework(framework: Framework) -> Option<&'static FrameworkDef> {
    FRAMEWORK_REGISTRY
        .iter()
        .find(|def| def.framework == framework)
}

/// Find the language definition for a specific language.
pub fn find_language(language: Language) -> Option<&'static LanguageDef> {
    LANGUAGE_REGISTRY
        .iter()
        .find(|def| def.language == language)
}

/// Check whether a language supports a project kind.
///
/// Replaces `Language::supports()`.
pub fn language_supports_kind(language: Language, kind: ProjectKind) -> bool {
    find_language(language)
        .map(|def| def.supported_kinds.contains(&kind))
        .unwrap_or(false)
}

/// Check whether a framework supports a project kind.
pub fn framework_supports_kind(framework: Framework, kind: ProjectKind) -> bool {
    find_framework(framework)
        .map(|def| def.supported_kinds.contains(&kind))
        .unwrap_or(false)
}

/// Infer the project kind from (language, optional framework).
///
/// This is the answer to "given `--language rust --framework axum`, what kind
/// should be used if the user omits `--kind`?"
///
/// - With a framework: returns `framework.default_kind` (if language matches)
/// - Without a framework: returns `language.default_kind`
pub fn infer_kind(language: Language, framework: Option<Framework>) -> ProjectKind {
    if let Some(fw) = framework {
        if let Some(def) = find_framework(fw) {
            // Only trust the framework's default if it belongs to this language.
            // A mismatch here is a caller bug; we return the language default
            // rather than panicking, so the validator can produce a clear error.
            if def.framework.language() == language {
                return def.default_kind;
            }
        }
    }
    find_language(language)
        .map(|def| def.default_kind)
        .unwrap_or(ProjectKind::Cli)
}

/// Infer the default framework for a (language, kind) pair.
///
/// Returns `None` when no default is registered (e.g. Rust + Cli).
/// Returns `Some(framework)` for the entry where `is_default = true`.
pub fn infer_framework(language: Language, kind: ProjectKind) -> Option<Framework> {
    FRAMEWORK_REGISTRY
        .iter()
        .find(|def| {
            def.is_default
                && def.framework.language() == language
                && def.supported_kinds.contains(&kind)
        })
        .map(|def| def.framework)
}

/// Infer the architecture for a (language, kind, optional framework) triple.
///
/// Priority (highest first):
/// 1. Framework-specific default (if framework supports the kind)
/// 2. Language + kind heuristic
/// 3. Layered (safe universal default)
pub fn infer_architecture(
    language: Language,
    kind: ProjectKind,
    framework: Option<Framework>,
) -> Architecture {
    if let Some(fw) = framework {
        if let Some(def) = find_framework(fw) {
            if def.supported_kinds.contains(&kind) {
                return def.default_architecture;
            }
        }
    }

    match (language, kind) {
        // TypeScript projects at scale benefit from feature-modular organisation.
        (Language::TypeScript, ProjectKind::WebBackend | ProjectKind::Fullstack) => {
            Architecture::FeatureModular
        }
        // Layered is the safest default: universally understood, minimal opinions.
        _ => Architecture::Layered,
    }
}

/// Validate that a (framework, language, kind) triple is consistent.
///
/// Returns `Ok(())` on success or a human-readable error string.
/// This is called by `Target::validate()` after the builder runs.
pub fn validate_framework_compatibility(
    framework: Framework,
    language: Language,
    kind: ProjectKind,
) -> Result<(), String> {
    if framework.language() != language {
        return Err(format!(
            "framework '{framework}' is a {} framework and cannot be used with {language}",
            framework.language()
        ));
    }

    if !framework_supports_kind(framework, kind) {
        let supported = find_framework(framework)
            .map(|d| {
                d.supported_kinds
                    .iter()
                    .map(|k| k.as_str())
                    .collect::<Vec<_>>()
                    .join(", ")
            })
            .unwrap_or_else(|| "unknown".into());

        return Err(format!(
            "framework '{framework}' supports [{supported}] but kind '{kind}' was requested"
        ));
    }

    Ok(())
}

/// Validate that a language supports the given kind.
pub fn validate_language_kind(language: Language, kind: ProjectKind) -> Result<(), String> {
    if !language_supports_kind(language, kind) {
        let supported = find_language(language)
            .map(|d| {
                d.supported_kinds
                    .iter()
                    .map(|k| k.as_str())
                    .collect::<Vec<_>>()
                    .join(", ")
            })
            .unwrap_or_else(|| "unknown".into());

        return Err(format!(
            "{language} supports [{supported}] but kind '{kind}' was requested"
        ));
    }
    Ok(())
}

// ── Registry integrity (checked in tests) ────────────────────────────────────

/// Assert that the registries are internally consistent.
///
/// Call this in a test; it panics with a clear message on any violation.
/// Catches registration errors at development time, not at user runtime.
#[doc(hidden)]
pub fn assert_registry_integrity() {
    for def in FRAMEWORK_REGISTRY {
        let lang = def.framework.language();

        // Every framework's language must be registered.
        assert!(
            find_language(lang).is_some(),
            "Framework {:?} references unregistered language {:?}",
            def.framework,
            lang
        );

        // default_kind must be in supported_kinds.
        assert!(
            def.supported_kinds.contains(&def.default_kind),
            "Framework {:?}: default_kind {:?} is not in supported_kinds {:?}",
            def.framework,
            def.default_kind,
            def.supported_kinds
        );
    }

    // At most one is_default=true per (language, kind) pair.
    for lang_def in LANGUAGE_REGISTRY {
        for &kind in lang_def.supported_kinds {
            if !kind.requires_framework() {
                continue;
            }
            let defaults: Vec<_> = FRAMEWORK_REGISTRY
                .iter()
                .filter(|f| {
                    f.is_default
                        && f.framework.language() == lang_def.language
                        && f.supported_kinds.contains(&kind)
                })
                .collect();

            assert!(
                defaults.len() <= 1,
                "Multiple default frameworks for ({:?}, {:?}): {:?}",
                lang_def.language,
                kind,
                defaults.iter().map(|d| d.framework).collect::<Vec<_>>()
            );
        }
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn registry_is_internally_consistent() {
        assert_registry_integrity();
    }

    #[test]
    fn every_language_default_kind_is_in_its_supported_kinds() {
        for def in LANGUAGE_REGISTRY {
            assert!(
                def.supported_kinds.contains(&def.default_kind),
                "{:?} default_kind {:?} not in supported_kinds",
                def.language,
                def.default_kind
            );
        }
    }

    // ── language_supports_kind ───────────────────────────────────────────────

    #[test]
    fn rust_supports_cli_backend_library_worker() {
        assert!(language_supports_kind(Language::Rust, ProjectKind::Cli));
        assert!(language_supports_kind(
            Language::Rust,
            ProjectKind::WebBackend
        ));
        assert!(language_supports_kind(Language::Rust, ProjectKind::Library));
        assert!(language_supports_kind(Language::Rust, ProjectKind::Worker));
        assert!(!language_supports_kind(
            Language::Rust,
            ProjectKind::WebFrontend
        ));
        assert!(!language_supports_kind(
            Language::Rust,
            ProjectKind::Fullstack
        ));
    }

    #[test]
    fn typescript_does_not_support_cli() {
        assert!(!language_supports_kind(
            Language::TypeScript,
            ProjectKind::Cli
        ));
    }

    #[test]
    fn go_does_not_support_frontend_or_fullstack() {
        assert!(!language_supports_kind(
            Language::Go,
            ProjectKind::WebFrontend
        ));
        assert!(!language_supports_kind(
            Language::Go,
            ProjectKind::Fullstack
        ));
    }

    // ── framework_supports_kind ──────────────────────────────────────────────

    #[test]
    fn axum_supports_only_web_backend() {
        let axum = Framework::Rust(RustFramework::Axum);
        assert!(framework_supports_kind(axum, ProjectKind::WebBackend));
        assert!(!framework_supports_kind(axum, ProjectKind::Cli));
        assert!(!framework_supports_kind(axum, ProjectKind::WebFrontend));
        assert!(!framework_supports_kind(axum, ProjectKind::Fullstack));
    }

    #[test]
    fn django_supports_backend_and_fullstack() {
        let django = Framework::Python(PythonFramework::Django);
        assert!(framework_supports_kind(django, ProjectKind::WebBackend));
        assert!(framework_supports_kind(django, ProjectKind::Fullstack));
        assert!(!framework_supports_kind(django, ProjectKind::Worker));
    }

    #[test]
    fn rocket_supports_backend_and_fullstack() {
        let rocket = Framework::Rust(RustFramework::Rocket);
        assert!(framework_supports_kind(rocket, ProjectKind::WebBackend));
        assert!(framework_supports_kind(rocket, ProjectKind::Fullstack));
    }

    #[test]
    fn svelte_supports_frontend_and_fullstack() {
        let svelte = Framework::TypeScript(TypeScriptFramework::Svelte);
        assert!(framework_supports_kind(svelte, ProjectKind::WebFrontend));
        assert!(framework_supports_kind(svelte, ProjectKind::Fullstack));
    }

    #[test]
    fn go_stdlib_supports_cli_and_backend() {
        let stdlib = Framework::Go(GoFramework::Stdlib);
        assert!(framework_supports_kind(stdlib, ProjectKind::Cli));
        assert!(framework_supports_kind(stdlib, ProjectKind::WebBackend));
    }

    // ── infer_kind ────────────────────────────────────────────────────────────

    #[test]
    fn axum_implies_web_backend() {
        let kind = infer_kind(Language::Rust, Some(Framework::Rust(RustFramework::Axum)));
        assert_eq!(kind, ProjectKind::WebBackend);
    }

    #[test]
    fn react_implies_web_frontend() {
        let kind = infer_kind(
            Language::TypeScript,
            Some(Framework::TypeScript(TypeScriptFramework::React)),
        );
        assert_eq!(kind, ProjectKind::WebFrontend);
    }

    #[test]
    fn django_implies_fullstack() {
        let kind = infer_kind(
            Language::Python,
            Some(Framework::Python(PythonFramework::Django)),
        );
        assert_eq!(kind, ProjectKind::Fullstack);
    }

    #[test]
    fn no_framework_uses_language_default() {
        assert_eq!(infer_kind(Language::Rust, None), ProjectKind::Cli);
        assert_eq!(infer_kind(Language::Python, None), ProjectKind::WebBackend);
        assert_eq!(
            infer_kind(Language::TypeScript, None),
            ProjectKind::WebFrontend
        );
        assert_eq!(infer_kind(Language::Go, None), ProjectKind::Cli);
    }

    #[test]
    fn wrong_language_framework_falls_back_gracefully() {
        // Python framework with Rust language → Rust's default (Cli)
        let kind = infer_kind(
            Language::Rust,
            Some(Framework::Python(PythonFramework::Django)),
        );
        assert_eq!(kind, ProjectKind::Cli);
    }

    #[test]
    fn nextjs_implies_fullstack() {
        let kind = infer_kind(
            Language::TypeScript,
            Some(Framework::TypeScript(TypeScriptFramework::NextJs)),
        );
        assert_eq!(kind, ProjectKind::Fullstack);
    }

    // ── infer_framework ───────────────────────────────────────────────────────

    #[test]
    fn rust_web_backend_defaults_to_axum() {
        assert_eq!(
            infer_framework(Language::Rust, ProjectKind::WebBackend),
            Some(Framework::Rust(RustFramework::Axum))
        );
    }

    #[test]
    fn rust_cli_has_no_default_framework() {
        assert_eq!(infer_framework(Language::Rust, ProjectKind::Cli), None);
    }

    #[test]
    fn python_web_backend_defaults_to_fastapi() {
        assert_eq!(
            infer_framework(Language::Python, ProjectKind::WebBackend),
            Some(Framework::Python(PythonFramework::FastApi))
        );
    }

    #[test]
    fn python_fullstack_defaults_to_django() {
        assert_eq!(
            infer_framework(Language::Python, ProjectKind::Fullstack),
            Some(Framework::Python(PythonFramework::Django))
        );
    }

    #[test]
    fn typescript_frontend_defaults_to_react() {
        assert_eq!(
            infer_framework(Language::TypeScript, ProjectKind::WebFrontend),
            Some(Framework::TypeScript(TypeScriptFramework::React))
        );
    }

    #[test]
    fn typescript_fullstack_defaults_to_nextjs() {
        assert_eq!(
            infer_framework(Language::TypeScript, ProjectKind::Fullstack),
            Some(Framework::TypeScript(TypeScriptFramework::NextJs))
        );
    }

    #[test]
    fn go_web_backend_defaults_to_gin() {
        assert_eq!(
            infer_framework(Language::Go, ProjectKind::WebBackend),
            Some(Framework::Go(GoFramework::Gin))
        );
    }

    // ── infer_architecture ────────────────────────────────────────────────────

    #[test]
    fn django_fullstack_gives_mvc() {
        let arch = infer_architecture(
            Language::Python,
            ProjectKind::Fullstack,
            Some(Framework::Python(PythonFramework::Django)),
        );
        assert_eq!(arch, Architecture::Mvc);
    }

    #[test]
    fn nestjs_gives_feature_modular() {
        let arch = infer_architecture(
            Language::TypeScript,
            ProjectKind::WebBackend,
            Some(Framework::TypeScript(TypeScriptFramework::NestJs)),
        );
        assert_eq!(arch, Architecture::FeatureModular);
    }

    #[test]
    fn rust_cli_gives_layered() {
        let arch = infer_architecture(Language::Rust, ProjectKind::Cli, None);
        assert_eq!(arch, Architecture::Layered);
    }

    #[test]
    fn typescript_backend_no_framework_gives_feature_modular() {
        let arch = infer_architecture(Language::TypeScript, ProjectKind::WebBackend, None);
        assert_eq!(arch, Architecture::FeatureModular);
    }

    #[test]
    fn go_backend_gives_layered() {
        let arch = infer_architecture(
            Language::Go,
            ProjectKind::WebBackend,
            Some(Framework::Go(GoFramework::Gin)),
        );
        assert_eq!(arch, Architecture::Layered);
    }

    // ── validate_framework_compatibility ─────────────────────────────────────

    #[test]
    fn axum_rust_web_backend_is_valid() {
        assert!(
            validate_framework_compatibility(
                Framework::Rust(RustFramework::Axum),
                Language::Rust,
                ProjectKind::WebBackend,
            )
            .is_ok()
        );
    }

    #[test]
    fn django_rust_fails_language_mismatch() {
        let err = validate_framework_compatibility(
            Framework::Python(PythonFramework::Django),
            Language::Rust,
            ProjectKind::WebBackend,
        )
        .unwrap_err();
        assert!(err.contains("python"));
        assert!(err.contains("rust"));
    }

    #[test]
    fn axum_rust_cli_fails_kind_not_supported() {
        let err = validate_framework_compatibility(
            Framework::Rust(RustFramework::Axum),
            Language::Rust,
            ProjectKind::Cli,
        )
        .unwrap_err();
        assert!(err.contains("web-backend"));
    }

    #[test]
    fn fastapi_python_worker_is_valid() {
        assert!(
            validate_framework_compatibility(
                Framework::Python(PythonFramework::FastApi),
                Language::Python,
                ProjectKind::Worker,
            )
            .is_ok()
        );
    }

    // ── validate_language_kind ────────────────────────────────────────────────

    #[test]
    fn rust_web_frontend_is_invalid() {
        let err = validate_language_kind(Language::Rust, ProjectKind::WebFrontend).unwrap_err();
        assert!(err.contains("web-backend") || err.contains("cli"));
    }

    #[test]
    fn python_cli_is_valid() {
        assert!(validate_language_kind(Language::Python, ProjectKind::Cli).is_ok());
    }

    #[test]
    fn go_fullstack_is_invalid() {
        assert!(validate_language_kind(Language::Go, ProjectKind::Fullstack).is_err());
    }
}
