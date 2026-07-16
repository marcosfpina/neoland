# Neoland Desktop

Tauri v2 native desktop application wrapping the [Neoland Web Console](../web/).

## Architecture

```
┌──────────────────────────────────────────┐
│           Tauri Shell (Rust)             │
│  ┌────────────────────────────────────┐  │
│  │   System Tray · Notifications      │  │
│  │   Global Shortcuts · Menu Bar      │  │
│  └────────────────────────────────────┘  │
│  ┌────────────────────────────────────┐  │
│  │   WebView (WebKit / Edge)          │  │
│  │   ┌──────────────────────────────┐ │  │
│  │   │  Leptos WASM SPA             │ │  │
│  │   │  (web/dist/)                 │ │  │
│  │   └──────────────────────────────┘ │  │
│  └────────────────────────────────────┘  │
│                  │ REST :3001             │
│                  ▼                        │
│     Neoland Control Plane (external)      │
└──────────────────────────────────────────┘
```

The desktop app reuses the exact same Leptos WASM bundle as the Web Console.
The Tauri Rust backend provides native OS features:

- **System Tray** — icon with Show/Hide/Quit menu, click to toggle window
- **Notifications** — native OS notifications for pipeline completion
- **Health Check** — verifies the Neoland server is reachable
- **Commands** — `get_server_url`, `notify`, `check_server_health` exposed to frontend

## Prerequisites (Linux)

```bash
# NixOS / nix develop
nix develop  # brings in webkitgtk, gtk3, glib, etc.

# Ubuntu / Debian
sudo apt install libwebkit2gtk-4.1-dev libgtk-3-dev libsoup-3.0-dev \
  libglib2.0-dev libcairo2-dev libpango1.0-dev libatk1.0-dev \
  libgdk-pixbuf-2.0-dev
```

## Development

```bash
# Start the Neoland server (in another terminal)
just serve

# Hot-reload desktop app with Trunk dev proxy
just desktop-dev

# Or manually:
cd desktop && cargo tauri dev
```

## Build

```bash
# Build WASM frontend + native desktop binary
just desktop-build

# Output:
#   desktop/src-tauri/target/release/neoland-desktop     (binary)
#   desktop/src-tauri/target/release/bundle/             (installers)
```

## Nix

```bash
# Validate build (compiles WASM + checks Tauri Rust code)
nix build .#neoland-desktop

# Full native build (inside nix develop)
just desktop-build
```

## Commands (invocable from the frontend via `@tauri-apps/api`)

| Command | Args | Returns |
|---|---|---|
| `get_server_url` | — | `String` (default: `http://127.0.0.1:3001`) |
| `notify` | `title: String, body: String` | `Result<(), String>` |
| `check_server_health` | `url: String` | `Result<String, String>` (JSON status) |

## Roadmap

- [x] Tauri v2 shell (Linux, macOS, Windows)
- [x] System tray integration
- [x] Native OS notifications
- [x] Nix derivation (`nix build .#neoland-desktop`)
- [ ] Embedded Neoland server (full offline mode)
- [ ] Global keyboard shortcuts
- [ ] Auto-start on login
- [ ] Deep link handling (`neoland://`)
- [ ] DMG / AppImage / MSI installers via CI
