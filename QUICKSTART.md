# Neoland Quick Start Guide

**Last Updated**: 2026-04-02

Get up and running with Neoland in under 5 minutes.

---

## Prerequisites

- NixOS or Nix package manager
- 2GB free RAM (for local inference)
- Terminal emulator

---

## Installation

### Option 1: Development Mode

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

For local encrypted secrets, edit `secrets/neoland.sops.env` with:

```bash
neoland-secrets
```

The `neoland`, `neoland-server`, `neoland-client`, `neoland-test`,
`neoland-doctor`, `neoland-restart`, `nsrv`, and `ncli` commands decrypt and
load that file automatically before launch.

---

## TUI Interface

### Layout

```
┌─────────────────────────────────────────────────────────────┐
│ 🚀 Neoland TUI | http://[::1]:50051 | ✅ Ready | Preset: ... │
├─────────────────────────────────────────┬───────────────────┤
│ 💬 Chat                                 │ ℹ️  Info           │
│                                         │                   │
│ [12:34:56] 👤 VOCÊ                      │ 📊 Config Atual   │
│ Hello, how are you?                     │                   │
│                                         │  Temperature: 0.70│
│ [12:34:58] 🤖 AI                        │  Top P: 0.90      │
│ I'm doing well! How can I help?         │  Max Tokens: 600  │
│                                         │                   │
│                                         │ ⌨️  Atalhos       │
│                                         │                   │
│                                         │  Ctrl+1-5: Presets│
│                                         │  Ctrl+L: Limpar   │
│                                         │  Tab: Sidebar     │
│                                         │  Esc: Sair        │
├─────────────────────────────────────────┴───────────────────┤
│ ✏️  Input (Ctrl+Enter para enviar)                          │
│ _                                                           │
└─────────────────────────────────────────────────────────────┘
```

### Keyboard Shortcuts

| Key            | Action                      |
| -------------- | --------------------------- |
| **Ctrl+Enter** | Send message                |
| **Ctrl+L**     | Clear chat                  |
| **Ctrl+1**     | Balanced preset (temp: 0.7) |
| **Ctrl+2**     | Creative preset (temp: 1.5) |
| **Ctrl+3**     | Precise preset (temp: 0.3)  |
| **Ctrl+4**     | Research preset (RAG: max)  |
| **Ctrl+5**     | Safe preset (commands: off) |
| **Tab**        | Toggle sidebar              |
| **j** / **k**  | Scroll down/up              |
| **Esc**        | Quit                        |

### Visual Feedback

- **✅ Ready**: System idle
- **⏳ Thinking...**: Processing inference

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
# Health check
curl http://localhost:3001/health
# Expected: OK

# Chat completion (OpenAI-compatible, authenticated)
curl -X POST http://localhost:3001/v1/chat/completions \
  -H "X-API-Key: neoland_admin_dev_key_change_in_production" \
  -H "Content-Type: application/json" \
  -d '{
    "messages": [{"role": "user", "content": "Hello"}],
    "stream": false
  }'
```

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
