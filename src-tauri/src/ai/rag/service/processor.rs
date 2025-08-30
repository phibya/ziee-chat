// File processing logic for RAG service with per-instance worker threads

use crate::ai::rag::{
    engines::{RAGEngine, RAGEngineFactory},
    models::RagInstanceFile,
    RAGResult,
};
use std::collections::HashSet;
use std::sync::Arc;
use tokio::sync::RwLock;
use tokio::time::{sleep, Duration};
use uuid::Uuid;

use super::queries::{
    get_engine_type_for_instance, get_pending_files_for_instance,
    get_rag_instances_with_pending_files, update_file_status, update_file_status_with_error,
    update_rag_instance_active_status,
};

/// Thread-safe registry to track active RAG instance worker threads
pub struct InstanceThreadRegistry {
    active_instances: Arc<RwLock<HashSet<Uuid>>>,
}

impl InstanceThreadRegistry {
    pub fn new() -> Self {
        Self {
            active_instances: Arc::new(RwLock::new(HashSet::new())),
        }
    }

    /// Check if an instance thread is already running
    pub async fn is_instance_active(&self, instance_id: Uuid) -> bool {
        let active = self.active_instances.read().await;
        active.contains(&instance_id)
    }

    /// Register a new instance thread
    pub async fn register_instance(&self, instance_id: Uuid) {
        let mut active = self.active_instances.write().await;
        active.insert(instance_id);
        tracing::info!("Registered worker thread for RAG instance: {}", instance_id);
    }

    /// Unregister an instance thread when it completes
    pub async fn unregister_instance(&self, instance_id: Uuid) {
        let mut active = self.active_instances.write().await;
        active.remove(&instance_id);
        tracing::info!(
            "Unregistered worker thread for RAG instance: {}",
            instance_id
        );
    }

    /// Get count of active instance threads
    pub async fn active_count(&self) -> usize {
        let active = self.active_instances.read().await;
        active.len()
    }
}

/// Main coordinator that monitors for new RAG instances and spawns worker threads
pub async fn process_pending_files() -> RAGResult<()> {
    static THREAD_REGISTRY: tokio::sync::OnceCell<InstanceThreadRegistry> =
        tokio::sync::OnceCell::const_new();
    let registry = THREAD_REGISTRY
        .get_or_init(|| async { InstanceThreadRegistry::new() })
        .await;

    // Get unique RAG instance IDs that have pending files
    let instance_ids_with_pending = get_rag_instances_with_pending_files().await?;

    if instance_ids_with_pending.is_empty() {
        return Ok(());
    }

    tracing::debug!(
        "Found {} unique RAG instances with pending files. Active threads: {}",
        instance_ids_with_pending.len(),
        registry.active_count().await
    );

    // Spawn worker threads for instances that don't have active threads yet
    for rag_instance_id in instance_ids_with_pending {
        if !registry.is_instance_active(rag_instance_id).await {
            tracing::info!(
                "Spawning new worker thread for RAG instance {}",
                rag_instance_id
            );

            // Register this instance as active
            registry.register_instance(rag_instance_id).await;

            // Clone necessary data for the worker thread
            let registry_clone = registry.active_instances.clone();

            // Spawn dedicated worker thread for this RAG instance
            tokio::spawn(async move {
                if let Err(e) = rag_instance_worker(rag_instance_id, registry_clone).await {
                    tracing::error!(
                        "Worker thread for RAG instance {} failed: {}",
                        rag_instance_id,
                        e
                    );
                }
            });
        } else {
            tracing::debug!(
                "RAG instance {} already has an active worker thread, skipping",
                rag_instance_id
            );
        }
    }

    Ok(())
}

/// Dedicated worker thread for processing files of a specific RAG instance
async fn rag_instance_worker(
    rag_instance_id: Uuid,
    registry: Arc<RwLock<HashSet<Uuid>>>,
) -> RAGResult<()> {
    tracing::info!(
        "Starting worker thread for RAG instance: {}",
        rag_instance_id
    );

    // Get engine type for this RAG instance
    let engine_type = match get_engine_type_for_instance(rag_instance_id).await {
        Ok(engine_type) => engine_type,
        Err(e) => {
            tracing::error!(
                "Failed to get engine type for RAG instance {}: {}",
                rag_instance_id,
                e
            );
            // Deactivate the RAG instance due to engine type failure
            if let Err(update_err) = update_rag_instance_active_status(rag_instance_id, false).await
            {
                tracing::error!("Failed to deactivate RAG instance: {}", update_err);
            }
            unregister_and_exit(rag_instance_id, &registry).await;
            return Err(e);
        }
    };

    // Create engine for this RAG instance
    let engine = match RAGEngineFactory::create_engine(engine_type, rag_instance_id) {
        Ok(engine) => engine,
        Err(e) => {
            tracing::error!(
                "Failed to create engine for RAG instance {}: {}",
                rag_instance_id,
                e
            );
            if let Err(update_err) = update_rag_instance_active_status(rag_instance_id, false).await
            {
                tracing::error!("Failed to deactivate RAG instance: {}", update_err);
            }
            unregister_and_exit(rag_instance_id, &registry).await;
            return Err(e);
        }
    };

    // Initialize the engine
    if let Err(e) = engine
        .initialize(serde_json::json!({}))
        .await
    {
        tracing::error!(
            "Failed to initialize engine for RAG instance {}: {}",
            rag_instance_id,
            e
        );
        if let Err(update_err) = update_rag_instance_active_status(rag_instance_id, false).await {
            tracing::error!("Failed to deactivate RAG instance: {}", update_err);
        }
        unregister_and_exit(rag_instance_id, &registry).await;
        return Err(e);
    }

    // Main processing loop for this RAG instance
    let mut consecutive_empty_checks = 0;
    const MAX_EMPTY_CHECKS: u32 = 3; // Terminate after 3 consecutive empty checks
    const CHECK_INTERVAL_SECS: u64 = 2; // Check every 2 seconds

    loop {
        // Get pending files for this specific RAG instance
        let pending_files = match get_pending_files_for_instance(rag_instance_id).await {
            Ok(files) => files,
            Err(e) => {
                tracing::error!(
                    "Failed to get pending files for RAG instance {}: {}",
                    rag_instance_id,
                    e
                );
                sleep(Duration::from_secs(CHECK_INTERVAL_SECS)).await;
                continue;
            }
        };

        if pending_files.is_empty() {
            consecutive_empty_checks += 1;
            tracing::debug!(
                "No pending files for RAG instance {} (check {}/{})",
                rag_instance_id,
                consecutive_empty_checks,
                MAX_EMPTY_CHECKS
            );

            if consecutive_empty_checks >= MAX_EMPTY_CHECKS {
                tracing::info!(
                    "No more files to process for RAG instance {}, terminating worker thread",
                    rag_instance_id
                );
                break;
            }

            sleep(Duration::from_secs(CHECK_INTERVAL_SECS)).await;
            continue;
        } else {
            consecutive_empty_checks = 0; // Reset counter when files are found
        }

        tracing::info!(
            "Processing {} files for RAG instance {}",
            pending_files.len(),
            rag_instance_id
        );

        // Process each file for this instance
        for file in pending_files {
            match process_single_file(&engine, &file).await {
                Ok(_) => {
                    tracing::info!(
                        "Successfully processed file: {} for RAG instance {}",
                        file.file_id,
                        rag_instance_id
                    );
                    if let Err(e) = update_file_status(&file.id, "completed").await {
                        tracing::error!("Failed to update file status for {}: {}", file.file_id, e);
                    }
                }
                Err(e) => {
                    tracing::error!(
                        "Failed to process file {} for RAG instance {}: {}",
                        file.file_id,
                        rag_instance_id,
                        e
                    );
                    if let Err(update_err) =
                        update_file_status_with_error(&file.id, "failed", &e.to_string()).await
                    {
                        tracing::error!(
                            "Failed to update error status for {}: {}",
                            file.file_id,
                            update_err
                        );
                    }
                }
            }
        }

        // Small delay before next check
        sleep(Duration::from_secs(CHECK_INTERVAL_SECS)).await;
    }

    unregister_and_exit(rag_instance_id, &registry).await;
    Ok(())
}

/// Unregister instance from registry and log thread termination
async fn unregister_and_exit(rag_instance_id: Uuid, registry: &Arc<RwLock<HashSet<Uuid>>>) {
    let mut active = registry.write().await;
    active.remove(&rag_instance_id);
    tracing::info!(
        "Worker thread for RAG instance {} completed and unregistered. Remaining active threads: {}",
        rag_instance_id,
        active.len()
    );
}

/// Process a single file using the RAG engine
async fn process_single_file(
    engine: &Box<dyn RAGEngine>,
    rag_file: &RagInstanceFile,
) -> RAGResult<()> {
    // Update status to processing
    update_file_status(&rag_file.id, "processing").await?;

    // Process the file with the RAG engine
    // Note: Content is now extracted from file storage by the engine itself
    // Processing options can be retrieved from database if needed
    engine
        .process_file(rag_file.file_id)
        .await?;

    Ok(())
}
