//! Application ports (traits) for external dependencies.
//!
//! In hexagonal architecture, ports define interfaces that the application
//! needs from the outside world. Adapters in `scarff-adapters` implement these.
//!
//! ## Port Types
//!
//! - **Driven (Output) Ports**: Called by application, implemented by infrastructure
//!   - `Filesystem`: File operations
//!   - `TemplateStore`: Template storage/retrieval
//!   - `TemplateRenderer`: Template rendering
//!
//! - **Driving (Input) Ports**: Called by external world, implemented by application
//!   - (Defined in CLI layer, implemented by services)

pub mod output;

pub use output::{Filesystem, TemplateRenderer, TemplateStore};
