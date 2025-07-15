use crate::ai::candle::{CandleError, CandleModel};
use crate::ai::candle_models::ModelFactory;
use crate::ai::openai_types::*;
use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::{IntoResponse, Response, Sse},
    routing::{get, post},
    Json, Router,
};
use candle_core::Device;
use futures::stream::{self, Stream};
use serde_json;
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};
use tokio::sync::Mutex;
use tokio_stream::StreamExt;
use uuid::Uuid;

pub struct ModelServerState {
    pub model: Arc<Mutex<Box<dyn CandleModel + Send + Sync>>>,
    pub tokenizer: Arc<tokenizers::Tokenizer>,
    pub model_id: String,
    pub model_name: String,
    pub architecture: String,
    pub started_at: i64,
}

impl ModelServerState {
    pub async fn new(
        model_path: &str,
        architecture: &str,
        model_id: &str,
        model_name: &str,
    ) -> Result<Self, CandleError> {
        println!("Loading model from: {}", model_path);

        let device = Device::Cpu; // TODO: Add GPU support
        let model = ModelFactory::create_model(architecture, model_path, &device)?;
        let tokenizer = ModelFactory::load_tokenizer(architecture, model_path)?;

        let started_at = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64;

        Ok(Self {
            model: Arc::new(Mutex::new(model)),
            tokenizer: Arc::new(tokenizer),
            model_id: model_id.to_string(),
            model_name: model_name.to_string(),
            architecture: architecture.to_string(),
            started_at,
        })
    }

    pub async fn new_with_specific_files(
        model_path: &str,
        architecture: &str,
        model_id: &str,
        model_name: &str,
        config_file: Option<&str>,
        tokenizer_file: Option<&str>,
        weight_file: Option<&str>,
        additional_weight_files: Option<&str>,
        _vocab_file: Option<&str>,
        _special_tokens_file: Option<&str>,
    ) -> Result<Self, CandleError> {
        println!("Loading model from: {} with specific files", model_path);
        
        if let Some(config) = config_file {
            println!("  Config file: {}", config);
        }
        if let Some(tokenizer) = tokenizer_file {
            println!("  Tokenizer file: {}", tokenizer);
        }
        if let Some(weight) = weight_file {
            println!("  Weight file: {}", weight);
        }
        if let Some(additional) = additional_weight_files {
            println!("  Additional weight files: {}", additional);
        }

        let device = Device::Cpu; // TODO: Add GPU support
        
        // For now, use the existing factory methods but with specific file awareness
        // TODO: Update ModelFactory to accept specific file paths
        let model = ModelFactory::create_model_with_files(
            architecture, 
            model_path, 
            &device,
            config_file,
            weight_file,
            additional_weight_files
        )?;
        
        let tokenizer = ModelFactory::load_tokenizer_with_file(
            architecture, 
            model_path,
            tokenizer_file
        )?;

        let started_at = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64;

        Ok(Self {
            model: Arc::new(Mutex::new(model)),
            tokenizer: Arc::new(tokenizer),
            model_id: model_id.to_string(),
            model_name: model_name.to_string(),
            architecture: architecture.to_string(),
            started_at,
        })
    }
}

pub fn create_model_server_router(state: ModelServerState) -> Router {
    Router::new()
        .route("/v1/chat/completions", post(chat_completions))
        .route("/v1/completions", post(completions))
        .route("/v1/models", get(list_models))
        .route("/v1/models/{model_id}", get(get_model))
        .route("/health", get(health_check))
        .route("/shutdown", post(shutdown_server))
        .with_state(Arc::new(state))
}

/// OpenAI-compatible chat completions endpoint
async fn chat_completions(
    State(state): State<Arc<ModelServerState>>,
    Json(request): Json<ChatCompletionRequest>,
) -> Response {
    if request.stream {
        stream_chat_completion(state, request).await
    } else {
        non_stream_chat_completion(state, request).await
    }
}

async fn non_stream_chat_completion(
    state: Arc<ModelServerState>,
    request: ChatCompletionRequest,
) -> Response {
    // Convert chat messages to a single prompt
    let prompt = messages_to_prompt(&request.messages);

    // Tokenize input
    let tokens = match state.tokenizer.encode(prompt.clone(), true) {
        Ok(encoding) => encoding.get_ids().to_vec(),
        Err(e) => {
            return Json(ErrorResponse::invalid_request(&format!(
                "Tokenization failed: {}",
                e
            )))
            .into_response();
        }
    };

    // Generate response
    let response_text = match generate_text(&state, &tokens, &request).await {
        Ok(text) => text,
        Err(e) => {
            return Json(ErrorResponse::server_error(&format!(
                "Generation failed: {}",
                e
            )))
            .into_response();
        }
    };

    let response_id = format!("chatcmpl-{}", Uuid::new_v4());
    let created = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs() as i64;

    let response = ChatCompletionResponse {
        id: response_id,
        object: "chat.completion".to_string(),
        created,
        model: state.model_name.clone(),
        choices: vec![ChatChoice {
            index: 0,
            message: ChatMessage {
                role: "assistant".to_string(),
                content: response_text.clone(),
                name: None,
            },
            finish_reason: Some("stop".to_string()),
        }],
        usage: Usage {
            prompt_tokens: tokens.len() as i32,
            completion_tokens: estimate_tokens(&response_text),
            total_tokens: tokens.len() as i32 + estimate_tokens(&response_text),
        },
    };

    Json(response).into_response()
}

async fn stream_chat_completion(
    state: Arc<ModelServerState>,
    request: ChatCompletionRequest,
) -> Response {
    let prompt = messages_to_prompt(&request.messages);

    let tokens = match state.tokenizer.encode(prompt, true) {
        Ok(encoding) => encoding.get_ids().to_vec(),
        Err(e) => {
            return Json(ErrorResponse::invalid_request(&format!(
                "Tokenization failed: {}",
                e
            )))
            .into_response();
        }
    };

    let response_id = format!("chatcmpl-{}", Uuid::new_v4());
    let created = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs() as i64;

    let stream = generate_text_stream(state.clone(), tokens, request, response_id.clone(), created);

    Sse::new(stream)
        .keep_alive(
            axum::response::sse::KeepAlive::new()
                .interval(std::time::Duration::from_secs(30))
                .text("keepalive"),
        )
        .into_response()
}

/// Legacy completions endpoint
async fn completions(
    State(state): State<Arc<ModelServerState>>,
    Json(request): Json<CompletionRequest>,
) -> Response {
    let tokens = match state.tokenizer.encode(request.prompt.clone(), true) {
        Ok(encoding) => encoding.get_ids().to_vec(),
        Err(e) => {
            return Json(ErrorResponse::invalid_request(&format!(
                "Tokenization failed: {}",
                e
            )))
            .into_response();
        }
    };

    let chat_request = ChatCompletionRequest {
        messages: vec![ChatMessage {
            role: "user".to_string(),
            content: request.prompt.clone(),
            name: None,
        }],
        model: request.model,
        temperature: request.temperature,
        top_p: request.top_p,
        top_k: request.top_k,
        max_tokens: request.max_tokens,
        stream: request.stream,
        stop: request.stop,
        frequency_penalty: None,
        presence_penalty: None,
        user: None,
    };

    let response_text = match generate_text(&state, &tokens, &chat_request).await {
        Ok(text) => text,
        Err(e) => {
            return Json(ErrorResponse::server_error(&format!(
                "Generation failed: {}",
                e
            )))
            .into_response();
        }
    };

    let response_id = format!("cmpl-{}", Uuid::new_v4());
    let created = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs() as i64;

    let response = CompletionResponse {
        id: response_id,
        object: "text_completion".to_string(),
        created,
        model: state.model_name.clone(),
        choices: vec![CompletionChoice {
            text: response_text.clone(),
            index: 0,
            finish_reason: Some("stop".to_string()),
        }],
        usage: Usage {
            prompt_tokens: tokens.len() as i32,
            completion_tokens: estimate_tokens(&response_text),
            total_tokens: tokens.len() as i32 + estimate_tokens(&response_text),
        },
    };

    Json(response).into_response()
}

async fn list_models(State(state): State<Arc<ModelServerState>>) -> Json<ModelsResponse> {
    let model_info = ModelInfo {
        id: state.model_id.clone(),
        object: "model".to_string(),
        created: state.started_at,
        owned_by: "candle".to_string(),
        permission: vec![],
        root: state.model_id.clone(),
        parent: None,
    };

    Json(ModelsResponse {
        object: "list".to_string(),
        data: vec![model_info],
    })
}

async fn get_model(
    State(state): State<Arc<ModelServerState>>,
    Path(model_id): Path<String>,
) -> Response {
    if model_id != state.model_id {
        return (
            StatusCode::NOT_FOUND,
            Json(ErrorResponse::invalid_request("Model not found")),
        )
            .into_response();
    }

    let model_info = ModelInfo {
        id: state.model_id.clone(),
        object: "model".to_string(),
        created: state.started_at,
        owned_by: "candle".to_string(),
        permission: vec![],
        root: state.model_id.clone(),
        parent: None,
    };

    Json(model_info).into_response()
}

async fn health_check() -> Json<serde_json::Value> {
    Json(serde_json::json!({
        "status": "healthy",
        "timestamp": SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs()
    }))
}

async fn shutdown_server() -> Json<serde_json::Value> {
    // TODO: Implement graceful shutdown
    Json(serde_json::json!({
        "message": "Shutdown initiated"
    }))
}

// Helper functions

fn messages_to_prompt(messages: &[ChatMessage]) -> String {
    let mut prompt = String::new();

    for message in messages {
        match message.role.as_str() {
            "system" => prompt.push_str(&format!("System: {}\n", message.content)),
            "user" => prompt.push_str(&format!("User: {}\n", message.content)),
            "assistant" => prompt.push_str(&format!("Assistant: {}\n", message.content)),
            _ => prompt.push_str(&format!("{}: {}\n", message.role, message.content)),
        }
    }

    prompt.push_str("Assistant: ");
    prompt
}

async fn generate_text(
    state: &Arc<ModelServerState>,
    tokens: &[u32],
    request: &ChatCompletionRequest,
) -> Result<String, CandleError> {
    // This is a simplified implementation
    // In a real implementation, you would:
    // 1. Convert tokens to tensor
    // 2. Run the model forward pass
    // 3. Sample from the output logits
    // 4. Decode tokens back to text

    let _model = state.model.lock().await;

    // For now, return a placeholder response
    let max_tokens = request.max_tokens.unwrap_or(100);
    let response = format!(
        "This is a placeholder response from model '{}'. \
         The model received {} input tokens and will generate up to {} tokens. \
         Temperature: {:?}, Top-P: {:?}",
        state.model_name,
        tokens.len(),
        max_tokens,
        request.temperature,
        request.top_p
    );

    Ok(response)
}

fn generate_text_stream(
    state: Arc<ModelServerState>,
    _tokens: Vec<u32>,
    _request: ChatCompletionRequest,
    response_id: String,
    created: i64,
) -> impl Stream<Item = Result<axum::response::sse::Event, std::convert::Infallible>> {
    let words = vec![
        "This",
        "is",
        "a",
        "streaming",
        "response",
        "from",
        "the",
        "model",
    ];

    // Clone values that need to be used in the final chunk
    let final_state = state.clone();
    let final_response_id = response_id.clone();

    stream::iter(words.into_iter().enumerate())
        .then(move |(i, word)| {
            let state = state.clone();
            let response_id = response_id.clone();
            let model_name = state.model_name.clone();

            async move {
                // Simulate some processing time
                tokio::time::sleep(std::time::Duration::from_millis(100)).await;

                let chunk = ChatCompletionChunk {
                    id: response_id,
                    object: "chat.completion.chunk".to_string(),
                    created,
                    model: model_name,
                    choices: vec![ChatChoiceDelta {
                        index: 0,
                        delta: ChatMessageDelta {
                            role: if i == 0 {
                                Some("assistant".to_string())
                            } else {
                                None
                            },
                            content: Some(format!("{} ", word)),
                        },
                        finish_reason: None,
                    }],
                };

                let data = serde_json::to_string(&chunk).unwrap();
                axum::response::sse::Event::default().data(data)
            }
        })
        .chain(stream::once(async move {
            // Send final chunk
            let final_chunk = ChatCompletionChunk {
                id: final_response_id,
                object: "chat.completion.chunk".to_string(),
                created,
                model: final_state.model_name.clone(),
                choices: vec![ChatChoiceDelta {
                    index: 0,
                    delta: ChatMessageDelta {
                        role: None,
                        content: None,
                    },
                    finish_reason: Some("stop".to_string()),
                }],
            };

            let data = serde_json::to_string(&final_chunk).unwrap();
            axum::response::sse::Event::default().data(data)
        }))
        .map(Ok)
}

fn estimate_tokens(text: &str) -> i32 {
    // Rough estimation: 1 token per 4 characters
    (text.len() / 4).max(1) as i32
}
