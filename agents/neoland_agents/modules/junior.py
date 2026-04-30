import dspy
from ..signatures.task_proposal import TaskProposal
from ..tools import call_rust_tool

class JuniorAgent(dspy.Module):
    """Criativo, sem filtro, gerador de hipóteses. Carta coringa do pipeline."""

    def __init__(self) -> None:
        super().__init__()
        
        # We define the tool IN the closure to capture the runtime session_id dynamically.
        # This solves the problem of static python tool registries in DSPy.
        pass

    def forward(self, task: str, context: str, session_id: str) -> dspy.Prediction:
        
        def run_shell_command(command: str) -> str:
            """Executes a bash command safely via the Rust Host. Use this to read files (cat/ls), explore the system, or modify files."""
            return call_rust_tool(session_id, "run_shell_command", {"command": command})
        
        propose = dspy.ReAct(TaskProposal, tools=[run_shell_command])
        
        # Add constraints at runtime
        propose.assert_constraints = [
            dspy.Assert(lambda x: 0.0 <= float(x.confidence) <= 1.0, "confidence must be float between 0.0 and 1.0"),
            dspy.Assert(lambda x: x.risk_level in ("low", "medium", "high"), "risk_level must be low, medium, high"),
        ]

        return propose(task=task, context=context)
