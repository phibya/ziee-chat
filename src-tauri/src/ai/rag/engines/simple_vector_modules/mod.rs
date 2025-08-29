// Simple Vector RAG Engine - Modular Organization Structure
//
// This folder demonstrates how the large simple_vector.rs file (3914 lines) 
// could be organized into focused, maintainable modules.
//
// The actual implementation remains in ../simple_vector.rs to ensure 
// compilation integrity and working functionality.
//
// This structure serves as:
// 1. Architectural documentation showing intended organization
// 2. A foundation for future modularization work
// 3. An example of successful type extraction (types.rs)
// 4. Demonstration of logical component boundaries
//
// To complete the modularization in the future:
// 1. Move implementations from simple_vector.rs to respective modules
// 2. Update trait implementations to match exact signatures
// 3. Fix all struct field mappings to match trait definitions
// 4. Test each module independently

pub mod types;      // All type definitions, structs, enums, and configurations
pub mod core;       // Main RAGSimpleVectorEngine struct and basic methods  
pub mod embeddings; // Embedding processing and batch operations
pub mod gleaning;   // Multi-pass gleaning functionality
pub mod overlap;    // Overlap management and semantic boundaries
pub mod sync;       // Cross-process synchronization
pub mod reranking;  // Enterprise reranking infrastructure  
// pub mod tokens;     // Token management and allocation systems

// Re-export types and main engine for external use
pub use types::*;
pub use core::RAGSimpleVectorEngine;