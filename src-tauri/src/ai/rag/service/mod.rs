// RAG Service module organization

pub mod core;
pub mod processor;
pub mod queries;

// Re-export the main service and status types
pub use core::{RAGService, RAGServiceStatus};
pub use queries::reset_processing_files_to_pending;
