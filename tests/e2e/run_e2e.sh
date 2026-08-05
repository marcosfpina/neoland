#!/usr/bin/env bash
# Strict functional E2E for the real Neoland TUI.

set -euo pipefail

E2E_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$E2E_DIR/../.." && pwd)"
REST_PORT="${NEOLAND_E2E_REST_PORT:-3101}"
GRPC_PORT="${NEOLAND_E2E_GRPC_PORT:-51051}"
SERVER_URL="http://127.0.0.1:${REST_PORT}"
GRPC_URL="http://127.0.0.1:${GRPC_PORT}"
GATEWAY_URL="${NEOLAND_E2E_GATEWAY_URL:-http://127.0.0.1:9}"
API_KEY="neoland_admin_dev_key_change_in_production"
TMP_ROOT="$(mktemp -d -t neoland-tui-e2e.XXXXXX)"
SERVER_PID=""

cleanup() {
    if [[ -n "$SERVER_PID" ]] && kill -0 "$SERVER_PID" 2>/dev/null; then
        kill "$SERVER_PID" 2>/dev/null || true
        wait "$SERVER_PID" 2>/dev/null || true
    fi
    rm -rf "$TMP_ROOT"
}
trap cleanup EXIT INT TERM

for tool in cargo curl expect; do
    command -v "$tool" >/dev/null || {
        echo "TUI E2E requires '$tool'" >&2
        exit 1
    }
done

cd "$PROJECT_ROOT"
cargo build --bin neoland

env -u DATABASE_URL -u NEOLAND_DATABASE_URL \
    AUDIT_LOG_PATH="$TMP_ROOT/audit.log" \
    XDG_CONFIG_HOME="$TMP_ROOT/server-config" \
    NEOLAND_SKIP_EMBEDDINGS=true \
    NEOLAND_NATS_ENABLED=false \
    NEOLAND_DSPY_URL=http://127.0.0.1:9 \
    RUST_LOG=error \
    target/debug/neoland server \
        --rest-port "$REST_PORT" \
        --grpc-port "$GRPC_PORT" \
        --web-dist web/dist \
        >"$TMP_ROOT/server.log" 2>&1 &
SERVER_PID=$!

for _ in $(seq 1 80); do
    if curl --fail --silent "$SERVER_URL/live" >/dev/null; then
        break
    fi
    if ! kill -0 "$SERVER_PID" 2>/dev/null; then
        echo "TUI E2E server exited during startup" >&2
        sed -n '1,240p' "$TMP_ROOT/server.log" >&2
        exit 1
    fi
    sleep 0.25
done
curl --fail --silent "$SERVER_URL/live" >/dev/null || {
    echo "TUI E2E server did not become live" >&2
    sed -n '1,240p' "$TMP_ROOT/server.log" >&2
    exit 1
}

run_tui() {
    local mode="$1"
    local config_dir="$TMP_ROOT/client-$mode"
    mkdir -p "$config_dir"
    expect "$E2E_DIR/tui_functional_test.exp" \
        "$PROJECT_ROOT/target/debug/neoland" \
        "$SERVER_URL" "$GRPC_URL" "$GATEWAY_URL" "$config_dir" "$mode" "$API_KEY"
}

# Missing-key behavior and authenticated degraded behavior are separate runs.
# Running a third time catches state/restart regressions in prefs and sessions.
run_tui missing-key
run_tui authenticated
run_tui restart

echo "TUI functional E2E: 3/3 passed"
