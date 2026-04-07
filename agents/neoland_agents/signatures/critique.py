import dspy


class SeniorCritique(dspy.Signature):
    """Senior ZeroTrust: ceticismo estruturado. Corta o que não sobrevive ao escrutínio."""

    task: str = dspy.InputField(desc="Tarefa original")
    proposal: str = dspy.InputField(desc="Output do Junior serializado em JSON")

    valid_parts: list[str] = dspy.OutputField(desc="Partes que sobrevivem ao escrutínio")
    rejected_parts: list[str] = dspy.OutputField(
        desc="Partes rejeitadas com motivo explícito de rejeição"
    )
    risk_assessment: str = dspy.OutputField(
        desc="Avaliação de risco real (não percebido). O que pode falhar e por quê."
    )
    escalate_to_architect: bool = dspy.OutputField(
        desc="True se a proposta exige validação de soundness estrutural pelo Architect"
    )
    refined_hypothesis: str = dspy.OutputField(
        desc="Hipótese refinada após ceticismo. Mais precisa e menos arriscada."
    )
