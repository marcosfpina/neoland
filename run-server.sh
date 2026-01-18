#!/usr/bin/env bash
# Script para executar o servidor Neoland (gRPC + REST)

set -e

echo "🚀 Iniciando Neoland Server..."
echo "📡 gRPC: http://[::1]:50051"
echo "🌐 REST: http://0.0.0.0:3001"
echo ""
echo "Pressione Ctrl+C para parar"
echo "─────────────────────────────────────"

nix develop --command cargo run --release --bin llamachat-server
