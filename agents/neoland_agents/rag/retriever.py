"""RAG retriever: busca documentos relevantes no pgvector."""
from __future__ import annotations

import asyncpg


class RAGRetriever:
    """Consulta o vector store PostgreSQL+pgvector do Neoland."""

    def __init__(self, db_url: str, top_k: int = 5, max_chars_per_doc: int = 500) -> None:
        self._db_url = db_url
        self.top_k = top_k
        self.max_chars_per_doc = max_chars_per_doc
        self._pool: asyncpg.Pool | None = None

    async def _get_pool(self) -> asyncpg.Pool:
        if self._pool is None:
            self._pool = await asyncpg.create_pool(self._db_url, min_size=1, max_size=3)
        return self._pool

    async def retrieve(self, query: str) -> str:
        """Retorna contexto formatado para injeção no TaskRequest."""
        try:
            pool = await self._get_pool()
            async with pool.acquire() as conn:
                rows = await conn.fetch(
                    """
                    SELECT content FROM documents
                    ORDER BY embedding <=> $1::vector
                    LIMIT $2
                    """,
                    query,
                    self.top_k,
                )
            docs = [row["content"][: self.max_chars_per_doc] for row in rows]
            return "\n\n---\n\n".join(docs) if docs else ""
        except Exception:
            return ""  # RAG falha gracefully — não bloqueia o pipeline

    async def close(self) -> None:
        if self._pool:
            await self._pool.close()
