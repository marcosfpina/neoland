#!/usr/bin/env bash
set -e

# Configuração
SOCKET_PATH="/tmp/mission-control.sock"
DAEMON_SCRIPT="nix/system-daemon.py"

echo "🛠️  [TEST] Iniciando Daemon (Background)..."
# Usar python3 direto (assumindo que o ambiente tem as libs, ou confiar no nix-shell do usuário)
# Em um ambiente Nix puro, isso deveria ser 'nix develop -c python3 ...'
python3 "$DAEMON_SCRIPT" > /dev/null 2>&1 &
DAEMON_PID=$!

# Aguardar inicialização do socket
echo "⏳ [TEST] Aguardando socket em $SOCKET_PATH..."
for i in {1..10}; do
    if [ -S "$SOCKET_PATH" ]; then
        echo "✅ [TEST] Socket detectado!"
        break
    fi
    sleep 0.5
done

if [ ! -S "$SOCKET_PATH" ]; then
    echo "❌ [TEST] Falha: Socket não criado após 5s."
    kill $DAEMON_PID
    exit 1
fi

# Testar endpoint via curl
echo "📡 [TEST] Consultando endpoint /metrics via UDS..."
RESPONSE=$(curl -s --unix-socket "$SOCKET_PATH" http://localhost/metrics)

# Validar JSON básico
if echo "$RESPONSE" | grep -q "timestamp"; then
    echo "✅ [TEST] Sucesso! Resposta recebida:"
    echo "$RESPONSE" | jq 'del(.processes) | del(.services)' # Mostrar resumo (sem listas longas)
else
    echo "❌ [TEST] Resposta inválida:"
    echo "$RESPONSE"
fi

# Limpeza
echo "🧹 [TEST] Encerrando daemon..."
kill $DAEMON_PID
rm -f "$SOCKET_PATH"
