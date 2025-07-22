use git2::{build::RepoBuilder, Cred, FetchOptions, RemoteCallbacks, Repository};
use serde::Serialize;
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::path::{Path, PathBuf};
use tokio::sync::mpsc;
use uuid::Uuid;
use std::process::Command;

#[derive(Debug, Clone, Serialize)]
pub struct GitProgress {
    pub phase: GitPhase,
    pub current: usize,
    pub total: usize,
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
        let cache_dir = crate::APP_DATA_DIR.join("caches");
        Self { cache_dir }
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

    /// Clone a repository with authentication and progress tracking
    pub async fn clone_repository(
        &self,
        repository_url: &str,
        repository_id: &Uuid,
        branch: Option<&str>,
        auth_token: Option<&str>,
        progress_tx: mpsc::UnboundedSender<GitProgress>,
    ) -> Result<std::path::PathBuf, GitError> {
        // print debug information
        println!(
            "Cloning repository: {}, ID: {}, Branch: {:?}, Auth Token: {:?}",
            repository_url, repository_id, branch, auth_token
        );
        // Generate cache key based on repository_id, URL, and branch
        let cache_key = Self::generate_cache_key(repository_id, repository_url, branch);
        let repo_cache_dir = self.cache_dir.join(cache_key);

        // Check if the cache folder already exists and is a valid git repository
        if repo_cache_dir.exists() && repo_cache_dir.join(".git").exists() {
            println!("Repository cache already exists at: {:?}", repo_cache_dir);

            // Check and fetch LFS files for cached repository
            let _ = progress_tx.send(GitProgress {
                phase: GitPhase::CheckingOut,
                current: 90,
                total: 100,
                message: "Checking for LFS files in cached repository...".to_string(),
            });

            // Scan for LFS pointers
            let repo_cache_dir_clone = repo_cache_dir.clone();
            let progress_tx_clone = progress_tx.clone();
            let auth_token_clone = auth_token.map(|s| s.to_string());

            let lfs_result = tokio::task::spawn_blocking(move || {
                // Use tokio runtime to run async function in blocking context
                let rt = tokio::runtime::Handle::current();
                rt.block_on(async {
                    if let Ok(lfs_pointers) =
                        Self::scan_for_lfs_pointers(&repo_cache_dir_clone).await
                    {
                        if !lfs_pointers.is_empty() {
                            println!(
                                "Found {} LFS pointers in cached repository",
                                lfs_pointers.len()
                            );
                            if let Err(e) = Self::fetch_lfs_files(
                                &repo_cache_dir_clone,
                                lfs_pointers,
                                &progress_tx_clone,
                                auth_token_clone.as_deref(),
                            )
                            .await
                            {
                                // Return the error if LFS fetch fails
                                return Err(e);
                            }
                        }
                    }
                    Ok(())
                })
            })
            .await
            .map_err(|e| GitError::Git(git2::Error::from_str(&e.to_string())))?;

            // Check if LFS fetch failed
            lfs_result?;

            // Send completion message
            let _ = progress_tx.send(GitProgress {
                phase: GitPhase::Complete,
                current: 100,
                total: 100,
                message: "Using existing repository cache with LFS files".to_string(),
            });

            return Ok(repo_cache_dir);
        }

        // Ensure cache directory exists
        tokio::fs::create_dir_all(&self.cache_dir).await?;

        let progress_tx_clone = progress_tx.clone();
        let repo_cache_dir_clone = repo_cache_dir.clone();
        let repository_url = repository_url.to_string();
        let auth_token = auth_token.map(|s| s.to_string());
        let branch = branch.map(|s| s.to_string());

        // Run git operations in a blocking task
        let result = tokio::task::spawn_blocking(move || {
            Self::clone_repository_blocking(
                &repository_url,
                &repo_cache_dir_clone,
                branch.as_deref(),
                auth_token.as_deref(),
                progress_tx_clone,
            )
        })
        .await
        .map_err(|e| GitError::Git(git2::Error::from_str(&e.to_string())))?;

        match result {
            Ok(_) => {
                let _ = progress_tx.send(GitProgress {
                    phase: GitPhase::Complete,
                    current: 100,
                    total: 100,
                    message: "Repository cloned successfully".to_string(),
                });
                Ok(repo_cache_dir)
            }
            Err(e) => {
                let _ = progress_tx.send(GitProgress {
                    phase: GitPhase::Error,
                    current: 0,
                    total: 100,
                    message: format!("Clone failed: {}", e),
                });
                Err(e)
            }
        }
    }

    fn clone_repository_blocking(
        repository_url: &str,
        target_dir: &Path,
        branch: Option<&str>,
        auth_token: Option<&str>,
        progress_tx: mpsc::UnboundedSender<GitProgress>,
    ) -> Result<(), GitError> {
        let mut callbacks = RemoteCallbacks::new();

        // Set up authentication
        callbacks.credentials(|_url, username_from_url, _allowed_types| {
            if let Some(token) = auth_token {
                // For GitHub and similar, use token as password with empty username
                Cred::userpass_plaintext(username_from_url.unwrap_or(""), token)
            } else {
                // Try default credentials
                Cred::default()
            }
        });

        // Set up progress callback
        callbacks.transfer_progress(|progress| {
            let phase = if progress.received_objects() == progress.total_objects() {
                if progress.indexed_deltas() == progress.total_deltas() {
                    GitPhase::CheckingOut
                } else {
                    GitPhase::Resolving
                }
            } else {
                GitPhase::Receiving
            };

            let current = if progress.total_objects() > 0 {
                (progress.received_objects() * 100) / progress.total_objects()
            } else {
                0
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

            let _ = progress_tx.send(GitProgress {
                phase,
                current: current as usize,
                total: 100,
                message,
            });

            true
        });

        // Set up fetch options
        let mut fetch_options = FetchOptions::new();
        fetch_options.remote_callbacks(callbacks);

        // Send connecting message
        let _ = progress_tx.send(GitProgress {
            phase: GitPhase::Connecting,
            current: 0,
            total: 100,
            message: format!("Connecting to {}", repository_url),
        });

        // Perform the clone using RepoBuilder
        let mut builder = RepoBuilder::new();
        builder.fetch_options(fetch_options);

        // Set branch if specified
        if let Some(branch_name) = branch {
            builder.branch(branch_name);
        }

        builder.clone(repository_url, target_dir)?;

        // Check if git LFS is needed and fetch LFS files
        // Send LFS progress message
        let _ = progress_tx.send(GitProgress {
            phase: GitPhase::CheckingOut,
            current: 95,
            total: 100,
            message: "Checking for LFS files...".to_string(),
        });

        // Scan for LFS pointers using async runtime
        let target_dir_pathbuf = target_dir.to_path_buf();
        let progress_tx_clone = progress_tx.clone();
        let auth_token_clone = auth_token.map(|s| s.to_string());

        // Create a temporary runtime for async operations
        let rt = tokio::runtime::Runtime::new()
            .map_err(|e| GitError::Git(git2::Error::from_str(&e.to_string())))?;

        let lfs_result = rt.block_on(async {
            if let Ok(lfs_pointers) = Self::scan_for_lfs_pointers(&target_dir_pathbuf).await {
                if !lfs_pointers.is_empty() {
                    println!("Found {} LFS pointers in repository", lfs_pointers.len());
                    if let Err(e) = Self::fetch_lfs_files(
                        &target_dir_pathbuf,
                        lfs_pointers,
                        &progress_tx_clone,
                        auth_token_clone.as_deref(),
                    )
                    .await
                    {
                        return Err(e);
                    }
                }
            }
            Ok(())
        });

        // Check if LFS fetch failed and propagate the error
        lfs_result?;

        Ok(())
    }

    /// Get repository cache directory
    pub fn get_cache_path(&self, repository_id: &Uuid) -> std::path::PathBuf {
        self.cache_dir.join(repository_id.to_string())
    }

    /// Check if repository is already cached
    pub async fn is_cached(&self, repository_id: &Uuid) -> bool {
        let repo_path = self.get_cache_path(repository_id);
        repo_path.exists() && repo_path.join(".git").exists()
    }

    /// Update existing repository (git pull)
    pub async fn update_repository(
        &self,
        repository_id: &Uuid,
        auth_token: Option<&str>,
        progress_tx: mpsc::UnboundedSender<GitProgress>,
    ) -> Result<(), GitError> {
        let repo_path = self.get_cache_path(repository_id);

        if !repo_path.exists() {
            return Err(GitError::NotFound("Repository not cached".to_string()));
        }

        let auth_token = auth_token.map(|s| s.to_string());
        let progress_tx_clone = progress_tx.clone();

        // Run git operations in a blocking task
        let result = tokio::task::spawn_blocking(move || {
            Self::update_repository_blocking(&repo_path, auth_token.as_deref(), progress_tx_clone)
        })
        .await
        .map_err(|e| GitError::Git(git2::Error::from_str(&e.to_string())))?;

        match result {
            Ok(_) => {
                let _ = progress_tx.send(GitProgress {
                    phase: GitPhase::Complete,
                    current: 100,
                    total: 100,
                    message: "Repository updated successfully".to_string(),
                });
                Ok(())
            }
            Err(e) => {
                let _ = progress_tx.send(GitProgress {
                    phase: GitPhase::Error,
                    current: 0,
                    total: 100,
                    message: format!("Update failed: {}", e),
                });
                Err(e)
            }
        }
    }

    fn update_repository_blocking(
        repo_path: &Path,
        auth_token: Option<&str>,
        progress_tx: mpsc::UnboundedSender<GitProgress>,
    ) -> Result<(), GitError> {
        let repo = Repository::open(repo_path)?;

        // Send connecting message
        let _ = progress_tx.send(GitProgress {
            phase: GitPhase::Connecting,
            current: 0,
            total: 100,
            message: "Updating repository...".to_string(),
        });

        // Set up callbacks for authentication
        let mut callbacks = RemoteCallbacks::new();
        callbacks.credentials(|_url, username_from_url, _allowed_types| {
            if let Some(token) = auth_token {
                Cred::userpass_plaintext(username_from_url.unwrap_or(""), token)
            } else {
                Cred::default()
            }
        });

        // Set up progress callback
        callbacks.transfer_progress(|progress| {
            let current = if progress.total_objects() > 0 {
                (progress.received_objects() * 100) / progress.total_objects()
            } else {
                50
            };

            let _ = progress_tx.send(GitProgress {
                phase: GitPhase::Receiving,
                current: current as usize,
                total: 100,
                message: format!("Fetching updates: {} objects", progress.received_objects()),
            });

            true
        });

        // Set up fetch options
        let mut fetch_options = FetchOptions::new();
        fetch_options.remote_callbacks(callbacks);

        // Fetch from origin
        let mut remote = repo.find_remote("origin")?;
        remote.fetch(&[] as &[&str], Some(&mut fetch_options), None)?;

        // Get the fetch head and merge
        let fetch_head = repo.refname_to_id("FETCH_HEAD")?;
        let analysis = repo.merge_analysis(&[&repo.find_annotated_commit(fetch_head)?])?;

        if analysis.0.is_up_to_date() {
            let _ = progress_tx.send(GitProgress {
                phase: GitPhase::Complete,
                current: 100,
                total: 100,
                message: "Repository is up to date".to_string(),
            });
        } else if analysis.0.is_fast_forward() {
            // Fast-forward merge
            let refname = "refs/heads/main"; // Assume main branch
            match repo.find_reference(refname) {
                Ok(mut reference) => {
                    reference.set_target(fetch_head, "Fast-forward")?;
                    repo.set_head(refname)?;
                    repo.checkout_head(Some(git2::build::CheckoutBuilder::default().force()))?;
                }
                Err(_) => {
                    // Branch doesn't exist, create it
                    repo.reference(refname, fetch_head, false, "Fast-forward")?;
                    repo.set_head(refname)?;
                    repo.checkout_head(Some(git2::build::CheckoutBuilder::default().force()))?;
                }
            }
        }

        Ok(())
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

    /// Scan repository for LFS pointers
    async fn scan_for_lfs_pointers(repo_path: &Path) -> Result<Vec<LfsPointer>, GitError> {
        let mut lfs_pointers = Vec::new();

        // Open the repository
        let repo = Repository::open(repo_path)?;
        let head = repo.head()?;
        let tree = head.peel_to_tree()?;

        // Walk the tree and check each file
        tree.walk(git2::TreeWalkMode::PreOrder, |root, entry| {
            if entry.kind() == Some(git2::ObjectType::Blob) {
                let path = PathBuf::from(root).join(entry.name().unwrap_or(""));
                let full_path = repo_path.join(&path);

                // Check if the file might be an LFS pointer (small size is a good indicator)
                if let Ok(metadata) = std::fs::metadata(&full_path) {
                    if metadata.len() < 1024 {
                        // LFS pointers are typically under 1KB
                        if let Ok(content) = std::fs::read(&full_path) {
                            if Self::is_lfs_pointer(&content) {
                                if let Ok(content_str) = String::from_utf8(content) {
                                    if let Some((oid, size)) = Self::parse_lfs_pointer(&content_str)
                                    {
                                        lfs_pointers.push(LfsPointer {
                                            oid,
                                            size,
                                            path: path.clone(),
                                        });
                                    }
                                }
                            }
                        }
                    }
                }
            }

            git2::TreeWalkResult::Ok
        })?;

        Ok(lfs_pointers)
    }

    /// Get the git-lfs binary path from the build directory
    fn get_git_lfs_binary_path() -> Result<PathBuf, GitError> {
        // Get the executable directory
        let exe_path = std::env::current_exe()
            .map_err(|e| GitError::Io(e))?;
        let exe_dir = exe_path.parent()
            .ok_or_else(|| GitError::Io(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                "Failed to get executable directory"
            )))?;
        
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
                format!("git-lfs binary not found at {:?}", binary_path)
            )));
        }
        
        Ok(binary_path)
    }

    /// Fetch LFS files using the embedded git-lfs binary
    async fn fetch_lfs_files(
        repo_path: &Path,
        lfs_pointers: Vec<LfsPointer>,
        progress_tx: &mpsc::UnboundedSender<GitProgress>,
        auth_token: Option<&str>,
    ) -> Result<(), GitError> {
        if lfs_pointers.is_empty() {
            return Ok(());
        }

        println!("Found {} LFS pointer files to fetch", lfs_pointers.len());

        // Get the git-lfs binary path
        let git_lfs_path = Self::get_git_lfs_binary_path()?;
        println!("Using git-lfs binary at: {:?}", git_lfs_path);

        let total_files = lfs_pointers.len();
        
        // Change to the repository directory
        let original_dir = std::env::current_dir()
            .map_err(|e| GitError::Io(e))?;
        std::env::set_current_dir(repo_path)
            .map_err(|e| GitError::Io(e))?;

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

        // Run git lfs pull to fetch all LFS files
        let _ = progress_tx.send(GitProgress {
            phase: GitPhase::CheckingOut,
            current: 92,
            total: 100,
            message: format!("Fetching {} LFS files...", total_files),
        });

        let mut cmd = Command::new(&git_lfs_path);
        cmd.args(&["pull"]);
        
        // Add environment variables
        for (key, value) in env_vars {
            cmd.env(key, value);
        }

        println!("Running git lfs pull...");
        let output = cmd.output()
            .map_err(|e| GitError::Io(e))?;

        // Restore original directory
        std::env::set_current_dir(original_dir)
            .map_err(|e| GitError::Io(e))?;

        if !output.status.success() {
            let error_msg = format!(
                "git lfs pull failed: {}\nstderr: {}",
                output.status,
                String::from_utf8_lossy(&output.stderr)
            );
            println!("{}", error_msg);

            // Send error progress update
            let _ = progress_tx.send(GitProgress {
                phase: GitPhase::Error,
                current: 0,
                total: 100,
                message: error_msg.clone(),
            });

            return Err(GitError::Git(git2::Error::from_str(&error_msg)));
        }

        println!("git lfs pull output: {}", String::from_utf8_lossy(&output.stdout));

        // All files fetched successfully
        let _ = progress_tx.send(GitProgress {
            phase: GitPhase::CheckingOut,
            current: 98,
            total: 100,
            message: format!("Successfully fetched all {} LFS files", total_files),
        });

        Ok(())
    }
}
