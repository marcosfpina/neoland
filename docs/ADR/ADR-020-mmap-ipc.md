# ADR-020: mmap IPC — Zero-Copy Intra-Host Communication

**Status**: Accepted  
**Date**: 2026-04-07  
**Decision Makers**: marcosfpina, voidnxlabs Architecture  
**Fase**: Ciclo 1 — Fase A

---

## Contexto

O control plane Rust e o pipeline DSPy Python rodam no mesmo host. A comunicação atual
é via HTTP (REST): o Rust chama `POST /v1/pipeline/run` e aguarda a resposta completa.
Isso é adequado para o fluxo principal, mas não para:

- Sinalização de abort em tempo real (o Rust precisa interromper um pipeline em execução)
- Monitoramento de progresso sub-segundo (confidence, risk_level do agente corrente)
- Telemetria sem adicionar latência HTTP em cada step do pipeline

A alternativa explorada foi usar UNIX sockets ou pipes — mas ambos exigem protocolo de
framing e são bidirecionais quando precisamos de latência mínima de leitura.

## Decisão

**Usar um arquivo mmap'd como região de memória compartilhada** entre o processo Rust
e o processo Python, com um layout binário fixo (`SharedFlags`, 64 bytes, 1 cache line).

O Rust escreve (control plane) e o Python lê e escreve (pipeline). O arquivo fica em
`/run/neoland/agent-flags.shm` (tmpfs no NixOS — zero disco I/O).

### Layout binário (`SharedFlags`)

```
Offset  Size  Tipo           Campo
──────  ────  ─────────────  ──────────────────────────────
0       1     AtomicBool     pipeline_active
1       1     AtomicBool     escalate_to_architect
2       1     AtomicBool     abort_requested
3       1     —              pad (alinhamento AtomicU32)
4       4     AtomicU32      junior_confidence (f32 bits)
8       1     AtomicU8       risk_level (0=low…3=critical)
9       7     —              pad
16      36    [u8; 36]       session_id (UUID UTF-8)
52      12    —              pad (completar 64 bytes)
Total   64
```

`repr(C, align(64))` garante layout determinístico e alinhamento de cache line.
O tamanho é verificado em tempo de compilação: `assert!(size_of::<SharedFlags>() == 64)`.

### Protocolo de escrita

| Quem escreve | Quem lê | Campo | Semântica |
|---|---|---|---|
| Rust (control plane) | Python | `abort_requested` | O Python checa a cada step |
| Python (pipeline) | Rust | `pipeline_active` | Rust sabe se o pipeline está rodando |
| Python (pipeline) | Rust | `escalate_to_architect` | Rust pode logar/alertar |
| Python (pipeline) | Rust | `junior_confidence` | Métricas em tempo real |
| Python (pipeline) | Rust | `risk_level` | Alertas e throttling |
| Rust (control plane) | Python | `session_id` | Contexto da sessão ativa |

### Componentes entregues

- `src/agents/flags.rs` — `SharedFlags`, `RiskLevel`, métodos `store_*`/`load_*`
- `src/agents/mmap.rs` — `MmapRegion::open()`, `flags()`, `flush()`
- `src/config.rs` — `MmapConfig { shm_path }` + `NEOLAND_SHM_PATH` env override
- `agents/neoland_agents/ipc/flags.py` — `AgentFlags` (context manager) com offsets idênticos

## Alternativas Consideradas

| Alternativa | Motivo de rejeição |
|---|---|
| UNIX socket | Framing necessário; overhead de syscall por mensagem |
| Pipe | Unidirecional; não adequado para flags de múltiplos campos |
| Shared memory POSIX (`shm_open`) | `memfd_create` mais difícil de integrar com Python `mmap`; file-backed é mais simples |
| Redis / SQLite | Dependência externa; latência de rede/disco |

## Consequências

**Positivas**:
- Leitura do `abort_requested` pelo Python: ~1 instrução de memória (sem syscall)
- Sem protocolo de serialização para campos simples
- Python usa `mmap.mmap` nativo — sem dependências extras
- Layout verificado em compile-time — drift entre Rust e Python impossível sem quebrar os testes

**Negativas / Riscos**:
- Layout binário é um contrato implícito: qualquer mudança em `SharedFlags` **exige** atualização simultânea em `flags.py`
- `session_id` usa escrita não-atômica ([u8; 36]) — assumimos single writer (Rust). Se isso mudar, migrar para lock ou mensagem NATS
- `/run/neoland/` precisa existir antes do start — responsabilidade do módulo NixOS (Fase 4)

## Próximos Passos

- Fase B: integrar `MmapRegion` no `orchestrator.rs` — setar `pipeline_active` e `session_id` antes de chamar `POST /v1/pipeline/run`
- Fase B: Python pipeline lê `abort_requested` no início de cada step do orchestrator
- NixOS module: criar `/run/neoland/` como `RuntimeDirectory` no systemd service
