//! Filesystem adapters.

mod local;
mod memory;

pub use local::LocalFilesystem;
pub use memory::MemoryFilesystem;
