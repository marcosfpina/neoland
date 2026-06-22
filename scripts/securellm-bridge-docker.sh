#!/usr/bin/env bash
# Docker wrapper for the local securellm-bridge sibling checkout.

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
BRIDGE_LINK="$PROJECT_ROOT/securellm-bridge"
BRIDGE_TARGET="${SECURELLM_BRIDGE_TARGET:-$PROJECT_ROOT/../securellm-bridge}"
CONTAINER="${SECURELLM_BRIDGE_CONTAINER:-neoland-securellm-proxy}"
IMAGE="${SECURELLM_BRIDGE_IMAGE:-neoland-securellm-proxy:local}"
VOLUME="${SECURELLM_BRIDGE_VOLUME:-neoland-securellm-data}"
PORT="${SECURELLM_BRIDGE_PORT:-8080}"
NETWORK_MODE="${SECURELLM_BRIDGE_NETWORK:-host}"
RUNTIME_DIR="$PROJECT_ROOT/.validate-logs/runtime"
ENV_FILE="$RUNTIME_DIR/securellm-bridge.env"

usage() {
    cat <<'USAGE'
Usage: scripts/securellm-bridge-docker.sh <command>

Commands:
  link     Create/repair ./securellm-bridge symlink to ../securellm-bridge
  build    Build the Docker image used by Neoland
  start    Build if needed and start the Docker container
  stop     Stop the Docker container
  status   Show container status
  health   Probe /api/health
  logs     Show recent container logs
USAGE
}

ensure_link() {
    if [[ -L "$BRIDGE_LINK" && -d "$BRIDGE_LINK" ]]; then
        return
    fi

    if [[ -e "$BRIDGE_LINK" && ! -L "$BRIDGE_LINK" ]]; then
        echo "Refusing to replace non-symlink path: $BRIDGE_LINK" >&2
        exit 1
    fi

    if [[ ! -d "$BRIDGE_TARGET" ]]; then
        echo "securellm-bridge checkout not found at: $BRIDGE_TARGET" >&2
        exit 1
    fi

    rm -f "$BRIDGE_LINK"
    ln -s ../securellm-bridge "$BRIDGE_LINK"
    echo "Linked $BRIDGE_LINK -> ../securellm-bridge"
}

write_env_file() {
    mkdir -p "$RUNTIME_DIR"
    {
        printf 'RUST_LOG=%s\n' "${RUST_LOG:-info}"
        printf 'DATABASE_URL=%s\n' 'sqlite:///var/lib/securellm/models.db?mode=rwc'
        printf 'LLAMACPP_ENABLED=true\n'
        printf 'LLAMACPP_BASE_URL=%s\n' "${LLAMACPP_BASE_URL:-http://127.0.0.1:8081}"
        printf 'LLAMACPP_MODEL_NAME=%s\n' "${LLAMACPP_MODEL_NAME:-local-model}"
    } >"$ENV_FILE"
}

image_exists() {
    docker image inspect "$IMAGE" >/dev/null 2>&1
}

container_exists() {
    docker container inspect "$CONTAINER" >/dev/null 2>&1
}

container_running() {
    [[ "$(docker inspect -f '{{.State.Running}}' "$CONTAINER" 2>/dev/null || echo false)" == "true" ]]
}

container_network_mode() {
    docker inspect -f '{{.HostConfig.NetworkMode}}' "$CONTAINER" 2>/dev/null || true
}

ensure_container_runtime_matches() {
    if ! container_exists; then
        return
    fi

    if [[ "$(container_network_mode)" == "$NETWORK_MODE" ]]; then
        return
    fi

    echo "Recreating $CONTAINER to use Docker network mode: $NETWORK_MODE"
    docker stop "$CONTAINER" >/dev/null 2>&1 || true
    docker rm "$CONTAINER" >/dev/null
}

build_image() {
    ensure_link
    docker build -t "$IMAGE" -f "$BRIDGE_LINK/docker/Dockerfile" "$BRIDGE_LINK"
}

wait_for_health() {
    local url="http://127.0.0.1:${PORT}/api/health"

    for _ in {1..45}; do
        if curl -sf "$url" >/dev/null 2>&1; then
            echo "SecureLLM Bridge healthy at $url"
            return 0
        fi
        sleep 1
    done

    echo "SecureLLM Bridge did not become healthy at $url" >&2
    docker logs --tail 80 "$CONTAINER" >&2 || true
    return 1
}

start_container() {
    ensure_link
    write_env_file

    if container_running; then
        if [[ "$(container_network_mode)" == "$NETWORK_MODE" ]]; then
            echo "SecureLLM Bridge already running: $CONTAINER"
            wait_for_health
            return
        fi
    fi

    ensure_container_runtime_matches

    if container_exists; then
        docker start "$CONTAINER" >/dev/null
        wait_for_health
        return
    fi

    if ! image_exists; then
        build_image
    fi

    local network_args=("--network" "$NETWORK_MODE")
    local port_args=()
    if [[ "$NETWORK_MODE" != "host" ]]; then
        port_args=("--publish" "${PORT}:8080")
    fi

    docker run -d \
        --name "$CONTAINER" \
        "${network_args[@]}" \
        "${port_args[@]}" \
        --env-file "$ENV_FILE" \
        --volume "$BRIDGE_LINK/config:/home/securellm/.config/securellm:ro" \
        --mount "type=volume,src=${VOLUME},dst=/var/lib/securellm" \
        "$IMAGE" >/dev/null

    wait_for_health
}

stop_container() {
    if container_exists; then
        docker stop "$CONTAINER" >/dev/null || true
        echo "Stopped $CONTAINER"
    else
        echo "SecureLLM Bridge container not found: $CONTAINER"
    fi
}

status_container() {
    docker ps -a --filter "name=^/${CONTAINER}$"
}

health_probe() {
    curl -sf "http://127.0.0.1:${PORT}/api/health"
}

show_logs() {
    docker logs --tail "${SECURELLM_BRIDGE_LOG_LINES:-120}" "$CONTAINER"
}

case "${1:-}" in
    link) ensure_link ;;
    build) build_image ;;
    start) start_container ;;
    stop) stop_container ;;
    status) status_container ;;
    health) health_probe ;;
    logs) show_logs ;;
    -h|--help|help|"") usage ;;
    *)
        usage >&2
        exit 2
        ;;
esac
