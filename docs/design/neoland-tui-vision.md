# Visão: TUI Agentica e Eficiente

Se o objetivo é uma TUI 100% "Agêntica" e orientada à eficiência máxima, o paradigma de "Painéis de Configuração Manuais" (como o `Llama Manager` de `Ctrl+L`) está morto.

## Por que o `Llama Manager` não faz sentido?
Em um ambiente Agêntico, o humano não deve ter que apertar `Ctrl+L`, escolher "Llama-3-8B.gguf" numa lista, mandar dar Run, esperar subir na memória e aí sim pedir a tarefa.
O humano diz **O QUE** ele quer. O Agente (ou o Control Plane Rust) descobre **COMO** e **ONDE** rodar.

## O Que Torna uma TUI "Agêntica"?
1. **Zero Configuration no Cliente:** A TUI é apenas o "Canal de Luz" entre o Cérebro (DSPy/Rust) e as Mãos (MCP/Humano).
2. **Context-Awareness Automático:** O Agente avalia a complexidade da tarefa ("É um `mkdir`? Uso Llama-3 local." / "É arquitetura RAG pesada? Uso DeepSeek/Claude").
3. **Comandos Livres (K-9s style):** Em vez de sub-telas de configuração, tudo pode ser disparado do Prompt com prefixos (`/model llama-3`, `/steer`, `/approve`).

## A Proposta de Refatoração
1. **Arrancar o `Llama Manager` da TUI:** Excluir os arquivos `llama_manager.rs` e `llama_manager_logic.rs`.
2. **Delegar pro Backend:** Se o humano realmente quiser forçar a troca de modelo, ele digita `/model gemma-2` e a TUI manda um POST pro `neoland server`. O *servidor* lida com o `securellm-bridge` ou script do `llamaswap`.
3. **Limpeza Extrema:** A TUI passa a ter apenas 1 Modo de Tela (`Workstation/Canvas`). Reduzimos centenas de linhas de código que prendem o TUI à máquina física, tornando-o puro e portável via SSH.
