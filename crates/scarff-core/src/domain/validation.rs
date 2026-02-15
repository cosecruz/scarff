use crate::domain::{
    entities::{ProjectStructure, Target, Template, TemplateRecord},
    error::DomainError,
};

/// Centralized domain validation.
///
/// All validation logic lives here, not scattered across entities.
pub struct DomainValidator;

impl DomainValidator {
    pub fn validate_target(target: &Target) -> Result<(), DomainError> {
        target.validate()
    }

    pub fn validate_template(template: &Template) -> Result<(), DomainError> {
        template.validate()
    }

    pub fn validate_template_record(record: &TemplateRecord) -> Result<(), DomainError> {
        record.validate()
    }

    pub fn validate_project_structure(structure: &ProjectStructure) -> Result<(), DomainError> {
        structure.validate()
    }
}
