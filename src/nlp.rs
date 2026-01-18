use anyhow::{Error as E, Result};
use candle_core::{DType, Device, Tensor};
use candle_nn::VarBuilder;
use candle_transformers::models::bert::{BertModel, Config};
use hf_hub::{api::sync::Api, Repo, RepoType};
use tokenizers::Tokenizer;

pub struct EmbeddingModel {
    model: BertModel,
    tokenizer: Tokenizer,
    device: Device,
}

#[derive(Debug, Clone)]
pub struct Document {
    pub id: String,
    pub content: String,
    pub metadata: String,
}

pub struct VectorStore {
    documents: Vec<Document>,
    embeddings: Vec<Tensor>,
    model: EmbeddingModel,
}

impl EmbeddingModel {
    pub fn new() -> Result<Self> {
        let device = Device::Cpu;
        
        let repo_id = "sentence-transformers/all-MiniLM-L6-v2".to_string();
        let api = Api::new()?;
        let repo = api.repo(Repo::new(repo_id, RepoType::Model));

        println!("Carregando modelo de Embeddings (MiniLM)...");
        let config_filename = repo.get("config.json")?;
        let tokenizer_filename = repo.get("tokenizer.json")?;
        let weights_filename = repo.get("model.safetensors")?;

        let config_str = std::fs::read_to_string(config_filename)?;
        let config: Config = serde_json::from_str(&config_str)?;
        
        let tokenizer = Tokenizer::from_file(tokenizer_filename).map_err(E::msg)?;
        
        let vb = unsafe { VarBuilder::from_mmaped_safetensors(&[weights_filename], DType::F32, &device)? };
        let model = BertModel::load(vb, &config)?;

        Ok(Self {
            model,
            tokenizer,
            device,
        })
    }

    pub fn embed(&self, text: &str) -> Result<Tensor> {
        let tokens = self.tokenizer.encode(text, true).map_err(E::msg)?;
        let token_ids = Tensor::new(tokens.get_ids(), &self.device)?.unsqueeze(0)?;
        let token_type_ids = token_ids.zeros_like()?; 

        // Fix: BERT forward takes 3 args: input_ids, token_type_ids, attention_mask (Option)
        let embeddings = self.model.forward(&token_ids, &token_type_ids, None)?;
        
        let (_b, t, _h) = embeddings.dims3()?;
        let sum = embeddings.sum(1)?;
        let pooled = (sum / (t as f64))?;
        
        let pooled_norm = pooled.broadcast_div(&pooled.sqr()?.sum_keepdim(1)?.sqrt()?)?;
        
        Ok(pooled_norm.squeeze(0)?)
    }
}

impl VectorStore {
    pub fn new() -> Result<Self> {
        let model = EmbeddingModel::new()?;
        Ok(Self {
            documents: Vec::new(),
            embeddings: Vec::new(),
            model,
        })
    }

    pub fn add_document(&mut self, content: &str, metadata: &str) -> Result<()> {
        let embedding = self.model.embed(content)?;
        let doc = Document {
            id: uuid::Uuid::new_v4().to_string(),
            content: content.to_string(),
            metadata: metadata.to_string(),
        };
        
        self.documents.push(doc);
        self.embeddings.push(embedding);
        Ok(())
    }

    pub fn search(&self, query: &str, top_k: usize) -> Result<Vec<(Document, f32)>> {
        let query_emb = self.model.embed(query)?;
        
        let mut scores: Vec<(usize, f32)> = self.embeddings.iter().enumerate().map(|(idx, doc_emb)| {
            // Compute cosine similarity via dot product (embeddings are normalized)
            let score = (query_emb.clone() * doc_emb.clone())
                .and_then(|t| t.sum_all())
                .and_then(|t| t.to_scalar::<f32>())
                .unwrap_or(0.0); // Fallback to 0.0 similarity on tensor operation failure
            (idx, score)
        }).collect();

        scores.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

        let results = scores.into_iter()
            .take(top_k)
            .map(|(idx, score)| (self.documents[idx].clone(), score))
            .collect();

        Ok(results)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_vector_store_basic() {
        // NOTE: This test requires internet access to download the MiniLM model 
        // or a pre-cached model in ~/.cache/huggingface
        let mut store = VectorStore::new().expect("Failed to create VectorStore");
        
        store.add_document("How to move a window?", "manual:move")
            .expect("Failed to add document");
        store.add_document("How to change workspace?", "manual:workspace")
            .expect("Failed to add document");
        
        let results = store.search("window movement", 1)
            .expect("Search failed");
            
        assert!(!results.is_empty());
        assert!(results[0].0.content.contains("move a window"));
        assert!(results[0].1 > 0.5); // Should have high similarity
    }
}