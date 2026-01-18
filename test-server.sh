#!/usr/bin/env bash
# Script para testar servidor com verbose logging

echo "🧪 Testando servidor Neoland..."
echo ""

# Testar REST
echo "1️⃣ Testando REST API..."
curl -s http://localhost:3001/health && echo " ✅ REST OK" || echo " ❌ REST FALHOU"
echo ""

# Testar gRPC
echo "2️⃣ Testando gRPC..."
timeout 30 nix develop --command cargo test --release test_grpc_chat_stream -- --nocapture 2>&1 | tail -20

echo ""
echo "3️⃣ Verificando cache HuggingFace..."
if [ -d ~/.cache/huggingface ]; then
    du -sh ~/.cache/huggingface/
else
    echo "⚠️  Cache não existe - modelo não foi baixado"
fi

echo ""
echo "4️⃣ Verificando processos..."
ps aux | grep llamachat-server | grep -v grep || echo "❌ Servidor não está rodando"
