import dspy
from ..signatures.decision import TechLeaderDecision


class TechLeaderAgent(dspy.Module):
    """Decisão final. Fecha o pipeline e gera o checkpoint ADR."""

    def __init__(self) -> None:
        super().__init__()
        self.decide = dspy.ChainOfThought(TechLeaderDecision)

    def forward(self, task: str, all_inputs: str) -> dspy.Prediction:
        return self.decide(task=task, all_inputs=all_inputs)
