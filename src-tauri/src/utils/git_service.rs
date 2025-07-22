use git2::{Repository, Cred, RemoteCallbacks, FetchOptions, build::RepoBuilder};
use std::path::Path;
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use tokio::sync::mpsc;
use serde::Serialize;
use uuid::Uuid;

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
        // Generate cache key based on repository_id, URL, and branch
        let cache_key = Self::generate_cache_key(repository_id, repository_url, branch);
        let repo_cache_dir = self.cache_dir.join(cache_key);
        
        // Remove existing repository if it exists
        if repo_cache_dir.exists() {
            tokio::fs::remove_dir_all(&repo_cache_dir).await?;
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
        }).await.map_err(|e| GitError::Git(git2::Error::from_str(&e.to_string())))?;

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
        }).await.map_err(|e| GitError::Git(git2::Error::from_str(&e.to_string())))?;

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
}