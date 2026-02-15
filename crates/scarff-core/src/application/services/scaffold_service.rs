//! Scaffold Service - main application orchestrator.
//!
//! This service coordinates the entire scaffolding workflow:
//! 1. Resolve template for target
//! 2. Render template with context
//! 3. Write to filesystem
//!
//! It implements the driving port (incoming) and uses driven ports (outgoing).

use std::path::{Path, PathBuf};
use tracing::{info, instrument, warn};

use crate::{
    application::{
        ApplicationError,
        ports::{Filesystem, TemplateRenderer, TemplateStore},
    },
    domain::{DomainValidator as validator, ProjectStructure, RenderContext, Target, Template},
    error::{ScarffError, ScarffResult},
};

/// Information about a template for display purposes.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TemplateInfo {
    pub id: String,
    pub name: String,
    pub description: String,
    pub language: String,
    pub kind: String,
    pub architecture: String,
    pub framework: Option<String>,
}

/// Main scaffolding service.
///
/// Orchestrates the template resolution, rendering, and writing workflow.
pub struct ScaffoldService {
    store: Box<dyn TemplateStore>,
    renderer: Box<dyn TemplateRenderer>,
    filesystem: Box<dyn Filesystem>,
}

impl ScaffoldService {
    /// Create a new scaffold service with the given adapters.
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use scarff_core::application::{ScaffoldService, ports::*};
    ///
    /// let service = ScaffoldService::new(
    ///     store,      // impl TemplateStore
    ///     renderer,   // impl TemplateRenderer
    ///     filesystem, // impl Filesystem
    /// );
    /// ```
    pub fn new(
        store: Box<dyn TemplateStore>,
        renderer: Box<dyn TemplateRenderer>,
        filesystem: Box<dyn Filesystem>,
    ) -> Self {
        Self {
            store,
            renderer,
            filesystem,
        }
    }

    /// Scaffold a new project.
    ///
    /// This is the main use case - creates a project from a target configuration.
    #[instrument(
        skip_all,
        fields(
            target = %target,
            project = %project_name.as_ref(),
            output_path = %output_path.as_ref().display()
        )
    )]
    pub fn scaffold(
        &self,
        target: Target,
        project_name: impl AsRef<str>,
        output_path: impl AsRef<Path>,
    ) -> ScarffResult<()> {
        info!(
            "Scaffolding {} {} project",
            target.language(),
            target.kind()
        );

        let project_name = project_name.as_ref();
        let output_path = output_path.as_ref();

        // 1. Validate target
        validator::validate_target(&target).map_err(ScarffError::Domain)?;

        // 2. Resolve template
        let template = self.resolve_template(&target)?;
        info!(template_name = %template.metadata.name, "Template resolved");

        // 3. Create render context
        let context = RenderContext::new(project_name);
        // TODO: depending on target.language render project_name to fit norm

        // 4. Render template
        let structure = self.renderer.render(&template, &context, output_path)?;
        // info!(
        //     files = structure.file_count(),
        //     directories = structure.directory_count(),
        //     "Template rendered"
        // );

        // 5. Write to filesystem
        self.write_structure(&structure)?;

        info!("Scaffold completed successfully");
        Ok(())
    }

    /// List all available templates.
    pub fn list_templates(&self) -> ScarffResult<Vec<TemplateInfo>> {
        let templates = self.store.list()?;

        Ok(templates
            .into_iter()
            .map(|t| TemplateInfo {
                id: format!("{}@{}", t.metadata.name, t.metadata.version),
                name: t.metadata.name.to_string(),
                description: t.metadata.description.to_string(),
                language: t
                    .matcher
                    .language
                    .map_or_else(|| "any".to_string(), |l| l.to_string()),
                kind: t
                    .matcher
                    .kind
                    .map_or_else(|| "any".to_string(), |k| k.to_string()),
                architecture: t
                    .matcher
                    .architecture
                    .map_or_else(|| "any".to_string(), |a| a.to_string()),
                framework: t.matcher.framework.map(|f| f.to_string()),
            })
            .collect())
    }

    /// Find templates matching a target.
    pub fn find_templates(&self, target: &Target) -> ScarffResult<Vec<TemplateInfo>> {
        let templates = self.store.find(target)?;

        Ok(templates
            .into_iter()
            .map(|t| TemplateInfo {
                id: format!("{}@{}", t.metadata.name, t.metadata.version),
                name: t.metadata.name.to_string(),
                description: t.metadata.description.to_string(),
                language: t
                    .matcher
                    .language
                    .map_or_else(|| "any".to_string(), |l| l.to_string()),
                kind: t
                    .matcher
                    .kind
                    .map_or_else(|| "any".to_string(), |k| k.to_string()),
                architecture: t
                    .matcher
                    .architecture
                    .map_or_else(|| "any".to_string(), |a| a.to_string()),
                framework: t.matcher.framework.map(|f| f.to_string()),
            })
            .collect())
    }

    // -------------------------------------------------------------------------
    // Internal Helpers
    // -------------------------------------------------------------------------

    /// Resolve the best matching template for a target.
    fn resolve_template(&self, target: &Target) -> ScarffResult<Template> {
        let matches = self.store.find(target)?;

        if matches.is_empty() {
            return Err(ApplicationError::TemplateResolution {
                reason: format!("No template matches target: {}", target),
            }
            .into());
        }

        if matches.len() == 1 {
            return Ok(matches.into_iter().next().unwrap());
        }

        // Multiple matches - select by specificity
        let max_specificity = matches
            .iter()
            .map(|t| t.matcher.specificity())
            .max()
            .unwrap();

        let best_matches: Vec<_> = matches
            .into_iter()
            .filter(|t| t.matcher.specificity() == max_specificity)
            .collect();

        if best_matches.len() > 1 {
            return Err(ApplicationError::TemplateResolution {
                reason: format!(
                    "Ambiguous: {} templates match with equal specificity",
                    best_matches.len()
                ),
            }
            .into());
        }

        Ok(best_matches.into_iter().next().unwrap())
    }

    /// Write project structure to filesystem with rollback on failure.
    fn write_structure(&self, structure: &ProjectStructure) -> ScarffResult<()> {
        // Check if project exists
        if self.filesystem.exists(&structure.root) {
            return Err(ApplicationError::ProjectExists {
                path: structure.root.clone(),
            }
            .into());
        }

        // Try to write everything
        match self.write_all(structure) {
            Ok(()) => {
                info!("Successfully wrote all files");
                Ok(())
            }
            Err(e) => {
                warn!("Write failed, attempting rollback");
                self.rollback(&structure.root);
                Err(e)
            }
        }
    }

    /// Write all entries in the structure.
    fn write_all(&self, structure: &ProjectStructure) -> ScarffResult<()> {
        // Create root
        self.filesystem.create_dir_all(&structure.root)?;

        // Write entries
        for entry in &structure.entries {
            match entry {
                crate::domain::FsEntry::Directory(dir) => {
                    let path = structure.root.join(&dir.path);
                    self.filesystem.create_dir_all(&path)?;
                }
                crate::domain::FsEntry::File(file) => {
                    let path = structure.root.join(&file.path);

                    // Ensure parent exists
                    if let Some(parent) = path.parent() {
                        self.filesystem.create_dir_all(parent)?;
                    }

                    self.filesystem.write_file(&path, &file.content)?;

                    if file.permissions.executable_flag() {
                        self.filesystem.set_permissions(&path, true)?;
                    }
                }
            }
        }

        Ok(())
    }

    /// Best-effort rollback on failure.
    fn rollback(&self, root: &Path) {
        if let Err(e) = self.filesystem.remove_dir_all(root) {
            warn!(
                error = %e,
                path = %root.display(),
                "Rollback failed"
            );
        } else {
            info!("Rollback successful");
        }
    }
}
