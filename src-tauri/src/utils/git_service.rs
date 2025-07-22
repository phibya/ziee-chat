use git2::{build::RepoBuilder, Cred, FetchOptions, RemoteCallbacks, Repository};
use serde::Serialize;
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::path::{Path, PathBuf};
use tokio::sync::mpsc;
use uuid::Uuid;
use std::process::{Command, Stdio};
use std::io::{BufRead, BufReader};

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
        self.clone_repository_without_lfs(repository_url, repository_id, branch, auth_token, progress_tx).await
    }

    /// Clone a repository without LFS files
    pub async fn clone_repository_without_lfs(
        &self,
        repository_url: &str,
        repository_id: &Uuid,
        branch: Option<&str>,
        auth_token: Option<&str>,
        progress_tx: mpsc::UnboundedSender<GitProgress>,
    ) -> Result<std::path::PathBuf, GitError> {
        // Generate cache key based on repository_id, URL, and branch
        let cache_key = Self::generate_cache_key(repository_id, repository_url, branch);
        let repo_cache_dir = self.cache_dir.join(cache_key);

        // Check if the cache folder already exists and is a valid git repository
        if repo_cache_dir.exists() && repo_cache_dir.join(".git").exists() {

            // Send completion message (no LFS files pulled yet)
            let _ = progress_tx.send(GitProgress {
                phase: GitPhase::Complete,
                current: 100,
                total: 100,
                message: "Using existing repository cache".to_string(),
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
                current: current as u64,
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

        // Don't fetch LFS files during initial clone

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
                current: current as u64,
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

    /// Pull specific LFS files based on file paths
    pub async fn pull_lfs_files(
        &self,
        repo_path: &Path,
        file_paths: &[String],
        auth_token: Option<&str>,
        progress_tx: mpsc::UnboundedSender<GitProgress>,
    ) -> Result<(), GitError> {
        if file_paths.is_empty() {
            return Ok(());
        }

        // First scan which of the requested files are LFS pointers
        let mut lfs_files = Vec::new();
        let mut total_size = 0u64;
        
        for file_path in file_paths {
            let full_path = repo_path.join(file_path);
            if let Ok(content) = std::fs::read(&full_path) {
                if Self::is_lfs_pointer(&content) {
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
                }
            }
        }

        if lfs_files.is_empty() {
            return Ok(());
        }


        // Fetch the LFS files with size-based progress
        Self::fetch_lfs_files_with_progress(repo_path, lfs_files, total_size, progress_tx, auth_token).await
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

    /// Fetch LFS files using the embedded git-lfs binary with size-based progress
    async fn fetch_lfs_files_with_progress(
        repo_path: &Path,
        lfs_pointers: Vec<LfsPointer>,
        total_size: u64,
        progress_tx: mpsc::UnboundedSender<GitProgress>,
        auth_token: Option<&str>,
    ) -> Result<(), GitError> {
        if lfs_pointers.is_empty() {
            return Ok(());
        }


        // Get the git-lfs binary path
        let git_lfs_path = Self::get_git_lfs_binary_path()?;

        let total_files = lfs_pointers.len();
        let mut downloaded_size = 0u64;
        
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
                    env_vars.push(("GIT_ASKPASS".to_string(), "echo".to_string()));
                    env_vars.push(("GIT_USERNAME".to_string(), parts[0].to_string()));
                    env_vars.push(("GIT_PASSWORD".to_string(), parts[1].to_string()));
                }
            } else {
                // Bearer token
                env_vars.push(("GIT_ASKPASS".to_string(), "echo".to_string()));
                env_vars.push(("GIT_PASSWORD".to_string(), token.to_string()));
            }
        }

        // Pull each file individually to track progress
        for (index, lfs_pointer) in lfs_pointers.iter().enumerate() {
            let file_path_str = lfs_pointer.path.to_string_lossy();
            let file_size = lfs_pointer.size;
            
            // Send initial progress update
            let _ = progress_tx.send(GitProgress {
                phase: GitPhase::CheckingOut,
                current: downloaded_size,
                total: total_size,
                message: format!(
                    "Starting LFS file {}/{}: {} ({} MB / {} MB)",
                    index + 1,
                    total_files,
                    lfs_pointer.path.display(),
                    downloaded_size / (1024 * 1024),
                    total_size / (1024 * 1024)
                ),
            });

            // Use blocking spawn for git-lfs since it doesn't have good async support
            let git_lfs_path_clone = git_lfs_path.clone();
            let file_path_string = file_path_str.to_string();
            let env_vars_clone = env_vars.clone();
            let progress_tx_clone = progress_tx.clone();
            let lfs_filename = lfs_pointer.path.file_name().unwrap_or_default().to_string_lossy().to_string();
            
            let result = tokio::task::spawn_blocking(move || {
                // Create a temporary file for progress
                let progress_file = std::env::temp_dir().join(format!("git-lfs-progress-{}.log", uuid::Uuid::new_v4()));
                let progress_file_path = progress_file.to_string_lossy().to_string();
                
                let mut cmd = Command::new(&git_lfs_path_clone);
                cmd.args(&["pull", "--include", &file_path_string]);
                
                // Add environment variables
                for (key, value) in &env_vars_clone {
                    cmd.env(key, value);
                }
                
                // Set GIT_LFS_PROGRESS to the temp file
                cmd.env("GIT_LFS_PROGRESS", &progress_file_path);

                
                // Spawn the child process
                let mut child = cmd.spawn()
                    .map_err(|e| GitError::Io(e))?;


                // Spawn a thread to read the progress file
                let progress_tx_file = progress_tx_clone.clone();
                let progress_file_clone = progress_file.clone();
                let lfs_filename_clone = lfs_filename.clone();
                let file_size_clone = file_size;
                let downloaded_size_clone = downloaded_size;
                let total_size_clone = total_size;
                
                let progress_reader_handle = std::thread::spawn(move || {
                    use std::time::{Duration, Instant};
                    
                    // Wait a moment for the file to be created
                    std::thread::sleep(Duration::from_millis(100));
                    
                    // Keep reading the file until the process completes
                    let mut last_pos = 0;
                    let mut last_update = Instant::now();
                    let mut last_bytes_so_far = 0u64;
                    
                    loop {
                        if let Ok(mut file) = std::fs::File::open(&progress_file_clone) {
                            use std::io::{Read, Seek, SeekFrom};
                            
                            // Seek to the last read position
                            if let Ok(_) = file.seek(SeekFrom::Start(last_pos)) {
                                let mut buffer = String::new();
                                if let Ok(bytes_read) = file.read_to_string(&mut buffer) {
                                    if bytes_read > 0 {
                                        last_pos += bytes_read as u64;
                                        
                                        // Parse each line
                                        for line in buffer.lines() {
                                            
                                            // Parse the plain text format: "download 1/1 55676250/4915916176 model-00003-of-00004.safetensors"
                                            let parts: Vec<&str> = line.split_whitespace().collect();
                                            if parts.len() >= 3 && parts[0] == "download" {
                                                // Parse the bytes progress (e.g., "55676250/4915916176")
                                                if let Some(progress_parts) = parts.get(2) {
                                                    let progress_split: Vec<&str> = progress_parts.split('/').collect();
                                                    if progress_split.len() == 2 {
                                                        if let (Ok(current), Ok(total)) = (
                                                            progress_split[0].parse::<u64>(),
                                                            progress_split[1].parse::<u64>()
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
                        
                        // Send update every 3 seconds
                        if last_update.elapsed() >= Duration::from_secs(3) && last_bytes_so_far > 0 {
                            let percent = if file_size_clone > 0 {
                                (last_bytes_so_far * 100) / file_size_clone
                            } else {
                                0
                            };
                            
                            // Send progress update
                            let _ = progress_tx_file.send(GitProgress {
                                phase: GitPhase::CheckingOut,
                                current: downloaded_size_clone + last_bytes_so_far.min(file_size_clone),
                                total: total_size_clone,
                                message: format!(
                                    "Downloading {}: {}% ({} MB / {} MB total)",
                                    lfs_filename_clone,
                                    percent,
                                    (downloaded_size_clone + last_bytes_so_far.min(file_size_clone)) / (1024 * 1024),
                                    total_size_clone / (1024 * 1024)
                                ),
                            });
                            
                            last_update = Instant::now();
                        }
                        
                        // Sleep briefly before checking again
                        std::thread::sleep(Duration::from_millis(100));
                        
                        // Check if we should stop
                        if !progress_file_clone.exists() {
                            // Send final update if we have progress
                            if last_bytes_so_far > 0 {
                                let _ = progress_tx_file.send(GitProgress {
                                    phase: GitPhase::CheckingOut,
                                    current: downloaded_size_clone + file_size_clone,
                                    total: total_size_clone,
                                    message: format!(
                                        "Completed downloading {} ({} MB)",
                                        lfs_filename_clone,
                                        file_size_clone / (1024 * 1024)
                                    ),
                                });
                            }
                            break;
                        }
                    }
                });


                // Wait for the process to complete
                let status = child.wait()
                    .map_err(|e| GitError::Io(e))?;
                
                // Clean up the progress file
                let _ = std::fs::remove_file(&progress_file);
                
                // The reader thread will exit when it sees the file is gone
                let _ = progress_reader_handle.join();
                
                Ok::<bool, GitError>(status.success())
            }).await
            .map_err(|e| GitError::Git(git2::Error::from_str(&e.to_string())))?;
            
            match result {
                Ok(true) => {
                    downloaded_size += file_size;
                }
                Ok(false) => {
                    let error_msg = format!("git lfs pull failed for {}", file_path_str);

                    // Send error progress update
                    let _ = progress_tx.send(GitProgress {
                        phase: GitPhase::Error,
                        current: downloaded_size,
                        total: total_size,
                        message: error_msg.clone(),
                    });

                    // Restore original directory before returning error
                    std::env::set_current_dir(&original_dir).ok();
                    return Err(GitError::Git(git2::Error::from_str(&error_msg)));
                }
                Err(e) => {
                    // Restore original directory before returning error
                    std::env::set_current_dir(&original_dir).ok();
                    return Err(e);
                }
            }

        }

        // Restore original directory
        std::env::set_current_dir(original_dir)
            .map_err(|e| GitError::Io(e))?;

        // All files fetched successfully
        let _ = progress_tx.send(GitProgress {
            phase: GitPhase::CheckingOut,
            current: total_size,
            total: total_size,
            message: format!(
                "Successfully fetched all {} LFS files ({} MB total)",
                total_files,
                total_size / (1024 * 1024)
            ),
        });

        Ok(())
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

        
        // Calculate total size
        let total_size: u64 = lfs_pointers.iter().map(|p| p.size).sum();
        
        // Use the new method with progress
        Self::fetch_lfs_files_with_progress(repo_path, lfs_pointers, total_size, progress_tx.clone(), auth_token).await
    }
}
