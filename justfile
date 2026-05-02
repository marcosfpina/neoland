# ─── Neoland Justfile — Developer Experience ──────────────────────────────
# Run `just` to list all commands
# Requires: `just` and Nix dev shell or Rust toolchain.
# Most commands work inside `nix develop` or with `direnv allow`.

set positional-arguments := true

# ─── Check & Lint ─────────────────────────────────────────────────────────

# Validate compilation (lib only, fast)
check:
    cargo check --lib

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

# Library unit tests
test:
    cargo test --lib

# All tests (lib + integration + bins)
test-all:
    cargo test

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

# Start gRPC + REST server
server:
    cargo run -- server

# Launch TUI client
client:
    cargo run -- client

# Environment diagnostics (JSON)
doctor:
    cargo run -- doctor --json

# Start DSPy agent pipeline (Python/FastAPI)
agents-start:
    cd agents && poetry run uvicorn neoland_agents.app:app --reload --port 8001

# ─── Frontend ─────────────────────────────────────────────────────────────

# Start Next.js dev server
frontend-dev:
    nix develop --command frontend-dev

# Production frontend build
frontend-build:
    nix develop --command frontend-build

# Lint frontend
frontend-lint:
    nix develop --command frontend-lint

# Full stack health check
frontend-stack:
    nix develop --command frontend-stack

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

# ─── Clean ────────────────────────────────────────────────────────────────

# Remove cargo artifacts
clean:
    cargo clean

# Remove all artifacts + Python venv
clean-all: clean
    rm -rf agents/.venv
