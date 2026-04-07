import dspy
from ..signatures.critique import SeniorCritique


class SeniorAgent(dspy.Module):
    """Cético. Refina e corta. Avalia risco real, não percebido."""

    def __init__(self) -> None:
        super().__init__()
        self.critique = dspy.ChainOfThought(SeniorCritique)

    def forward(self, task: str, proposal: str) -> dspy.Prediction:
        return self.critique(task=task, proposal=proposal)
