// Git service (main git operations)
mod service;
pub use service::{GitError, GitPhase, GitProgress, GitService};

// LFS functionality
pub mod lfs;
pub use lfs::{FilePullMode, LfsError, LfsMetadata, LfsPhase, LfsPointer, LfsProgress, LfsService};
