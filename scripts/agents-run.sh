#!/usr/bin/env bash
# Runs the Python agent pipeline with Neoland defaults loaded in one place.

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
DEFAULT_SOPS_ENV_FILE="$PROJECT_ROOT/secrets/neoland.sops.env"

load_sops_env_if_available() {
    local sops_env_file="${NEOLAND_SOPS_ENV_FILE:-$DEFAULT_SOPS_ENV_FILE}"

    if [[ ! -f "$sops_env_file" ]]; then
        return
    fi

    if ! command -v sops >/dev/null 2>&1; then
        echo "SOPS file exists but sops is unavailable; continuing without decrypted secrets." >&2
        return
    fi

    set -a
    # shellcheck disable=SC1090
    source <(sops decrypt --output-type dotenv "$sops_env_file")
    set +a
}

set_agent_defaults() {
    export NEOLAND_DSPY_URL="${NEOLAND_DSPY_URL:-http://127.0.0.1:8001}"
    export NEOLAND_LLM_PROVIDER="${NEOLAND_LLM_PROVIDER:-securellm}"

    if [[ "$NEOLAND_LLM_PROVIDER" == "securellm" ]]; then
        export NEOLAND_LLM_MODEL="${NEOLAND_LLM_MODEL:-llamacpp/local-model}"
        export SECURELLM_BASE_URL="${SECURELLM_BASE_URL:-http://127.0.0.1:8080/v1}"
        export LLM_API_KEY="${LLM_API_KEY:-securellm-local}"
    fi
}

run_pipeline() {
    load_sops_env_if_available
    set_agent_defaults
    cd "$PROJECT_ROOT/agents"
    exec poetry run uvicorn neoland_agents.app:app \
        --host 127.0.0.1 \
        --port 8001 \
        --workers "${NEOLAND_PIPELINE_WORKERS:-2}"
}

case "${1:-start}" in
    start) run_pipeline ;;
    *)
        echo "Usage: scripts/agents-run.sh [start]" >&2
        exit 2
        ;;
esac
