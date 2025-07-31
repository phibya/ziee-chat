// Git service (main git operations)
mod service;
pub use service::{GitService, GitProgress, GitPhase, GitError};

// LFS functionality
pub mod lfs;
pub use lfs::{LfsError, LfsMetadata, LfsPointer, LfsService, FilePullMode, LfsProgress, LfsPhase};