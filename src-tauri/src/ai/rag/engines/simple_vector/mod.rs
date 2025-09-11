pub mod core; // Main RAGSimpleVectorEngine struct and basic methods
pub mod indexing; // File processing, indexing methods, and embedding operations
pub mod overlap; // Overlap management and semantic boundaries
pub mod queries; // Database query functions
pub mod querying; // Query processing methods
pub mod types; // All type definitions, structs, enums, and configurations
pub mod utils; // Utility functions and helpers
// Enterprise reranking infrastructure
// pub mod tokens;     // Token management and allocation systems

// Re-export types and main engine for external use
pub use core::RAGSimpleVectorEngine;
pub use types::*;
