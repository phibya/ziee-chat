// Utility functions for RAG operations

use crate::ai::rag::types::{RAGInstanceInfo, RAGModel, RAGModels};
use crate::database::models::rag_instance::{RAGEngineSettings, RAGInstance, RAGInstanceErrorCode};
use crate::database::queries::{models, providers, rag_instances, rag_providers};
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

    let embedding_model = models::get_model_by_id(embedding_model_id)
        .await
        .map_err(|_| RAGInstanceErrorCode::DatabaseError)?
        .ok_or(RAGInstanceErrorCode::EmbeddingModelNotFound)?;

    let embedding_model_provider = providers::get_provider_by_id(embedding_model.provider_id)
        .await
        .map_err(|_| RAGInstanceErrorCode::DatabaseError)?
        .ok_or(RAGInstanceErrorCode::ProviderNotFound)?;

    // Create AI provider for embedding model
    // Note: Keeping existing approach for RAG compatibility - both approaches coexist
    let embedding_ai_provider = crate::ai::model_manager::create_ai_provider_with_model_id(
        &embedding_model_provider,
        Some(embedding_model_id),
    )
    .await
    .map_err(|_| RAGInstanceErrorCode::ProviderConnectionFailed)?;

    let embedding_rag_model = RAGModel {
        model: embedding_model,
        ai_provider: embedding_ai_provider.into(),
    };

    // 4. Create LLM model with AI provider if specified (optional)
    let llm_rag_model = if let Some(llm_model_id) = instance.llm_model_id {
        let llm_model = models::get_model_by_id(llm_model_id)
            .await
            .map_err(|_| RAGInstanceErrorCode::DatabaseError)?
            .ok_or(RAGInstanceErrorCode::LlmModelNotFound)?;

        let llm_model_provider = providers::get_provider_by_id(llm_model.provider_id)
            .await
            .map_err(|_| RAGInstanceErrorCode::DatabaseError)?
            .ok_or(RAGInstanceErrorCode::ProviderNotFound)?;

        // Create AI provider for LLM model
        // Note: Keeping existing approach for RAG compatibility - both approaches coexist
        let llm_ai_provider = crate::ai::model_manager::create_ai_provider_with_model_id(
            &llm_model_provider,
            Some(llm_model_id),
        )
        .await
        .map_err(|_| RAGInstanceErrorCode::ProviderConnectionFailed)?;

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
