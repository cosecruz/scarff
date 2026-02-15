//! Simple variable substitution renderer.

use std::path::Path;

use scarff_core::{
    application::ports::TemplateRenderer,
    domain::{
        DirectoryToCreate, DomainValidator as validator, FileSpec, FileToWrite, FsEntry,
        Permissions, ProjectStructure, RenderContext, Template, TemplateContent, TemplateNode,
        TemplateSource,
    },
    error::ScarffResult,
};
use tracing::instrument;

/// Simple renderer using basic variable substitution.
pub struct SimpleRenderer;

impl SimpleRenderer {
    /// Create a new simple renderer.
    pub fn new() -> Self {
        Self
    }
}

impl Default for SimpleRenderer {
    fn default() -> Self {
        Self::new()
    }
}

impl TemplateRenderer for SimpleRenderer {
    #[instrument(skip_all)]
    fn render(
        &self,
        template: &Template,
        context: &RenderContext,
        output_root: &Path,
    ) -> ScarffResult<ProjectStructure> {
        // Validate template first
        validator::validate_template(template).map_err(scarff_core::error::ScarffError::Domain)?;

        let mut structure = ProjectStructure::new(output_root);

        // Render each node
        for node in &template.tree.nodes {
            match node {
                TemplateNode::File(spec) => {
                    let content = render_content(&spec.content, context)?;
                    structure.add_file(spec.path.as_path(), content, spec.permissions);
                }
                TemplateNode::Directory(spec) => {
                    structure.add_directory(spec.path.as_path(), spec.permissions);
                }
            }
        }

        // Validate final structure
        validator::validate_project_structure(&structure)
            .map_err(scarff_core::error::ScarffError::Domain)?;

        Ok(structure)
    }
}

fn render_content(content: &TemplateContent, ctx: &RenderContext) -> ScarffResult<String> {
    match content {
        TemplateContent::Literal(source) => Ok(source.as_str().to_string()),
        TemplateContent::Parameterized(source) => Ok(ctx.render(source.as_str())),
        TemplateContent::External(_) => Err(
            scarff_core::application::ApplicationError::RenderingFailed {
                reason: "External templates not supported by SimpleRenderer".into(),
            }
            .into(),
        ),
    }
}
