//! Application layer for Scarff.
//!
//! This layer contains:
//! - **Services**: Use case orchestration (ScaffoldService, TemplateService)
//! - **Ports**: Interface definitions (traits) for external dependencies
//! - **Errors**: Application-specific error types
//!
//! The application layer coordinates the domain layer but contains no
//! business logic itself. All business rules live in `crate::domain`.

pub mod error;
pub mod ports;
pub mod services;

// Re-export main services
pub use services::{
    ScaffoldService,
    TemplateInfo, // DTO for template metadata
    TemplateService,
};

// Re-export port traits (for adapter implementation)
pub use ports::{Filesystem, TemplateRenderer, TemplateStore};

pub use error::ApplicationError;
