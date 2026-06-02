"""Configuração do pipeline via variáveis de ambiente."""
import os


# Providers suportados pelo pipeline DSPy/LiteLLM.
# "local" usa ollama (sem key); os demais lêem {PROVIDER}_API_KEY.
_PROVIDER_DEFAULTS: dict[str, str] = {
    "deepseek": "deepseek-chat",
    "groq": "llama-3.1-8b-instant",
    "gemini": "gemini/gemini-1.5-flash",
    "openai": "gpt-4o-mini",
    "local": "ollama_chat/llama3.2",
}


def _resolve_api_key(provider: str) -> str:
    """Busca key na ordem: LLM_API_KEY → {PROVIDER}_API_KEY → vazio."""
    generic = os.getenv("LLM_API_KEY", "")
    if generic:
        return generic
    return os.getenv(f"{provider.upper()}_API_KEY", "")


class Settings:
    dspy_url: str = os.getenv("NEOLAND_DSPY_URL", "http://localhost:8001")
    db_url: str = os.getenv("DATABASE_URL", "postgresql://localhost/neoland")
    # Default: deepseek — funciona sem OPENAI_API_KEY.
    # Override via NEOLAND_LLM_PROVIDER=groq|gemini|openai|local
    llm_provider: str = os.getenv("NEOLAND_LLM_PROVIDER", "deepseek")
    llm_model: str = os.getenv(
        "NEOLAND_LLM_MODEL",
        _PROVIDER_DEFAULTS.get(os.getenv("NEOLAND_LLM_PROVIDER", "deepseek"), "deepseek-chat"),
    )
    checkpoint_dir: str = os.getenv("NEOLAND_CHECKPOINT_DIR", "/var/lib/neoland/checkpoints/adr")
    pipeline_port: int = int(os.getenv("NEOLAND_PIPELINE_PORT", "8001"))
    rag_top_k: int = int(os.getenv("NEOLAND_RAG_TOP_K", "5"))

    def dspy_lm(self):
        import dspy

        api_key = _resolve_api_key(self.llm_provider) or None

        if self.llm_provider == "local":
            # ollama local — sem key, base url configurável
            ollama_base = os.getenv("OLLAMA_BASE_URL", "http://localhost:11434")
            return dspy.LM(
                model=self.llm_model,
                api_base=ollama_base,
                api_key="ollama",  # litellm exige valor não-vazio
            )

        return dspy.LM(
            model=f"{self.llm_provider}/{self.llm_model}",
            api_key=api_key,
        )


settings = Settings()
