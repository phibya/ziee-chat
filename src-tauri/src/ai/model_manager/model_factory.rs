use uuid::Uuid;

use crate::database::queries::models::{get_model_by_id, get_provider_by_model_id};
use crate::ai::{AIModel, AIProvider};
use crate::ai::ModelInstance;
use crate::ai::create_ai_provider_with_model_id;

/// Create an AIModel instance by model UUID
/// 
/// This function provides a simplified way to create AI models by encapsulating:
/// 1. Database lookups for model and provider info
/// 2. AIProvider creation using existing infrastructure  
/// 3. Wrapping in ModelInstance for clean API usage
///
/// # Arguments
/// * `model_id` - UUID of the model to create
///
/// # Returns
/// * `Box<dyn AIModel>` - AIModel instance ready for use
///
/// # Example
/// ```rust
/// let ai_model = create_ai_model(model_id).await?;
/// let response = ai_model.chat(SimplifiedChatRequest {
///     messages: vec![ChatMessage::text("user", "Hello!")],
///     stream: false,
/// }).await?;
/// ```
pub async fn create_ai_model(
    model_id: Uuid,
) -> Result<Box<dyn AIModel>, Box<dyn std::error::Error + Send + Sync>> {
    // 1. Load model from database
    let model = get_model_by_id(model_id).await
        .map_err(|e| format!("Failed to get model {}: {}", model_id, e))?
        .ok_or_else(|| format!("Model {} not found", model_id))?;
    
    // 2. Load provider from database
    let provider = get_provider_by_model_id(model_id).await
        .map_err(|e| format!("Failed to get provider for model {}: {}", model_id, e))?
        .ok_or_else(|| format!("Provider for model {} not found", model_id))?;
    
    // 3. Create the underlying AIProvider using existing infrastructure
    let ai_provider = create_ai_provider_with_model_id(&provider, Some(model_id))
        .await
        .map_err(|e| format!("Failed to create AI provider for model {}: {}", model_id, e))?;
    
    // 4. Wrap in ModelInstance to provide AIModel interface
    let model_instance = ModelInstance::new(model, ai_provider);
    
    Ok(Box::new(model_instance))
}

/// Create an AIModel instance with custom provider
/// 
/// This is useful for testing or when you already have a provider instance
/// and want to wrap it with a specific model configuration.
///
/// # Arguments
/// * `model_id` - UUID of the model to create
/// * `provider` - Pre-created AIProvider instance
///
/// # Returns
/// * `Box<dyn AIModel>` - AIModel instance ready for use
pub async fn create_ai_model_with_provider(
  model_id: Uuid,
  provider: Box<dyn AIProvider>,
) -> Result<Box<dyn AIModel>, Box<dyn std::error::Error + Send + Sync>> {
    // Load model from database
    let model = get_model_by_id(model_id).await
        .map_err(|e| format!("Failed to get model {}: {}", model_id, e))?
        .ok_or_else(|| format!("Model {} not found", model_id))?;
    
    // Wrap with the provided provider
    let model_instance = ModelInstance::new(model, provider);
    
    Ok(Box::new(model_instance))
}
