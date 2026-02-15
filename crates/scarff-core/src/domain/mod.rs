// ============================================================================
//  CLEAN MODULE BOUNDARIES
// ============================================================================

//! Core domain layer for Scarff.
//!
//! This module contains pure business logic with ZERO external dependencies.
//! All I/O, templating, and rendering concerns are handled via ports (traits)
//! defined in the application layer.
//!
//! ## Hexagonal Architecture Compliance
//!
//! - **No async**: Domain logic is synchronous
//! - **No I/O**: No filesystem, network, or external calls
//! - **No external crates**: Only std library + thiserror
//! - **Immutable entities**: All domain objects are Clone + PartialEq
//! - **Rich domain model**: Behavior lives in entities, not services
//!
// Public API - what the world sees
pub mod capabilities;
pub mod entities;
pub mod error;
pub mod value_objects;

// Private implementation details - not visible outside domain
mod validation;

// mod validator;

// Re-exports for convenience
pub use entities::{
    project_structure::{DirectoryToCreate, FileToWrite, FsEntry, ProjectStructure},
    target::{Target, TargetBuilder},
    template::{
        ContentTemplateId, DirectorySpec, FileSpec, RenderContext, TargetMatcher,
        TargetMatcherBuilder, Template, TemplateBuilder, TemplateContent, TemplateId,
        TemplateMetadata, TemplateNode, TemplateRecord, TemplateSource, TemplateTree,
    },
};

pub use error::{DomainError, ErrorCategory};

pub use value_objects::{
    Architecture, Framework, Language, ProjectKind, PythonFramework, RustFramework,
    TypeScriptFramework,
};

// Internal only - not re-exported
pub use entities::common::{Permissions, RelativePath};
pub use validation::DomainValidator;
// pub use common::{Permissions, RelativePath};
// pub use error::DomainError;

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use super::*;
    use entities::*;
    use value_objects::*;

    // ========================================================================
    // Value Object Tests
    // ========================================================================

    #[test]
    fn language_parses_correctly() {
        assert_eq!(Language::from_str("rust").unwrap(), Language::Rust);
        assert_eq!(Language::from_str("RS").unwrap(), Language::Rust);
        assert!(Language::from_str("java").is_err());
    }

    #[test]
    fn language_supports_project_kinds() {
        assert!(Language::Rust.supports(ProjectKind::Cli));
        assert!(Language::Rust.supports(ProjectKind::WebBackend));
        assert!(!Language::Rust.supports(ProjectKind::WebFrontend));

        assert!(Language::TypeScript.supports(ProjectKind::WebFrontend));
        assert!(!Language::TypeScript.supports(ProjectKind::Cli)); // Technically possible but not "supported"
    }

    #[test]
    fn framework_compatibility() {
        let fw = Framework::Rust(RustFramework::Axum);
        assert!(fw.is_compatible_with(Language::Rust, ProjectKind::WebBackend));
        assert!(!fw.is_compatible_with(Language::Python, ProjectKind::WebBackend));
        assert!(!fw.is_compatible_with(Language::Rust, ProjectKind::WebFrontend));
    }

    #[test]
    fn framework_inference() {
        assert_eq!(
            Framework::infer(Language::Rust, ProjectKind::WebBackend),
            Some(Framework::Rust(RustFramework::Axum))
        );
        assert_eq!(
            Framework::infer(Language::Rust, ProjectKind::Cli),
            None // CLI doesn't need framework
        );
    }

    #[test]
    fn architecture_compatibility() {
        assert!(Architecture::Layered.is_compatible_with(Language::Rust, ProjectKind::Cli, None));

        // MVC only with Django
        let django = Some(Framework::Python(PythonFramework::Django));
        assert!(Architecture::Mvc.is_compatible_with(
            Language::Python,
            ProjectKind::Fullstack,
            django
        ));
        assert!(!Architecture::Mvc.is_compatible_with(Language::Rust, ProjectKind::Cli, None));
    }

    // ========================================================================
    // Target Builder Tests (Typestate)
    // ========================================================================

    #[test]
    fn target_builder_basic() {
        let target = Target::builder().language(Language::Rust).build().unwrap();

        assert_eq!(target.language(), Language::Rust);
        assert_eq!(target.kind(), ProjectKind::Cli); // Default
        assert_eq!(target.architecture(), Architecture::Layered); // Default
    }

    #[test]
    fn target_builder_full() {
        let target = Target::builder()
            .language(Language::Rust)
            .kind(ProjectKind::WebBackend)
            .unwrap()
            .framework(Framework::Rust(RustFramework::Axum))
            .unwrap()
            .architecture(Architecture::Clean)
            .build()
            .unwrap();

        assert_eq!(
            target.framework(),
            Some(Framework::Rust(RustFramework::Axum))
        );
        assert_eq!(target.architecture(), Architecture::Clean);
    }

    #[test]
    fn target_builder_rejects_incompatible_framework() {
        let result = Target::builder()
            .language(Language::Rust)
            .framework(Framework::Python(PythonFramework::Django));

        assert!(result.is_err());
    }

    #[test]
    fn target_builder_rejects_incompatible_kind() {
        let result = Target::builder()
            .language(Language::Rust)
            .kind(ProjectKind::WebFrontend);

        assert!(result.is_err());
    }

    #[test]
    fn target_requires_framework_for_web() {
        // Should auto-infer framework
        let target = Target::builder()
            .language(Language::Python)
            .kind(ProjectKind::WebBackend)
            .unwrap()
            .build()
            .unwrap();

        assert!(target.framework().is_some());
    }

    // ========================================================================
    // Template Tests
    // ========================================================================

    #[test]
    fn template_builder_success() {
        let template = Template::builder()
            .id(TemplateId::new("test", "1.0.0"))
            .matcher(TargetMatcher::builder().language(Language::Rust).build())
            .metadata(TemplateMetadata::new("Test"))
            .add_node(TemplateNode::Directory(DirectorySpec::new("src")))
            .build()
            .unwrap();

        assert_eq!(template.id.name(), "test");
        assert!(template.matches(&Target::builder().language(Language::Rust).build().unwrap()));
    }

    #[test]
    fn template_builder_rejects_empty_tree() {
        let result = Template::builder()
            .id(TemplateId::new("test", "1.0.0"))
            .matcher(TargetMatcher::default())
            .metadata(TemplateMetadata::new("Test"))
            .build();

        assert!(result.is_err());
    }

    #[test]
    fn template_validates_duplicate_paths() {
        let template = Template::builder()
            .id(TemplateId::new("test", "1.0.0"))
            .matcher(TargetMatcher::default())
            .metadata(TemplateMetadata::new("Test"))
            .add_node(TemplateNode::Directory(DirectorySpec::new("src")))
            .add_node(TemplateNode::Directory(DirectorySpec::new("src"))) // Duplicate
            .build()
            .unwrap();

        assert!(template.validate().is_err());
    }

    #[test]
    fn template_specificity_calculation() {
        let template = Template::builder()
            .id(TemplateId::new("specific", "1.0.0"))
            .matcher(
                TargetMatcher::builder()
                    .language(Language::Rust)
                    .kind(ProjectKind::Cli)
                    .build(),
            )
            .metadata(TemplateMetadata::new("Specific"))
            .add_node(TemplateNode::Directory(DirectorySpec::new("src")))
            .build()
            .unwrap();

        assert_eq!(template.specificity(), 2);
    }

    #[test]
    fn template_id_parsing() {
        let id = TemplateId::parse("my-template@2.0.0").unwrap();
        assert_eq!(id.name(), "my-template");
        assert_eq!(id.version(), "2.0.0");

        assert!(TemplateId::parse("invalid").is_err());
        assert!(TemplateId::parse("too@many@ats").is_err());
    }

    #[test]
    #[should_panic]
    fn template_id_rejects_at_in_name() {
        TemplateId::new("invalid@name", "1.0.0");
    }

    // ========================================================================
    // Matcher Tests
    // ========================================================================

    #[test]
    fn target_matcher_wildcard_matches_all() {
        let matcher = TargetMatcher::default();
        let target = Target::builder().language(Language::Rust).build().unwrap();

        assert!(matcher.matches(&target));
    }

    #[test]
    fn target_matcher_specificity() {
        let m1 = TargetMatcher::default();
        let m2 = TargetMatcher::builder().language(Language::Rust).build();
        let m3 = TargetMatcher::builder()
            .language(Language::Rust)
            .kind(ProjectKind::Cli)
            .build();

        assert_eq!(m1.specificity(), 0);
        assert_eq!(m2.specificity(), 1);
        assert_eq!(m3.specificity(), 2);
    }

    #[test]
    fn target_matcher_partial_match() {
        let matcher = TargetMatcher::builder().language(Language::Rust).build();

        let rust_cli = Target::builder()
            .language(Language::Rust)
            .kind(ProjectKind::Cli)
            .unwrap()
            .build()
            .unwrap();

        let python_cli = Target::builder()
            .language(Language::Python)
            .kind(ProjectKind::Cli)
            .unwrap()
            .build()
            .unwrap();

        assert!(matcher.matches(&rust_cli));
        assert!(!matcher.matches(&python_cli));
    }

    // ========================================================================
    // Project Structure Tests
    // ========================================================================

    #[test]
    fn project_structure_builds_correctly() {
        let structure = ProjectStructure::new("/tmp/test")
            .with_directory("src", Permissions::read_write())
            .with_file(
                "src/main.rs",
                "fn main() {}".into(),
                Permissions::read_write(),
            );

        assert_eq!(structure.entry_count(), 2);
        assert_eq!(structure.files().count(), 1);
        assert_eq!(structure.directories().count(), 1);
    }

    #[test]
    fn project_structure_validates_duplicates() {
        let structure = ProjectStructure::new("/tmp/test")
            .with_file("main.rs", "".into(), Permissions::read_write())
            .with_file("main.rs", "".into(), Permissions::read_write());

        assert!(structure.validate().is_err());
    }

    #[test]
    fn project_structure_validates_empty() {
        let structure = ProjectStructure::new("/tmp/test");
        assert!(structure.validate().is_err());
    }

    // ========================================================================
    // Render Context Tests
    // ========================================================================

    #[test]
    fn render_context_standard_variables() {
        let ctx = RenderContext::new("my awesome project");

        assert_eq!(ctx.get("PROJECT_NAME"), Some("my awesome project"));
        assert_eq!(ctx.get("PROJECT_NAME_SNAKE"), Some("my_awesome_project"));
        assert_eq!(ctx.get("PROJECT_NAME_KEBAB"), Some("my-awesome-project"));
        assert_eq!(ctx.get("PROJECT_NAME_PASCAL"), Some("MyAwesomeProject"));
    }

    #[test]
    fn render_context_custom_variables() {
        let ctx = RenderContext::new("test").with_variable("AUTHOR", "Alice");

        assert_eq!(ctx.get("AUTHOR"), Some("Alice"));
    }

    #[test]
    fn render_context_renders_template() {
        let ctx = RenderContext::new("my-project");
        let template = "Project: {{PROJECT_NAME}}, Year: {{YEAR}}";

        assert_eq!(ctx.render(template), "Project: my-project, Year: 2026");
    }
}
