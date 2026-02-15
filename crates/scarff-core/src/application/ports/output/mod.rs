//! Driven (output) ports - implemented by infrastructure.
//!
//! These traits define what the application needs from external systems.
//! The `scarff-adapters` crate provides implementations.

use crate::application::ApplicationError;
use crate::domain::{
    ProjectStructure, RenderContext, Target, Template, TemplateId, TemplateRecord,
};
use crate::error::ScarffResult;
use std::path::Path;

/// Port for filesystem operations.
///
/// Implemented by:
/// - `scarff_adapters::filesystem::LocalFilesystem` (production)
/// - `scarff_adapters::filesystem::MemoryFilesystem` (testing)
///
/// ## Design Notes
///
/// - All paths are relative to avoid absolute path issues
/// - Permissions are capability-based, not Unix-specific
/// - Async-ready (can be extended with async-trait later)
pub trait Filesystem: Send + Sync {
    /// Create a directory and all parent directories.
    fn create_dir_all(&self, path: &Path) -> ScarffResult<()>;

    /// Write content to a file.
    fn write_file(&self, path: &Path, content: &str) -> ScarffResult<()>;

    /// Set file permissions.
    fn set_permissions(&self, path: &Path, executable: bool) -> ScarffResult<()>;

    /// Check if path exists.
    fn exists(&self, path: &Path) -> bool;

    /// Remove a directory and all contents.
    fn remove_dir_all(&self, path: &Path) -> ScarffResult<()>;
}

/// Port for template storage and retrieval.
///
/// Implemented by:
/// - `scarff_adapters::template_store::InMemoryStore` (built-in templates)
/// - `scarff_adapters::template_store::FileSystemStore` (future: user templates)
/// - `scarff_adapters::template_store::RemoteStore` (future: registry)
pub trait TemplateStore: Send + Sync {
    /// Find all templates matching a target.
    fn find(&self, target: &Target) -> ScarffResult<Vec<Template>>;

    /// Get a specific template by ID.
    fn get(&self, id: &TemplateId) -> ScarffResult<Template>;

    /// List all available templates.
    fn list(&self) -> ScarffResult<Vec<Template>>;

    /// Insert or update a template.
    fn insert(&self, template: Template) -> ScarffResult<()>;

    /// Remove a template.
    fn remove(&self, id: &TemplateId) -> ScarffResult<()>;
}

/// Port for template rendering.
///
/// Implemented by:
/// - `scarff_adapters::renderer::SimpleRenderer` (variable substitution)
/// - `scarff_adapters::renderer::TeraRenderer` (future: Tera templates)
/// - `scarff_adapters::renderer::HandlebarsRenderer` (future: Handlebars)
pub trait TemplateRenderer: Send + Sync {
    /// Render a template into a project structure.
    ///
    /// # Arguments
    ///
    /// * `template` - The template to render
    /// * `context` - Variable substitution context
    /// * `output_root` - Root directory for output paths
    fn render(
        &self,
        template: &Template,
        context: &RenderContext,
        output_root: &Path,
    ) -> ScarffResult<ProjectStructure>;
}
