# Neoland v0.1.0-rc.1

**First public release candidate — TUI + CLI + API.**

> **Historical draft:** this file predates the canonical release gate. Claims
> below describe a partial validation snapshot and must not be read as current
> release evidence. See [`ROADMAP.md`](ROADMAP.md).

Partial validation recorded on 2026-06-02:

| Gate | Result |
|------|--------|
| Rust unit tests (226) | ✅ |
| Clippy (0 warnings) | ✅ |
| E2E REST tests (22/22) | ✅ |
| Python contracts (26/26) | ✅ |
| Doctor (`ok: true`) | ✅ |
| Full-stack smoke (9/9 layers) | Historical claim; current real-LLM smoke remains pending |

## What's in this release

### Control plane (Rust)
- REST API (15 endpoints) + gRPC with full auth middleware (RBAC, rate limiting, audit)
- Four-stage DSPy agent orchestration: Junior → Senior → Architect → TechLeader
- Live steering and breakpoint resolution via SSE + human-in-the-loop API
- Session management with PostgreSQL persistence and checkpoint relay
- `neoland doctor --json` reports each runtime layer independently

### Agent pipeline (Python · DSPy 3.x)
- 26 contract tests covering all Pydantic schemas and agent signatures — no LLM required
- Checkpoint artifacts persisted to `$NEOLAND_CHECKPOINT_DIR`
- FastAPI health endpoint integrated into doctor

### TUI (Rust · ratatui)
- 3-column layout (Sessions / Conversation / Reasoning)
- 4 themes (Tokyo Night, Neon Glass, High Contrast, Monochrome)
- Chat bubbles + code blocks + streaming SSE subscriber
- Pipeline tree with confidence badges and RWA anchors
- Command system (`/help`, `/provider`, `/theme`, `/search`, etc.)

### Infrastructure
- Nix-first dev shell: single `nix develop` wires DATABASE_URL, all NEOLAND_* env vars, SOPS secrets
- SecureLLM Bridge, ml-ops-api, and llama.cpp Docker configurations aligned and validated
- GPU passthrough via CDI (`nvidia.com/gpu=0`) for ml-ops-api VRAM-aware routing
- `just smoke` and `just preflight` as canonical release validation commands

### Security
- SOPS-encrypted secrets in `secrets/neoland.sops.env`
- HashiCorp Vault integration with env fallback chain
- Structured audit logging, brute-force detection, input sanitization

## Historical next slice (formerly called v0.2.0 Honest Preview)

- [ ] Leptos WASM Web Console connected to backend (REST + SSE streaming)
- [ ] Build WASM in Nix flake (`nix build .#neoland-web`)
- [ ] `cargo test --workspace` includes Web crate
- [ ] README with real screenshots (TUI + Web Console)

## Performance (measured)

| Endpoint | RPS | p99 |
|----------|-----|-----|
| `GET /live` | 37,600 | 12 ms |
| `GET /health` | 258 | 309 ms (IO-bound — polls all layers) |

## Known limitations

- **pgvector**: vector store runs in-memory without `CREATE EXTENSION vector`
- **LLM API key**: full pipeline execution requires `OPENAI_API_KEY` (or equivalent litellm provider key) — smoke validates routing, not model responses
- **SecureLLM Bridge Redis**: caching disabled without Redis; core proxy functionality unaffected
- **CPU inference**: Candle/Qwen fallback is a dev path (~5-10 tok/s); production throughput via llama.cpp/vLLM behind ml-ops-api
- **Non-Nix install path**: Ubuntu bare metal setup is documented but not yet validated end-to-end

## Upgrade notes

This is the first tagged release. No migration required.

Set `NEOLAND_CHECKPOINT_DIR` if the default (`~/.local/share/neoland/checkpoints/adr`) is not suitable for your environment.

Use `just server` (not `./target/release/neoland server` directly) to ensure SOPS secrets and `DATABASE_URL` are loaded from the dev shell.

---

Full changelog: [`ROADMAP.md`](ROADMAP.md)
