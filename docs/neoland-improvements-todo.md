# Plano de Execução: Melhorias Contínuas (Neoland)

Este documento detalha um plano de ação estruturado para abordar as oportunidades de melhoria identificadas no projeto Neoland, priorizando a estabilidade e a prontidão para produção (buscando atingir 100% de *Production Readiness*).

## Fase A: Estabilidade e Prevenção de Falhas (Prioridade Alta)

Estas tarefas focam em evitar problemas críticos em produção, como vazamentos de memória e falhas silenciosas.

- [x] **A.1. Implementar LRU Cache no VectorStore (`src/storage/vector_store.rs`)**
  - **Problema:** O armazenamento em memória atual cresce indefinidamente.
  - **Ação:** Substituir ou envelopar o armazenamento atual (provavelmente um `HashMap` ou vetor) com uma estrutura de dados de evicção (ex: crate `lru` ou customizada) baseada em capacidade máxima ou tempo de vida (TTL).
  - **Critério de Aceite:** O consumo de memória do VectorStore em memória não deve ultrapassar o limite configurado sob carga pesada.

- [x] **A.2. Finalizar Comando `neoland doctor` (`src/cli.rs` / `src/commands.rs`)**
  - **Problema:** A TUI sugere o uso de `neoland doctor` em erros de conexão, mas a ferramenta de diagnóstico não está completa ou devidamente detalhada.
  - **Ação:** Expandir o comando para verificar conectividade de rede (gRPC, REST), acesso ao banco de dados (PostgreSQL/pgvector), disponibilidade do Vault e status dos backends de inferência (ml-offload-api).
  - **Critério de Aceite:** O usuário pode rodar `neoland doctor` e obter um relatório claro do que está falhando no ambiente.

## Fase B: Observabilidade e Testes (Prioridade Média-Alta)

Estas tarefas melhoram a visibilidade do sistema e garantem que regressões não ocorram.

- [ ] **B.1. Habilitar OpenTelemetry Tracing (`src/logging.rs` e Python Agents)**
  - **Problema:** Dificuldade de rastrear a origem de latências altas (citado em `high-latency.md`), especialmente cruzando a fronteira Rust -> Python.
  - **Ação:** Configurar o exportador OTLP no Rust (crate `opentelemetry`) e no Python (`opentelemetry-sdk`). Propagar o `traceparent` (Correlation IDs) nos cabeçalhos gRPC/REST.
  - **Critério de Aceite:** Traces completos visíveis em uma ferramenta compatível (Jaeger/Prometheus/Grafana) desde a TUI até a resposta do Agente.

- [ ] **B.2. Corrigir Testes Ignorados e Atingir 80% de Cobertura**
  - **Problema:** O `docs/neoland-roadmap.md` e o `docs/neoland-progress.md` citam ~26 testes ignorados (incluindo stubs gRPC).
  - **Ação:** Investigar o porquê de os testes em `tests/` e `src/` estarem com `#[ignore]`. Corrigi-los e adicionar testes unitários para a camada TUI e integrações de fallback.
  - **Critério de Aceite:** `cargo test` passa 100% (sem `ignored` críticos) e cobertura sobe para >= 80%.

## Fase C: Escalabilidade, Performance e Segurança (Prioridade Média)

Estas tarefas preparam o sistema para escalar de forma segura com mais usuários, agentes e poder computacional.

- [ ] **C.1. Suporte para GPUs e Endpoints Externos (LLMs)**
  - **Problema:** A inferência atual depende muito de CPU local, gerando gargalos de performance e limitando a escalabilidade. O gerenciamento dinâmico (como Nvidia Brev) pode ser caótico se não estruturado corretamente.
  - **Ação:** Adicionar suporte nativo à configuração de endpoints externos (compatíveis com OpenAI/vLLM). Estruturar um pipeline de roteamento robusto via Circuit Breaker para alternar fluidamente entre clusters GPU remotos e o fallback local em CPU, abstraindo a complexidade da infraestrutura externa.
  - **Critério de Aceite:** O sistema roteia requisições para GPUs remotas de forma transparente, executando *failover* para o fallback local em caso de falha externa.

- [ ] **C.2. Gerenciamento de Conexões (PgBouncer)**
  - **Problema:** O aumento de agentes Python causará exaustão de conexões no PostgreSQL.
  - **Ação:** Atualizar os manifestos/arquivos do Kubernetes ou docker-compose (se aplicável) para incluir o PgBouncer. Configurar o Rust e o Python para apontar para a porta do PgBouncer.
  - **Critério de Aceite:** Sistema suporta 100+ agentes simultâneos sem estourar o limite de conexões do DB.

- [ ] **C.3. Proteção Automática (Rate Limiting Dinâmico / IP Blocking)**
  - **Problema:** O documento de mitigação de força bruta prevê bloqueios automáticos, mas a implementação pode estar incompleta.
  - **Ação:** Expandir o RateLimiter em `src/server/mod.rs` para banir temporariamente IPs que excedam a cota de forma agressiva (Fail2Ban like).
  - **Critério de Aceite:** IPs maliciosos são colocados em quarentena automaticamente.

- [ ] **C.4. Pré-aquecimento de Modelos (Cold Starts)**
  - **Problema:** Modelos Qwen locais demoram a responder no primeiro uso.
  - **Ação:** Criar um script ou endpoint em `ml-offload-api` que force o carregamento do modelo na inicialização do serviço.
  - **Critério de Aceite:** O primeiro request após o deploy tem latência comparável aos subsequentes.

## Fase D: Documentação Abrangente (Prioridade Contínua)

Garantir que os múltiplos subprojetos do Neoland sejam mantíveis, compreensíveis e fáceis de integrar.

- [ ] **D.1. Documentar Agentes Python (`agents/README.md`)**
  - **Ação:** Detalhar a arquitetura baseada em DSPy, o pipeline RAG, a comunicação IPC com o Rust e como adicionar novos agentes/skills.
- [ ] **D.2. Documentar Backend Core (`src/README.md`)**
  - **Ação:** Explicar o ciclo de vida da engine Rust, a ponte de ML (`ml_offload`), gerenciamento de estado (`AppState`) e o fluxo de inferência e fallback.
- [ ] **D.3. Documentar Infraestrutura e Observabilidade (`deploy/README.md`)**
  - **Ação:** Guias sobre o uso de Nix, Prometheus, Vault, SOPS, e os manifestos Kubernetes associados.
- [ ] **D.4. Documentar Sistema de Skills (`skills/README.md`)**
  - **Ação:** Criar um guia padrão sobre como escrever, testar e integrar novas skills (ex: `Linux_Server_Master`, `security-architect`) para os agentes Python.
