# ─── Neoland Justfile — Developer Experience ──────────────────────────────
# Run `just` to list all commands.
# Requires: `just` inside `nix develop` (or `direnv allow`).
#
# ⚠️  SOPS Secrets:
#   server / client / doctor load secrets from secrets/neoland.sops.env.
#   -raw variants skip SOPS — set env vars manually if the age key is absent:
#     export NEOLAND_ADMIN_API_KEY="..." NEOLAND_USER_API_KEY="..."
#
# Short aliases available in devShell: ncli, nsrv, nd, nt

set positional-arguments

# ─── Quick shortcuts (most used) ──────────────────────────────────────────

# TUI client — alias for `client` (loads SOPS)
tui: client

# Dev stack: sobe servidor, aguarda /health, abre TUI. Ctrl+C mata tudo.
dev:
    #!/usr/bin/env bash
    set -euo pipefail
    trap 'kill $(jobs -p) 2>/dev/null; wait 2>/dev/null || true' EXIT INT TERM
    echo "→ Iniciando servidor Neoland..."
    bash scripts/neoland-run.sh server &
    SERVER_PID=$!
    echo "→ Aguardando servidor em :3001..."
    until curl -sf http://localhost:3001/health >/dev/null 2>&1; do
        sleep 0.3
        if ! kill -0 "$SERVER_PID" 2>/dev/null; then
            echo "✗ Servidor encerrou antes de ficar ready."
            exit 1
        fi
    done
    echo "✓ Servidor pronto. Abrindo TUI..."
    bash scripts/neoland-run.sh client

# Dev stack Web: sobe servidor + Web Console (Trunk proxy). Single port :8080.
dev-web:
    #!/usr/bin/env bash
    set -euo pipefail
    trap 'kill $(jobs -p) 2>/dev/null; wait 2>/dev/null || true' EXIT INT TERM
    echo "→ Iniciando servidor Neoland em :3001..."
    bash scripts/neoland-run.sh server &
    SERVER_PID=$!
    echo "→ Aguardando /health..."
    until curl -sf http://localhost:3001/health >/dev/null 2>&1; do
        sleep 0.3
        if ! kill -0 "$SERVER_PID" 2>/dev/null; then
            echo "✗ Servidor encerrou antes de ficar ready."
            exit 1
        fi
    done
    echo "✓ Servidor pronto."
    echo "→ Iniciando Web Console em http://localhost:8080..."
    cd web && trunk serve

# Servidor standalone — cliente roda em outro terminal (ncli ou just tui)
serve: server

# Bootstrap all-inclusive: symlink + Docker bridge + agents + server + checks + smoke
bootstrap:
    bash scripts/bootstrap-local.sh run

# Bootstrap full validation, including long end-to-end task
bootstrap-full:
    bash scripts/bootstrap-local.sh full

# Start local stack only
up:
    bash scripts/bootstrap-local.sh start

# Stop local stack
down:
    bash scripts/bootstrap-local.sh stop

# CI pipeline: check → fmt-check → clippy → test (all must pass)
ci:
    cargo check --lib
    cargo fmt --check
    cargo clippy --all-targets -- -D warnings
    cargo test --lib
    cargo test -p neoland-web

# Auth-specific CI: compile check + auth tests
ci-auth:
    cargo check --lib
    cargo test --lib auth::

# Auto-fix: format + clippy suggestions applied in-place
fix:
    cargo fmt
    cargo clippy --fix --allow-staged --all-targets 2>/dev/null || true

# Watch mode: re-check lib on every save (requires cargo-watch in devShell)
watch:
    cargo watch -x 'check --lib'

# ─── TUI development ──────────────────────────────────────────────────────

# ASCII preview of TUI layout states (no running server needed)
visual:
    cargo test --lib tui::ui::tests::dump_visual -- --ignored --nocapture

# TUI unit tests only (fast, no secrets)
test-tui:
    cargo test --lib tui::

# ─── Check & Lint ─────────────────────────────────────────────────────────

# Validate compilation (lib only, fast)
check:
    cargo check --lib

# Check WASM compilation (Web Console)
check-wasm:
    nix develop --command cargo check -p neoland-web --target wasm32-unknown-unknown

# Clippy for WASM target
clippy-wasm:
    nix develop --command cargo clippy -p neoland-web --target wasm32-unknown-unknown -- -D warnings

# Build Web Console WASM (release)
build-web:
    nix develop --command trunk build --release
    @echo "✓ web/dist/ ready"

# Lint with clippy (deny all warnings)
clippy:
    cargo clippy --all-targets -- -D warnings

# Format all Rust code
fmt:
    cargo fmt

# Check formatting without modifying
fmt-check:
    cargo fmt -- --check

# ─── Test ─────────────────────────────────────────────────────────────────

# Library unit tests (no secrets needed)
test:
    cargo test --lib

# Unit tests with SOPS secrets loaded
test-with-secrets:
    bash scripts/neoland-run.sh test -- --lib

# Web Console unit tests (no WASM runtime needed)
test-web:
    cargo test -p neoland-web

# Web Console WASM integration tests (requires nix develop for WASM target + headless browser)
test-web-wasm:
    nix develop --command wasm-pack test --headless --chrome web/

# Web Console WASM integration tests (Firefox)
test-web-wasm-firefox:
    nix develop --command wasm-pack test --headless --firefox web/

# ─── Desktop ──────────────────────────────────────────────────────────────

# Build desktop app (requires Tauri system deps — see desktop/README.md)
desktop-build:
    cd desktop && cargo tauri build

# Dev mode: hot-reload desktop app with Trunk proxy
desktop-dev:
    cd desktop && cargo tauri dev

# Check desktop Rust code compiles
desktop-check:
    cd desktop/src-tauri && cargo check

# All tests (lib + integration + bins)
test-all:
    cargo test

# All tests with SOPS secrets loaded
test-all-with-secrets:
    bash scripts/neoland-run.sh test

# CLI-specific tests
test-cli:
    cargo test cli::tests

# Agent pipeline contract tests
test-agents-contract:
    cd agents && poetry run pytest tests/ -m contract -v

# Agent pipeline integration tests (requires LLM_API_KEY)
test-agents-integration:
    cd agents && poetry run pytest tests/ -m integration -v

# ─── Build ────────────────────────────────────────────────────────────────

# Release binary
build:
    cargo build --release

# Debug binary
build-debug:
    cargo build

# ─── Run ──────────────────────────────────────────────────────────────────

# Start gRPC + REST server (loads SOPS secrets automatically)
server:
    bash scripts/neoland-run.sh server

# Cargo direct (no SOPS — set env vars manually first)
server-raw:
    cargo run -- server

# Launch TUI client (loads SOPS secrets automatically)
client:
    bash scripts/neoland-run.sh client

# Cargo direct (no SOPS — set env vars manually first)
client-raw:
    cargo run -- client

# Environment diagnostics with SOPS secrets loaded
doctor:
    bash scripts/neoland-run.sh doctor --json

# Cargo direct (no SOPS)
doctor-raw:
    cargo run -- doctor --json

# Start DSPy agent pipeline (Python/FastAPI)
agents-start:
    bash scripts/agents-run.sh start

# Create/repair local securellm-bridge symlink
bridge-link:
    bash scripts/securellm-bridge-docker.sh link

# Build SecureLLM Bridge Docker image
bridge-build:
    bash scripts/securellm-bridge-docker.sh build

# Start SecureLLM Bridge Docker container
bridge-start:
    bash scripts/securellm-bridge-docker.sh start

# Stop SecureLLM Bridge Docker container
bridge-stop:
    bash scripts/securellm-bridge-docker.sh stop

# Show SecureLLM Bridge Docker status
bridge-status:
    bash scripts/securellm-bridge-docker.sh status

# Probe SecureLLM Bridge health endpoint
bridge-health:
    bash scripts/securellm-bridge-docker.sh health

# Show recent SecureLLM Bridge logs
bridge-logs:
    bash scripts/securellm-bridge-docker.sh logs

# Start local operational stack: bridge + agents + server
stack-start:
    bash scripts/neoland-stack.sh start

# Stop local operational stack
stack-stop:
    bash scripts/neoland-stack.sh stop

# Show local operational stack status
stack-status:
    bash scripts/neoland-stack.sh status

# ─── Secrets ──────────────────────────────────────────────────────────────

# Edit encrypted secrets via SOPS
secrets-edit:
    sops secrets/neoland.sops.env

# View decrypted secrets
secrets-view:
    sops -d secrets/neoland.sops.env

# ─── DX Setup ─────────────────────────────────────────────────────────────

# Install git hooks
setup-hooks:
    bash scripts/setup-hooks.sh

# Enable direnv (one-time)
setup-direnv:
    echo "use flake" > .envrc

# Full setup: hooks + direnv
setup-all: setup-hooks setup-direnv

# ─── Validation ───────────────────────────────────────────────────────────

# Production readiness validation suite
validate:
    bash scripts/validate-production-readiness.sh

# Dependency vulnerability scan
audit:
    cargo audit

# Show current roadmap and release status
roadmap:
    cat ROADMAP.md

# Full-stack smoke: Neoland → SecureLLM → agents; ml-ops is optional by default
smoke:
    bash scripts/smoke-full-stack.sh

# Capture screenshots of TUI and Web Console (requires running server + display)
screenshot:
    #!/usr/bin/env bash
    set -euo pipefail
    echo "→ Capturando TUI screenshot..."
    cargo test --lib tui::ui::tests::dump_visual -- --ignored --nocapture 2>&1 | tee /tmp/neoland-tui-dump.txt
    echo "✓ TUI dump salvo em /tmp/neoland-tui-dump.txt"
    echo ""
    echo "→ Para capturar o Web Console, abra http://localhost:8080 no browser"
    echo "  e use a ferramenta de screenshot do seu OS ou DevTools."
    echo "  (just dev-web precisa estar rodando em outro terminal)"

# Long smoke: includes POST /v1/agents/task through the full DSPy pipeline
smoke-task:
    bash scripts/smoke-full-stack.sh --task

# SLO validation: load test /live and /health against defined targets (requires hey)
validate-slo:
    bash scripts/validate-slo.sh

# Release preflight: all gates (tests, lint, build, smoke, doctor)
preflight:
    bash scripts/release-preflight.sh

# ─── Clean ────────────────────────────────────────────────────────────────

# Remove cargo artifacts
clean:
    cargo clean

# Remove all artifacts + Python venv
clean-all: clean
    rm -rf agents/.venv
