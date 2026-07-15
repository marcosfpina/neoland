# ADR-022: WmBackend Trait Abstraction (deferred)

**Status:** Deferred  
**Data:** 2026-07-15  
**Contexto:** Remoção da dependência `hyprland-ipc` do ai-agent-os

---

## Contexto

O neoland dependia de `hyprland-ipc`, um crate interno ao repo `ai-agent-os`, via rev git
pinada. Isso criava uma inversão de camadas (control plane dependendo de agents-OS) e tornava
o IPC intestável em CI headless.

**Decisão imediata (2026-07-15):** o código do `hyprland-ipc` foi internalizado diretamente
em `src/hyprland_ops.rs`. A dependência git e o path override em `.cargo/config.toml` foram
removidos. O crate `neoland` passou a falar com o socket do Hyprland sem intermediários.

---

## Decisão futura: trait `WmBackend`

Quando portabilidade de compositor ou testabilidade em CI headless tornarem-se requisitos,
extrair a seguinte abstração:

```rust
// src/wm/backend.rs
pub trait WmBackend: Send + Sync {
    async fn dispatch(&self, cmd: &str) -> Result<String>;
    async fn get_active_workspace(&self) -> Result<Workspace>;
    async fn get_clients(&self) -> Result<Vec<Window>>;
}

// impl real
pub struct HyprlandBackend { socket_path: PathBuf }
impl WmBackend for HyprlandBackend { ... }

// impl para testes
pub struct MockBackend { responses: VecDeque<String> }
impl WmBackend for MockBackend { ... }
```

`HyprlandIPC` vira um wrapper fino sobre `HyprlandBackend`. Os callers recebem
`Arc<dyn WmBackend>` via injeção.

### Gatilhos para priorizar

- CI headless precisar testar código que hoje chama `HyprlandIPC`
- Suporte a Sway/KWin entrar no roadmap
- `hyprland_ops.rs` crescer além de ~200 linhas e virar difícil de rastrear sem interface

### Custo estimado

~1 dia. Sem breaking changes nos callers se feito com `Arc<dyn WmBackend>` desde o início.

---

## Consequências

- **Agora:** zero dependências externas para IPC de compositor; acoplamento ai-agent-os → neoland eliminado.
- **Futuro (ao implementar o trait):** testabilidade completa em CI headless e portabilidade de compositor.
