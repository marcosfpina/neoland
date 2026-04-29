# Neoland — Roadmap To Production

**Última atualização**: 2026-04-28  
**Objetivo deste documento**: ser a fonte única de tracking até produção.  
**Leitura atual do projeto**: **72/100 rumo a prod**  
**Regra**: quando README, checkpoints antigos e código divergirem, o código vence.

**Definição de prod para Neoland**: release com acesso público e condição de compartilhar o projeto publicamente de forma responsável.  
Isso significa não apenas “funciona em ambiente interno”, mas “pode ser exposto, demonstrado e adotado sem esconder riscos operacionais centrais”.

---

## Resumo Executivo

Neoland já tem um núcleo técnico forte:

- control plane Rust com auth, health, métricas, OpenAPI e rotas reais de agentes
- pipeline Python DSPy com contrato tipado e checkpoints ADR
- SSE já exposto no backend para eventos de pipeline
- mmap IPC entregue em base estrutural
- frontend reposicionado em torno de `Pipeline`, `Sessions`, `ADR` e `Services`
- suíte Rust validada localmente via `nix develop --command cargo test --lib --quiet`: **221 passed, 17 ignored**

O principal trabalho que falta para produção **não é fundação**. É alinhamento operacional:

- fechar gaps entre backend real e frontend consumindo apenas payload final
- remover dependências de narrativa herdada de Matrix onde não houver backend verdadeiro
- transformar observabilidade, deploy, runbooks e rollout em rotina verificável
- reduzir drift documental, hoje alto
- fechar os mínimos de exposição pública responsável: release discipline, defaults seguros, onboarding honesto e superfície pública coerente

Mas a ordem correta importa: **estabilização do produto principal vem antes do fechamento operacional**.  
Enquanto ainda houver bugs relevantes em TUI, fluxo principal e superfícies centrais, Neoland deve ser tratado como **pré-release técnico**, não como release candidate.

Há também uma trilha explícita de `DX verification`: não basta termos componentes, precisamos provar que eles funcionam conforme prometido nas docs e superfícies.  
Ver: [`docs/DX_ROADMAP.md`](/home/kernelcore/master/neoland/docs/DX_ROADMAP.md)

Há agora também uma trilha explícita de `LLM runtime alignment`, porque a stack de inferência precisa de uma topologia oficial e health correto entre `Neoland`, `securellm-bridge`, `ml-ops-api` e `llama.cpp/vLLM`.
Ver: [`docs/LLM_RUNTIME_ROADMAP.md`](/home/kernelcore/master/neoland/docs/LLM_RUNTIME_ROADMAP.md)

---

## Estado Integral Atual

### 1. Control Plane Rust — **85%**

**Confirmado no código**

- `POST /v1/agents/task`
- `GET /v1/agents/session/:id`
- `GET /v1/agents/health`
- `GET /v1/agents/events`
- `GET /v1/agents/events/:session`
- `GET /health`, `GET /ready`, `GET /live`
- OpenAPI e Swagger em `src/openapi.rs`

**Leitura**

Base pronta para beta operacional single-host. O trabalho restante aqui é mais de integração, rollout e endurecimento final do que de arquitetura base.

### 2. Pipeline Python DSPy — **80%**

**Confirmado no código**

- contrato espelhado entre `agents/neoland_agents/schemas/api.py` e `src/agents/client.rs`
- pipeline multiagente ativo em `agents/neoland_agents/app.py`
- ADR-019 e ADR-020 aceitas e implementadas parcialmente
- sessão persistida via `src/agents/session.rs`

**Gaps**

- `GET /v1/pipeline/session/{session_id}` no app Python segue stub
- Fase B do mmap ainda precisa integrar abort/progresso fino no fluxo completo
- streaming fino por stage ainda não está consumido ponta a ponta

### 3. Frontend Workbench — **55%**

**Confirmado no código**

- páginas alinhadas ao produto em `matrix/frontend/app/pipeline`, `sessions`, `adr`, `services`
- adapters tipados em `matrix/frontend/lib/neoland`
- ADR Vault já renderiza arquivos reais de checkpoint
- Services já usa health/readiness/liveness reais

**Gaps críticos**

- `PipelineLiveView` ainda afirma que stage streaming “não está exposto”, mas o backend já expõe SSE
- `PipelineRunner` ainda espera payload final, sem consumir `/v1/agents/events/:session`
- home ainda mistura workbench real com métricas derivadas do backend Matrix
- `app/api/neoland/stats/route.ts` depende de `NEOLAND_MATRIX_URL` e usa decisão `"accepted"`, enquanto o contrato Neoland trabalha com `approve | reject | defer | escalate`
- `Sessions` continua lookup-first porque não há listagem real de sessões

### 4. Operações, SRE e Deploy — **65%**

**Confirmado no código e docs**

- métricas Prometheus
- probes de health/readiness/liveness
- runbooks em `docs/runbooks/`
- backup scripts
- Nix-first setup

**Gaps**

- falta checklist de release única e verificável
- falta validação rotineira de restore, rollback e smoke de deploy
- falta convergir serviço Rust, pipeline Python, frontend e runtime dirs em um fluxo de operação simples
- falta convergir a topologia de inferência em uma única leitura operacional

### 5. Segurança e Governança — **80%**

**Confirmado**

- auth/RBAC
- audit logging
- Vault/secrets path
- validação/rate limit
- documentação de segurança e ADRs

**Gaps**

- mTLS/edge hardening não está fechado como gate operacional
- falta declarar claramente o mínimo aceitável de prod versus backlog enterprise

### 6. Documentação e Fonte de Verdade — **40%**

Hoje este é um dos maiores riscos de execução.

**Drifts encontrados**

- `ROADMAP.md` antigo falava em SSE como próximo passo, mas SSE já existe
- `README.md` vende 96/100 e features mais avançadas do que o caminho operacional validado
- `docs/PROGRESS.md`, `docs/PRODUCTION_CHECKPOINT.md` e `docs/PROJECT_SNAPSHOT.md` usam percentuais e escopos diferentes
- houve drift documental sobre o path do `matrix`, mas o path do projeto não deve ser renomeado por este roadmap

---

## O Que Está Bloqueando Produção De Verdade

### P-1. Bugs e inconsistências na superfície principal ainda bloqueiam release

Impacto:
- não adianta endurecer operação se a experiência central ainda falha
- release público com TUI e fluxo principal instáveis gera retrabalho e perda de confiança
- smoke operacional perde valor se o produto-base ainda não está estável

### P0. Frontend ainda não usa o streaming que o backend já expõe

Impacto:
- operador não vê progressão real do pipeline
- produto vende “live view” sem consumir a fonte mais importante do sistema

### P1. Dependência residual do mundo Matrix onde Neoland deveria ser a verdade

Impacto:
- risco de dashboards mostrarem números inconsistentes
- ambiguidade de ownership entre workbench Neoland e analytics herdado

### P2. Falta de um caminho único de deploy e validação

Impacto:
- projeto parece mais pronto do que realmente está para operação contínua
- ambiente de dev e ambiente de release ainda não estão fechando em um único ritual simples

### P2.5. Runtime de inferência ainda tem naming e health ambíguos

Impacto:
- difícil saber se o target principal do Neoland é `securellm-bridge`, `ml-ops-api` ou `llama.cpp`
- `doctor` pode acusar erro no endpoint errado
- troubleshooting fica lento e sujeito a falsas conclusões

### P3. Documentação inflada e desalinhada

Impacto:
- decisões ruins de priorização
- onboarding mais lento
- percepção falsa de readiness

### P4. Critério de public release ainda não está explicitado no produto inteiro

Impacto:
- difícil dizer com honestidade “isto já pode ser compartilhado publicamente”
- risco de expor superfícies incompletas como se fossem maduras
- messaging, defaults e demo path ainda podem prometer mais do que a operação sustenta

---

## Roadmap De Execução Até Prod

## Fase -1 — Estabilização Do Produto Principal
**Status**: `in_progress`  
**Objetivo**: fechar bugs e regressões na experiência central antes do hardening de release.

- [ ] corrigir bugs relevantes no TUI
- [ ] estabilizar fluxo principal de execução e inspeção
- [ ] reduzir inconsistências visíveis entre superfícies do produto
- [ ] garantir que os caminhos principais funcionem de forma repetível em ambiente de desenvolvimento real
- [ ] cruzar bugs e regressões com a trilha de DX verification

**Saída da fase**

- produto principal utilizável sem comportamento quebrado recorrente
- base confiável para iniciar checklist real de release

## Fase 0 — Verdade Única e Corte de Escopo
**Status**: `in_progress`  
**Objetivo**: parar drift e definir o que é “prod” para Neoland agora.

- [x] consolidar um roadmap único na raiz
- [ ] alinhar `README.md`, `docs/PROGRESS.md` e `docs/PROJECT_SNAPSHOT.md` a esta leitura
- [ ] alinhar documentação antiga ao path já corrigido do `matrix`, sem mudar a estrutura atual do repo
- [ ] declarar explicitamente o alvo de prod: **public release responsável do workbench Neoland**, com escopo controlado e sem claims infladas

**Saída da fase**

- uma narrativa única de produto
- zero contradição sobre o que já existe e o que falta

## Fase 1 — Verdade Do Produto Nas Superfícies
**Status**: `next`  
**Objetivo**: fazer o centerpiece do produto refletir a verdade operacional.

- [ ] consumir `GET /v1/agents/events/:session` no frontend
- [ ] renderizar `StageStarted`, `StageDone`, `StageOutput`, `AdrCheckpoint`, `PipelineError`
- [ ] parar de mostrar copy dizendo que stage streaming não existe
- [ ] persistir no client o `session_id` retornado para reconectar ao stream
- [ ] definir estado degradado quando SSE falhar, sem mockar progresso

**Gate para avançar**

- operador consegue iniciar task e ver progressão real por stage sem refresh

## Fase 2 — Session e ADR Forensics Fechados
**Status**: `next`  
**Objetivo**: tornar sessões e checkpoints navegáveis de ponta a ponta.

- [ ] enriquecer `Sessions` com fluxo claro de entrada a partir do resultado do pipeline
- [ ] expor, no backend, o mínimo necessário para browsing real de sessões se isso for requisito imediato
- [ ] ligar melhor sessão -> checkpoint -> ADR Vault
- [ ] padronizar leitura dos arquivos de checkpoint e campos obrigatórios

**Gate para avançar**

- qualquer execução real pode ser reconstituída por sessão e checkpoint

## Fase 3 — Desacoplamento de Matrix Residual
**Status**: `next`  
**Objetivo**: deixar claro o que é infra reaproveitada e o que é dependência de produto.

- [ ] revisar `app/page.tsx` e `app/api/neoland/stats/route.ts`
- [ ] remover ou reclassificar métricas que dependem de backend Matrix não canônico
- [ ] normalizar decisão `accepted` para o vocabulário Neoland ou eliminar a dependência
- [ ] manter analytics herdado apenas onde houver contrato explícito e ownership claro

**Gate para avançar**

- frontend principal não depende de payload sem fonte de verdade Neoland

## Fase 4 — Operação Single-Host Confiável
**Status**: `planned`  
**Objetivo**: transformar a stack atual em serviço operável sem improviso, depois da estabilização do produto principal.

- [ ] fechar topologia oficial de inferência e gateway
- [ ] fechar runtime dirs, envs e secrets para Rust + Python + frontend
- [ ] validar smoke de boot completo
- [ ] validar backup e restore de sessão/checkpoints
- [ ] documentar rollout, rollback e incident response mínimos
- [ ] definir pacote único de health checks para preflight
- [ ] definir baseline de exposição pública: portas, auth obrigatória, envs inseguros proibidos, comportamento seguro por default

**Gate para avançar**

- subir stack, validar health, rodar task real e recuperar de falha com roteiro curto

## Fase 5 — Release Candidate
**Status**: `planned`  
**Objetivo**: preparar go/no-go real para release público responsável.

- [ ] smoke E2E do fluxo principal
- [ ] validar alertas e métricas mínimas em ambiente de staging
- [ ] congelar contratos do frontend com backend real
- [ ] revisar hardening que é obrigatório para primeira release, separando do backlog enterprise
- [ ] revisar README, quickstart, positioning e screenshots sob ótica de divulgação pública honesta
- [ ] definir checklist de “safe to share”: instalação, auth, health, known limitations, recovery path

**Gate para avançar**

- release candidata passa checklist completo sem intervenção manual ad hoc

## Fase 6 — Produção
**Status**: `planned`  
**Objetivo**: publicar com escopo controlado e responsabilidade operacional.

- [ ] deploy em ambiente alvo
- [ ] soak period com tasks reais
- [ ] revisão de incidentes da primeira semana
- [ ] replanejamento para multi-host, ledger mais profundo e expansão de analytics
- [ ] publicação externa com documentação e limitações explícitas

---

## Backlog Após Prod Inicial

Isto **não** deve bloquear o primeiro corte de produção:

- multi-tenancy
- compliance automation completa
- mTLS end-to-end em todos os planos
- analytics avançado de ranking
- automações enterprise de adr-ledger
- mobile / desktop app

---

## Tracking Board

| ID | Item | Status | Criticidade |
|----|------|--------|-------------|
| P-1 | Corrigir bugs do TUI e estabilizar superfície principal | In progress | Alta |
| DX-1 | Inventariar promessas do produto por superfície | Next | Alta |
| DX-2 | Validar jornadas principais Nix e Ubuntu | Next | Alta |
| DX-3 | Classificar claims em verified / partial / stale | Next | Alta |
| P0 | Unificar documentação e score real | In progress | Alta |
| P1 | Consumir SSE no Pipeline Live View | Next | Alta |
| P2 | Fechar fluxo sessão -> ADR -> forensics | Next | Alta |
| P3 | Remover dependência ambígua de Matrix no core UI | Next | Alta |
| P4 | Definir deploy single-host repetível | Planned | Alta |
| P5 | Validar backup/restore e rollback | Planned | Alta |
| P6 | Checklist de release candidate | Planned | Alta |
| P7 | Ajustar README para não superestimar readiness | Planned | Média |
| P8 | Alinhar docs antigas ao path já corrigido do `matrix` | Planned | Média |
| P9 | Revisar home para foco total em workbench operacional | Planned | Média |
| P10 | Session listing ou alternativa oficial | Planned | Média |
| P11 | Integrar Fase B do mmap no fluxo real | Planned | Média |
| P12 | Separar backlog enterprise do primeiro corte | In progress | Média |
| LLM-1 | Congelar topologia `Neoland -> SecureLLM Bridge -> ml-ops-api -> llama.cpp/vLLM` | In progress | Alta |
| LLM-2 | Corrigir semântica de `ml_api_url` no Neoland | Next | Alta |
| LLM-3 | Ajustar `doctor` para health do gateway real | Next | Alta |
| LLM-4 | Subir `securellm-api-server` com `ml-ops` habilitado | Next | Alta |
| LLM-5 | Fixar `LLAMACPP_URL` e forwarding no `ml-ops-api` | Next | Alta |
| LLM-6 | Validar E2E completo com task real | Planned | Alta |

---

## Critério De Produção

Consideraremos Neoland pronto para o primeiro corte quando estes pontos forem verdadeiros ao mesmo tempo:

- TUI e fluxo principal não apresentam bugs relevantes que comprometam a demo, uso básico ou avaliação pública
- uma task real inicia no frontend e mostra progressão real até decisão final
- sessão e checkpoint resultantes podem ser inspecionados sem dados simulados
- health, readiness e liveness refletem o estado real da stack
- deploy e rollback funcionam com procedimento curto e reproduzível
- documentação principal não contradiz o comportamento do código
- o projeto pode ser mostrado publicamente sem depender de explicações defensivas sobre flows quebrados, defaults inseguros ou superfícies enganosas
- a experiência pública mínima de instalação, autenticação e uso inicial está clara e suportável

---

## Regras De Atualização

- atualizar este arquivo ao fechar qualquer item P0-P12
- não subir score geral sem evidência operacional ou teste correspondente
- registrar feature como pronta só quando estiver ligada ao fluxo real, não apenas implementada isoladamente
- usar percentuais por stream apenas quando houver critério observável

---

## Próxima Janela Recomendada

Ordem sugerida para o próximo ciclo curto:

1. P-1 — estabilizar TUI e fluxo principal
2. P1 — SSE real no frontend
3. P3 — limpar dependência ambígua de Matrix na home e stats
4. LLM-1/LLM-3 — fechar topologia e health do runtime de inferência
5. P2 — fechar navegação entre pipeline, sessão e ADR
6. P4/P5 — ritual de deploy, restore e rollback

Se fizermos isso, Neoland sai de “tecnicamente impressionante, mas ainda instável na superfície principal” para “workbench control-plane estável e pronto para fechamento operacional de release”.
