-- NEOLAND Vector Store Migration
-- Phase 4.7: Performance Optimization
-- Created: 2026-01-31
--
-- This migration creates the persistent vector store schema using PostgreSQL + pgvector

-- Enable pgvector extension
CREATE EXTENSION IF NOT EXISTS vector;

-- Create documents table with vector embeddings
CREATE TABLE IF NOT EXISTS documents (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    content TEXT NOT NULL,
    metadata TEXT NOT NULL,
    embedding vector(384) NOT NULL,  -- MiniLM-L6 produces 384-dimensional embeddings
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Create HNSW index for fast similarity search
-- HNSW (Hierarchical Navigable Small World) provides O(log n) search complexity
CREATE INDEX IF NOT EXISTS documents_embedding_idx
ON documents
USING hnsw (embedding vector_cosine_ops)
WITH (m = 16, ef_construction = 64);

-- Create GIN index for metadata full-text search
CREATE INDEX IF NOT EXISTS documents_metadata_idx
ON documents
USING gin (to_tsvector('english', metadata));

-- Create trigger to update updated_at timestamp
CREATE OR REPLACE FUNCTION update_updated_at_column()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = NOW();
    RETURN NEW;
END;
$$ language 'plpgsql';

CREATE TRIGGER update_documents_updated_at
    BEFORE UPDATE ON documents
    FOR EACH ROW
    EXECUTE FUNCTION update_updated_at_column();

-- Grant permissions (adjust user as needed)
-- GRANT ALL PRIVILEGES ON TABLE documents TO neoland;
-- GRANT USAGE, SELECT ON ALL SEQUENCES IN SCHEMA public TO neoland;

-- Verify installation
DO $$
BEGIN
    -- Check pgvector extension
    IF NOT EXISTS (
        SELECT 1 FROM pg_extension WHERE extname = 'vector'
    ) THEN
        RAISE EXCEPTION 'pgvector extension not installed';
    END IF;

    -- Check table creation
    IF NOT EXISTS (
        SELECT 1 FROM information_schema.tables
        WHERE table_name = 'documents'
    ) THEN
        RAISE EXCEPTION 'documents table not created';
    END IF;

    -- Check HNSW index
    IF NOT EXISTS (
        SELECT 1 FROM pg_indexes
        WHERE indexname = 'documents_embedding_idx'
    ) THEN
        RAISE EXCEPTION 'HNSW index not created';
    END IF;

    RAISE NOTICE 'Vector store migration completed successfully';
END $$;
