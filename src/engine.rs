use anyhow::{Error as E, Result};
use candle_core::{Device, Tensor};
use candle_transformers::generation::LogitsProcessor;
use candle_transformers::models::quantized_llama::ModelWeights as Qwen2;
use hf_hub::{api::sync::Api, Repo, RepoType};
use tokenizers::Tokenizer;
use crate::nlp::VectorStore;

#[derive(Clone)]
pub struct GenerationConfig {
    pub temperature: f64,
    pub top_p: f64,
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

        let default_system_prompt = "You are a Linux System Architect assistant embedded in a Hyprland environment.
If context is provided below, use it to answer.
Always output commands in format [[CMD:action:args]] when executing system actions.".to_string();

        println!("Initializing Local Engine (Qwen 1.8B)...");
        let api = Api::new()?;
        let repo = api.repo(Repo::new(repo_id, RepoType::Model));
        let model_path = repo.get(&filename)?;

        let tokenizer_repo = api.repo(Repo::new("Qwen/Qwen1.5-1.8B-Chat".to_string(), RepoType::Model));
        let tokenizer_filename = tokenizer_repo.get("tokenizer.json")?;
        let tokenizer = Tokenizer::from_file(tokenizer_filename).map_err(E::msg)?;

        let mut file = std::fs::File::open(&model_path)?;
        let content = candle_core::quantized::gguf_file::Content::read(&mut file).map_err(E::msg)?;
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
        let base_system_prompt = config
            .system_prompt
            .as_ref()
            .unwrap_or(&self.default_system_prompt);

        let full_system_prompt = format!("{}{}", base_system_prompt, context_injection);
        let templated_prompt = format!(
            "<|im_start|>system\n{}\n<|im_end|>\n<|im_start|>user\n{}\n<|im_end|>\n<|im_start|>assistant\n",
            full_system_prompt, user_input
        );

        let tokens = self.tokenizer.encode(templated_prompt, true).map_err(E::msg)?;
        let mut tokens = tokens.get_ids().to_vec();

        // Create logits processor with config parameters
        let logits_processor = LogitsProcessor::new(
            299792458,
            Some(config.temperature),
            Some(config.top_p),
        );

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
                callback(token_str);
            }
        }
        Ok(context_doc_ids)
    }
}