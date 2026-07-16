# Neoland Quick Start Guide

**Last Updated**: 2026-07-16 · **Version**: v0.2.0 Honest Preview

Get up and running with Neoland in under 5 minutes.

---

## Prerequisites

- 2GB free RAM (for local inference)
- Terminal emulator (for TUI) or modern browser (for Web Console)
- One of:
  - **Nix / NixOS** (recommended)
  - Ubuntu or comparable bare metal environment

---

## Quick Start (Nix)

```bash
git clone <this-repo> && cd neoland
nix develop

# TUI: server + client together
just dev

# Web Console: server + browser SPA on http://localhost:8080
just dev-web
```

That's it. Two commands. Under 5 minutes.

---

## Installation

### Option 1: Nix Dev Shell (Recommended)

```bash
cd neoland
nix develop

# Aliases set by the dev shell:
nsrv          # neoland server  (:3001 REST, :50051 gRPC)
ncli          # neoland client  (TUI)
nd            # neoland doctor  (diagnostics)
nt            # cargo test --lib (unit tests)

# Or use just recipes:
just server   # start server
just tui      # TUI client
just dev-web  # server + Web Console on :8080
just dev      # server + TUI
just doctor   # environment diagnostics
```

### Option 2: Docker Compose (Full Stack)

Sobe PostgreSQL (pgvector), NATS, DSPy pipeline e control plane.

```bash
cd deploy/docker/
cp .env.example .env && $EDITOR .env   # set LLM_API_KEY

eval $(ssh-agent) && ssh-add ~/.ssh/id_ed25519
DOCKER_BUILDKIT=1 docker compose build --ssh default
docker compose up -d

curl http://localhost:3001/health | jq .
```

### Option 3: Build Release Binary

```bash
nix develop --command cargo build --bin neoland --release
# Binary: ./target/release/neoland
```

---

## Basic Usage

### 1. Start the Server

```bash
# Default: gRPC :50051, REST :3001, Web Console: web/dist/
neoland server

# Custom ports + Web Console path
neoland server --grpc-port 50052 --rest-port 3002 --web-dist /var/www/neoland

# Or via env var
NEOLAND_WEB_DIST_DIR=/opt/neoland/web neoland server
```

### 2. Launch TUI Client

```bash
neoland client
neoland client --neoland-gateway-url http://gpu-server:8080
```

### 3. Open Web Console

```bash
# Dev mode: single command (Trunk proxy handles CORS)
just dev-web
# → open http://localhost:8080

# Production: build WASM, serve via Neoland
just build-web              # → web/dist/
neoland server --web-dist web/dist
# → open http://localhost:3001
```

---

## Web Console

The Web Console is a Leptos WASM SPA with 3 panels:

```
┌─ Topbar ──────────────────────────────────────────────────────────┐
│  [☰ Sessions]  NEOLAND://CORE  [⚙ Reasoning]                      │
├─ Workspace ───────────────────────────────────────────────────────┤
│ ┌ Sessions ────────┐ ┌ Conversation ───────────────────────┐      │
│ │ [+ New Chat]     │ │                                    │      │
│ │ ● Session 1  5m  │ │  󰚩 Neoland                         │      │
│ │ ● Session 2  1h  │ │  ╭────────────────────────────────╮│      │
│ └──────────────────┘ │  │ Response streaming via SSE...   ││      │
│                      │  ╰────────────────────────────────╯│      │
│                      └────────────────────────────────────┘      │
└──────────────────────────────────────────────────────────────────┘
```

- **Sessions panel**: list, create, and switch between sessions
- **Conversation**: chat messages with live SSE streaming from agent pipeline
- **Reasoning panel**: pipeline tree with confidence badges and ADR status

### Development

```bash
# Build WASM
just build-web         # trunk build --release → web/dist/

# Check + test
just check-wasm        # cargo check --target wasm32-unknown-unknown
just clippy-wasm       # cargo clippy for WASM target
just test-web          # 16 unit tests
```

---

## Key Commands

| Command | Description |
|---------|-------------|
| `just dev` | Server + TUI (single terminal) |
| `just dev-web` | Server + Web Console on :8080 |
| `just server` | Server only |
| `just tui` | TUI client only |
| `just ci` | Full CI: check → fmt → clippy → test (core + web) |
| `just test-web` | Web Console unit tests (16) |
| `just build-web` | Build WASM bundle |
| `just doctor` | Environment diagnostics |
| `just smoke` | Full-stack smoke test |

---

## Testing

```bash
# All tests
cargo test --workspace          # 275 core + 16 web

# Web Console only
cargo test -p neoland-web       # 16 tests

# Python contracts (no LLM needed)
cd agents && poetry run pytest tests/ -m contract -v

# WASM compilation
cargo check -p neoland-web --target wasm32-unknown-unknown
```

---

## TUI Interface

```
│  󰽥 Neoland  │ 󰤣  │ 󰍉 local  │ 󰔎 0ms  󰭹 Conversation              󰀄  󰢻  │
│─────────────────────────────────────────────────────────────────────│
│╭ 󰙯 Sessions ──────╮╭ 󰭹 Conversation ────────────────╮╭ 󰒝 Reasoning ──╮│
││  ✚ New Chat  ^n   ││  󰚩 Neoland                     ││ Pipeline Tree ││
││  󰅂 ⠋ Active Task   ││  ╭────────────────────────────╮││  ├─ Jr [92%]  ││
││    ● Past Session  ││  │ Response from agent...      │││  ├─ Sr [88%]  ││
│╰───────────────────╯│  ╰────────────────────────────╯││  ╰─ ADR ✓     ││
│                      ╰────────────────────────────────╯╰──────────────╯│
│  ╭──────────────────────────────────────────────────────────────────╮  │
│  │ L local │  󰢱 balanced │ Enter enviar  ·  Tab painéis  ·  ? help  │  │
│  ╰──────────────────────────────────────────────────────────────────╯  │
```

### Keyboard Shortcuts

| Key | Action |
|-----|--------|
| **Enter** | Send message / Steer |
| **Ctrl+C** / **Esc** | Quit |
| **Tab** | Switch panels (Sessions → Conversation → Composer) |
| **Ctrl+1-5** | Preset profiles |
| **/help** | Show all commands |
| **/theme** | Cycle themes (4 themes) |
| **/search** | Search sessions |
| **/why** | Show agent reasoning |

---

## Runtime Stack

```
┌──────────┐  ┌──────────┐
│  Browser │  │ Terminal │
│  Leptos  │  │  ratatui │
└────┬─────┘  └────┬─────┘
     │ REST :3001   │
     │ SSE (EventSource)
     │ gRPC-web :50051
     ▼              ▼
┌─────────────────────────┐
│   Neoland Control Plane │
│   (Rust · axum · tonic) │
└───────────┬─────────────┘
            │
    ┌───────┴───────┐
    │ SecureLLM     │
    │ Bridge :8080  │
    └───────┬───────┘
            │
    ┌───────┴───────┐
    │ ml-ops-api    │
    │ :8083         │
    └───────┬───────┘
            │
    ┌───────┴───────┐
    │ llama.cpp     │
    │ :8081 / vLLM  │
    └───────────────┘
```

---

## Troubleshooting

### Web Console blank page

```bash
# Ensure WASM is built
just build-web
ls web/dist/   # should contain index.html + .wasm + .js

# Check the path
neoland server --web-dist web/dist
```

### Connection refused

```bash
pgrep -f "neoland server"
lsof -i :3001
```

### Model download issues

```bash
rm -rf ~/.cache/huggingface/hub/models--Qwen*
neoland server
```

---

## Next Steps

- [`README.md`](../README.md) — full project overview
- [`ROADMAP.md`](../ROADMAP.md) — release roadmap
- [`docs/neoland-architecture.md`](neoland-architecture.md) — architecture deep-dive
- [`docs/neoland-project-snapshot.md`](neoland-project-snapshot.md) — codebase inventory
- [`docs/ADR/`](ADR/) — architectural decisions
