// Chunking module - split from chunking.rs into organized submodules

pub mod r#trait;
pub mod types;
pub mod token_based;

// Re-export public items
pub use r#trait::ChunkingProcessor;
pub use types::{ChunkingStrategy, ContentType, ChunkSelector};
pub use token_based::TokenBasedChunker;