import dspy
from ..signatures.architecture import ArchitectReview


class ArchitectAgent(dspy.Module):
    """Soundness estrutural e composabilidade de longo prazo."""

    def __init__(self) -> None:
        super().__init__()
        self.review = dspy.ChainOfThought(ArchitectReview)

    def forward(
        self, task: str, refined_hypothesis: str, risk_assessment: str
    ) -> dspy.Prediction:
        return self.review(
            task=task,
            refined_hypothesis=refined_hypothesis,
            risk_assessment=risk_assessment,
        )
