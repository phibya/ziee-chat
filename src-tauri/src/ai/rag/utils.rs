// Utility functions for RAG operations

use uuid::Uuid;
use crate::database::models::rag_instance::{RAGInstance, RAGEngineSettings};
use crate::database::queries::{rag_instances, providers, models};
use crate::ai::rag::types::{RAGModel, RAGModels, RAGInstanceInfo};

/// Get AI provider, models, and RAG engine settings using rag_instance_id
pub async fn get_rag_instance_info(
    rag_instance_id: Uuid,
) -> Result<RAGInstanceInfo, Box<dyn std::error::Error + Send + Sync>> {
    // 1. Get RAG instance from database
    let instance = rag_instances::get_rag_instance_by_id(rag_instance_id)
        .await
        .map_err(|e| format!("Failed to get RAG instance: {}", e))?
        .ok_or("RAG instance not found")?;

    // 2. Get provider information
    let provider = providers::get_provider_by_id(instance.provider_id)
        .await
        .map_err(|e| format!("Failed to get provider: {}", e))?
        .ok_or("Provider not found")?;

    // 3. Create embedding model with AI provider (required)
    let embedding_model_id = instance.embedding_model_id
        .ok_or("Embedding model ID not configured for this RAG instance")?;
    
    let embedding_model = models::get_model_by_id(embedding_model_id)
        .await
        .map_err(|e| format!("Failed to get embedding model: {}", e))?
        .ok_or("Embedding model not found")?;

    // Create AI provider for embedding model
    let embedding_ai_provider = crate::ai::model_manager::create_ai_provider_with_model_id(
        &provider,
        Some(embedding_model_id),
    )
    .await
    .map_err(|e| format!("Failed to create embedding AI provider: {}", e))?;

    let embedding_rag_model = RAGModel {
        model: embedding_model,
        ai_provider: embedding_ai_provider.into(),
    };

    // 4. Create LLM model with AI provider if specified (optional)
    let llm_rag_model = if let Some(llm_model_id) = instance.llm_model_id {
        let llm_model = models::get_model_by_id(llm_model_id)
            .await
            .map_err(|e| format!("Failed to get LLM model: {}", e))?
            .ok_or("LLM model not found")?;

        // Create AI provider for LLM model
        let llm_ai_provider = crate::ai::model_manager::create_ai_provider_with_model_id(
            &provider,
            Some(llm_model_id),
        )
        .await
        .map_err(|e| format!("Failed to create LLM AI provider: {}", e))?;

        Some(RAGModel {
            model: llm_model,
            ai_provider: llm_ai_provider.into(),
        })
    } else {
        None
    };

    Ok(RAGInstanceInfo {
        instance,
        provider,
        models: RAGModels {
            embedding_model: embedding_rag_model,
            llm_model: llm_rag_model,
        },
    })
}

/// Get RAG engine settings from RAG instance
pub fn get_rag_engine_settings(instance: &RAGInstance) -> &RAGEngineSettings {
    &instance.engine_settings
}

/// Create AI provider with LLM model (for text generation tasks)
pub async fn create_llm_provider(
    rag_instance_id: Uuid,
    user_id: Uuid,
) -> Result<RAGModel, Box<dyn std::error::Error + Send + Sync>> {
    // Get RAG instance
    let instance = rag_instances::get_rag_instance(rag_instance_id, user_id)
        .await
        .map_err(|e| format!("Failed to get RAG instance: {}", e))?
        .ok_or("RAG instance not found")?;

    // Get provider
    let provider = providers::get_provider_by_id(instance.provider_id)
        .await
        .map_err(|e| format!("Failed to get provider: {}", e))?
        .ok_or("Provider not found")?;

    // Get LLM model (required)
    let llm_model_id = instance.llm_model_id
        .ok_or("LLM model ID not configured for this RAG instance")?;
    
    let llm_model = models::get_model_by_id(llm_model_id)
        .await
        .map_err(|e| format!("Failed to get LLM model: {}", e))?
        .ok_or("LLM model not found")?;

    // Create AI provider with LLM model
    let ai_provider = crate::ai::model_manager::create_ai_provider_with_model_id(
        &provider,
        Some(llm_model_id),
    )
    .await
    .map_err(|e| format!("Failed to create AI provider: {}", e))?;

    Ok(RAGModel {
        model: llm_model,
        ai_provider: ai_provider.into(),
    })
}

/// Create AI provider with embedding model (for embedding tasks)
pub async fn create_embedding_provider(
    rag_instance_id: Uuid,
    user_id: Uuid,
) -> Result<RAGModel, Box<dyn std::error::Error + Send + Sync>> {
    // Get RAG instance
    let instance = rag_instances::get_rag_instance(rag_instance_id, user_id)
        .await
        .map_err(|e| format!("Failed to get RAG instance: {}", e))?
        .ok_or("RAG instance not found")?;

    // Get provider
    let provider = providers::get_provider_by_id(instance.provider_id)
        .await
        .map_err(|e| format!("Failed to get provider: {}", e))?
        .ok_or("Provider not found")?;

    // Get embedding model (required)
    let embedding_model_id = instance.embedding_model_id
        .ok_or("Embedding model ID not configured for this RAG instance")?;
    
    let embedding_model = models::get_model_by_id(embedding_model_id)
        .await
        .map_err(|e| format!("Failed to get embedding model: {}", e))?
        .ok_or("Embedding model not found")?;

    // Create AI provider with embedding model
    let ai_provider = crate::ai::model_manager::create_ai_provider_with_model_id(
        &provider,
        Some(embedding_model_id),
    )
    .await
    .map_err(|e| format!("Failed to create AI provider: {}", e))?;

    Ok(RAGModel {
        model: embedding_model,
        ai_provider: ai_provider.into(),
    })
}