#!/usr/bin/env bash
# Script para executar o cliente Neoland (GTK4)

set -e

echo "🎨 Iniciando Neoland Client (GTK4)..."
echo "🔌 Conectando em: http://[::1]:50051"
echo ""
echo "⚠️  IMPORTANTE: O servidor deve estar rodando!"
echo "   Execute './run-server.sh' em outro terminal"
echo "─────────────────────────────────────"

nix develop --command cargo run --release --bin llamachat-client
