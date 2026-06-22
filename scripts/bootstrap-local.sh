#!/usr/bin/env bash
# All-inclusive local bootstrap for the Neoland SecureLLM workflow.

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
LOG_DIR="$PROJECT_ROOT/.validate-logs/bootstrap"
TIMESTAMP="$(date +%Y%m%d-%H%M%S)"
LOG="$LOG_DIR/bootstrap-$TIMESTAMP.log"
RUN_CHECKS="${NEOLAND_BOOTSTRAP_CHECKS:-quick}"
RUN_SMOKE="${NEOLAND_BOOTSTRAP_SMOKE:-1}"

mkdir -p "$LOG_DIR"

usage() {
    cat <<'USAGE'
Usage: scripts/bootstrap-local.sh [command]

Commands:
  run       Prepare symlink, start local stack, run quick checks and smoke
  full      Same as run, with full Rust checks and long task smoke
  start     Prepare symlink and start local stack only
  stop      Stop local stack
  status    Show local stack status

Environment:
  NEOLAND_BOOTSTRAP_CHECKS=quick|full|none   default: quick
  NEOLAND_BOOTSTRAP_SMOKE=1|0                default: 1
USAGE
}

log() {
    echo "$*" | tee -a "$LOG"
}

section() {
    log ""
    log "── $*"
}

run_step() {
    local label="$1"
    shift

    section "$label"
    if "$@" 2>&1 | tee -a "$LOG"; then
        log "[OK] $label"
    else
        local exit_code=$?
        log "[FAIL] $label (exit $exit_code)"
        return "$exit_code"
    fi
}

require_tool() {
    local tool="$1"
    if ! command -v "$tool" >/dev/null 2>&1; then
        log "[FAIL] Required tool not found: $tool"
        return 1
    fi
}

preflight_tools() {
    section "Tooling"
    require_tool docker
    require_tool cargo
    require_tool poetry
    require_tool curl
    require_tool just
    log "[OK] Required local tools are available"
}

quick_checks() {
    run_step "Rust format check" cargo fmt -- --check
    run_step "Rust lib check" cargo check --lib
    run_step "TUI tests" cargo test --lib tui::
    run_step "Health tests" cargo test --lib health::tests
}

full_checks() {
    quick_checks
    run_step "REST API integration tests" cargo test --test rest_api_test
    run_step "Rust library tests" cargo test --lib
}

start_stack() {
    run_step "SecureLLM symlink" bash "$SCRIPT_DIR/securellm-bridge-docker.sh" link
    run_step "Local stack start" bash "$SCRIPT_DIR/neoland-stack.sh" start
    run_step "Local stack status" bash "$SCRIPT_DIR/neoland-stack.sh" status
}

run_bootstrap() {
    log "Neoland local bootstrap — $TIMESTAMP"
    log "Log: $LOG"

    preflight_tools
    start_stack

    case "$RUN_CHECKS" in
        quick) quick_checks ;;
        full) full_checks ;;
        none) log "Skipping checks because NEOLAND_BOOTSTRAP_CHECKS=none" ;;
        *)
            log "[FAIL] Invalid NEOLAND_BOOTSTRAP_CHECKS=$RUN_CHECKS"
            return 2
            ;;
    esac

    if [[ "$RUN_SMOKE" == "1" ]]; then
        run_step "SecureLLM smoke" bash "$SCRIPT_DIR/smoke-full-stack.sh"
    else
        log "Skipping smoke because NEOLAND_BOOTSTRAP_SMOKE=0"
    fi

    section "Ready"
    log "REST:    http://127.0.0.1:3001"
    log "gRPC:    127.0.0.1:50051"
    log "Agents:  http://127.0.0.1:8001"
    log "Bridge:  http://127.0.0.1:8080"
    log "Logs:    $PROJECT_ROOT/.validate-logs"
}

case "${1:-run}" in
    run) run_bootstrap ;;
    full)
        RUN_CHECKS=full
        export NEOLAND_SMOKE_RUN_TASK=1
        run_bootstrap
        ;;
    start)
        log "Neoland local bootstrap start — $TIMESTAMP"
        preflight_tools
        start_stack
        ;;
    stop) bash "$SCRIPT_DIR/neoland-stack.sh" stop ;;
    status) bash "$SCRIPT_DIR/neoland-stack.sh" status ;;
    -h|--help|help) usage ;;
    *)
        usage >&2
        exit 2
        ;;
esac
