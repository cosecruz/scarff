//! Application services - orchestrate use cases.
//!
//! Services coordinate the domain layer and ports to accomplish
//! high-level use cases like "scaffold a project" or "resolve template".

pub mod scaffold_service;
pub mod template_service;

pub use scaffold_service::{ScaffoldService, TemplateInfo};
pub use template_service::TemplateService;
