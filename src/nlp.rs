use std::num::NonZeroUsize;

use anyhow::{Error as E, Result};
use candle_core::{DType, Device, Tensor};
use candle_nn::VarBuilder;
use candle_transformers::models::bert::{BertModel, Config};
use hf_hub::{api::sync::Api, Repo, RepoType};
use lru::LruCache;
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
    cache: LruCache<String, (Document, Tensor)>,
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

        let vb = unsafe {
            VarBuilder::from_mmaped_safetensors(&[weights_filename], DType::F32, &device)?
        };
        let model = BertModel::load(vb, &config)?;

        Ok(Self { model, tokenizer, device })
    }

    pub fn embed(&self, text: &str) -> Result<Tensor> {
        let tokens = self.tokenizer.encode(text, true).map_err(E::msg)?;
        let token_ids = Tensor::new(tokens.get_ids(), &self.device)?.unsqueeze(0)?;
        let token_type_ids = token_ids.zeros_like()?;

        // Fix: BERT forward takes 3 args: input_ids, token_type_ids, attention_mask
        // (Option)
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
        Self::with_capacity(1000)
    }

    pub fn with_capacity(capacity: usize) -> Result<Self> {
        let model = EmbeddingModel::new()?;
        let cache = LruCache::new(
            NonZeroUsize::new(capacity).ok_or_else(|| anyhow::anyhow!("Capacity must be > 0"))?,
        );
        Ok(Self { cache, model })
    }

    pub fn len(&self) -> usize {
        self.cache.len()
    }

    pub fn is_empty(&self) -> bool {
        self.cache.is_empty()
    }

    pub fn add_document(&mut self, content: &str, metadata: &str) -> Result<()> {
        let embedding = self.model.embed(content)?;
        let doc_id = uuid::Uuid::new_v4().to_string();
        let doc = Document {
            id: doc_id.clone(),
            content: content.to_string(),
            metadata: metadata.to_string(),
        };

        self.cache.put(doc_id, (doc, embedding));
        Ok(())
    }

    pub fn search(&self, query: &str, top_k: usize) -> Result<Vec<(Document, f32)>> {
        let query_emb = self.model.embed(query)?;

        // Note: LruCache::iter() returns items from most-recently-used to
        // least-recently-used. For search, we iterate over all current items in
        // the cache.
        let mut scores: Vec<(Document, f32)> = self
            .cache
            .iter()
            .map(|(_id, (doc, doc_emb))| {
                // Compute cosine similarity via dot product (embeddings are normalized)
                let score = (query_emb.clone() * doc_emb.clone())
                    .and_then(|t| t.sum_all())
                    .and_then(|t| t.to_scalar::<f32>())
                    .unwrap_or(0.0); // Fallback to 0.0 similarity on tensor operation failure
                (doc.clone(), score)
            })
            .collect();

        scores.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

        let results = scores.into_iter().take(top_k).collect();

        Ok(results)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_document_default_fields_are_strings() {
        let doc = Document { id: String::new(), content: String::new(), metadata: String::new() };
        assert!(doc.id.is_empty());
        assert!(doc.content.is_empty());
        assert!(doc.metadata.is_empty());
    }

    #[test]
    fn test_document_empty_metadata() {
        let doc = Document {
            id: "abc".to_string(),
            content: "some content".to_string(),
            metadata: String::new(),
        };
        assert!(doc.metadata.is_empty());
        assert_eq!(doc.content, "some content");
    }

    #[test]
    fn test_document_unicode_content() {
        let doc = Document {
            id: "unicode-1".to_string(),
            content: "こんにちは世界 — Olá mundo — مرحبا بالعالم".to_string(),
            metadata: "lang:multilingual".to_string(),
        };
        assert!(doc.content.contains("こんにちは"));
        assert!(doc.content.contains("Olá"));
        assert!(doc.content.contains("مرحبا"));
    }

    #[test]
    fn test_document_large_content() {
        let large = "x".repeat(10_000);
        let doc = Document {
            id: "large-1".to_string(),
            content: large.clone(),
            metadata: "size:10k".to_string(),
        };
        assert_eq!(doc.content.len(), 10_000);
        let cloned = doc.clone();
        assert_eq!(cloned.content.len(), 10_000);
    }

    #[test]
    fn test_document_special_characters_in_metadata() {
        let doc = Document {
            id: "spec-1".to_string(),
            content: "test".to_string(),
            metadata: "file:/tmp/test file (copy).pdf:page=3&lang=en".to_string(),
        };
        assert!(doc.metadata.contains("/tmp/"));
        assert!(doc.metadata.contains("page=3"));
    }

    #[test]
    fn test_document_source_metadata_convention() {
        let doc = Document {
            id: "conv-1".to_string(),
            content: "architecture overview".to_string(),
            metadata: "source:adr-0001.md".to_string(),
        };
        let parts: Vec<&str> = doc.metadata.split(':').collect();
        assert_eq!(parts.len(), 2);
        assert_eq!(parts[0], "source");
        assert!(parts[1].ends_with(".md"));
    }

    #[test]
    fn test_document_creation() {
        let doc = Document {
            id: "test-123".to_string(),
            content: "How to move a window?".to_string(),
            metadata: "manual:move".to_string(),
        };
        assert_eq!(doc.id, "test-123");
        assert_eq!(doc.content, "How to move a window?");
        assert_eq!(doc.metadata, "manual:move");
    }

    #[test]
    fn test_document_clone() {
        let doc = Document {
            id: "test-456".to_string(),
            content: "Test content".to_string(),
            metadata: "test:meta".to_string(),
        };
        let cloned = doc.clone();
        assert_eq!(doc.id, cloned.id);
        assert_eq!(doc.content, cloned.content);
        assert_eq!(doc.metadata, cloned.metadata);
    }

    #[test]
    fn test_document_debug() {
        let doc = Document {
            id: "test".to_string(),
            content: "content".to_string(),
            metadata: "meta".to_string(),
        };
        let debug = format!("{:?}", doc);
        assert!(debug.contains("test"));
        assert!(debug.contains("content"));
    }

    #[test]
    #[ignore] // Requires internet access or cached MiniLM model
    fn test_vector_store_basic() {
        let mut store = VectorStore::new().expect("Failed to create VectorStore");

        store
            .add_document("How to move a window?", "manual:move")
            .expect("Failed to add document");
        store
            .add_document("How to change workspace?", "manual:workspace")
            .expect("Failed to add document");

        let results = store.search("window movement", 1).expect("Search failed");

        assert!(!results.is_empty());
        assert!(results[0].0.content.contains("move a window"));
        assert!(results[0].1 > 0.5); // Should have high similarity
    }

    #[test]
    #[ignore] // Requires internet access or cached MiniLM model
    fn test_vector_store_empty_search() {
        let store = VectorStore::new().expect("Failed to create VectorStore");
        let results = store.search("anything", 5).expect("Search failed");
        assert!(results.is_empty());
    }

    #[test]
    #[ignore] // Requires internet access or cached MiniLM model
    fn test_vector_store_top_k_limit() {
        let mut store = VectorStore::new().expect("Failed to create VectorStore");

        store.add_document("Doc A", "meta:a").expect("Failed to add doc");
        store.add_document("Doc B", "meta:b").expect("Failed to add doc");
        store.add_document("Doc C", "meta:c").expect("Failed to add doc");

        let results = store.search("doc", 2).expect("Search failed");
        assert!(results.len() <= 2);
    }

    #[test]
    #[ignore] // Requires internet access or cached MiniLM model
    fn test_embedding_model_consistency() {
        let model = EmbeddingModel::new().expect("Failed to create embedding model");
        let emb1 = model.embed("hello world").expect("Failed to embed");
        let emb2 = model.embed("hello world").expect("Failed to embed");
        // Same input should produce same embedding
        let diff = (&emb1 - &emb2)
            .and_then(|t| t.sqr())
            .and_then(|t| t.sum_all())
            .and_then(|t| t.to_scalar::<f32>())
            .unwrap_or(f32::MAX);
        assert!(diff < 1e-6, "Same input should produce identical embeddings");
    }

    #[test]
    #[ignore]
    fn test_vector_store_lru_eviction() {
        let mut store = VectorStore::with_capacity(2).expect("Failed to create store");

        store.add_document("Doc 1", "m1").unwrap();
        store.add_document("Doc 2", "m2").unwrap();
        assert_eq!(store.len(), 2);

        // This should evict Doc 1
        store.add_document("Doc 3", "m3").unwrap();
        assert_eq!(store.len(), 2);

        let results = store.search("Doc 1", 10).unwrap();
        // Doc 1 should not be in results
        for (doc, _) in results {
            assert_ne!(doc.content, "Doc 1");
        }

        let results = store.search("Doc 3", 10).unwrap();
        assert!(results.iter().any(|(d, _)| d.content == "Doc 3"));
    }
}
