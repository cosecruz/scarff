//! Infrastructure adapters for Scarff.
//!
//! This crate implements the ports defined in `scarff-core::application::ports`.
//! It contains all external dependencies and I/O operations.

pub mod builtin_templates;
pub mod filesystem;
pub mod renderer;
pub mod template_loader;
pub mod template_store;

// Re-export commonly used adapters
pub use filesystem::{LocalFilesystem, MemoryFilesystem};
pub use renderer::SimpleRenderer;
pub use template_store::InMemoryStore;
