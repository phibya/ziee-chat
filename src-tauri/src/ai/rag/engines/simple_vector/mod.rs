pub mod core; // Main RAGSimpleVectorEngine struct and basic methods
pub mod embeddings; // Embedding processing and batch operations
pub mod overlap; // Overlap management and semantic boundaries
pub mod types; // All type definitions, structs, enums, and configurations // Enterprise reranking infrastructure
               // pub mod tokens;     // Token management and allocation systems

// Re-export types and main engine for external use
pub use core::RAGSimpleVectorEngine;
pub use types::*;
