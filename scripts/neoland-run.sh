#!/usr/bin/env bash

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
DEFAULT_SOPS_ENV_FILE="$PROJECT_ROOT/secrets/neoland.sops.env"

load_sops_env() {
    local sops_env_file="$1"

    if ! command -v sops >/dev/null 2>&1; then
        echo "❌ SOPS is required to decrypt $sops_env_file" >&2
        echo "   Hint: enter \`nix develop\` or install \`sops\` locally." >&2
        exit 1
    fi

    echo "🔐 Loading secrets from $sops_env_file" >&2

    set -a
    # shellcheck disable=SC1090
    source <(sops decrypt --output-type dotenv "$sops_env_file")
    set +a
}

apply_runtime_defaults() {
    export NEOLAND_DSPY_URL="${NEOLAND_DSPY_URL:-http://127.0.0.1:8001}"
    export NEOLAND_GATEWAY_URL="${NEOLAND_GATEWAY_URL:-http://127.0.0.1:8080}"
    export NEOLAND_PIPELINE_TIMEOUT_SECS="${NEOLAND_PIPELINE_TIMEOUT_SECS:-300}"
}

main() {
    local sops_env_file="${NEOLAND_SOPS_ENV_FILE:-$DEFAULT_SOPS_ENV_FILE}"
    if [[ -f "$sops_env_file" ]]; then
        load_sops_env "$sops_env_file"
    fi

    apply_runtime_defaults

    exec cargo run --manifest-path "$PROJECT_ROOT/Cargo.toml" --bin neoland -- "$@"
}

main "$@"
