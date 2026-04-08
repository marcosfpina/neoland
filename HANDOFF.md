# HANDOFF.md

## Contexto salvo em 2026-04-08

### Objetivo

Construir `neoland-ui` como repo separado em `~/master/neoland-ui`, mantendo `~/master/neoland` limpo.

O foco do frontend foi corrigido durante a análise:
- `neoland-ui` nao e um chat genérico
- `neoland-ui` e o painel operacional do control plane Neoland
- a peça central e `Pipeline Live View`

### Repos correlacionados

- Backend/control plane: `~/master/neoland`
- Frontend/gateway: `~/master/neoland-ui`

### Arquivos importantes ja preparados em `~/master/neoland-ui`

- `NEOLAND-UI.md`
- `AGENTS.md`

`AGENTS.md` ja inclui:
- posicionamento do produto
- correlacao entre os dois repos
- ownership de backend vs frontend
- regras de contrato
- workflow cross-repo
- direcao de UX e ordem de implementacao

### Base real a adaptar

Existe uma base archived confirmada em:

- `/home/kernelcore/dev/_ARCHIVE_2025_12_10/frontend/backend/`

Ela contem:
- Next.js App Router
- shadcn/ui
- Tailwind
- Bun runtime
- socket.io
- layout/dashboard generico que precisa ser adaptado para Neoland

### Backend verificado no repo `~/master/neoland`

Estas integracoes foram verificadas no codigo:

- `POST /v1/agents/task`
- `GET /v1/agents/session/:id`
- `GET /v1/agents/health`
- `GET /health`
- `GET /ready`
- `GET /live`

Arquivos-chave:
- `~/master/neoland/src/server/mod.rs`
- `~/master/neoland/src/agents/orchestrator.rs`
- `~/master/neoland/src/agents/client.rs`
- `~/master/neoland/agents/neoland_agents/app.py`
- `~/master/neoland/agents/neoland_agents/schemas/api.py`
- `~/master/neoland/docs/ADR/ADR-019-multi-agent-dspy-pipeline.md`
- `~/master/neoland/docs/ADR/ADR-020-mmap-ipc.md`

### Leitura correta do projeto

O centro atual do Neoland nao e so TUI/chat.

O foco real identificado no codigo e:
- control plane em Rust
- pipeline multiagente em Python DSPy
- sessao persistida
- ADR/checkpoints
- eventos/NATS
- futura telemetria/abort via mmap IPC

### Proximo passo recomendado

Iniciar no repo `~/master/neoland-ui` com esta sequencia:

1. copiar/adaptar a base archived
2. remover branding e features genericas do dashboard antigo
3. aplicar a estrutura definida em `NEOLAND-UI.md`
4. criar `lib/types.ts` espelhando contratos reais do backend
5. criar gateway Bun com rotas `pipeline`, `services`, `adr`, `settings`
6. atacar primeiro a vertical slice central: `/pipeline`

### Observacao importante

Durante uma suposicao inicial errada, foi criada uma pasta vazia `neoland-ui/` dentro de `~/master/neoland`.

Estado esperado:
- ela nao faz parte da estrategia final
- o repo correto para o frontend e `~/master/neoland-ui`
- se necessario, avaliar depois a remocao dessa pasta no repo backend
