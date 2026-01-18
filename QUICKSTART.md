# Neoland - CLI Unificado 🚀

## Execução Rápida

### Opção 1: Nova CLI (Recomendado)

**Servidor**:

```bash
nix develop --command cargo run --bin neoland -- server
# Ou com portas customizadas:
nix develop --command cargo run --bin neoland -- server --grpc-port 50052 --rest-port 3002
```

**Cliente** (TUI - Em Desenvolvimento):

```bash
nix develop --command cargo run --bin neoland -- client
```

**Health Check**:

```bash
nix develop --command cargo run --bin neoland -- test
```

**Restart**:

```bash
nix develop --command cargo run --bin neoland -- restart
```

---

### Opção 2: Build Release

```bash
# Build única vez
nix develop --command cargo build --release --bins

# Executar
./target/release/neoland server
./target/release/neoland client
./target/release/neoland test
./target/release/neoland --help
```

---

### Opção 3: Legacy (Scripts Shell - Deprecated)

Ainda funcionam, mas serão removidos:

```bash
./run-server.sh
./run-client.sh  # GTK4, será substituído por TUI
```

---

## 📖 CLI Subcomandos

### `neoland server`

Inicia o servidor gRPC + REST.

**Flags**:

- `--grpc-port <PORT>` - Porta gRPC (padrão: 50051)
- `--rest-port <PORT>` - Porta REST (padrão: 3001)
- `--log-level <LEVEL>` - Nível de logging: trace/debug/info/warn/error (padrão: info)

**Exemplo**:

```bash
nix develop --command cargo run --bin neoland -- \
  server \
  --grpc-port 50052 \
  --rest-port 3002 \
  --log-level debug
```

---

### `neoland client`

Inicia o cliente TUI (Terminal User Interface).

> ⚠️ **Em Desenvolvimento**: Fase 2 (TUI ratatui) ainda não implementada.  
> Use temporariamente: `nix develop --command cargo run --bin llamachat-client` (GTK4 legacy)

**Flags**:

- `--server-url <URL>` - URL do servidor gRPC (padrão: `http://[::1]:50051`)

---

### `neoland test`

Executa health checks completos no servidor.

**Flags**:

- `--rest-endpoint <URL>` - Endpoint REST (padrão: `http://localhost:3001`)
- `--grpc-endpoint <URL>` - Endpoint gRPC (padrão: `http://[::1]:50051`)

**Testes executados**:

1. REST health endpoint (`/health`)
2. Verificação de conexão gRPC
3. Verificação de processos (`ps aux`)

---

### `neoland restart`

Reinicia o servidor (mata processo antigo e inicia novo).

**Flags**:

- `--grpc-port <PORT>` - Porta gRPC (padrão: 50051)
- `--rest-port <PORT>` - Porta REST (padrão: 3001)

---

## 🔍 Verificação de Saúde

### Testar gRPC (com servidor rodando):

```bash
nix develop --command cargo test --release test_grpc_chat_stream
```

### Testar REST API:

```bash
curl http://localhost:3001/health
# Resposta esperada: OK
```

---

## 🎯 Atalhos do Cliente (GTK4 Legacy)

| Atalho         | Ação                         |
| -------------- | ---------------------------- |
| **Ctrl+1**     | Preset Balanceado            |
| **Ctrl+2**     | Preset Criativo              |
| **Ctrl+3**     | Preset Preciso               |
| **Ctrl+4**     | Preset Pesquisa (RAG Max)    |
| **Ctrl+5**     | Preset Seguro (sem comandos) |
| **Ctrl+L**     | Limpar chat                  |
| **Ctrl+Enter** | Enviar mensagem              |
| **F12**        | Toggle Hyprland scratchpad   |

---

## 📦 Modelos de IA

Na primeira execução, os modelos serão baixados automaticamente:

- **LLM**: Qwen 1.8B Chat (~1.1 GB)
- **Embeddings**: MiniLM-L6-v2 (~90 MB)

**Cache**: `~/.cache/huggingface/`

---

## ⚠️ Troubleshooting

### Erro: "Connection refused"

- ✅ Certifique-se de que o **servidor está rodando** (Terminal 1)
- ✅ Verifique se a porta **50051** está livre: `lsof -i :50051`

### Erro: "Failed to load model"

- ✅ Verifique conexão com internet (download de modelos)
- ✅ Limpe cache: `rm -rf ~/.cache/huggingface/hub/models--Qwen*`

### Cliente GTK4 não abre

- ✅ Verifique display server: `echo $WAYLAND_DISPLAY` ou `echo $DISPLAY`
- ✅ Execute em ambiente gráfico (não SSH sem X11 forwarding)

---

## 🧪 Teste Completo

```bash
# Terminal 1: Iniciar servidor
nix develop --command cargo run --bin neoland -- server

# Terminal 2: Executar health check
nix develop --command cargo run --bin neoland -- test

# Terminal 2: Executar cliente (GTK4 legacy)
nix develop --command cargo run --bin llamachat-client
```

---

## 📊 Endpoints

- **gRPC**: `http://[::1]:50051` (cliente GTK4)
- **REST**: `http://0.0.0.0:3001` (API OpenAI-compatible)
- **Health**: `http://localhost:3001/health`

---

## 🛠️ Build & Desenvolvimento

```bash
# Verificar código
nix develop --command cargo check --all-targets

# Build release (todos binaries)
nix develop --command cargo build --release --bins

# Build específico
nix develop --command cargo build --release --bin neoland

# Executar testes
nix develop --command cargo test --release

# Limpar build
cargo clean
```

---

## 🚀 Próximas Fases

- **Fase 2**: TUI moderna com ratatui (substituir GTK4)
- **Fase 3**: Integração profunda com intelagent-core (phantom)
- **Fase 4**: Integração com securellm-bridge (fallback multi-provider)
