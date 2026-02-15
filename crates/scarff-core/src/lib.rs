//! Scarff Core - Hexagonal Architecture Implementation
//!
//! This crate provides the domain and application layers for the Scarff
//! project scaffolding tool, following hexagonal (ports and adapters) architecture.
//!
//! ## Architecture Overview
//!
//! ```text
//! ┌─────────────────────────────────────────┐
//! │           scarff-cli (CLI)              │
//! │     (Implements Driving Ports)          │
//! └──────────────────┬──────────────────────┘
//!                    │ calls
//!                    ▼
//! ┌─────────────────────────────────────────┐
//! │         Application Services            │
//! │   (ScaffoldService, TemplateService)    │
//! │         Orchestrates Use Cases          │
//! └──────────────────┬──────────────────────┘
//!                    │ uses
//!                    ▼
//! ┌─────────────────────────────────────────┐
//! │      Application Ports (Traits)           │
//! │   (Driven: Store, Filesystem, Render)   │
//! └──────────────────┬──────────────────────┘
//!                    │ implemented by
//!                    ▼
//! ┌─────────────────────────────────────────┐
//! │      scarff-adapters (Infrastructure)     │
//! │   (InMemoryStore, LocalFilesystem, etc) │
//! └─────────────────────────────────────────┘
//!                    │
//!                    ▼
//! ┌─────────────────────────────────────────┐
//! │         Domain Layer (Pure Logic)         │
//! │   (Target, Template, ProjectStructure)   │
//! │         No External Dependencies         │
//! └─────────────────────────────────────────┘
//! ```
//!
//! ## Usage
//!
//! ```rust,no_run
//! use scarff_core::{
//!     application::{ScaffoldService, TemplateService},
//!     domain::{Target, Language, ProjectKind},
//! };
//!
//! // 1. Create target
//! let target = Target::builder()
//!     .language(Language::Rust)
//!     .kind(ProjectKind::Cli).unwrap()
//!     .build()
//!     .unwrap();
//!
//! // 2. Use application service (with injected adapters)
//! let service = ScaffoldService::new(store, filesystem, renderer);
//! service.scaffold(target, "my-project", "./output").unwrap();
//! ```

// Re-export domain layer (stable, well-defined API)
pub mod domain;

// Re-export application layer (orchestration logic)
pub mod application;

// Re-export error types
pub mod error;

// Public API - what external crates should use
pub mod prelude {
    pub use crate::application::{
        ScaffoldService, TemplateService,
        ports::{Filesystem, TemplateRenderer, TemplateStore},
    };
    pub use crate::domain::{
        Architecture, Framework, Language, ProjectKind, ProjectStructure, RenderContext, Target,
        TargetBuilder, Template, TemplateId, TemplateMetadata,
    };
    pub use crate::error::{ScarffError, ScarffResult};
}

// Version info
pub const VERSION: &str = env!("CARGO_PKG_VERSION");
