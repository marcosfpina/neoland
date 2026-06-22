#!/usr/bin/env bash
# Local operational stack: SecureLLM Bridge + DSPy agents + Neoland server.

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
LOG_DIR="$PROJECT_ROOT/.validate-logs/runtime"
SERVER_PID_FILE="$LOG_DIR/neoland-server.pid"
AGENTS_PID_FILE="$LOG_DIR/neoland-agents.pid"

mkdir -p "$LOG_DIR"

usage() {
    cat <<'USAGE'
Usage: scripts/neoland-stack.sh <command>

Commands:
  start   Start SecureLLM Bridge, agents pipeline, and Neoland server
  stop    Stop agents pipeline, Neoland server, and SecureLLM Bridge
  status  Probe all local stack endpoints
USAGE
}

is_url_ready() {
    curl -sf --max-time 3 "$1" >/dev/null 2>&1
}

wait_url() {
    local name="$1"
    local url="$2"

    for _ in {1..60}; do
        if is_url_ready "$url"; then
            echo "$name ready at $url"
            return 0
        fi
        sleep 1
    done

    echo "$name did not become ready at $url" >&2
    return 1
}

pid_alive() {
    local pid_file="$1"
    [[ -f "$pid_file" ]] && kill -0 "$(cat "$pid_file")" 2>/dev/null
}

start_agents() {
    if is_url_ready "http://127.0.0.1:8001/health"; then
        echo "Agents pipeline already ready at http://127.0.0.1:8001/health"
        return
    fi

    if pid_alive "$AGENTS_PID_FILE"; then
        echo "Agents pipeline process already running with PID $(cat "$AGENTS_PID_FILE")"
        wait_url "Agents pipeline" "http://127.0.0.1:8001/health"
        return
    fi

    nohup setsid bash "$SCRIPT_DIR/agents-run.sh" start >"$LOG_DIR/agents.log" 2>&1 </dev/null &
    local pid="$!"
    disown "$pid" 2>/dev/null || true
    echo "$pid" >"$AGENTS_PID_FILE"
    wait_url "Agents pipeline" "http://127.0.0.1:8001/health"
}

start_server() {
    if is_url_ready "http://127.0.0.1:3001/live"; then
        echo "Neoland server already ready at http://127.0.0.1:3001/live"
        return
    fi

    if pid_alive "$SERVER_PID_FILE"; then
        echo "Neoland server process already running with PID $(cat "$SERVER_PID_FILE")"
        wait_url "Neoland server" "http://127.0.0.1:3001/live"
        return
    fi

    nohup setsid bash "$SCRIPT_DIR/neoland-run.sh" server >"$LOG_DIR/server.log" 2>&1 </dev/null &
    local pid="$!"
    disown "$pid" 2>/dev/null || true
    echo "$pid" >"$SERVER_PID_FILE"
    wait_url "Neoland server" "http://127.0.0.1:3001/live"
}

stop_pid_file() {
    local label="$1"
    local pid_file="$2"

    if ! [[ -f "$pid_file" ]]; then
        echo "$label PID file not found"
        return
    fi

    local pid
    pid="$(cat "$pid_file")"
    if kill -0 "$pid" 2>/dev/null; then
        kill "$pid" 2>/dev/null || true
        for _ in {1..10}; do
            if ! kill -0 "$pid" 2>/dev/null; then
                break
            fi
            sleep 0.5
        done
        if kill -0 "$pid" 2>/dev/null; then
            kill -9 "$pid" 2>/dev/null || true
            echo "Force-stopped $label PID $pid"
        fi
        echo "Stopped $label PID $pid"
    else
        echo "$label PID $pid is not running"
    fi
    rm -f "$pid_file"
}

stop_matching_processes() {
    local label="$1"
    shift
    local patterns=("$@")
    local found=0

    for pattern in "${patterns[@]}"; do
        while IFS= read -r pid; do
            [[ -z "$pid" ]] && continue
            [[ "$pid" == "$$" ]] && continue
            if kill -0 "$pid" 2>/dev/null; then
                kill "$pid" 2>/dev/null || true
                for _ in {1..10}; do
                    if ! kill -0 "$pid" 2>/dev/null; then
                        break
                    fi
                    sleep 0.5
                done
                if kill -0 "$pid" 2>/dev/null; then
                    kill -9 "$pid" 2>/dev/null || true
                    echo "Force-stopped $label matching '$pattern' PID $pid"
                fi
                echo "Stopped $label matching '$pattern' PID $pid"
                found=1
            fi
        done < <(pgrep -f "$pattern" 2>/dev/null || true)
    done

    if [[ "$found" -eq 0 ]]; then
        echo "No unmanaged $label processes found"
    fi
}

stop_port_listeners() {
    local label="$1"
    local port="$2"
    local found=0
    local pids

    pids="$(ss -ltnp "sport = :$port" 2>/dev/null | grep -oE 'pid=[0-9]+' | cut -d= -f2 | sort -u || true)"
    while IFS= read -r pid; do
        [[ -z "$pid" ]] && continue
        if kill -0 "$pid" 2>/dev/null; then
            kill "$pid" 2>/dev/null || true
            for _ in {1..10}; do
                if ! kill -0 "$pid" 2>/dev/null; then
                    break
                fi
                sleep 0.5
            done
            if kill -0 "$pid" 2>/dev/null; then
                kill -9 "$pid" 2>/dev/null || true
                echo "Force-stopped $label listener on :$port PID $pid"
            else
                echo "Stopped $label listener on :$port PID $pid"
            fi
            found=1
        fi
    done <<<"$pids"

    if [[ "$found" -eq 0 ]]; then
        echo "No $label listeners found on :$port"
    fi
}

start_stack() {
    bash "$SCRIPT_DIR/securellm-bridge-docker.sh" start
    start_agents
    start_server
    echo "Stack logs: $LOG_DIR"
}

stop_stack() {
    stop_pid_file "Neoland server" "$SERVER_PID_FILE"
    stop_matching_processes "Neoland server" \
        "target/debug/neoland server" \
        "cargo run --manifest-path .*/Cargo.toml --bin neoland -- server"
    stop_pid_file "Agents pipeline" "$AGENTS_PID_FILE"
    stop_matching_processes "Agents pipeline" "uvicorn neoland_agents.app:app"
    stop_port_listeners "Agents pipeline" 8001
    bash "$SCRIPT_DIR/securellm-bridge-docker.sh" stop
}

status_stack() {
    bash "$SCRIPT_DIR/securellm-bridge-docker.sh" status
    local checks=(
        "Neoland live|http://127.0.0.1:3001/live"
        "Neoland health|http://127.0.0.1:3001/health"
        "Agents health|http://127.0.0.1:8001/health"
        "Bridge health|http://127.0.0.1:8080/api/health"
    )

    for item in "${checks[@]}"; do
        local label="${item%%|*}"
        local url="${item#*|}"
        if is_url_ready "$url"; then
            echo "[OK]   $label"
        else
            echo "[MISS] $label"
        fi
    done
}

case "${1:-}" in
    start) start_stack ;;
    stop) stop_stack ;;
    status) status_stack ;;
    -h|--help|help|"") usage ;;
    *)
        usage >&2
        exit 2
        ;;
esac
