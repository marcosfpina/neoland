from typing import Literal
import dspy


class TechLeaderDecision(dspy.Signature):
    """Tech-Leader: decisão final + ADR auto-gerado para checkpoint entre sessões."""

    task: str = dspy.InputField(desc="Tarefa original")
    all_inputs: str = dspy.InputField(
        desc="JSON com outputs de todos os agentes anteriores (junior, senior, architect)"
    )

    decision: Literal["approve", "reject", "defer", "escalate"] = dspy.OutputField(
        desc="Decisão: approve=implementar, reject=descartar, defer=revisitar depois, escalate=precisa de input externo"
    )
    rationale: str = dspy.OutputField(desc="Justificativa clara e objetiva da decisão")
    action_items: list[str] = dspy.OutputField(
        desc="Próximos passos concretos e acionáveis"
    )
    adr_title: str = dspy.OutputField(
        desc="Título do ADR gerado, formato: 'ADR-NNN: Título Descritivo'"
    )
    session_summary: str = dspy.OutputField(
        desc="Resumo conciso para contexto da próxima sessão. O que foi decidido e por quê."
    )
