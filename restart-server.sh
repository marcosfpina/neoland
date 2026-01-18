#!/usr/bin/env bash
# Script para reiniciar o servidor Neoland após correção

echo "🔄 Reiniciando servidor Neoland..."

# Encontrar e matar processo antigo
OLD_PID=$(ps aux | grep llamachat-server | grep -v grep | awk '{print $2}' | head -1)

if [ -n "$OLD_PID" ]; then
    echo "🛑 Parando servidor antigo (PID: $OLD_PID)..."
    kill $OLD_PID
    sleep 2
fi

echo "🚀 Iniciando novo servidor com suporte IPv6..."
./run-server.sh
