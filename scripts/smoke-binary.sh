#!/usr/bin/env bash
#
# Smoke test do artefato de release.
#
# Valida o que `--help` não valida: que o binário SOBE, ATENDE e ENCERRA
# limpo — a partir de um diretório fora da árvore do repo, sem devShell,
# sem cargo, sem sops. É o gate do Alpha: se este script passa, o binário
# entregue funciona numa máquina que só tem ele.
#
#   ./scripts/smoke-binary.sh [caminho-do-binário]
#
# Default: ./result/bin/neoland (saída de `nix build .#neoland`).

set -euo pipefail

BINARY="${1:-./result/bin/neoland}"

if [[ ! -x "$BINARY" ]]; then
    echo "❌ binário não encontrado ou não executável: $BINARY" >&2
    echo "   rode antes: nix build .#neoland" >&2
    exit 1
fi

BINARY="$(readlink -f "$BINARY")"

# Portas altas fixas para evitar colisão com o servidor de dev (3001/50051).
REST_PORT="${SMOKE_REST_PORT:-38101}"
GRPC_PORT="${SMOKE_GRPC_PORT:-38151}"

WORKDIR="$(mktemp -d)"
SERVER_PID=""

cleanup() {
    if [[ -n "$SERVER_PID" ]] && kill -0 "$SERVER_PID" 2>/dev/null; then
        kill -KILL "$SERVER_PID" 2>/dev/null || true
    fi
    rm -rf "$WORKDIR"
}
trap cleanup EXIT

echo "┌─ Neoland binary smoke"
echo "│  binário : $BINARY"
echo "│  workdir : $WORKDIR  (fora da árvore do repo)"
echo "│  REST    : 127.0.0.1:$REST_PORT"
echo "└─"

# Ambiente deliberadamente mínimo: nada de DATABASE_URL (o servidor deve
# subir degradado, com o pipeline desligado), nada de sops, nada de HOME
# do dev. NEOLAND_SKIP_EMBEDDINGS evita o download de ~90MB do
# HuggingFace que de outro modo bloqueia o boot na primeira execução.
cd "$WORKDIR"
env -i \
    PATH="/usr/bin:/bin" \
    HOME="$WORKDIR" \
    NEOLAND_SKIP_EMBEDDINGS=true \
    "$BINARY" server --rest-port "$REST_PORT" --grpc-port "$GRPC_PORT" \
    > "$WORKDIR/server.log" 2>&1 &
SERVER_PID=$!

echo "▶ aguardando /health responder (timeout 60s)..."
ready=false
for _ in $(seq 1 120); do
    if ! kill -0 "$SERVER_PID" 2>/dev/null; then
        echo "❌ servidor morreu durante o boot. Log:" >&2
        cat "$WORKDIR/server.log" >&2
        exit 1
    fi
    if curl -fsS "http://127.0.0.1:$REST_PORT/health" >/dev/null 2>&1; then
        ready=true
        break
    fi
    sleep 0.5
done

if [[ "$ready" != true ]]; then
    echo "❌ /health não respondeu em 60s. Log:" >&2
    cat "$WORKDIR/server.log" >&2
    exit 1
fi
echo "  ✔ /health respondeu"

# /health devolve JSON com status/version/components
health_body="$(curl -fsS "http://127.0.0.1:$REST_PORT/health")"
for field in status version components; do
    if ! grep -q "\"$field\"" <<<"$health_body"; then
        echo "❌ /health sem campo '$field': $health_body" >&2
        exit 1
    fi
done
echo "  ✔ /health tem status, version e components"

if ! curl -fsS "http://127.0.0.1:$REST_PORT/live" | grep -q '"alive":true'; then
    echo "❌ /live não reportou alive:true" >&2
    exit 1
fi
echo "  ✔ /live reporta alive"

if ! curl -fsS "http://127.0.0.1:$REST_PORT/metrics" | grep -q '# HELP'; then
    echo "❌ /metrics não devolveu formato Prometheus" >&2
    exit 1
fi
echo "  ✔ /metrics em formato Prometheus"

# O spec precisa estar embutido no binário — o Web Console e clientes
# gerados dependem dele.
if ! curl -fsS "http://127.0.0.1:$REST_PORT/openapi.json" | grep -q '"openapi"'; then
    echo "❌ /openapi.json não devolveu o spec" >&2
    exit 1
fi
echo "  ✔ /openapi.json servido pelo binário"

# Sem DATABASE_URL o pipeline fica desligado; a rota deve dizer isso em
# vez de derrubar o processo.
if ! curl -fsS "http://127.0.0.1:$REST_PORT/v1/agents/health" | grep -q '"status"'; then
    echo "❌ /v1/agents/health não respondeu" >&2
    exit 1
fi
echo "  ✔ /v1/agents/health responde com o pipeline desligado"

echo "▶ SIGTERM deve drenar e sair com 0..."
kill -TERM "$SERVER_PID"

exit_code=""
for _ in $(seq 1 60); do
    if ! kill -0 "$SERVER_PID" 2>/dev/null; then
        wait "$SERVER_PID" && exit_code=0 || exit_code=$?
        break
    fi
    sleep 0.5
done

if [[ -z "$exit_code" ]]; then
    echo "❌ servidor não encerrou 30s após SIGTERM" >&2
    cat "$WORKDIR/server.log" >&2
    exit 1
fi

if [[ "$exit_code" != 0 ]]; then
    echo "❌ saída suja após SIGTERM: exit $exit_code" >&2
    cat "$WORKDIR/server.log" >&2
    exit 1
fi
echo "  ✔ encerrou com exit 0"

if ! grep -q "Shutdown complete" "$WORKDIR/server.log"; then
    echo "❌ log não confirma drenagem ('Shutdown complete' ausente)" >&2
    cat "$WORKDIR/server.log" >&2
    exit 1
fi
echo "  ✔ log confirma drenagem completa"

SERVER_PID=""
echo "✅ smoke passou — o binário roda sozinho"
