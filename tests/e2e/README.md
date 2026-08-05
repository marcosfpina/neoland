# Neoland TUI functional E2E

The production TUI gate drives the compiled `neoland client` inside a real
pseudo-terminal with `expect`. It starts an isolated control plane and runs the
client three times to cover:

- startup and restart with isolated persisted state;
- missing API key and authenticated degraded-pipeline behavior;
- `/help`, `/theme`, `/stream`, `/preset`, `/provider`, `/queue`, `/search`, and `/new`;
- visible error handling and clean Ctrl+C shutdown.

Run it from the repository root:

```bash
nix develop --command bash tests/e2e/run_e2e.sh
```

This deterministic suite does not claim that an external LLM worked. The
production preflight separately runs `scripts/smoke-full-stack.sh --task` with
PostgreSQL, DSPy, and real provider credentials. Both gates are required for a
release.
