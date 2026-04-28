# Neoland Quick Start Guide

**Last Updated**: 2026-04-26

Get up and running with Neoland in under 5 minutes.

---

## Prerequisites

- 2GB free RAM (for local inference)
- Terminal emulator
- One of:
  - Nix / NixOS recommended
  - Ubuntu or comparable bare metal environment supported

---

## Installation

### Option 1: Development Mode With Nix (Recommended)

```bash
cd /home/kernelcore/master/neoland
nix develop

# Dev-shell commands
neoland-secrets
neoland-server
neoland-client --ml-api-url http://localhost:9000
neoland-doctor --json

# One-shot commands also work
nix develop --command neoland-server
nix develop --command neoland-test --json
```

### Option 2: Build Release Binary

```bash
nix develop --command cargo build --bin neoland --release
# Binary: ./target/release/neoland
```

---

## Basic Usage

### 1. Start the Server

```bash
# Default ports: gRPC=50051, REST=3001
neoland server

# In nix develop, you can use the shortcut command
neoland-server
nix develop --command neoland-server

# Custom configuration
neoland server --grpc-port 50052 --rest-port 3002 --log-level debug
```

The server will:

- Download models on first run (~1.2GB total)
- Start gRPC service on `[::]:<grpc-port>`
- Start REST API on `0.0.0.0:<rest-port>`

### 2. Launch TUI Client

**In a new terminal**:

```bash
# Connect to default local server
neoland client

# In nix develop, you can use the shortcut command
neoland-client
nix develop --command neoland-client --ml-api-url http://localhost:9000

# Custom endpoints
neoland client \
  --server-url http://[::1]:50051 \
  --ml-api-url http://localhost:9000
```

## Secrets

For local encrypted secrets, the preferred workflow is to edit
`secrets/neoland.sops.env` with:

```bash
neoland-secrets
```

The `neoland`, `neoland-server`, `neoland-client`, `neoland-test`,
`neoland-doctor`, `neoland-restart`, `nsrv`, and `ncli` commands decrypt and
load that file automatically before launch.

If you are on bare metal Ubuntu or do not want to use SOPS, exporting the same
environment variables manually is also valid:

```bash
export NEOLAND_ADMIN_API_KEY=...
export NEOLAND_USER_API_KEY=...
export NEOLAND_READONLY_API_KEY=...
```

SOPS is recommended, not mandatory.

---

## TUI Interface

### Layout

```
╭─ Neoland ── ● Connected  gpt-4o-mini  1.2k tok  42ms ──────╮
│                                                              │
│  12:34:56  you                                               │
│  Hello, how are you?                                         │
│                                                              │
│  12:34:58  assistant                                         │
│  I'm doing well! How can I help you today?                   │
│                                                              │
│  12:35:02  assistant                          ⠹ thinking... │
│                                                              │
│                                         ↑↓ scroll  g bottom │
╰─────────────────────────────────────────────────────────────╯
╭─ Message ───────────────────────── Enter send  Ctrl+C quit ─╮
│ type here█                                                   │
╰──────────────────────────────────────────────────────────────╯
```

### Keyboard Shortcuts

| Key | Action |
|-----|--------|
| **Enter** | Send message |
| **Ctrl+C** / **Esc** | Quit |
| **Ctrl+L** | Clear chat |
| **↑ / ↓** | Scroll chat |
| **PgUp / PgDn** | Scroll fast |
| **g** (buffer empty) | Jump to bottom |
| **Ctrl+← / →** | Word left/right |
| **Alt+B / Alt+F** | Word left/right (alt) |
| **Ctrl+A / Ctrl+E** | Line start/end |
| **Ctrl+W** | Delete word back |
| **Ctrl+U** | Kill to start |
| **Ctrl+K** | Kill to end |
| **Ctrl+1** | Balanced preset (temp 0.7) |
| **Ctrl+2** | Creative preset (temp 1.5) |
| **Ctrl+3** | Precise preset (temp 0.3) |
| **Ctrl+4** | Research preset |
| **Ctrl+5** | Safe preset (no commands) |
| **Tab** | Toggle sidebar |

### Visual Feedback

- **● Connected** (green): pipeline reachable
- **● Degraded** (yellow): partial connectivity
- **● Offline** (red): no connection
- **⠹ thinking...**: braille spinner while waiting for response

---

## Testing

### Health Check

```bash
neoland test

# JSON output for scripts or CI smoke checks
neoland test --json
```

Verifies:

1. REST API (`/health` endpoint)
2. gRPC connectivity (real connection attempt)
3. Process status

### REST API Direct Test

```bash
# Health check (no auth)
curl http://localhost:3001/health

# OpenAPI spec (no auth)
curl http://localhost:3001/openapi.json | jq .info
# Swagger UI: http://localhost:3001/swagger-ui/

# Chat completion (requires API key)
curl -X POST http://localhost:3001/v1/chat/completions \
  -H "X-API-Key: $NEOLAND_ADMIN_API_KEY" \
  -H "Content-Type: application/json" \
  -d '{"messages": [{"role": "user", "content": "Hello"}], "stream": false}'
```

> **IMPORTANTE — Dev keys**: Por padrão, se `NEOLAND_ADMIN_API_KEY` não estiver
> definida no ambiente, o servidor sobe com uma chave de desenvolvimento
> (`neoland_admin_dev_key_change_in_production`). **Nunca use essa chave em
> produção.** Defina sempre as variáveis de ambiente antes de subir o servidor:
>
> ```bash
> export NEOLAND_ADMIN_API_KEY="$(openssl rand -hex 32)"
> export NEOLAND_USER_API_KEY="$(openssl rand -hex 32)"
> export NEOLAND_READONLY_API_KEY="$(openssl rand -hex 32)"
> ```

---

## Advanced Configuration

### Environment Variables

```bash
# ML Offload API URL (if using external GPU service)
export ML_API_URL="http://gpu-server.local:9000"

neoland client --ml-api-url $ML_API_URL
```

### Preset Customization

Edit `src/tui/presets.rs`:

```rust
pub fn creative() -> Self {
    Self {
        temperature: 1.8,  // Increase randomness
        max_tokens: 800,   // Longer responses
        // ...
    }
}
```

Rebuild: `cargo build --release`

---

## NixOS Integration

### System Service

Import the bundled module into your NixOS config:

```nix
imports = [ /home/kernelcore/master/neoland/modules/applications/neoland.nix ];

services.neoland = {
  enable = true;
  grpcPort = 50051;
  restPort = 3001;
  openFirewall = true;
  environmentFile = "/run/secrets/neoland.env";
};
```

Rebuild: `sudo nixos-rebuild switch`

Use the environment file for deployment secrets such as:

- `DATABASE_URL`
- `NEOLAND_ADMIN_API_KEY`
- `NEOLAND_USER_API_KEY`
- `NEOLAND_READONLY_API_KEY`
- `VAULT_ADDR`
- `VAULT_TOKEN`

---

## Troubleshooting

### Model Download Issues

```bash
# Clear cache and retry
rm -rf ~/.cache/huggingface/hub/models--Qwen*
neoland server
```

### Connection Refused

```bash
# Check if server is running
pgrep -f "neoland server"

# Check port availability
lsof -i :50051
```

### TUI Not Rendering

```bash
# Verify terminal capabilities
echo $TERM
# Should be: xterm-256color, alacritty, or similar

# Force color support
export TERM=xterm-256color
neoland client
```

### High Memory Usage

Server memory grows with chat history. Restart periodically:

```bash
neoland restart
```

---

## Performance Tips

1. **Use ml-offload-api**: Offload to GPU for 10-50x speedup
2. **Reduce max_tokens**: Lower values = faster responses
3. **Disable RAG**: Set `context_top_k: 0` if not needed

---

## Next Steps

- Read `ARCHITECTURE.md` for technical details
- Explore integration points (`ml-offload-api`, `securellm-bridge`)
- Check the bundled NixOS module: `modules/applications/neoland.nix`

---

## Support

Internal project - contact VoidNxSEC team for issues.

---

**Pro Tip**: Use `neoland --help` or `neoland <command> --help` for detailed CLI documentation.
