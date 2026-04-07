"""Configuração do pipeline via variáveis de ambiente."""
import os


class Settings:
    dspy_url: str = os.getenv("NEOLAND_DSPY_URL", "http://localhost:8001")
    db_url: str = os.getenv("DATABASE_URL", "postgresql://localhost/neoland")
    llm_provider: str = os.getenv("NEOLAND_LLM_PROVIDER", "openai")
    llm_model: str = os.getenv("NEOLAND_LLM_MODEL", "gpt-4o-mini")
    llm_api_key: str = os.getenv("LLM_API_KEY", "")
    checkpoint_dir: str = os.getenv("NEOLAND_CHECKPOINT_DIR", "/var/lib/neoland/checkpoints/adr")
    pipeline_port: int = int(os.getenv("NEOLAND_PIPELINE_PORT", "8001"))
    rag_top_k: int = int(os.getenv("NEOLAND_RAG_TOP_K", "5"))

    def dspy_lm(self):
        import dspy
        return dspy.LM(
            model=f"{self.llm_provider}/{self.llm_model}",
            api_key=self.llm_api_key or None,
        )


settings = Settings()
