# Neoland Core — Backend Engine em Rust

Este diretório contém a implementação principal do Neoland, desenvolvida em Rust para garantir alta performance, segurança de memória e concorrência robusta. O backend atua como o Control Plane do sistema, gerenciando a inferência de LLMs, o armazenamento vetorial e a orquestração de agentes.

## 🏗️ Arquitetura do Sistema

O backend é modular e estruturado em camadas:

### 1. Motor de Inferência (`engine.rs`)
*   **Local Engine:** Utiliza a biblioteca `candle` para inferência local (CPU/GPU) do modelo Qwen 1.8B (GGUF).
*   **Context Injection:** Integração nativa com o Vector Store para injeção de contexto RAG em tempo real.
*   **Streaming:** Suporte a streaming de tokens via callbacks, otimizado para interfaces TUI e gRPC.

### 2. Camada de Servidor (`server/`)
*   **Dual-Stack:** Provê simultaneamente uma interface **gRPC** (alta performance para comunicação interna/TUI) e uma interface **REST/Axum** (compatível com OpenAI para integrações externas).
*   **Segurança Avançada:**
    *   **Auth (`auth.rs`):** Sistema RBAC com 3 níveis de permissão.
    *   **Audit (`audit.rs`):** Log imutável de ações críticas e detecção de ataques de força bruta.
    *   **Rate Limiting:** Controle de vazão dinâmico por IP ou API Key.
*   **Observabilidade:** Endpoints de `/metrics` (Prometheus) e `/health` (Liveness/Readiness probes).

### 3. Gerenciamento de LLMs (`llm/`)
*   **Unified Client:** Uma abstração que unifica provedores locais e remotos.
*   **Resiliência:** Implementa **Circuit Breakers** e estratégias de roteamento baseadas em latência EMA (*Exponential Moving Average*).
*   **Fallbacks:** Alternância automática entre backends de alta performance (GPU) e backends de segurança (locais) em caso de falha.

### 4. Armazenamento e RAG (`storage/`, `nlp.rs`)
*   **Hybrid Vector Store:**
    *   **In-Memory:** Para buscas ultrarrápidas durante a geração.
    *   **Persistent (pgvector):** Armazenamento de longo prazo com suporte a busca semântica em PostgreSQL.
*   **Zero-Copy IPC:** Implementação do lado Rust para a memória compartilhada usada na comunicação com os agentes Python.

### 5. Interface TUI (`tui/`)
*   Uma interface de terminal moderna e responsiva para interação direta com o sistema, comandos de gerenciamento e chat.

## 🛠️ Tecnologias Principais

*   **Tokio:** Runtime assíncrono para I/O escalável.
*   **Tonic / Axum:** Frameworks para gRPC e REST.
*   **Candle:** Framework de ML minimalista da HuggingFace.
*   **SQLx:** Acesso seguro e tipado ao PostgreSQL.
*   **Prometheus:** Coleta de métricas de performance e uso.

## 🚀 Desenvolvimento e Build

O projeto utiliza o sistema de build do Cargo, mas é recomendado o uso do **Nix** para garantir um ambiente reprodutível:

```bash
# Build do projeto
cargo build --release

# Executar testes (alguns requerem modelos locais)
cargo test -- --ignored

# Executar o servidor
cargo run --bin neoland-server
```

## 📝 Variáveis de Ambiente Críticas

| Variável | Descrição |
| :--- | :--- |
| `DATABASE_URL` | Conexão com PostgreSQL/pgvector |
| `VAULT_TOKEN` | Token para integração com HashiCorp Vault |
| `NEOLAND_SHM_PATH` | Caminho para a memória compartilhada de IPC |
| `AUDIT_LOG_PATH` | Local dos logs de auditoria |
