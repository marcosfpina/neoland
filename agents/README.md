# Neoland Agents — DSPy Multi-Agent Pipeline

Este diretório contém o cérebro cognitivo do Neoland: um pipeline de múltiplos agentes construído com **DSPy** e **FastAPI**. Ele é responsável por processar tarefas complexas, realizar pesquisas (RAG), e gerar decisões arquiteturais (ADRs) de forma estruturada.

## 🏗️ Arquitetura do Pipeline

O pipeline segue uma hierarquia de decisão inspirada em uma equipe de engenharia real:

1.  **Junior Agent (`junior.py`):** Recebe a tarefa inicial e o contexto do RAG. Propõe uma hipótese e identifica incertezas.
2.  **Senior Agent (`senior.py`):** Revisa a proposta do Junior. Valida partes da solução, rejeita outras e decide se a complexidade exige a intervenção de um Arquiteto.
3.  **Architect Agent (`architect.py`) [Condicional]:** Executado apenas se escalado pelo Senior. Foca na integridade estrutural, composicionalidade e riscos de longo prazo.
4.  **Tech Leader Agent (`tech_leader.py`):** Consolida todos os inputs anteriores e toma a decisão final, gerando um ADR (Architectural Decision Record) e itens de ação.

## ⚡ Zero-Copy IPC (Rust ↔ Python)

Uma das características mais avançadas do Neoland é a comunicação via **Shared Memory** localizada em `neoland_agents/ipc/flags.py`.

*   **O que é:** Um arquivo mapeado em memória (`mmap`) compartilhado entre o processo Rust (Control Plane) e o processo Python (DSPy Pipeline).
*   **Vantagem:** Permite sincronização de estado (como flags de aborto, níveis de risco e confiança do agente) com latência próxima de zero e sem o overhead de serialização JSON/gRPC para flags críticas.
*   **Layout:** O layout binário de 64 bytes é rigorosamente mantido em sincronia com `src/agents/flags.rs` no backend Rust.

## 🚀 Como Executar

### Pré-requisitos
*   Python 3.13+
*   Poetry (recomendado) ou ambiente virtual configurado via Nix (`nix develop`).

### Instalação
```bash
cd agents
poetry install
```

### Inicialização
O pipeline roda como um serviço FastAPI na porta `8001` por padrão:
```bash
# Via script de conveniência (se disponível no root)
# ou manualmente:
poetry run uvicorn neoland_agents.app:app --host 0.0.1 --port 8001
```

## 🛠️ Principais Módulos

*   **`pipeline/orchestrator.py`:** O motor que encadeia os agentes e gerencia o fluxo de dados.
*   **`pipeline/checkpoint.py`:** Sistema de persistência que salva o estado de cada execução (JSON + SQL) para auditoria e recuperação de desastres.
*   **`rag/retriever.py`:** Integração com o Vector Store para fornecer contexto relevante aos agentes.
*   **`signatures/`:** Definições tipadas (DSPy Signatures) para as entradas e saídas de cada LLM.

## 📝 Variáveis de Ambiente

Configuráveis via `.env` ou exportadas no shell:

| Variável | Padrão | Descrição |
| :--- | :--- | :--- |
| `NEOLAND_LLM_PROVIDER` | `openai` | Provedor de LLM (openai, anthropic, etc.) |
| `NEOLAND_LLM_MODEL` | `gpt-4o-mini` | Modelo de linguagem principal |
| `LLM_API_KEY` | - | Chave de API para o provedor |
| `DATABASE_URL` | - | URL do PostgreSQL para persistência |
| `NEOLAND_SHM_PATH` | `/run/neoland/agent-flags.shm` | Caminho para o arquivo de memória compartilhada |

## 🧪 Testes

A suíte de testes está em `tests/` e utiliza `pytest`:
```bash
# Testes de contrato (rápidos)
poetry run pytest -m contract

# Testes de integração (requerem LLM/DB)
poetry run pytest -m integration
```
