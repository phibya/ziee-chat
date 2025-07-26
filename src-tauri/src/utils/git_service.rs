use crate::utils::cancellation::CancellationToken;
use git2::{build::RepoBuilder, Cred, FetchOptions, RemoteCallbacks};
use serde::Serialize;
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::path::{Path, PathBuf};
use std::process::Command;
use tokio::sync::mpsc;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize)]
pub struct GitProgress {
    pub phase: GitPhase,
    pub current: u64,
    pub total: u64,
    pub message: String,
}

#[derive(Debug, Clone, Serialize)]
pub enum GitPhase {
    Connecting,
    Receiving,
    Resolving,
    CheckingOut,
    Complete,
    Error,
}

#[derive(Debug, thiserror::Error)]
pub enum GitError {
    #[error("Git error: {0}")]
    Git(#[from] git2::Error),
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Authentication failed: {0}")]
    Auth(String),
    #[error("Repository not found: {0}")]
    NotFound(String),
    #[error("Invalid URL: {0}")]
    InvalidUrl(String),
    #[error("Operation was cancelled")]
    Cancelled,
}

pub struct GitService {
    cache_dir: std::path::PathBuf,
}

/// LFS pointer information
#[derive(Debug, Clone)]
struct LfsPointer {
    oid: String,
    size: u64,
    path: PathBuf,
}

impl GitService {
    pub fn new() -> Self {
        let cache_dir = crate::get_app_data_dir().join("caches/models/git");
        Self { cache_dir }
    }

    /// Get the git-lfs binary path from the build directory
    fn get_git_lfs_binary_path() -> Result<PathBuf, GitError> {
        // Get the executable directory
        let exe_path = std::env::current_exe().map_err(|e| GitError::Io(e))?;
        let exe_dir = exe_path.parent().ok_or_else(|| {
            GitError::Io(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                "Failed to get executable directory",
            ))
        })?;

        // Determine the binary name based on the platform
        let binary_name = if cfg!(windows) {
            "git-lfs.exe"
        } else {
            "git-lfs"
        };

        let binary_path = exe_dir.join(binary_name);

        // Check if the binary exists
        if !binary_path.exists() {
            return Err(GitError::Io(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                format!("git-lfs binary not found at {:?}", binary_path),
            )));
        }

        Ok(binary_path)
    }

    /// Generate a unique cache key based on repository_id, URL, and branch
    fn generate_cache_key(
        repository_id: &Uuid,
        repository_url: &str,
        branch: Option<&str>,
    ) -> String {
        let mut hasher = DefaultHasher::new();
        repository_id.hash(&mut hasher);
        repository_url.hash(&mut hasher);
        branch.hash(&mut hasher);
        let hash = hasher.finish();
        format!("{}-{:x}", repository_id, hash)
    }

    /// Clone a repository with cancellation support (LFS files not included in initial clone)
    pub async fn clone_repository(
        &self,
        repository_url: &str,
        repository_id: &Uuid,
        branch: Option<&str>,
        auth_token: Option<&str>,
        progress_tx: mpsc::UnboundedSender<GitProgress>,
        cancellation_token: Option<CancellationToken>,
    ) -> Result<std::path::PathBuf, GitError> {
        // Check for cancellation before starting
        if let Some(ref token) = cancellation_token {
            if token.is_cancelled().await {
                return Err(GitError::Cancelled);
            }
        }

        // Generate cache key based on repository_id, URL, and branch
        let cache_key = Self::generate_cache_key(repository_id, repository_url, branch);
        let repo_cache_dir = self.cache_dir.join(cache_key);

        // Check if the cache folder already exists and is a valid git repository
        let is_existing_repo = repo_cache_dir.exists() && repo_cache_dir.join(".git").exists();

        // Ensure cache directory exists
        tokio::fs::create_dir_all(&self.cache_dir).await?;

        let progress_tx_clone = progress_tx.clone();
        let repo_cache_dir_clone = repo_cache_dir.clone();
        let repository_url = repository_url.to_string();
        let auth_token = auth_token.map(|s| s.to_string());
        let branch = branch.map(|s| s.to_string());

        // Create a cancellation flag for the blocking task
        let cancelled_flag = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false));
        let cancelled_flag_task = cancelled_flag.clone();

        // Spawn a task to monitor cancellation and update the flag
        let cancellation_monitor = if let Some(token) = cancellation_token.clone() {
            let flag = cancelled_flag.clone();
            Some(tokio::spawn(async move {
                while !flag.load(std::sync::atomic::Ordering::Relaxed) {
                    if token.is_cancelled().await {
                        flag.store(true, std::sync::atomic::Ordering::Relaxed);
                        break;
                    }
                    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
                }
            }))
        } else {
            None
        };

        // Run git operations in a blocking task (merged implementation from clone_repository_blocking)
        let result = tokio::task::spawn_blocking(move || {
            if is_existing_repo {
                // Repository exists, pull latest changes
                let _ = progress_tx_clone.send(GitProgress {
                    phase: GitPhase::Connecting,
                    current: 10,
                    total: 100,
                    message: "Opening existing repository".to_string(),
                });

                let repo = match git2::Repository::open(&repo_cache_dir_clone) {
                    Ok(repo) => repo,
                    Err(e) => {
                        let _ = progress_tx_clone.send(GitProgress {
                            phase: GitPhase::Error,
                            current: 0,
                            total: 100,
                            message: format!("Failed to open repository: {}", e),
                        });
                        return Err(GitError::Git(e));
                    }
                };

                // Check for cancellation before pull
                if let Some(ref token) = cancellation_token {
                    let rt = tokio::runtime::Handle::try_current();
                    if let Ok(handle) = rt {
                        let token_clone = token.clone();
                        let cancelled = handle.block_on(async { token_clone.is_cancelled().await });
                        if cancelled {
                            return Err(GitError::Cancelled);
                        }
                    }
                }

                let _ = progress_tx_clone.send(GitProgress {
                    phase: GitPhase::Connecting,
                    current: 30,
                    total: 100,
                    message: format!("Fetching updates from {}", repository_url),
                });

                // Set up callbacks for fetch operation
                let mut callbacks = RemoteCallbacks::new();

                // Set up authentication
                callbacks.credentials(|_url, username_from_url, _allowed_types| {
                    if let Some(token) = auth_token.as_deref() {
                        Cred::userpass_plaintext(username_from_url.unwrap_or(""), token)
                    } else {
                        Cred::default()
                    }
                });

                // Set up progress callback
                let cancelled_flag_callback = cancelled_flag_task.clone();
                let progress_tx_callback = progress_tx_clone.clone();
                callbacks.transfer_progress(move |progress| {
                    // Check for cancellation using atomic flag
                    if cancelled_flag_callback.load(std::sync::atomic::Ordering::Relaxed) {
                        println!("Git fetch cancelled by user");
                        return false;
                    }

                    // Use git2's byte progress if available, otherwise estimate from objects
                    let current_bytes = if progress.received_bytes() > 0 {
                        progress.received_bytes() as u64
                    } else {
                        // Fallback: estimate bytes from objects (roughly 10KB per object)
                        progress.received_objects() as u64 * 10240
                    };
                    
                    // Git2 doesn't provide total_bytes, so estimate from objects
                    let total_bytes = if progress.total_objects() > 0 {
                        progress.total_objects() as u64 * 10240
                    } else {
                        100 * 1024 * 1024 // Default 100MB estimate
                    };

                    let _ = progress_tx_callback.send(GitProgress {
                        phase: GitPhase::Receiving,
                        current: current_bytes,
                        total: total_bytes,
                        message: format!(
                            "Receiving objects: {} / {}",
                            progress.received_objects(),
                            progress.total_objects()
                        ),
                    });

                    true
                });

                let mut fetch_options = git2::FetchOptions::new();
                fetch_options.remote_callbacks(callbacks);

                // Get the origin remote and fetch
                let mut remote = match repo.find_remote("origin") {
                    Ok(remote) => remote,
                    Err(e) => {
                        let _ = progress_tx_clone.send(GitProgress {
                            phase: GitPhase::Error,
                            current: 0,
                            total: 100,
                            message: format!("Failed to find origin remote: {}", e),
                        });
                        return Err(GitError::Git(e));
                    }
                };

                // Fetch from remote
                match remote.fetch(&[] as &[&str], Some(&mut fetch_options), None) {
                    Ok(_) => {
                        let _ = progress_tx_clone.send(GitProgress {
                            phase: GitPhase::CheckingOut,
                            current: 90,
                            total: 100,
                            message: "Updating working directory".to_string(),
                        });

                        // Get the target branch or default to main/master
                        let branch_name = branch.as_deref().unwrap_or("main");
                        let remote_branch_name = format!("origin/{}", branch_name);

                        // Try to find the remote branch
                        match repo.find_branch(&remote_branch_name, git2::BranchType::Remote) {
                            Ok(remote_branch) => {
                                let target_commit = remote_branch.get().target().unwrap();

                                // Reset HEAD to the remote branch
                                let target_commit_obj = repo.find_commit(target_commit).unwrap();
                                match repo.reset(
                                    &target_commit_obj.as_object(),
                                    git2::ResetType::Hard,
                                    None,
                                ) {
                                    Ok(_) => Ok(()),
                                    Err(e) => {
                                        let _ = progress_tx_clone.send(GitProgress {
                                            phase: GitPhase::Error,
                                            current: 0,
                                            total: 100,
                                            message: format!("Failed to reset to latest: {}", e),
                                        });
                                        Err(GitError::Git(e))
                                    }
                                }
                            }
                            Err(_) => {
                                // Try master if main doesn't exist
                                let master_branch_name = "origin/master";
                                match repo.find_branch(master_branch_name, git2::BranchType::Remote)
                                {
                                    Ok(remote_branch) => {
                                        let target_commit = remote_branch.get().target().unwrap();
                                        let target_commit_obj =
                                            repo.find_commit(target_commit).unwrap();
                                        match repo.reset(
                                            &target_commit_obj.as_object(),
                                            git2::ResetType::Hard,
                                            None,
                                        ) {
                                            Ok(_) => Ok(()),
                                            Err(e) => {
                                                let _ = progress_tx_clone.send(GitProgress {
                                                    phase: GitPhase::Error,
                                                    current: 0,
                                                    total: 100,
                                                    message: format!(
                                                        "Failed to reset to latest: {}",
                                                        e
                                                    ),
                                                });
                                                Err(GitError::Git(e))
                                            }
                                        }
                                    }
                                    Err(e) => {
                                        let _ = progress_tx_clone.send(GitProgress {
                                            phase: GitPhase::Error,
                                            current: 0,
                                            total: 100,
                                            message: format!("Failed to find remote branch: {}", e),
                                        });
                                        Err(GitError::Git(e))
                                    }
                                }
                            }
                        }
                    }
                    Err(e) => {
                        if e.code() == git2::ErrorCode::User {
                            if let Some(ref token) = cancellation_token {
                                let rt = tokio::runtime::Handle::try_current();
                                if let Ok(handle) = rt {
                                    let token_clone = token.clone();
                                    let cancelled =
                                        handle.block_on(async { token_clone.is_cancelled().await });
                                    if cancelled {
                                        return Err(GitError::Cancelled);
                                    }
                                }
                            }
                        }

                        let _ = progress_tx_clone.send(GitProgress {
                            phase: GitPhase::Error,
                            current: 0,
                            total: 100,
                            message: format!("Failed to fetch updates: {}", e),
                        });
                        Err(GitError::Git(e))
                    }
                }
            } else {
                // Repository doesn't exist, perform initial clone
                let mut callbacks = RemoteCallbacks::new();

                // Set up authentication
                callbacks.credentials(|_url, username_from_url, _allowed_types| {
                    if let Some(token) = auth_token.as_deref() {
                        // For GitHub and similar, use token as password with empty username
                        Cred::userpass_plaintext(username_from_url.unwrap_or(""), token)
                    } else {
                        // Try default credentials
                        Cred::default()
                    }
                });

                // Set up progress callback with cancellation check
                let cancelled_flag_callback = cancelled_flag_task.clone();
                let progress_tx_callback = progress_tx_clone.clone();
                callbacks.transfer_progress(move |progress| {
                    // Check for cancellation using atomic flag
                    if cancelled_flag_callback.load(std::sync::atomic::Ordering::Relaxed) {
                        println!("Git clone cancelled by user");
                        return false; // Cancel the operation
                    }

                    let phase = if progress.received_objects() == progress.total_objects() {
                        if progress.indexed_deltas() == progress.total_deltas() {
                            GitPhase::CheckingOut
                        } else {
                            GitPhase::Resolving
                        }
                    } else {
                        GitPhase::Receiving
                    };

                    // Use git2's byte progress if available, otherwise estimate from objects
                    let current_bytes = if progress.received_bytes() > 0 {
                        progress.received_bytes() as u64
                    } else {
                        // Fallback: estimate bytes from objects (roughly 10KB per object)
                        progress.received_objects() as u64 * 10240
                    };
                    
                    // Git2 doesn't provide total_bytes, so estimate from objects
                    let total_bytes = if progress.total_objects() > 0 {
                        progress.total_objects() as u64 * 10240
                    } else {
                        100 * 1024 * 1024 // Default 100MB estimate
                    };

                    let message = match phase {
                        GitPhase::Receiving => format!(
                            "Receiving objects: {} / {}",
                            progress.received_objects(),
                            progress.total_objects()
                        ),
                        GitPhase::Resolving => format!(
                            "Resolving deltas: {} / {}",
                            progress.indexed_deltas(),
                            progress.total_deltas()
                        ),
                        GitPhase::CheckingOut => "Checking out files...".to_string(),
                        _ => "Processing...".to_string(),
                    };

                    let _ = progress_tx_callback.send(GitProgress {
                        phase,
                        current: current_bytes,
                        total: total_bytes,
                        message,
                    });

                    true
                });

                // Set up fetch options
                let mut fetch_options = FetchOptions::new();
                fetch_options.remote_callbacks(callbacks);

                // Send connecting message
                let _ = progress_tx_clone.send(GitProgress {
                    phase: GitPhase::Connecting,
                    current: 0,
                    total: 100,
                    message: format!("Connecting to {}", repository_url),
                });

                // Perform the clone using RepoBuilder
                let mut builder = RepoBuilder::new();
                builder.fetch_options(fetch_options);

                // Set branch if specified
                if let Some(branch_name) = branch.as_deref() {
                    builder.branch(branch_name);
                }

                // Check for cancellation before clone
                if let Some(ref token) = cancellation_token {
                    let rt = tokio::runtime::Handle::try_current();
                    if let Ok(handle) = rt {
                        let token_clone = token.clone();
                        let cancelled = handle.block_on(async { token_clone.is_cancelled().await });
                        if cancelled {
                            return Err(GitError::Cancelled);
                        }
                    }
                }

                match builder.clone(&repository_url, &repo_cache_dir_clone) {
                    Ok(_) => {
                        // Don't fetch LFS files during initial clone
                        Ok(())
                    }
                    Err(e) => {
                        // Check if error was due to cancellation
                        if e.code() == git2::ErrorCode::User {
                            // Progress callback returned false, likely due to cancellation
                            if let Some(ref token) = cancellation_token {
                                let rt = tokio::runtime::Handle::try_current();
                                if let Ok(handle) = rt {
                                    let token_clone = token.clone();
                                    let cancelled =
                                        handle.block_on(async { token_clone.is_cancelled().await });
                                    if cancelled {
                                        return Err(GitError::Cancelled);
                                    }
                                }
                            }
                        }

                        // Send error progress before returning
                        let _ = progress_tx_clone.send(GitProgress {
                            phase: GitPhase::Error,
                            current: 0,
                            total: 100,
                            message: format!("Clone failed: {}", e),
                        });
                        Err(GitError::Git(e))
                    }
                }
            }
        })
        .await
        .map_err(|e| GitError::Git(git2::Error::from_str(&e.to_string())))?;

        // Clean up the cancellation monitor
        if let Some(monitor) = cancellation_monitor {
            monitor.abort();
        }

        match result {
            Ok(_) => {
                let message = if is_existing_repo {
                    "Repository updated successfully"
                } else {
                    "Repository cloned successfully"
                };

                let _ = progress_tx.send(GitProgress {
                    phase: GitPhase::Complete,
                    current: 1, // Completion - we don't know exact bytes, so use 1:1 ratio  
                    total: 1,
                    message: message.to_string(),
                });
                Ok(repo_cache_dir)
            }
            Err(e) => {
                let message = if is_existing_repo {
                    format!("Update failed: {}", e)
                } else {
                    format!("Clone failed: {}", e)
                };

                let _ = progress_tx.send(GitProgress {
                    phase: GitPhase::Error,
                    current: 0,
                    total: 0,
                    message,
                });
                Err(e)
            }
        }
    }

    /// Build repository URL from repository configuration
    pub fn build_repository_url(base_url: &str, repository_path: &str) -> String {
        // Remove trailing slash from base_url
        let base_url = base_url.trim_end_matches('/');

        match base_url {
            url if url.contains("github.com") => {
                format!("{}/{}.git", base_url, repository_path)
            }
            url if url.contains("huggingface.co") => {
                format!("{}/{}", base_url, repository_path)
            }
            _ => {
                format!("{}/{}.git", base_url, repository_path)
            }
        }
    }

    /// Check if a file is an LFS pointer
    fn is_lfs_pointer(content: &[u8]) -> bool {
        // LFS pointers start with "version https://git-lfs.github.com/spec/v1"
        content.starts_with(b"version https://git-lfs.github.com/spec/v1")
    }

    /// Parse LFS pointer content
    fn parse_lfs_pointer(content: &str) -> Option<(String, u64)> {
        let mut oid = None;
        let mut size = None;

        for line in content.lines() {
            if let Some(oid_value) = line.strip_prefix("oid sha256:") {
                oid = Some(oid_value.to_string());
            } else if let Some(size_str) = line.strip_prefix("size ") {
                if let Ok(size_value) = size_str.parse::<u64>() {
                    size = Some(size_value);
                }
            }
        }

        match (oid, size) {
            (Some(o), Some(s)) => Some((o, s)),
            _ => None,
        }
    }

    /// Pull specific LFS files based on file paths with cancellation support
    pub async fn pull_lfs_files_with_cancellation(
        &self,
        repo_path: &Path,
        file_paths: &[String],
        auth_token: Option<&str>,
        progress_tx: mpsc::UnboundedSender<GitProgress>,
        cancellation_token: Option<CancellationToken>,
    ) -> Result<(), GitError> {
        println!(
            "pull_lfs_files_with_cancellation called with {} files",
            file_paths.len()
        );

        // Send initial debug message to verify channel is working
        let _ = progress_tx.send(GitProgress {
            phase: GitPhase::Connecting,
            current: 0,
            total: 0, // Will be updated once we know the total size
            message: "Starting LFS file scan...".to_string(),
        });

        if file_paths.is_empty() {
            println!("No LFS files to pull - sending completion message");
            // Send a completion message even if no files to pull
            let _ = progress_tx.send(GitProgress {
                phase: GitPhase::Complete,
                current: 100,
                total: 100,
                message: "No LFS files to download".to_string(),
            });
            return Ok(());
        }

        // Check for cancellation before starting
        if let Some(ref token) = cancellation_token {
            if token.is_cancelled().await {
                return Err(GitError::Cancelled);
            }
        }

        // First scan which of the requested files are LFS pointers
        let mut lfs_files = Vec::new();
        let mut total_size = 0u64;

        println!("Scanning {} files for LFS pointers...", file_paths.len());
        for file_path in file_paths {
            // Check for cancellation during scan
            if let Some(ref token) = cancellation_token {
                if token.is_cancelled().await {
                    return Err(GitError::Cancelled);
                }
            }

            let full_path = repo_path.join(file_path);
            println!("Checking file: {}", full_path.display());

            if let Ok(content) = std::fs::read(&full_path) {
                if Self::is_lfs_pointer(&content) {
                    println!("Found LFS pointer: {}", file_path);
                    if let Ok(content_str) = String::from_utf8(content) {
                        if let Some((oid, size)) = Self::parse_lfs_pointer(&content_str) {
                            lfs_files.push(LfsPointer {
                                oid,
                                size,
                                path: PathBuf::from(file_path),
                            });
                            total_size += size;
                        }
                    }
                } else {
                    println!("File {} is not an LFS pointer", file_path);
                }
            } else {
                println!("Could not read file: {}", full_path.display());
            }
        }

        println!(
            "Found {} LFS files with total size {} bytes",
            lfs_files.len(),
            total_size
        );

        if lfs_files.is_empty() {
            println!("No LFS files found - sending completion message");
            // Send a completion message even if no LFS files found
            let _ = progress_tx.send(GitProgress {
                phase: GitPhase::Complete,
                current: 0,
                total: 0,
                message: "No LFS files found to download".to_string(),
            });
            return Ok(());
        }

        // Now perform the actual LFS file fetching (merged from fetch_lfs_files_with_progress_and_cancellation)
        if lfs_files.is_empty() {
            return Ok(());
        }

        // Check for cancellation before starting
        if let Some(ref token) = cancellation_token {
            if token.is_cancelled().await {
                return Err(GitError::Cancelled);
            }
        }

        println!(
            "Found {} LFS pointer files to fetch (total size: {} bytes)",
            lfs_files.len(),
            total_size
        );

        // Get the git-lfs binary path
        let git_lfs_path = Self::get_git_lfs_binary_path()?;
        println!("Using git-lfs binary at: {:?}", git_lfs_path);

        let total_files = lfs_files.len();

        // Change to the repository directory
        let original_dir = std::env::current_dir().map_err(|e| GitError::Io(e))?;
        std::env::set_current_dir(repo_path).map_err(|e| GitError::Io(e))?;

        // Set up authentication environment variables if needed
        let mut env_vars = Vec::new();
        if let Some(token) = auth_token {
            if token.contains(':') {
                // Basic auth format: username:password
                let parts: Vec<&str> = token.splitn(2, ':').collect();
                if parts.len() == 2 {
                    env_vars.push(("GIT_ASKPASS", "echo"));
                    env_vars.push(("GIT_USERNAME", parts[0]));
                    env_vars.push(("GIT_PASSWORD", parts[1]));
                }
            } else {
                // Bearer token
                env_vars.push(("GIT_ASKPASS", "echo"));
                env_vars.push(("GIT_PASSWORD", token));
            }
        }

        // Check for cancellation before starting LFS pull
        if let Some(ref token) = cancellation_token {
            if token.is_cancelled().await {
                // Restore original directory before returning
                let _ = std::env::set_current_dir(original_dir);
                return Err(GitError::Cancelled);
            }
        }

        // Instead of using git lfs pull (which doesn't give good progress),
        // we'll download files individually for better progress tracking
        let mut downloaded_size = 0u64;

        for (index, lfs_pointer) in lfs_files.iter().enumerate() {
            // Check for cancellation before each file
            if let Some(ref token) = cancellation_token {
                if token.is_cancelled().await {
                    let _ = std::env::set_current_dir(&original_dir);
                    return Err(GitError::Cancelled);
                }
            }

            let file_name = lfs_pointer
                .path
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("unknown");

            // Send progress update for starting this file
            let _ = progress_tx.send(GitProgress {
                phase: GitPhase::CheckingOut,
                current: downloaded_size,
                total: total_size,
                message: format!("Starting {} ({} of {})", file_name, index + 1, total_files),
            });

            println!(
                "Sent initial progress update for file {} ({} of {})",
                file_name,
                index + 1,
                total_files
            );

            // Try to download this specific LFS file with real-time progress tracking
            let file_path_str = lfs_pointer.path.to_string_lossy().to_string();
            let lfs_filename = lfs_pointer
                .path
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("unknown")
                .to_string();
            let file_size = lfs_pointer.size;

            println!(
                "Downloading LFS file: {} ({} bytes)",
                lfs_pointer.path.display(),
                lfs_pointer.size
            );

            // Create a temporary file for progress tracking
            let progress_file =
                std::env::temp_dir().join(format!("git-lfs-progress-{}.log", Uuid::new_v4()));
            let progress_file_path = progress_file.to_string_lossy().to_string();

            let mut cmd = Command::new(&git_lfs_path);
            cmd.args(&["pull", "--include", &file_path_str]);

            // Add environment variables
            for (key, value) in &env_vars {
                cmd.env(key, value);
            }

            // Set GIT_LFS_PROGRESS to the temp file for real-time progress
            cmd.env("GIT_LFS_PROGRESS", &progress_file_path);

            // Spawn the child process
            let mut child = cmd.spawn().map_err(|e| GitError::Io(e))?;

            // Create a shared cancellation flag for the background thread
            let cancelled_flag = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false));
            let cancelled_flag_thread = cancelled_flag.clone();

            // Spawn a background task to read the progress file in real-time
            let progress_tx_clone = progress_tx.clone();
            let progress_file_clone = progress_file.clone();
            let lfs_filename_clone = lfs_filename.clone();

            let progress_reader_handle = std::thread::spawn(move || {
                use std::io::{Read, Seek, SeekFrom};
                use std::time::{Duration, Instant};

                // Wait a moment for the file to be created
                std::thread::sleep(Duration::from_millis(100));
                println!(
                    "Progress reader thread started for file: {}",
                    progress_file_clone.display()
                );

                // Send an initial progress update to show we're starting
                let _ = progress_tx_clone.send(GitProgress {
                    phase: GitPhase::CheckingOut,
                    current: 0,
                    total: 100,
                    message: format!("Starting download of {}", lfs_filename_clone),
                });
                println!(
                    "Sent initial background progress update for {}",
                    lfs_filename_clone
                );

                // Keep reading the file until the process completes
                let mut last_pos = 0;
                let mut last_update = Instant::now();
                let mut last_bytes_so_far = 0u64;

                loop {
                    if let Ok(mut file) = std::fs::File::open(&progress_file_clone) {
                        // Seek to the last read position
                        if let Ok(_) = file.seek(SeekFrom::Start(last_pos)) {
                            let mut buffer = String::new();
                            if let Ok(bytes_read) = file.read_to_string(&mut buffer) {
                                if bytes_read > 0 {
                                    last_pos += bytes_read as u64;

                                    // Parse each line for progress info
                                    for line in buffer.lines() {
                                        // Parse git-lfs progress format: "download 1/1 55676250/4915916176 model-00003-of-00004.safetensors"
                                        let parts: Vec<&str> = line.split_whitespace().collect();
                                        if parts.len() >= 3 && parts[0] == "download" {
                                            // Parse the bytes progress (e.g., "55676250/4915916176")
                                            if let Some(progress_parts) = parts.get(2) {
                                                let progress_split: Vec<&str> =
                                                    progress_parts.split('/').collect();
                                                if progress_split.len() == 2 {
                                                    if let (Ok(current), Ok(_total)) = (
                                                        progress_split[0].parse::<u64>(),
                                                        progress_split[1].parse::<u64>(),
                                                    ) {
                                                        last_bytes_so_far = current;
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }

                    // Send update every 1 second with real progress
                    if last_update.elapsed() >= Duration::from_secs(1) && last_bytes_so_far > 0 {
                        let percent = if file_size > 0 {
                            ((last_bytes_so_far * 100) / file_size).min(100)
                        } else {
                            0
                        };

                        // Send real-time progress update with actual byte values
                        let _ = progress_tx_clone.send(GitProgress {
                            phase: GitPhase::CheckingOut,
                            current: last_bytes_so_far,
                            total: file_size,
                            message: format!(
                                "Downloading {}: {}% ({:.1} MB / {:.1} MB)",
                                lfs_filename_clone,
                                percent,
                                last_bytes_so_far as f64 / (1024.0 * 1024.0),
                                file_size as f64 / (1024.0 * 1024.0)
                            ),
                        });

                        last_update = Instant::now();
                    }

                    // Sleep briefly before checking again
                    std::thread::sleep(Duration::from_millis(200));

                    // Check for cancellation using the atomic flag
                    if cancelled_flag_thread.load(std::sync::atomic::Ordering::Relaxed) {
                        println!("Background progress reader thread cancelled");
                        break;
                    }

                    // Check if we should stop (file removed or process finished)
                    if !progress_file_clone.exists() {
                        println!("Progress file no longer exists, exiting reader thread");
                        break;
                    }
                }

                println!("Progress reader thread exiting for {}", lfs_filename_clone);
            });

            // Wait for the process to complete with cancellation checking
            println!("Waiting for git-lfs process to complete...");
            let status = loop {
                // Check for cancellation
                if let Some(ref token) = cancellation_token {
                    if token.is_cancelled().await {
                        println!("Cancellation requested, killing git-lfs process");

                        // Signal the background thread to stop
                        cancelled_flag.store(true, std::sync::atomic::Ordering::Relaxed);

                        // Kill the child process
                        let _ = child.kill();
                        let _ = child.wait();

                        // Clean up the progress file
                        let _ = std::fs::remove_file(&progress_file);

                        // Wait for the progress reader to finish
                        let _ = progress_reader_handle.join();

                        // Restore original directory before returning
                        let _ = std::env::set_current_dir(&original_dir);

                        return Err(GitError::Cancelled);
                    }
                }

                // Try to get the exit status without blocking
                match child.try_wait().map_err(|e| GitError::Io(e))? {
                    Some(status) => break status,
                    None => {
                        // Process is still running, wait a bit and check again
                        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
                    }
                }
            };

            println!("Git-lfs process completed with status: {:?}", status);

            // Clean up the progress file
            println!("Cleaning up progress file: {}", progress_file.display());
            let _ = std::fs::remove_file(&progress_file);

            // Wait for the progress reader to finish
            println!("Waiting for progress reader thread to finish...");
            let _ = progress_reader_handle.join();
            println!("Progress reader thread finished");

            if !status.success() {
                let error_msg =
                    format!("Failed to download LFS file {}", lfs_pointer.path.display());
                println!("{}", error_msg);

                // Restore original directory before returning error
                let _ = std::env::set_current_dir(&original_dir);

                // Send error progress update
                let _ = progress_tx.send(GitProgress {
                    phase: GitPhase::Error,
                    current: 0,
                    total: 100,
                    message: error_msg.clone(),
                });

                return Err(GitError::Git(git2::Error::from_str(&error_msg)));
            }

            // Update downloaded size
            downloaded_size += lfs_pointer.size;

            println!(
                "Successfully downloaded: {} ({} bytes)",
                lfs_pointer.path.display(),
                lfs_pointer.size
            );

            // Send updated progress after file completion
            let _ = progress_tx.send(GitProgress {
                phase: GitPhase::CheckingOut,
                current: downloaded_size,
                total: total_size,
                message: format!("Completed {} ({} of {})", file_name, index + 1, total_files),
            });

            println!(
                "Sent completion progress update for file {} ({} of {})",
                file_name,
                index + 1,
                total_files
            );
        }

        // Restore original directory
        std::env::set_current_dir(original_dir).map_err(|e| GitError::Io(e))?;

        // Check for cancellation one final time
        if let Some(ref token) = cancellation_token {
            if token.is_cancelled().await {
                return Err(GitError::Cancelled);
            }
        }

        // All files fetched successfully
        match progress_tx.send(GitProgress {
            phase: GitPhase::Complete,
            current: total_size,
            total: total_size,
            message: format!("Successfully downloaded all {} LFS files", total_files),
        }) {
            Ok(_) => println!("Successfully sent LFS completion progress message"),
            Err(e) => println!("Failed to send LFS completion progress message: {}", e),
        }

        println!(
            "LFS download completed: {} files, {} total bytes",
            total_files, total_size
        );
        Ok(())
    }
}
