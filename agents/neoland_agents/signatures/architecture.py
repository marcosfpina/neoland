import dspy


class ArchitectReview(dspy.Signature):
    """Architect: soundness estrutural e composabilidade de longo prazo."""

    task: str = dspy.InputField(desc="Tarefa original")
    refined_hypothesis: str = dspy.InputField(desc="Hipótese refinada pelo Senior")
    risk_assessment: str = dspy.InputField(desc="Avaliação de risco do Senior")

    structural_soundness: bool = dspy.OutputField(
        desc="A proposta é estruturalmente sólida e pode ser implementada?"
    )
    composability_score: float = dspy.OutputField(
        desc="Score de composabilidade de 0.0 a 1.0"
    )
    long_term_concerns: list[str] = dspy.OutputField(
        desc="Preocupações de longo prazo não bloqueantes mas importantes"
    )
    recommended_structure: str = dspy.OutputField(
        desc="Estrutura de implementação recomendada (módulos, camadas, contratos)"
    )
    blockers: list[str] = dspy.OutputField(
        desc="Blockers concretos que impedem execução. Lista vazia se nenhum."
    )
