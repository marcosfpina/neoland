# Agentic Breakpoints

This branch introduces Agentic Breakpoints, a feature that allows the system to halt an agent's execution right before it performs a critical action (like running a destructive shell command). It pauses the DSPy pipeline, shifts the TUI into an interactive `[Steer / Approve / Reject]` state, and waits for human operator input.
