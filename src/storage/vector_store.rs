// NEOLAND Persistent Vector Store with PostgreSQL + pgvector
// Phase 4.7: Performance Optimization
//
// This replaces the in-memory VectorStore with a persistent PostgreSQL-backed
// implementation using the pgvector extension for efficient similarity search.

use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use pgvector::Vector;
use sqlx::{postgres::PgPoolOptions, PgPool, Row};
use uuid::Uuid;

use crate::nlp::EmbeddingModel;

/// Document stored in the database with embeddings
#[derive(Debug, Clone, sqlx::FromRow)]
pub struct StoredDocument {
    pub id: Uuid,
    pub content: String,
    pub metadata: String,
    #[sqlx(try_from = "Vec<f32>")]
    pub embedding: Vector,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Persistent vector store using PostgreSQL + pgvector
pub struct PersistentVectorStore {
    pool: PgPool,
    model: EmbeddingModel,
}

impl PersistentVectorStore {
    /// Create new persistent vector store
    ///
    /// # Arguments
    ///
    /// * `database_url` - PostgreSQL connection string
    /// * `max_connections` - Maximum number of database connections (default: 20)
    ///
    /// # Example
    ///
    /// ```no_run
    /// use neoland::storage::PersistentVectorStore;
    ///
    /// # async fn example() -> anyhow::Result<()> {
    /// let store = PersistentVectorStore::new(
    ///     "postgresql://user:pass@localhost/neoland",
    ///     Some(20)
    /// ).await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn new(database_url: &str, max_connections: Option<u32>) -> Result<Self> {
        tracing::info!("Initializing persistent vector store");

        // Create connection pool
        let pool = PgPoolOptions::new()
            .max_connections(max_connections.unwrap_or(20))
            .connect(database_url)
            .await
            .context("Failed to connect to PostgreSQL")?;

        tracing::info!("Connected to PostgreSQL");

        // Initialize database schema
        Self::init_schema(&pool).await?;

        // Load embedding model
        let model = EmbeddingModel::new().context("Failed to load embedding model")?;

        tracing::info!("Persistent vector store initialized successfully");

        Ok(Self { pool, model })
    }

    /// Initialize database schema with pgvector extension
    async fn init_schema(pool: &PgPool) -> Result<()> {
        tracing::info!("Initializing database schema");

        // Enable pgvector extension
        sqlx::query("CREATE EXTENSION IF NOT EXISTS vector")
            .execute(pool)
            .await
            .context("Failed to create pgvector extension")?;

        tracing::debug!("pgvector extension enabled");

        // Create documents table
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS documents (
                id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
                content TEXT NOT NULL,
                metadata TEXT NOT NULL,
                embedding vector(384) NOT NULL,
                created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
                updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
            )
            "#,
        )
        .execute(pool)
        .await
        .context("Failed to create documents table")?;

        tracing::debug!("Documents table created");

        // Create index for similarity search using HNSW (Hierarchical Navigable Small World)
        // This provides O(log n) search complexity vs O(n) for brute force
        sqlx::query(
            r#"
            CREATE INDEX IF NOT EXISTS documents_embedding_idx
            ON documents
            USING hnsw (embedding vector_cosine_ops)
            WITH (m = 16, ef_construction = 64)
            "#,
        )
        .execute(pool)
        .await
        .context("Failed to create HNSW index")?;

        tracing::debug!("HNSW index created for fast similarity search");

        // Create index for metadata filtering
        sqlx::query(
            r#"
            CREATE INDEX IF NOT EXISTS documents_metadata_idx
            ON documents
            USING gin (to_tsvector('english', metadata))
            "#,
        )
        .execute(pool)
        .await
        .context("Failed to create metadata index")?;

        tracing::info!("Database schema initialized successfully");

        Ok(())
    }

    /// Add a document to the vector store
    ///
    /// # Arguments
    ///
    /// * `content` - Document content to embed and store
    /// * `metadata` - Associated metadata (e.g., source, category, tags)
    ///
    /// # Returns
    ///
    /// UUID of the created document
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use neoland::storage::PersistentVectorStore;
    /// # async fn example(store: &PersistentVectorStore) -> anyhow::Result<()> {
    /// let doc_id = store.add_document(
    ///     "How to move a window in Hyprland?",
    ///     "source:manual,category:window-management"
    /// ).await?;
    /// println!("Document added with ID: {}", doc_id);
    /// # Ok(())
    /// # }
    /// ```
    pub async fn add_document(&self, content: &str, metadata: &str) -> Result<Uuid> {
        tracing::debug!("Adding document: {} chars, metadata: {}", content.len(), metadata);

        // Generate embedding
        let embedding_tensor = self.model.embed(content).context("Failed to generate embedding")?;

        // Convert Candle tensor to Vec<f32>
        let embedding_data = embedding_tensor
            .to_vec1::<f32>()
            .context("Failed to convert embedding to vec")?;

        // Create pgvector::Vector
        let embedding = Vector::from(embedding_data);

        // Insert into database
        let doc_id: Uuid = sqlx::query_scalar(
            r#"
            INSERT INTO documents (content, metadata, embedding)
            VALUES ($1, $2, $3)
            RETURNING id
            "#,
        )
        .bind(content)
        .bind(metadata)
        .bind(&embedding)
        .fetch_one(&self.pool)
        .await
        .context("Failed to insert document")?;

        tracing::info!("Document added successfully: {}", doc_id);

        Ok(doc_id)
    }

    /// Search for similar documents using cosine similarity
    ///
    /// # Arguments
    ///
    /// * `query` - Query text to search for
    /// * `top_k` - Number of results to return
    /// * `metadata_filter` - Optional metadata filter (e.g., "source:manual")
    ///
    /// # Returns
    ///
    /// Vector of (document, similarity_score) tuples, sorted by relevance
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use neoland::storage::PersistentVectorStore;
    /// # async fn example(store: &PersistentVectorStore) -> anyhow::Result<()> {
    /// let results = store.search(
    ///     "window movement shortcuts",
    ///     5,
    ///     Some("category:window-management")
    /// ).await?;
    ///
    /// for (doc, score) in results {
    ///     println!("Score: {:.3} - {}", score, doc.content);
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub async fn search(
        &self,
        query: &str,
        top_k: usize,
        metadata_filter: Option<&str>,
    ) -> Result<Vec<(StoredDocument, f32)>> {
        tracing::debug!(
            "Searching: query='{}', top_k={}, filter={:?}",
            query,
            top_k,
            metadata_filter
        );

        // Generate query embedding
        let query_tensor = self.model.embed(query).context("Failed to generate query embedding")?;

        let query_data = query_tensor
            .to_vec1::<f32>()
            .context("Failed to convert query embedding to vec")?;

        let query_embedding = Vector::from(query_data);

        // Execute similarity search with optional metadata filter
        let results = if let Some(filter) = metadata_filter {
            sqlx::query(
                r#"
                SELECT
                    id,
                    content,
                    metadata,
                    embedding,
                    created_at,
                    updated_at,
                    1 - (embedding <=> $1) as similarity
                FROM documents
                WHERE metadata ILIKE $2
                ORDER BY embedding <=> $1
                LIMIT $3
                "#,
            )
            .bind(&query_embedding)
            .bind(format!("%{}%", filter))
            .bind(top_k as i64)
            .fetch_all(&self.pool)
            .await
            .context("Failed to execute similarity search with filter")?
        } else {
            sqlx::query(
                r#"
                SELECT
                    id,
                    content,
                    metadata,
                    embedding,
                    created_at,
                    updated_at,
                    1 - (embedding <=> $1) as similarity
                FROM documents
                ORDER BY embedding <=> $1
                LIMIT $2
                "#,
            )
            .bind(&query_embedding)
            .bind(top_k as i64)
            .fetch_all(&self.pool)
            .await
            .context("Failed to execute similarity search")?
        };

        // Map results to (StoredDocument, similarity_score)
        let documents: Vec<(StoredDocument, f32)> = results
            .into_iter()
            .map(|row| {
                let doc = StoredDocument {
                    id: row.get("id"),
                    content: row.get("content"),
                    metadata: row.get("metadata"),
                    embedding: row.get("embedding"),
                    created_at: row.get("created_at"),
                    updated_at: row.get("updated_at"),
                };
                let similarity: f32 = row.get("similarity");
                (doc, similarity)
            })
            .collect();

        tracing::debug!("Found {} results", documents.len());

        Ok(documents)
    }

    /// Get a document by ID
    pub async fn get_document(&self, id: Uuid) -> Result<Option<StoredDocument>> {
        let doc = sqlx::query_as::<_, StoredDocument>(
            r#"
            SELECT id, content, metadata, embedding, created_at, updated_at
            FROM documents
            WHERE id = $1
            "#,
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await
        .context("Failed to fetch document")?;

        Ok(doc)
    }

    /// Delete a document by ID
    pub async fn delete_document(&self, id: Uuid) -> Result<bool> {
        let result = sqlx::query("DELETE FROM documents WHERE id = $1")
            .bind(id)
            .execute(&self.pool)
            .await
            .context("Failed to delete document")?;

        Ok(result.rows_affected() > 0)
    }

    /// Delete all documents matching a metadata pattern
    ///
    /// Useful for GDPR "right to be forgotten" compliance
    pub async fn delete_by_metadata(&self, metadata_pattern: &str) -> Result<u64> {
        let result = sqlx::query("DELETE FROM documents WHERE metadata ILIKE $1")
            .bind(format!("%{}%", metadata_pattern))
            .execute(&self.pool)
            .await
            .context("Failed to delete documents by metadata")?;

        tracing::info!(
            "Deleted {} documents matching '{}'",
            result.rows_affected(),
            metadata_pattern
        );

        Ok(result.rows_affected())
    }

    /// Get total number of documents
    pub async fn count(&self) -> Result<i64> {
        let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM documents")
            .fetch_one(&self.pool)
            .await
            .context("Failed to count documents")?;

        Ok(count)
    }

    /// Get database statistics
    pub async fn get_stats(&self) -> Result<VectorStoreStats> {
        let stats: VectorStoreStats = sqlx::query_as(
            r#"
            SELECT
                COUNT(*) as total_documents,
                pg_size_pretty(pg_total_relation_size('documents')) as table_size,
                pg_size_pretty(pg_indexes_size('documents')) as index_size,
                (
                    SELECT COUNT(*)
                    FROM pg_stat_user_indexes
                    WHERE indexrelname = 'documents_embedding_idx'
                ) > 0 as has_hnsw_index
            FROM documents
            "#,
        )
        .fetch_one(&self.pool)
        .await
        .context("Failed to fetch statistics")?;

        Ok(stats)
    }

    /// Close the connection pool gracefully
    pub async fn close(self) {
        tracing::info!("Closing persistent vector store");
        self.pool.close().await;
    }
}

/// Vector store statistics
#[derive(Debug, sqlx::FromRow)]
pub struct VectorStoreStats {
    pub total_documents: i64,
    pub table_size: String,
    pub index_size: String,
    pub has_hnsw_index: i64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    #[ignore] // Requires PostgreSQL with pgvector extension
    async fn test_persistent_vector_store() {
        let database_url = std::env::var("DATABASE_URL").unwrap_or_else(|_| {
            "postgresql://postgres:postgres@localhost/neoland_test".to_string()
        });

        let store = PersistentVectorStore::new(&database_url, Some(5))
            .await
            .expect("Failed to create persistent vector store");

        // Add documents
        let doc1_id = store
            .add_document("How to move a window?", "source:manual,category:window")
            .await
            .expect("Failed to add document 1");

        let doc2_id = store
            .add_document("How to change workspace?", "source:manual,category:workspace")
            .await
            .expect("Failed to add document 2");

        // Search
        let results = store.search("window movement", 2, None).await.expect("Search failed");

        assert!(!results.is_empty());
        assert!(results[0].0.content.contains("move a window"));
        assert!(results[0].1 > 0.5); // High similarity

        // Get document
        let doc = store
            .get_document(doc1_id)
            .await
            .expect("Failed to get document")
            .expect("Document not found");

        assert_eq!(doc.content, "How to move a window?");

        // Delete document
        let deleted = store.delete_document(doc2_id).await.expect("Failed to delete document");

        assert!(deleted);

        // Get stats
        let stats = store.get_stats().await.expect("Failed to get stats");
        assert!(stats.total_documents > 0);

        // Cleanup
        let _ = store.delete_document(doc1_id).await;
        store.close().await;
    }

    #[tokio::test]
    #[ignore]
    async fn test_metadata_filtering() {
        let database_url = std::env::var("DATABASE_URL").unwrap_or_else(|_| {
            "postgresql://postgres:postgres@localhost/neoland_test".to_string()
        });

        let store = PersistentVectorStore::new(&database_url, Some(5))
            .await
            .expect("Failed to create store");

        // Add documents with different metadata
        let _doc1 = store
            .add_document("Window operations", "category:window,source:manual")
            .await
            .unwrap();

        let _doc2 = store
            .add_document("Workspace switching", "category:workspace,source:manual")
            .await
            .unwrap();

        // Search with filter
        let results = store.search("operations", 10, Some("category:window")).await.unwrap();

        assert_eq!(results.len(), 1);
        assert!(results[0].0.content.contains("Window"));

        // Cleanup
        let _ = store.delete_by_metadata("source:manual").await;
        store.close().await;
    }
}
