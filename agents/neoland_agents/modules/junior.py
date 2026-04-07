import dspy
from ..signatures.task_proposal import TaskProposal


class JuniorAgent(dspy.Module):
    """Criativo, sem filtro, gerador de hipóteses. Carta coringa do pipeline."""

    def __init__(self) -> None:
        super().__init__()
        self.propose = dspy.ChainOfThought(TaskProposal)
        # Força tipos corretos — evita que o LLM retorne confidence como string
        self.propose.assert_constraints = [
            dspy.Assert(
                lambda x: 0.0 <= float(x.confidence) <= 1.0,
                "confidence must be a float between 0.0 and 1.0",
            ),
            dspy.Assert(
                lambda x: x.risk_level in ("low", "medium", "high"),
                "risk_level must be one of: low, medium, high",
            ),
        ]

    def forward(self, task: str, context: str) -> dspy.Prediction:
        return self.propose(task=task, context=context)
