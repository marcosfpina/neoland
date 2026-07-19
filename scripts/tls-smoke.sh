#!/usr/bin/env bash
# TLS/mTLS smoke — valida que os listeners REST e gRPC servem TLS nativo
# e rejeitam conexões sem client cert quando um CA está configurado.
#
# Uso: bash scripts/tls-smoke.sh [caminho-do-binario]
# Pré-requisito: secrets/tls/ gerado por scripts/gen-certs.sh --auto

set -euo pipefail

BIN="${1:-./target/debug/neoland}"
TLS_DIR="secrets/tls"
REST_URL="https://localhost:3001/health"
SERVER_LOG="$(mktemp)"

if [[ ! -f "$TLS_DIR/ca/ca.crt" ]]; then
  echo "Certificados não encontrados — rodando gen-certs.sh --auto"
  bash scripts/gen-certs.sh --auto
fi

echo "▶ Subindo server com mTLS (log: $SERVER_LOG)..."
RUST_BACKTRACE=1 \
NEOLAND_SKIP_EMBEDDINGS=1 \
NEOLAND_TLS_CA_CERT="$TLS_DIR/ca/ca.crt" \
NEOLAND_TLS_SERVER_CERT="$TLS_DIR/server/server.crt" \
NEOLAND_TLS_SERVER_KEY="$TLS_DIR/server/server.key" \
"$BIN" server > "$SERVER_LOG" 2>&1 &
SERVER_PID=$!

cleanup() {
  status=$?
  # Diagnóstico ANTES do kill — senão o curl -v examina um server morto
  if [[ $status -ne 0 ]]; then
    echo "── diagnóstico: server vivo? ──"
    if kill -0 "$SERVER_PID" 2>/dev/null; then
      echo "server VIVO (pid $SERVER_PID) — listeners:"
      ss -tlnp 2>/dev/null | grep -E '3001|50051' || echo "  (nenhum listener em 3001/50051)"
    else
      wait "$SERVER_PID" 2>/dev/null
      echo "server MORTO — exit code: $?"
    fi
    echo "── diagnóstico: curl -v ──"
    curl -v --max-time 10 \
      --cacert "$TLS_DIR/ca/ca.crt" \
      --cert "$TLS_DIR/client/client.crt" \
      --key "$TLS_DIR/client/client.key" \
      "$REST_URL" 2>&1 | tail -30 || true
    echo "── diagnóstico: server log (completo) ──"
    cat "$SERVER_LOG" || true
    echo "── diagnóstico: curl --version ──"
    curl --version | head -1
  fi
  kill "$SERVER_PID" 2>/dev/null || true
  exit $status
}
trap cleanup EXIT

echo "▶ 1/4 mTLS com client cert deve responder (readiness ≤30s)..."
ready=""
for _ in $(seq 1 30); do
  if curl -sf --max-time 5 \
    --cacert "$TLS_DIR/ca/ca.crt" \
    --cert "$TLS_DIR/client/client.crt" \
    --key "$TLS_DIR/client/client.key" \
    "$REST_URL" > /dev/null 2>&1; then
    ready=1
    break
  fi
  sleep 1
done
if [[ -z "$ready" ]]; then
  echo "  ❌ mTLS com client cert não respondeu em 30s"
  exit 56
fi
echo "  ✅ aceito"

echo "▶ 2/4 Sem client cert deve ser rejeitado..."
if curl -s --max-time 10 --cacert "$TLS_DIR/ca/ca.crt" "$REST_URL" > /dev/null 2>&1; then
  echo "  ❌ ERRO: conexão sem client cert foi aceita"; exit 1
fi
echo "  ✅ rejeitado"

echo "▶ 3/4 HTTP puro deve falhar..."
if curl -s --max-time 10 "http://localhost:3001/health" > /dev/null 2>&1; then
  echo "  ❌ ERRO: HTTP puro foi aceito"; exit 1
fi
echo "  ✅ rejeitado"

echo "▶ 4/4 Handshake TLS no gRPC (:50051)..."
echo | openssl s_client -connect localhost:50051 \
  -CAfile "$TLS_DIR/ca/ca.crt" \
  -cert "$TLS_DIR/client/client.crt" \
  -key "$TLS_DIR/client/client.key" 2>/dev/null \
  | grep -q "Verify return code: 0"
echo "  ✅ handshake TLSv1.3 ok"

echo "✅ TLS smoke: 4/4"
