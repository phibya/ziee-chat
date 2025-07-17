use serde::{Deserialize, Serialize};

#[derive(Debug, Clone)]
pub struct TokenizerConfig {
    pub eos_token: String,
    pub eos_token_id: u32,
    pub bos_token: String,
    pub bos_token_id: u32,
    pub unk_token: String,
    pub unk_token_id: u32,
    pub chat_template: Option<String>,
    pub model_max_length: u32,
    pub pad_token: Option<String>,
}

#[derive(Debug, Clone)]
pub struct ModelConfig {
    pub max_position_embeddings: u32,
    pub vocab_size: u32,
    pub hidden_size: u32,
    pub model_type: String,
    pub architectures: Vec<String>,
}

impl Default for TokenizerConfig {
    fn default() -> Self {
        Self {
            eos_token: "</s>".to_string(),
            eos_token_id: 2,
            bos_token: "<s>".to_string(),
            bos_token_id: 1,
            unk_token: "<unk>".to_string(),
            unk_token_id: 0,
            chat_template: None,
            model_max_length: 2048,
            pad_token: None,
        }
    }
}

impl Default for ModelConfig {
    fn default() -> Self {
        Self {
            max_position_embeddings: 2048,
            vocab_size: 32000,
            hidden_size: 2048,
            model_type: "llama".to_string(),
            architectures: vec!["LlamaForCausalLM".to_string()],
        }
    }
}