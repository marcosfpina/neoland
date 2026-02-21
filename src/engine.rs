use anyhow::{Error as E, Result};
use candle_core::{Device, Tensor};
use candle_transformers::{
    generation::LogitsProcessor, models::quantized_llama::ModelWeights as Qwen2,
};
use hf_hub::{api::sync::Api, Repo, RepoType};
use tokenizers::Tokenizer;

use crate::nlp::VectorStore;

#[derive(Clone)]
pub struct GenerationConfig {
    pub temperature: f64,
    pub top_p: f64,
    pub typical_p: f64,
    pub epsilon_cutoff: f64,
    pub eta_cutoff: f64,
    pub tail_free_sampling: f64,
    pub top_a: f64,
    pub max_tokens: usize,
    pub repetition_penalty: f32,
    pub context_top_k: usize,
    pub context_similarity_threshold: f32,
    pub disable_context: bool,
    pub system_prompt: Option<String>,
    pub enable_commands: bool,
    pub allowed_commands: Vec<String>,
}

impl Default for GenerationConfig {
    fn default() -> Self {
        Self {
            temperature: 0.7,
            top_p: 0.9,
            typical_p: 1.0,
            epsilon_cutoff: 0.0,
            eta_cutoff: 0.0,
            tail_free_sampling: 1.0,
            top_a: 0.0,
            max_tokens: 600,
            repetition_penalty: 1.1,
            context_top_k: 2,
            context_similarity_threshold: 0.3,
            disable_context: false,
            system_prompt: None,
            enable_commands: true,
            allowed_commands: vec![],
        }
    }
}

pub struct LocalEngine {
    model: Qwen2,
    tokenizer: Tokenizer,
    device: Device,
    default_system_prompt: String,
}

impl LocalEngine {
    pub fn new() -> Result<Self> {
        let device = Device::Cpu;
        let repo_id = "Qwen/Qwen1.5-1.8B-Chat-GGUF".to_string();
        let filename = "qwen1_5-1_8b-chat-q4_k_m.gguf".to_string();

        let default_system_prompt = "You are a Linux System Architect assistant embedded in a \
                                     Hyprland environment.
If context is provided below, use it to answer.
Always output commands in format [[CMD:action:args]] when executing system actions."
            .to_string();

        println!("Initializing Local Engine (Qwen 1.8B)...");
        let api = Api::new()?;
        let repo = api.repo(Repo::new(repo_id, RepoType::Model));
        let model_path = repo.get(&filename)?;

        let tokenizer_repo =
            api.repo(Repo::new("Qwen/Qwen1.5-1.8B-Chat".to_string(), RepoType::Model));
        let tokenizer_filename = tokenizer_repo.get("tokenizer.json")?;
        let tokenizer = Tokenizer::from_file(tokenizer_filename).map_err(E::msg)?;

        let mut file = std::fs::File::open(&model_path)?;
        let content =
            candle_core::quantized::gguf_file::Content::read(&mut file).map_err(E::msg)?;
        let model = Qwen2::from_gguf(content, &mut file, &device)?;

        println!("Local Engine Ready.");
        Ok(Self { model, tokenizer, device, default_system_prompt })
    }

    pub fn generate_stream(
        &mut self,
        user_input: &str,
        vector_store: Option<&VectorStore>,
        config: &GenerationConfig,
        callback: impl Fn(String),
    ) -> Result<Vec<String>> {
        let mut context_doc_ids = Vec::new();
        let mut context_injection = String::new();

        // Context retrieval based on config
        if !config.disable_context {
            if let Some(store) = vector_store {
                if let Ok(results) = store.search(user_input, config.context_top_k) {
                    if !results.is_empty() {
                        context_injection.push_str("\n### RELEVANT CONTEXT:\n");
                        for (doc, score) in results {
                            if score > config.context_similarity_threshold {
                                let line = format!("- {}\n", doc.content);
                                context_injection.push_str(&line);
                                context_doc_ids.push(doc.id.clone());
                            }
                        }
                        context_injection.push_str("### END CONTEXT\n");
                    }
                }
            }
        }

        // Use custom system prompt or default
        let base_system_prompt =
            config.system_prompt.as_ref().unwrap_or(&self.default_system_prompt);

        let full_system_prompt = format!("{}{}", base_system_prompt, context_injection);
        let templated_prompt = format!(
            "<|im_start|>system\n{}\n<|im_end|>\n<|im_start|>user\n{}\n<|im_end|>\\
             n<|im_start|>assistant\n",
            full_system_prompt, user_input
        );

        let tokens = self.tokenizer.encode(templated_prompt, true).map_err(E::msg)?;
        let mut tokens = tokens.get_ids().to_vec();

        // Create logits processor with config parameters
        let mut logits_processor =
            LogitsProcessor::new(299792458, Some(config.temperature), Some(config.top_p));

        let gen_start = std::time::Instant::now();
        let mut output_tokens: u32 = 0;

        for _ in 0..config.max_tokens {
            let input = Tensor::new(&tokens[tokens.len() - 1..], &self.device)?.unsqueeze(0)?;
            let logits = self.model.forward(&input, tokens.len() - 1)?;
            let logits = logits.squeeze(0)?.squeeze(0)?;
            let next_token = logits_processor.sample(&logits)?;
            tokens.push(next_token);

            if let Ok(token_str) = self.tokenizer.decode(&[next_token], true) {
                if token_str.contains("<|im_end|>") || token_str.contains("<|endoftext|>") {
                    break;
                }
                output_tokens += 1;
                callback(token_str);
            }
        }

        // Record LLM generation metrics
        let elapsed = gen_start.elapsed().as_secs_f64();
        crate::metrics::utils::record_llm_request(
            "local",
            "qwen-1.8b",
            "success",
            0, // prompt tokens not counted at this layer
            output_tokens,
            elapsed,
        );

        Ok(context_doc_ids)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generation_config_defaults() {
        let config = GenerationConfig::default();
        assert!((config.temperature - 0.7).abs() < f64::EPSILON);
        assert!((config.top_p - 0.9).abs() < f64::EPSILON);
        assert!((config.typical_p - 1.0).abs() < f64::EPSILON);
        assert_eq!(config.max_tokens, 600);
        assert!((config.repetition_penalty - 1.1).abs() < f32::EPSILON);
        assert_eq!(config.context_top_k, 2);
        assert!((config.context_similarity_threshold - 0.3).abs() < f32::EPSILON);
        assert!(!config.disable_context);
        assert!(config.system_prompt.is_none());
        assert!(config.enable_commands);
        assert!(config.allowed_commands.is_empty());
    }

    #[test]
    fn test_generation_config_clone() {
        let config = GenerationConfig::default();
        let cloned = config.clone();
        assert_eq!(config.max_tokens, cloned.max_tokens);
        assert!((config.temperature - cloned.temperature).abs() < f64::EPSILON);
    }

    #[test]
    fn test_generation_config_custom_system_prompt() {
        let mut config = GenerationConfig::default();
        config.system_prompt = Some("Custom assistant prompt".to_string());
        assert_eq!(config.system_prompt.as_deref(), Some("Custom assistant prompt"));
    }

    #[test]
    fn test_generation_config_custom_values() {
        let config = GenerationConfig {
            temperature: 0.1,
            top_p: 0.5,
            max_tokens: 100,
            disable_context: true,
            enable_commands: false,
            allowed_commands: vec!["ls".to_string(), "pwd".to_string()],
            ..Default::default()
        };
        assert!((config.temperature - 0.1).abs() < f64::EPSILON);
        assert!((config.top_p - 0.5).abs() < f64::EPSILON);
        assert_eq!(config.max_tokens, 100);
        assert!(config.disable_context);
        assert!(!config.enable_commands);
        assert_eq!(config.allowed_commands.len(), 2);
    }

    #[test]
    fn test_generation_config_boundary_values() {
        let config = GenerationConfig {
            temperature: 0.0,
            top_p: 0.0,
            max_tokens: 0,
            context_top_k: 0,
            context_similarity_threshold: 0.0,
            ..Default::default()
        };
        assert!((config.temperature - 0.0).abs() < f64::EPSILON);
        assert_eq!(config.max_tokens, 0);
        assert_eq!(config.context_top_k, 0);
    }

    #[test]
    #[ignore] // Requires HuggingFace model download (~1GB)
    fn test_local_engine_initialization() {
        let engine = LocalEngine::new();
        assert!(engine.is_ok(), "LocalEngine should initialize with cached model");
    }

    #[test]
    #[ignore] // Requires HuggingFace model download (~1GB)
    fn test_local_engine_generate_stream() {
        use std::sync::{Arc, Mutex};
        let mut engine = LocalEngine::new().expect("Failed to create engine");
        let config = GenerationConfig { max_tokens: 10, ..Default::default() };
        let tokens = Arc::new(Mutex::new(Vec::new()));
        let tokens_clone = tokens.clone();
        let result = engine.generate_stream("Hello", None, &config, move |token| {
            tokens_clone.lock().unwrap().push(token);
        });
        assert!(result.is_ok());
    }
}
