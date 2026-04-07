import dspy
from ..signatures.architecture import ArchitectReview


class ArchitectAgent(dspy.Module):
    """Soundness estrutural e composabilidade de longo prazo."""

    def __init__(self) -> None:
        super().__init__()
        self.review = dspy.ChainOfThought(ArchitectReview)
        self.review.assert_constraints = [
            dspy.Assert(
                lambda x: 0.0 <= float(x.composability_score) <= 1.0,
                "composability_score must be a float between 0.0 and 1.0",
            ),
        ]

    def forward(
        self, task: str, refined_hypothesis: str, risk_assessment: str
    ) -> dspy.Prediction:
        return self.review(
            task=task,
            refined_hypothesis=refined_hypothesis,
            risk_assessment=risk_assessment,
        )
