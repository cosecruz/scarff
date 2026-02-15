//! Domain value objects: Language, ProjectKind, Framework, Architecture.
//!
//! # Design
//!
//! These are pure value types — `Copy`, equality-by-value, no identity.
//! They hold NO capability logic. All compatibility and inference lives in
//! `capabilities.rs`. This file's only job is to define the types, their
//! string representations, and their `FromStr` parsers.
//!
//! # Adding New Variants
//!
//! 1. Add the enum variant here
//! 2. Add the `as_str` arm and the `FromStr` arm here
//! 3. Add a capability entry in `capabilities.rs`
//! 4. Done — nothing else changes

use crate::domain::error::DomainError;
use serde::{Deserialize, Serialize};
use std::fmt;
use std::str::FromStr;

// ── Language ─────────────────────────────────────────────────────────────────

/// A supported programming language.
///
/// To add a new language: add a variant here, then add a `LanguageDef` in
/// `capabilities.rs`. No other files change.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Language {
    Rust,
    Python,
    TypeScript,
    Go,
}

impl Language {
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Rust => "rust",
            Self::Python => "python",
            Self::TypeScript => "typescript",
            Self::Go => "go",
        }
    }

    pub const fn file_extension(&self) -> &'static str {
        match self {
            Self::Rust => "rs",
            Self::Python => "py",
            Self::TypeScript => "ts",
            Self::Go => "go",
        }
    }

    /// Whether this language supports the given project kind.
    ///
    /// Delegates to `capabilities::language_supports_kind`. Do not add match
    /// arms here — register capabilities in `capabilities.rs` instead.
    pub fn supports(self, kind: ProjectKind) -> bool {
        crate::domain::capabilities::language_supports_kind(self, kind)
    }
}

impl fmt::Display for Language {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

impl FromStr for Language {
    type Err = DomainError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_ascii_lowercase().as_str() {
            "rust" | "rs" => Ok(Self::Rust),
            "python" | "py" => Ok(Self::Python),
            "typescript" | "ts" => Ok(Self::TypeScript),
            "go" | "golang" => Ok(Self::Go),
            other => Err(DomainError::InvalidTarget(format!(
                "unknown language: {other}"
            ))),
        }
    }
}

// ── ProjectKind ───────────────────────────────────────────────────────────────

/// The type of project to scaffold.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum ProjectKind {
    Cli,
    WebBackend,
    WebFrontend,
    Fullstack,
    Worker,
    Library,
}

impl ProjectKind {
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Cli => "cli",
            Self::WebBackend => "web-backend",
            Self::WebFrontend => "web-frontend",
            Self::Fullstack => "fullstack",
            Self::Worker => "worker",
            Self::Library => "library",
        }
    }

    /// Whether this kind typically requires a framework.
    ///
    /// Used by `Target::validate()` to check that a framework was provided
    /// (or successfully inferred) for web-oriented kinds.
    pub const fn requires_framework(self) -> bool {
        matches!(self, Self::WebBackend | Self::WebFrontend | Self::Fullstack)
    }

    /// Default kind for a language when the user omits `--kind`.
    ///
    /// Delegates to `capabilities::infer_kind`.
    pub fn default_for(language: Language) -> Self {
        crate::domain::capabilities::infer_kind(language, None)
    }
}

impl fmt::Display for ProjectKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

impl FromStr for ProjectKind {
    type Err = DomainError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_ascii_lowercase().as_str() {
            "cli" => Ok(Self::Cli),
            "web-backend" | "backend" | "api" | "webbackend" => Ok(Self::WebBackend),
            "web-frontend" | "frontend" | "webfrontend" => Ok(Self::WebFrontend),
            "fullstack" => Ok(Self::Fullstack),
            "worker" => Ok(Self::Worker),
            "library" | "lib" => Ok(Self::Library),
            other => Err(DomainError::InvalidTarget(format!(
                "unknown project kind: {other}"
            ))),
        }
    }
}

// ── Framework ─────────────────────────────────────────────────────────────────

/// A framework, namespaced by its language.
///
/// Adding a framework: add the variant to the inner enum, add `as_str` arm,
/// add `FromStr` arm in the CLI layer, then add a `FrameworkDef` in
/// `capabilities.rs`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Framework {
    Rust(RustFramework),
    Python(PythonFramework),
    TypeScript(TypeScriptFramework),
    Go(GoFramework),
}

/// Rust-ecosystem web frameworks.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum RustFramework {
    Axum,
    Actix,
    Rocket,
}

/// Python web frameworks.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum PythonFramework {
    FastApi,
    Django,
    Flask,
}

/// TypeScript/JavaScript frameworks.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum TypeScriptFramework {
    Express,
    NestJs,
    React,
    Vue,
    NextJs,
    Svelte,
}

/// Go web frameworks.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum GoFramework {
    Gin,
    Echo,
    Stdlib,
}

impl Framework {
    /// The language this framework belongs to.
    ///
    /// This is the only piece of framework knowledge that lives here rather
    /// than in `capabilities.rs`, because it is an intrinsic property of the
    /// type (Axum is always a Rust framework) rather than a capability rule.
    pub fn language(&self) -> Language {
        match self {
            Self::Rust(_) => Language::Rust,
            Self::Python(_) => Language::Python,
            Self::TypeScript(_) => Language::TypeScript,
            Self::Go(_) => Language::Go,
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Rust(RustFramework::Axum) => "axum",
            Self::Rust(RustFramework::Actix) => "actix",
            Self::Rust(RustFramework::Rocket) => "rocket",
            Self::Python(PythonFramework::FastApi) => "fastapi",
            Self::Python(PythonFramework::Django) => "django",
            Self::Python(PythonFramework::Flask) => "flask",
            Self::TypeScript(TypeScriptFramework::Express) => "express",
            Self::TypeScript(TypeScriptFramework::NestJs) => "nestjs",
            Self::TypeScript(TypeScriptFramework::React) => "react",
            Self::TypeScript(TypeScriptFramework::Vue) => "vue",
            Self::TypeScript(TypeScriptFramework::NextJs) => "nextjs",
            Self::TypeScript(TypeScriptFramework::Svelte) => "svelte",
            Self::Go(GoFramework::Gin) => "gin",
            Self::Go(GoFramework::Echo) => "echo",
            Self::Go(GoFramework::Stdlib) => "stdlib",
        }
    }

    /// Whether this framework supports the given (language, kind) combination.
    ///
    /// Delegates to `capabilities::validate_framework_compatibility`.
    pub fn is_compatible_with(self, language: Language, kind: ProjectKind) -> bool {
        crate::domain::capabilities::validate_framework_compatibility(self, language, kind).is_ok()
    }

    /// Infer the default framework for a (language, kind) pair.
    ///
    /// Delegates to `capabilities::infer_framework`.
    pub fn infer(language: Language, kind: ProjectKind) -> Option<Self> {
        crate::domain::capabilities::infer_framework(language, kind)
    }
}

impl fmt::Display for Framework {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

// ── Architecture ──────────────────────────────────────────────────────────────

/// Architectural patterns for project organisation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Architecture {
    /// Classic presentation → application → domain → infrastructure layering.
    Layered,
    /// Model-View-Controller (primarily for Django's MVT fullstack).
    Mvc,
    /// Clean / Hexagonal / Onion (ports-and-adapters).
    Clean,
    /// Feature-first modular structure (NestJs, large TypeScript projects).
    FeatureModular,
}

impl Architecture {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Layered => "layered",
            Self::Mvc => "mvc",
            Self::Clean => "clean",
            Self::FeatureModular => "feature-modular",
        }
    }

    /// Whether this architecture is compatible with the given constraints.
    ///
    /// Rather than hard-coding framework-specific rules here (the old
    /// anti-pattern), this now uses a simple universal rule:
    ///
    /// - `Mvc` is only valid when `capabilities::infer_architecture` would
    ///   also produce `Mvc` for this combination. This keeps the rule
    ///   centrally defined.
    /// - All other architectures are universally compatible (the user can
    ///   always override the default; we just pick a sensible one).
    pub fn is_compatible_with(
        self,
        language: Language,
        kind: ProjectKind,
        framework: Option<Framework>,
    ) -> bool {
        match self {
            // MVC is only valid when the inferred architecture is also MVC.
            // This avoids duplicating the "Django fullstack = MVC" rule here.
            Self::Mvc => {
                crate::domain::capabilities::infer_architecture(language, kind, framework)
                    == Architecture::Mvc
            }
            // All other architectures are user-overridable.
            Self::Layered | Self::Clean | Self::FeatureModular => true,
        }
    }

    /// Infer a reasonable architecture. Delegates to `capabilities`.
    pub fn infer(language: Language, kind: ProjectKind, framework: Option<Framework>) -> Self {
        crate::domain::capabilities::infer_architecture(language, kind, framework)
    }
}

impl fmt::Display for Architecture {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

impl FromStr for Architecture {
    type Err = DomainError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_ascii_lowercase().as_str() {
            "layered" => Ok(Self::Layered),
            "mvc" => Ok(Self::Mvc),
            "clean" | "hexagonal" | "onion" => Ok(Self::Clean),
            "feature-modular" | "modular" | "featuremodular" => Ok(Self::FeatureModular),
            other => Err(DomainError::InvalidTarget(format!(
                "unknown architecture: {other}"
            ))),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn language_display_is_lowercase() {
        assert_eq!(Language::Rust.to_string(), "rust");
        assert_eq!(Language::TypeScript.to_string(), "typescript");
    }

    #[test]
    fn language_from_str_accepts_aliases() {
        assert_eq!("rs".parse::<Language>().unwrap(), Language::Rust);
        assert_eq!("py".parse::<Language>().unwrap(), Language::Python);
        assert_eq!("ts".parse::<Language>().unwrap(), Language::TypeScript);
        assert_eq!("golang".parse::<Language>().unwrap(), Language::Go);
    }

    #[test]
    fn language_from_str_unknown_errors() {
        assert!("java".parse::<Language>().is_err());
        assert!("".parse::<Language>().is_err());
    }

    #[test]
    fn project_kind_from_str_accepts_aliases() {
        assert_eq!(
            "backend".parse::<ProjectKind>().unwrap(),
            ProjectKind::WebBackend
        );
        assert_eq!(
            "api".parse::<ProjectKind>().unwrap(),
            ProjectKind::WebBackend
        );
        assert_eq!(
            "frontend".parse::<ProjectKind>().unwrap(),
            ProjectKind::WebFrontend
        );
        assert_eq!("lib".parse::<ProjectKind>().unwrap(), ProjectKind::Library);
    }

    #[test]
    fn project_kind_requires_framework_for_web_kinds() {
        assert!(ProjectKind::WebBackend.requires_framework());
        assert!(ProjectKind::WebFrontend.requires_framework());
        assert!(ProjectKind::Fullstack.requires_framework());
        assert!(!ProjectKind::Cli.requires_framework());
        assert!(!ProjectKind::Library.requires_framework());
        assert!(!ProjectKind::Worker.requires_framework());
    }

    #[test]
    fn project_kind_default_for_delegates_to_capabilities() {
        assert_eq!(ProjectKind::default_for(Language::Rust), ProjectKind::Cli);
        assert_eq!(
            ProjectKind::default_for(Language::Python),
            ProjectKind::WebBackend
        );
    }

    #[test]
    fn framework_language_is_correct() {
        assert_eq!(
            Framework::Rust(RustFramework::Axum).language(),
            Language::Rust
        );
        assert_eq!(
            Framework::Python(PythonFramework::Django).language(),
            Language::Python
        );
    }

    #[test]
    fn architecture_mvc_only_compatible_with_django_fullstack() {
        // MVC is only valid when the inferred arch would also be MVC.
        let django = Some(Framework::Python(PythonFramework::Django));
        assert!(Architecture::Mvc.is_compatible_with(
            Language::Python,
            ProjectKind::Fullstack,
            django
        ));
        assert!(!Architecture::Mvc.is_compatible_with(Language::Rust, ProjectKind::Cli, None));
        assert!(!Architecture::Mvc.is_compatible_with(
            Language::Python,
            ProjectKind::WebBackend,
            None
        ));
    }

    #[test]
    fn architecture_clean_is_universally_compatible() {
        assert!(Architecture::Clean.is_compatible_with(Language::Rust, ProjectKind::Cli, None));
        assert!(Architecture::Clean.is_compatible_with(
            Language::Python,
            ProjectKind::Fullstack,
            Some(Framework::Python(PythonFramework::Django))
        ));
    }

    #[test]
    fn architecture_from_str_accepts_aliases() {
        assert_eq!(
            "hexagonal".parse::<Architecture>().unwrap(),
            Architecture::Clean
        );
        assert_eq!(
            "onion".parse::<Architecture>().unwrap(),
            Architecture::Clean
        );
        assert_eq!(
            "modular".parse::<Architecture>().unwrap(),
            Architecture::FeatureModular
        );
    }
}

// /// Extended architecture patterns for advanced usage.
// #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
// pub enum ArchitecturePattern {
//     Layered,
//     Hexagonal, // Explicit hexagonal/ports-and-adapters
//     Onion,     // Concentric circles
//     Clean,     // Uncle Bob's Clean Architecture
//     FeatureModular,
//     Microkernel,
// }

// impl ArchitecturePattern {
//     /// Convert to basic Architecture enum.
//     pub fn to_architecture(self) -> Architecture {
//         match self {
//             Self::Layered => Architecture::Layered,
//             Self::Hexagonal | Self::Onion | Self::Clean => Architecture::Clean,
//             Self::FeatureModular => Architecture::FeatureModular,
//             Self::Microkernel => Architecture::Layered, // Maps to layered for simplicity
//         }
//     }

//     /// Get default directory structure for this pattern.
//     pub fn default_structure(&self) -> Vec<&'static str> {
//         match self {
//             Self::Layered => vec![
//                 "src/presentation",
//                 "src/application",
//                 "src/domain",
//                 "src/infrastructure",
//             ],
//             Self::Hexagonal => vec![
//                 "src/domain",
//                 "src/application/ports",
//                 "src/application/services",
//                 "src/adapters/in",
//                 "src/adapters/out",
//                 "src/configuration",
//             ],
//             Self::Onion => vec![
//                 "src/core/domain",
//                 "src/core/use_cases",
//                 "src/interfaces",
//                 "src/infrastructure",
//             ],
//             Self::Clean => vec![
//                 "src/entities",
//                 "src/use_cases",
//                 "src/interface_adapters",
//                 "src/frameworks",
//             ],
//             Self::FeatureModular => vec!["src/features", "src/shared"],
//             Self::Microkernel => vec!["src/core", "src/plugins", "src/platform"],
//         }
//     }
// }
