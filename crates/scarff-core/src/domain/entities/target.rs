//! The `Target` aggregate root and its typestate builder.
//!
//! A `Target` is the fully-resolved, validated description of the project the
//! user wants to scaffold. All fields are inferred or validated at build time;
//! once a `Target` exists it is guaranteed consistent.
//!
//! # Typestate builder
//!
//! The builder uses two phantom marker types (`NoLanguage` / `HasLanguage`) to
//! enforce at *compile time* that a language is set before any other field.
//! Runtime validation (`validate`) is still called at `build()` to catch
//! cross-field invariants that cannot be expressed in the type system.
//!
//! # Domain purity
//!
//! This module must not import `tracing`. Observability is the responsibility
//! of the application and CLI layers, not the domain.

use std::fmt;
use std::marker::PhantomData;

use crate::domain::{
    capabilities,
    error::DomainError,
    value_objects::{Architecture, Framework, Language, ProjectKind},
};

// ── Aggregate root ────────────────────────────────────────────────────────────

/// A fully-validated project scaffolding target.
///
/// Every field is guaranteed consistent on construction:
/// - `language` supports `kind`
/// - `framework` (if present) belongs to `language` and supports `kind`
/// - `architecture` is compatible with `(language, kind, framework)`
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Target {
    language: Language,
    kind: ProjectKind,
    framework: Option<Framework>,
    architecture: Architecture,
}

impl Target {
    /// Start building a new `Target`.
    pub fn builder() -> TargetBuilder<NoLanguage> {
        TargetBuilder::new()
    }

    pub const fn language(&self) -> Language {
        self.language
    }
    pub const fn kind(&self) -> ProjectKind {
        self.kind
    }
    pub const fn framework(&self) -> Option<Framework> {
        self.framework
    }
    pub const fn architecture(&self) -> Architecture {
        self.architecture
    }

    /// Validate this target's internal consistency.
    ///
    /// Called automatically by the builder. Available for re-validation after
    /// deserialization or external construction.
    pub fn validate(&self) -> Result<(), DomainError> {
        // 1. Language must support the kind.
        if let Err(reason) = capabilities::validate_language_kind(self.language, self.kind) {
            return Err(DomainError::IncompatibleLanguageKind {
                language: self.language.to_string(),
                kind: self.kind.to_string(),
                reason,
            });
        }

        // 2. Framework must be compatible if present; required if kind needs one.
        match self.framework {
            Some(fw) => {
                if let Err(reason) =
                    capabilities::validate_framework_compatibility(fw, self.language, self.kind)
                {
                    return Err(DomainError::IncompatibleFramework {
                        framework: fw.to_string(),
                        context: format!("{} + {}", self.language, self.kind),
                        reason,
                    });
                }
            }
            None if self.kind.requires_framework() => {
                return Err(DomainError::MissingRequiredField { field: "framework" });
            }
            None => {}
        }

        // 3. Architecture must be compatible.
        if !self
            .architecture
            .is_compatible_with(self.language, self.kind, self.framework)
        {
            return Err(DomainError::InvalidArchitecture {
                architecture: self.architecture.to_string(),
                reason: format!("incompatible with {} + {}", self.language, self.kind),
            });
        }

        Ok(())
    }
}

impl fmt::Display for Target {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} {} ({})", self.language, self.kind, self.architecture)?;
        if let Some(fw) = self.framework {
            write!(f, " + {fw}")?;
        }
        Ok(())
    }
}

// ── Typestate markers ─────────────────────────────────────────────────────────

/// Marker: language has not yet been set.
pub struct NoLanguage;
/// Marker: language has been set; other fields may now be configured.
pub struct HasLanguage;

// ── Builder ───────────────────────────────────────────────────────────────────

/// Typestate builder for [`Target`].
///
/// Compile-time guarantee: `kind`, `framework`, and `architecture` are only
/// accessible after `language` has been set.
pub struct TargetBuilder<L> {
    language: Option<Language>,
    kind: Option<ProjectKind>,
    framework: Option<Framework>,
    architecture: Option<Architecture>,
    _marker: PhantomData<L>,
}

impl TargetBuilder<NoLanguage> {
    pub fn new() -> Self {
        Self {
            language: None,
            kind: None,
            framework: None,
            architecture: None,
            _marker: PhantomData,
        }
    }

    /// Set the language. This transitions the builder to `HasLanguage`.
    pub fn language(self, language: Language) -> TargetBuilder<HasLanguage> {
        TargetBuilder {
            language: Some(language),
            kind: self.kind,
            framework: self.framework,
            architecture: self.architecture,
            _marker: PhantomData,
        }
    }
}

impl Default for TargetBuilder<NoLanguage> {
    fn default() -> Self {
        Self::new()
    }
}

impl TargetBuilder<HasLanguage> {
    /// Set the project kind.
    ///
    /// Rejects immediately if the language does not support this kind.
    pub fn kind(mut self, kind: ProjectKind) -> Result<Self, DomainError> {
        let lang = self.language.expect("typestate guarantees language is set");
        if let Err(reason) = capabilities::validate_language_kind(lang, kind) {
            return Err(DomainError::IncompatibleLanguageKind {
                language: lang.to_string(),
                kind: kind.to_string(),
                reason,
            });
        }
        self.kind = Some(kind);
        Ok(self)
    }

    /// Set the framework.
    ///
    /// Rejects immediately if the framework belongs to a different language.
    /// Full kind-compatibility is validated at `build()` time once the kind
    /// is also known.
    pub fn framework(mut self, framework: Framework) -> Result<Self, DomainError> {
        let lang = self.language.expect("typestate guarantees language is set");
        if framework.language() != lang {
            return Err(DomainError::IncompatibleFramework {
                framework: framework.to_string(),
                context: format!("language {lang}"),
                reason: format!(
                    "framework '{}' belongs to {} not {}",
                    framework,
                    framework.language(),
                    lang
                ),
            });
        }
        self.framework = Some(framework);
        Ok(self)
    }

    /// Set the architecture. Always succeeds; validated at `build()`.
    pub fn architecture(mut self, architecture: Architecture) -> Self {
        self.architecture = Some(architecture);
        self
    }

    /// Build and validate the `Target`, inferring any unset fields.
    ///
    /// Inference order:
    /// 1. `kind` — from framework's default_kind, or language default
    /// 2. `framework` — auto-inferred if kind requires one and none was set
    /// 3. `architecture` — inferred from (language, kind, framework)
    pub fn build(self) -> Result<Target, DomainError> {
        let language = self.language.expect("typestate guarantees language is set");

        // Infer kind: prefer framework's default, then language default.
        let kind = self
            .kind
            .unwrap_or_else(|| capabilities::infer_kind(language, self.framework));

        // Infer framework: only when the kind requires one.
        let framework = self.framework.or_else(|| {
            if kind.requires_framework() {
                capabilities::infer_framework(language, kind)
            } else {
                None
            }
        });

        // Infer architecture from the resolved triple.
        let architecture = self
            .architecture
            .unwrap_or_else(|| capabilities::infer_architecture(language, kind, framework));

        let target = Target {
            language,
            kind,
            framework,
            architecture,
        };

        target.validate()?;
        Ok(target)
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::value_objects::{PythonFramework, RustFramework, TypeScriptFramework};

    fn rust() -> TargetBuilder<HasLanguage> {
        Target::builder().language(Language::Rust)
    }
    fn python() -> TargetBuilder<HasLanguage> {
        Target::builder().language(Language::Python)
    }
    fn ts() -> TargetBuilder<HasLanguage> {
        Target::builder().language(Language::TypeScript)
    }

    // ── Basic inference ───────────────────────────────────────────────────────

    #[test]
    fn rust_defaults_to_cli_layered_no_framework() {
        let t = rust().build().unwrap();
        assert_eq!(t.language(), Language::Rust);
        assert_eq!(t.kind(), ProjectKind::Cli);
        assert_eq!(t.architecture(), Architecture::Layered);
        assert_eq!(t.framework(), None);
    }

    #[test]
    fn python_defaults_to_web_backend_with_fastapi() {
        let t = python().build().unwrap();
        assert_eq!(t.kind(), ProjectKind::WebBackend);
        assert_eq!(
            t.framework(),
            Some(Framework::Python(PythonFramework::FastApi))
        );
    }

    #[test]
    fn typescript_defaults_to_web_frontend_with_react() {
        let t = ts().build().unwrap();
        assert_eq!(t.kind(), ProjectKind::WebFrontend);
        assert_eq!(
            t.framework(),
            Some(Framework::TypeScript(TypeScriptFramework::React))
        );
    }

    // ── Framework implies kind ────────────────────────────────────────────────

    #[test]
    fn axum_implies_web_backend() {
        let t = rust()
            .framework(Framework::Rust(RustFramework::Axum))
            .unwrap()
            .build()
            .unwrap();
        assert_eq!(t.kind(), ProjectKind::WebBackend);
    }

    #[test]
    fn django_implies_fullstack_and_mvc() {
        let t = python()
            .framework(Framework::Python(PythonFramework::Django))
            .unwrap()
            .build()
            .unwrap();
        assert_eq!(t.kind(), ProjectKind::Fullstack);
        assert_eq!(t.architecture(), Architecture::Mvc);
    }

    #[test]
    fn nextjs_implies_fullstack() {
        let t = ts()
            .framework(Framework::TypeScript(TypeScriptFramework::NextJs))
            .unwrap()
            .build()
            .unwrap();
        assert_eq!(t.kind(), ProjectKind::Fullstack);
    }

    // ── Explicit overrides respected ──────────────────────────────────────────

    #[test]
    fn explicit_kind_overrides_framework_default() {
        // Rocket supports both backend and fullstack; user explicitly picks backend.
        let t = rust()
            .kind(ProjectKind::WebBackend)
            .unwrap()
            .framework(Framework::Rust(RustFramework::Rocket))
            .unwrap()
            .build()
            .unwrap();
        assert_eq!(t.kind(), ProjectKind::WebBackend);
    }

    #[test]
    fn explicit_architecture_overrides_inference() {
        let t = python()
            .framework(Framework::Python(PythonFramework::Django))
            .unwrap()
            .architecture(Architecture::Clean)
            .build()
            .unwrap();
        // Clean overrides the MVC default.
        assert_eq!(t.architecture(), Architecture::Clean);
    }

    // ── Validation errors ─────────────────────────────────────────────────────

    #[test]
    fn wrong_language_framework_is_rejected_immediately() {
        let result = rust().framework(Framework::Python(PythonFramework::Django));
        assert!(result.is_err());
    }

    #[test]
    fn incompatible_kind_for_language_is_rejected() {
        let result = rust().kind(ProjectKind::WebFrontend);
        assert!(result.is_err());
    }

    #[test]
    fn display_includes_all_fields() {
        let t = rust()
            .kind(ProjectKind::WebBackend)
            .unwrap()
            .framework(Framework::Rust(RustFramework::Axum))
            .unwrap()
            .build()
            .unwrap();
        let s = t.to_string();
        assert!(s.contains("rust"));
        assert!(s.contains("web-backend"));
        assert!(s.contains("axum"));
    }

    // ── Auto-framework inference for web kinds ────────────────────────────────

    #[test]
    fn rust_web_backend_auto_infers_axum() {
        let t = rust()
            .kind(ProjectKind::WebBackend)
            .unwrap()
            .build()
            .unwrap();
        assert_eq!(t.framework(), Some(Framework::Rust(RustFramework::Axum)));
    }

    #[test]
    fn go_web_backend_auto_infers_gin() {
        let t = Target::builder()
            .language(Language::Go)
            .kind(ProjectKind::WebBackend)
            .unwrap()
            .build()
            .unwrap();
        assert_eq!(
            t.framework(),
            Some(Framework::Go(
                crate::domain::value_objects::GoFramework::Gin
            ))
        );
    }

    // ── Validate re-entrant ───────────────────────────────────────────────────

    #[test]
    fn validate_on_valid_target_is_ok() {
        let t = rust().build().unwrap();
        assert!(t.validate().is_ok());
    }
}
