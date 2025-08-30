// Chunking module - split from chunking.rs into organized submodules

pub mod token_based;
pub mod r#trait;
pub mod types;

// Re-export public items
pub use r#trait::ChunkingProcessor;
pub use token_based::TokenBasedChunker;
pub use types::{ChunkSelector, ChunkingStrategy, ContentType};
