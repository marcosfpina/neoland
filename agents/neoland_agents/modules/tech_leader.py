import dspy
from ..signatures.decision import TechLeaderDecision


class TechLeaderAgent(dspy.Module):
    """Decisão final. Fecha o pipeline e gera o checkpoint ADR."""

    def __init__(self) -> None:
        super().__init__()
        self.decide = dspy.ChainOfThought(TechLeaderDecision)
        self.decide.assert_constraints = [
            dspy.Assert(
                lambda x: x.decision in ("approve", "reject", "defer", "escalate"),
                "decision must be one of: approve, reject, defer, escalate",
            ),
        ]

    def forward(self, task: str, all_inputs: str) -> dspy.Prediction:
        return self.decide(task=task, all_inputs=all_inputs)
