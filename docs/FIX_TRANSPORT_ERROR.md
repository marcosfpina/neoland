# ⚠️ INSTRUÇÕES PARA RESOLVER ERRO DE TRANSPORTE

## Problema Identificado

O servidor estava escutando em `0.0.0.0:50051` (IPv4 only), mas o cliente tentava conectar em `[::1]:50051` (IPv6).

## Solução Aplicada

✅ Servidor agora escuta em `[::]:50051` (IPv4 + IPv6)

## Como Testar Agora

### 1. Parar o servidor antigo

```bash
# Encontre o PID do servidor
ps aux | grep llamachat-server | grep -v grep

# Mate o processo (substitua PID pelo número encontrado)
kill 205812  # ou o PID que você encontrou
```

### 2. Recompilar e reiniciar servidor

```bash
# Terminal 1
./run-server.sh
```

### 3. Testar cliente (em outro terminal)

```bash
# Terminal 2
./run-client.sh
```

## Verificação Rápida

```bash
# Verificar se servidor está escutando em IPv6
lsof -i :50051

# Deve mostrar algo como:
# COMMAND     PID USER   FD   TYPE DEVICE SIZE/OFF NODE NAME
# llamachat XXXXX user   10u  IPv6 XXXXXX      0t0  TCP *:50051 (LISTEN)
```

## Teste de Conectividade

```bash
# Testar REST (deve retornar "OK")
curl http://localhost:3001/health

# Testar gRPC (com servidor rodando)
nix develop --command cargo test --release test_grpc_chat_stream
```

---

**Status**: Correção aplicada, aguardando recompilação do servidor.
