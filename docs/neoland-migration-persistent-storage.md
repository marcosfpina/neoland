# Migration Guide: In-Memory to Persistent Vector Store

**Phase**: 4.7 - Performance Optimization
**Date**: 2026-01-31
**Impact**: Data Persistence, Production Readiness

---

## Overview

This guide covers the migration from the in-memory `VectorStore` to the persistent `PersistentVectorStore` backed by PostgreSQL with pgvector extension.

### Why Migrate?

**Current State (In-Memory)**:
- ❌ Data lost on server restart
- ❌ Limited by RAM capacity
- ❌ Single-server only (no horizontal scaling)
- ❌ No backup/recovery
- ✅ Fast (no network overhead)

**Target State (PostgreSQL + pgvector)**:
- ✅ Data persists across restarts
- ✅ Scales to millions of documents
- ✅ Horizontal scaling ready
- ✅ Backup/recovery with PostgreSQL tools
- ✅ Fast with HNSW indexing (O(log n) search)
- ✅ ACID guarantees

---

## Prerequisites

### 1. PostgreSQL 15+ Installation

```bash
# NixOS (recommended)
nix-shell -p postgresql_15

# Or add to configuration.nix
{
  services.postgresql = {
    enable = true;
    package = pkgs.postgresql_15;
    enableTCPIP = true;
    authentication = ''
      host neoland neoland 127.0.0.1/32 md5
    '';
  };
}

# Ubuntu/Debian
sudo apt update
sudo apt install postgresql-15 postgresql-contrib-15

# macOS (Homebrew)
brew install postgresql@15

# Start PostgreSQL
sudo systemctl start postgresql
# or
pg_ctl -D /var/lib/postgresql/data -l logfile start
```

### 2. pgvector Extension Installation

```bash
# NixOS
nix-shell -p pgvector

# Ubuntu/Debian
sudo apt install postgresql-15-pgvector

# From source (if not available in repos)
git clone https://github.com/pgvector/pgvector.git
cd pgvector
make
sudo make install
```

### 3. Create Database and User

```sql
-- Connect as postgres user
sudo -u postgres psql

-- Create database and user
CREATE DATABASE neoland;
CREATE USER neoland WITH ENCRYPTED PASSWORD 'your_secure_password';
GRANT ALL PRIVILEGES ON DATABASE neoland TO neoland;

-- Connect to neoland database
\c neoland

-- Enable pgvector extension
CREATE EXTENSION IF NOT EXISTS vector;

-- Grant permissions
GRANT ALL ON SCHEMA public TO neoland;

-- Verify installation
SELECT * FROM pg_extension WHERE extname = 'vector';
```

---

## Migration Steps

### Step 1: Update Dependencies

Already added to `Cargo.toml`:

```toml
# Persistent Storage (Phase 4.7)
sqlx = { version = "0.8", features = ["runtime-tokio-rustls", "postgres", "uuid", "chrono"] }
pgvector = { version = "0.4", features = ["sqlx"] }
```

Build with new dependencies:

```bash
cargo build --release
```

### Step 2: Set Database URL

```bash
# Production
export DATABASE_URL="postgresql://neoland:password@localhost/neoland"

# Development
export DATABASE_URL="postgresql://neoland:password@localhost/neoland_dev"

# Or in .env file
echo "DATABASE_URL=postgresql://neoland:password@localhost/neoland" > .env
```

### Step 3: Update Code

#### Before (In-Memory VectorStore)

```rust
use neoland::nlp::VectorStore;

// Create in-memory store
let mut store = VectorStore::new()?;

// Add documents (lost on restart)
store.add_document("How to move a window?", "manual:move")?;

// Search
let results = store.search("window movement", 5)?;
```

#### After (Persistent VectorStore)

```rust
use neoland::storage::PersistentVectorStore;

// Create persistent store (async)
let store = PersistentVectorStore::new(
    &std::env::var("DATABASE_URL")?,
    Some(20) // max connections
).await?;

// Add documents (persisted to PostgreSQL)
let doc_id = store.add_document(
    "How to move a window?",
    "source:manual,category:window"
).await?;

// Search with optional metadata filter
let results = store.search(
    "window movement",
    5,
    Some("category:window") // optional filter
).await?;

// Close gracefully (optional, handled by Drop)
store.close().await;
```

### Step 4: Data Migration (If Existing Data)

If you have existing data in the in-memory store that needs to be migrated:

```rust
use neoland::nlp::VectorStore;
use neoland::storage::PersistentVectorStore;

async fn migrate_data() -> Result<()> {
    // Load old in-memory store
    let old_store = VectorStore::new()?;

    // Create new persistent store
    let new_store = PersistentVectorStore::new(
        &std::env::var("DATABASE_URL")?,
        Some(20)
    ).await?;

    // Migrate documents (if you have a way to iterate them)
    // Note: In-memory VectorStore doesn't expose document iteration
    // This would require adding a method like:
    // for doc in old_store.get_all_documents() {
    //     new_store.add_document(&doc.content, &doc.metadata).await?;
    // }

    tracing::info!("Migration complete");
    Ok(())
}
```

**Note**: The current `VectorStore` doesn't expose a method to iterate all documents. If you need to migrate existing data, you'll need to:

1. Re-add documents from your original data source, OR
2. Add a `get_all_documents()` method to the old `VectorStore`

### Step 5: Update Server Code

Update `src/server/mod.rs` or wherever VectorStore is initialized:

```rust
// Replace VectorStore initialization
// let vector_store = Arc::new(Mutex::new(VectorStore::new()?));

// With PersistentVectorStore
let database_url = std::env::var("DATABASE_URL")
    .context("DATABASE_URL environment variable not set")?;

let vector_store = PersistentVectorStore::new(&database_url, Some(20))
    .await
    .context("Failed to initialize persistent vector store")?;

let vector_store = Arc::new(vector_store); // No Mutex needed, already thread-safe
```

### Step 6: Schema Initialization

The schema is automatically initialized on first connection:

```sql
-- Created automatically by PersistentVectorStore::init_schema()

CREATE EXTENSION IF NOT EXISTS vector;

CREATE TABLE IF NOT EXISTS documents (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    content TEXT NOT NULL,
    metadata TEXT NOT NULL,
    embedding vector(384) NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS documents_embedding_idx
ON documents
USING hnsw (embedding vector_cosine_ops)
WITH (m = 16, ef_construction = 64);

CREATE INDEX IF NOT EXISTS documents_metadata_idx
ON documents
USING gin (to_tsvector('english', metadata));
```

### Step 7: Test the Migration

```bash
# Run tests
cargo test --test persistent_vector_store_test

# Or manually test
cargo run --bin neoland -- server

# In another terminal
curl -X POST http://localhost:3001/v1/chat/completions \
  -H "Content-Type: application/json" \
  -H "X-API-Key: ${API_KEY}" \
  -d '{
    "messages": [
      {"role": "user", "content": "test vector store"}
    ]
  }'
```

---

## Performance Tuning

### HNSW Index Parameters

The HNSW index is created with default parameters:

```sql
CREATE INDEX documents_embedding_idx
ON documents
USING hnsw (embedding vector_cosine_ops)
WITH (m = 16, ef_construction = 64);
```

**Tuning**:

| Parameter | Default | Description | Impact |
|-----------|---------|-------------|--------|
| `m` | 16 | Max connections per node | Higher = better recall, more memory |
| `ef_construction` | 64 | Dynamic candidate list size | Higher = better index quality, slower build |

For production with millions of documents:

```sql
-- Drop existing index
DROP INDEX IF EXISTS documents_embedding_idx;

-- Create optimized index
CREATE INDEX documents_embedding_idx
ON documents
USING hnsw (embedding vector_cosine_ops)
WITH (m = 32, ef_construction = 128);
```

### Connection Pool Sizing

```rust
let store = PersistentVectorStore::new(
    &database_url,
    Some(50) // Increase for high concurrency
).await?;
```

**Guidelines**:
- Development: 5-10 connections
- Production (single server): 20-50 connections
- Production (multiple servers): `connections_per_server * num_servers <= postgres_max_connections`

### PostgreSQL Configuration

Add to `postgresql.conf`:

```ini
# Memory
shared_buffers = 4GB              # 25% of RAM
effective_cache_size = 12GB       # 75% of RAM
work_mem = 50MB                   # Per connection

# Connections
max_connections = 200

# Performance
random_page_cost = 1.1            # For SSD
effective_io_concurrency = 200

# Vacuum (for pgvector)
autovacuum = on
autovacuum_max_workers = 4
```

### Query Performance

Monitor slow queries:

```sql
-- Enable query logging
ALTER DATABASE neoland SET log_min_duration_statement = 1000; -- 1 second

-- View slow queries
SELECT query, mean_exec_time, calls
FROM pg_stat_statements
WHERE query LIKE '%documents%'
ORDER BY mean_exec_time DESC
LIMIT 10;
```

---

## Backup and Recovery

### Backup Vector Store

```bash
# Backup entire database
pg_dump -U neoland -d neoland -F c -f neoland_backup.dump

# Backup only documents table
pg_dump -U neoland -d neoland -t documents -F c -f documents_backup.dump

# Backup with pgvector extension
pg_dump -U neoland -d neoland --create --clean -F c -f neoland_full.dump
```

### Restore Vector Store

```bash
# Restore entire database
pg_restore -U neoland -d neoland -c neoland_backup.dump

# Restore specific table
pg_restore -U neoland -d neoland -t documents documents_backup.dump
```

### Automated Backups

Integrated with NEOLAND backup scripts (`scripts/backup/backup.sh`):

```bash
# Run daily backup (includes vector store)
./scripts/backup/backup.sh

# Restore from backup
./scripts/backup/restore.sh --backup-path /var/backups/neoland/20260131_120000
```

---

## Monitoring

### Key Metrics

```sql
-- Total documents
SELECT COUNT(*) FROM documents;

-- Table size
SELECT pg_size_pretty(pg_total_relation_size('documents')) as total_size;

-- Index size
SELECT pg_size_pretty(pg_indexes_size('documents')) as index_size;

-- Index usage
SELECT
    indexrelname,
    idx_scan,
    idx_tup_read,
    idx_tup_fetch
FROM pg_stat_user_indexes
WHERE tablename = 'documents';
```

### Prometheus Metrics

Add to your metrics module:

```rust
use prometheus::{IntGauge, register_int_gauge};

lazy_static! {
    static ref VECTOR_STORE_DOCUMENTS: IntGauge =
        register_int_gauge!(
            "neoland_vector_store_documents_total",
            "Total number of documents in vector store"
        ).unwrap();

    static ref VECTOR_STORE_SIZE_BYTES: IntGauge =
        register_int_gauge!(
            "neoland_vector_store_size_bytes",
            "Total size of vector store in bytes"
        ).unwrap();
}

// Update metrics periodically
let count = store.count().await?;
VECTOR_STORE_DOCUMENTS.set(count);
```

---

## Troubleshooting

### Issue: "extension vector does not exist"

**Solution**:

```sql
-- Connect as superuser
sudo -u postgres psql

-- Install extension
CREATE EXTENSION IF NOT EXISTS vector;
```

### Issue: "HNSW index build is too slow"

**Solution**:

```sql
-- Build index with lower parameters initially
CREATE INDEX documents_embedding_idx
ON documents
USING hnsw (embedding vector_cosine_ops)
WITH (m = 8, ef_construction = 32);

-- Rebuild with higher parameters later (off-peak hours)
REINDEX INDEX CONCURRENTLY documents_embedding_idx;
```

### Issue: "Too many connections"

**Solution**:

```bash
# Reduce connection pool size
let store = PersistentVectorStore::new(&database_url, Some(10)).await?;

# Or increase PostgreSQL max_connections
# Edit postgresql.conf:
# max_connections = 300
sudo systemctl restart postgresql
```

### Issue: "Search is slow"

**Check**:

1. Index exists: `\d documents`
2. Index is being used: `EXPLAIN ANALYZE SELECT ...`
3. Table statistics are up to date: `ANALYZE documents;`
4. Increase `ef_search` at query time (not yet implemented)

---

## Rollback Plan

If migration fails, rollback to in-memory store:

1. **Stop NEOLAND server**

2. **Revert code changes**:
   ```bash
   git revert <migration_commit>
   ```

3. **Rebuild**:
   ```bash
   cargo build --release
   ```

4. **Restart server**:
   ```bash
   cargo run --release --bin neoland -- server
   ```

**Data Loss**: In-memory store will be empty after restart. Re-add documents from source.

---

## Testing

### Unit Tests

```bash
# Run persistent vector store tests
DATABASE_URL=postgresql://postgres:postgres@localhost/neoland_test \
  cargo test --lib storage::vector_store::tests -- --ignored
```

### Integration Tests

```bash
# Run full integration tests
DATABASE_URL=postgresql://postgres:postgres@localhost/neoland_test \
  cargo test --test grpc_integration_test
```

### Load Test

```bash
# Benchmark search performance
DATABASE_URL=postgresql://postgres:postgres@localhost/neoland \
  cargo bench --bench inference_benchmark
```

---

## Production Checklist

- [ ] PostgreSQL 15+ installed with pgvector extension
- [ ] Database created with appropriate user permissions
- [ ] DATABASE_URL environment variable set
- [ ] Connection pool sized appropriately (20-50 connections)
- [ ] HNSW index created with optimized parameters
- [ ] Backup strategy configured (daily automated backups)
- [ ] Monitoring metrics exposed (/metrics endpoint)
- [ ] Slow query logging enabled
- [ ] Load testing completed (verify performance targets)
- [ ] Disaster recovery tested (backup/restore validation)

---

## Performance Targets

| Metric | Target | Measurement |
|--------|--------|-------------|
| **Insert Latency** | < 100ms | p99 for single document |
| **Search Latency** | < 200ms | p99 for top-10 search |
| **Throughput** | 500 RPS | Sustained load |
| **Storage** | 1M+ documents | Tested capacity |
| **Index Build** | < 30 min | For 1M documents |

---

## References

- [pgvector Documentation](https://github.com/pgvector/pgvector)
- [HNSW Algorithm](https://arxiv.org/abs/1603.09320)
- [PostgreSQL Performance Tuning](https://wiki.postgresql.org/wiki/Performance_Optimization)
- [sqlx Documentation](https://docs.rs/sqlx/)
- [NEOLAND Disaster Recovery Plan](./disaster_recovery.md)

---

## Support

For migration issues:
- Check logs: `tail -f /var/log/neoland/server.log`
- Test connection: `psql $DATABASE_URL -c "SELECT version();"`
- Contact: infrastructure@company.com
