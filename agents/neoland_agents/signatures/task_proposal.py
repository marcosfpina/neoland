from typing import Literal
import dspy


class TaskProposal(dspy.Signature):
    """Junior: gerador criativo de hipóteses. Sem filtro de risco — auto-consciência via unknowns."""

    task: str = dspy.InputField(desc="Tarefa a resolver")
    context: str = dspy.InputField(desc="Contexto RAG de documentos e sessões anteriores")

    hypothesis: str = dspy.OutputField(desc="Hipótese de solução sem autocensura")
    confidence: float = dspy.OutputField(desc="Confiança subjetiva de 0.0 a 1.0")
    risk_level: Literal["low", "medium", "high"] = dspy.OutputField(
        desc="Nível de risco percebido"
    )
    unknowns: list[str] = dspy.OutputField(
        desc="O que o agente não sabe mas pode importar para a decisão"
    )
    innovation_vectors: list[str] = dspy.OutputField(
        desc="Vetores de inovação ou abordagens não convencionais identificados"
    )
