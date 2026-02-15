//! In-memory template store with built-in templates.

use std::{
    collections::HashMap,
    sync::{Arc, RwLock},
};

use scarff_core::{
    application::ports::TemplateStore,
    domain::{DomainValidator as validator, Target, Template, TemplateId},
    error::ScarffResult,
};

use crate::builtin_templates;

/// Thread-safe in-memory template store.
#[derive(Clone)]
pub struct InMemoryStore {
    inner: Arc<RwLock<HashMap<TemplateId, Template>>>,
}

impl InMemoryStore {
    /// Create a new empty store.
    pub fn new() -> Self {
        Self {
            inner: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Create a store with built-in templates loaded.
    pub fn with_builtin() -> ScarffResult<Self> {
        let store = Self::new();
        store.load_builtin()?;
        Ok(store)
    }

    /// Load built-in templates.
    pub fn load_builtin(&self) -> ScarffResult<()> {
        let templates = builtin_templates::all_templates()?;

        for template in templates {
            self.insert(template)?;
        }

        Ok(())
    }

    /// Get the number of templates.
    pub fn len(&self) -> usize {
        self.inner.read().unwrap().len()
    }

    /// Check if store is empty.
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Clear all templates.
    pub fn clear(&self) -> ScarffResult<()> {
        let mut inner = self
            .inner
            .write()
            .map_err(|_| scarff_core::application::ApplicationError::StoreLockError)?;
        inner.clear();
        Ok(())
    }
}

impl Default for InMemoryStore {
    fn default() -> Self {
        Self::new()
    }
}

impl TemplateStore for InMemoryStore {
    fn find(&self, target: &Target) -> ScarffResult<Vec<Template>> {
        let inner = self
            .inner
            .read()
            .map_err(|_| scarff_core::application::ApplicationError::StoreLockError)?;

        Ok(inner
            .values()
            .filter(|t| t.matcher.matches(target))
            .cloned()
            .collect())
    }

    fn get(&self, id: &TemplateId) -> ScarffResult<Template> {
        let inner = self
            .inner
            .read()
            .map_err(|_| scarff_core::application::ApplicationError::StoreLockError)?;

        inner.get(id).cloned().ok_or_else(|| {
            scarff_core::application::ApplicationError::TemplateResolution {
                reason: format!("Template not found: {}", id),
            }
            .into()
        })
    }

    fn list(&self) -> ScarffResult<Vec<Template>> {
        let inner = self
            .inner
            .read()
            .map_err(|_| scarff_core::application::ApplicationError::StoreLockError)?;

        Ok(inner.values().cloned().collect())
    }

    fn insert(&self, template: Template) -> ScarffResult<()> {
        // Validate before insertion
        validator::validate_template(&template)
            .map_err(|e| scarff_core::error::ScarffError::Domain(e))?;

        let mut inner = self
            .inner
            .write()
            .map_err(|_| scarff_core::application::ApplicationError::StoreLockError)?;

        let id = TemplateId::new(
            template.metadata.name.to_string(),
            template.metadata.version.to_string(),
        );

        inner.insert(id, template);
        Ok(())
    }

    fn remove(&self, id: &TemplateId) -> ScarffResult<()> {
        let mut inner = self
            .inner
            .write()
            .map_err(|_| scarff_core::application::ApplicationError::StoreLockError)?;

        inner.remove(id).ok_or_else(|| {
            scarff_core::application::ApplicationError::TemplateResolution {
                reason: format!("Template not found: {}", id),
            };
        });

        Ok(())
    }
}
