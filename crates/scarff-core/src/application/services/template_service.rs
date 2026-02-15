//! Template Service - template management operations.
//!
//! Handles template CRUD operations and metadata queries.
//! Separated from ScaffoldService for single responsibility.

use crate::{
    application::{ApplicationError, ports::TemplateStore},
    domain::{Target, Template, TemplateId},
    error::ScarffResult,
};

/// Service for template operations.
pub struct TemplateService {
    store: Box<dyn TemplateStore>,
}

impl TemplateService {
    /// Create a new template service.
    pub fn new(store: Box<dyn TemplateStore>) -> Self {
        Self { store }
    }

    /// Get a template by ID.
    pub fn get(&self, id: &TemplateId) -> ScarffResult<Template> {
        self.store.get(id)
    }

    /// Add or update a template.
    pub fn save(&self, template: Template) -> ScarffResult<()> {
        self.store.insert(template)
    }

    /// Remove a template.
    pub fn remove(&self, id: &TemplateId) -> ScarffResult<()> {
        self.store.remove(id)
    }

    /// Find templates matching a target.
    pub fn find(&self, target: &Target) -> ScarffResult<Vec<Template>> {
        self.store.find(target)
    }

    /// List all templates.
    pub fn list(&self) -> ScarffResult<Vec<Template>> {
        self.store.list()
    }
}
