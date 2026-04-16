# ADR-021: Neoland Core Architecture Baseline

**Status**: Accepted  
**Date**: 2026-04-16  
**Decision Makers**: antigravity, marcosfpina  
**Fase**: Ciclo 1 — Fase B (Integração de Ecossistema)

---

## Contexto

Após a execução do scan de métricas global via Cerebro, consolidou-se a necessidade de formalizar a arquitetura do **Neoland** como o "Control Plane" central de um ecossistema distribuído. O projeto cresceu de um simples orquestrador de agentes para uma plataforma multi-camadas envolvendo Rust, Python, Next.js e IPC de alta performance.

### Dados de Baseline (Métricas Cerebro 2026-04-16)
- **Saúde**: 95.5 / 100
- **Volume**: ~105k LoC em 456 arquivos
- **Linguagens**: Rust (Core), Python (Pipeline), TypeScript (Frontend), Shell/Nix (Infra)
- **Security**: 80.0 (Baseline original) → 100.0 (Pós-remediação SOPS)

## Decisão

Formalizar o modelo de **4 Camadas de Operação** como o padrão para toda a evolução do Neoland:

### 1. Interface Layer (Neoland UI)
- **Tecnologia**: Next.js 15, React 19, Tailwind CSS.
- **Responsabilidade**: Visualização operacional, gerenciamento de sessões e inspeção de ADRs.
- **Segurança**: Zero-Trust via API Keys gerenciadas por **SOPS** (`NEOLAND_USER_API_KEY`).

### 2. Control Plane (Rust Core)
- **Tecnologia**: Rust (Axum, Tonic, Tokio).
- **Responsabilidade**: Orquestração de tarefas, gerenciamento de mmap IPC, ponte gRPC/REST e persistência de sessões no PostgreSQL.
- **Design Core**: `src/server/mod.rs` e `src/agents/orchestrator.rs`.

### 3. Pipeline Intelligence (DSPy Agents)
- **Tecnologia**: Python 3.12+, DSPy, FastAPI.
- **Responsabilidade**: Execução lógica da tarefa em estágios (Junior → Senior → Architect → TechLeader).
- **Isolamento**: PID isolation via systemd-run e mensageria via NATS.

### 4. Governance & Ledger (ADR Vault)
- **Tecnologia**: File-based + `adr-ledger` integrações.
- **Responsabilidade**: Persistência imutável de decisões arquiteturais assinadas criptograficamente.

## Alternativas Consideradas

- **Monolito Python**: Rejeitado por falta de performance no control plane e dificuldades em gerenciar IPC compartilhado binário.
- **Kubernetes-only**: Rejeitado para manter o "Nix-first" e a capacidade de rodar localmente no NixOS com overhead mínimo.

## Consequências

**Positivas**:
- Separação clara de responsabilidades entre orquestração (Rust) e inteligência (Python).
- Documentação "at scale" agora possui um ponto de referência centralizado.
- Melhoria imediata na postura de segurança com o uso de SOPS no frontend.

**Negativas**:
- Aumento da complexidade de deployment (necessita de Rust, Python e Node.js runtimes).
- Necessidade de manter contratos de API (gRPC/mmap) sincronizados entre linguagens.

## Próximos Passos

1. Gerar o `PROJECT_SNAPSHOT.md` para consumo automatizado.
2. Integrar as métricas do Cerebro diretamente no dashboard do Neoland UI.
3. Iniciar o Ciclo 1 — Fase C (Integração Phantom para scans em tempo real no pipeline).
