// Utility functions for RAG operations

use crate::ai::rag::types::{RAGInstanceInfo, RAGModels};
use crate::database::models::rag_instance::RAGInstanceErrorCode;
use crate::database::queries::{rag_instances, rag_providers};
use uuid::Uuid;

/// Get AI provider, models, and RAG engine settings using rag_instance_id
pub async fn get_rag_instance_info(
    rag_instance_id: Uuid,
) -> Result<RAGInstanceInfo, RAGInstanceErrorCode> {
    // 1. Get RAG instance from database
    let instance = rag_instances::get_rag_instance_by_id(rag_instance_id)
        .await
        .map_err(|_| RAGInstanceErrorCode::DatabaseError)?
        .ok_or(RAGInstanceErrorCode::RagInstanceNotFound)?;

    // 2. Get provider information
    let provider = rag_providers::get_rag_provider_by_id(instance.provider_id)
        .await
        .map_err(|_| RAGInstanceErrorCode::DatabaseError)?
        .ok_or(RAGInstanceErrorCode::ProviderNotFound)?;

    // 3. Create embedding model with AI provider (required)
    let embedding_model_id = instance
        .embedding_model_id
        .ok_or(RAGInstanceErrorCode::EmbeddingModelNotConfig)?;

    // Create AI model for embedding using the new simplified factory
    let embedding_ai_model = crate::ai::model_manager::model_factory::create_ai_model(embedding_model_id)
        .await
        .map_err(|_| RAGInstanceErrorCode::ProviderConnectionFailed)?;

    // 4. Create LLM model if specified (optional)
    let llm_ai_model = if let Some(llm_model_id) = instance.llm_model_id {
        let ai_model = crate::ai::model_manager::model_factory::create_ai_model(llm_model_id)
            .await
            .map_err(|_| RAGInstanceErrorCode::ProviderConnectionFailed)?;
        Some(ai_model)
    } else {
        None
    };

    Ok(RAGInstanceInfo {
        instance,
        provider,
        models: RAGModels {
            embedding_model: embedding_ai_model.into(),
            llm_model: llm_ai_model.map(|m| m.into()),
        },
    })
}

