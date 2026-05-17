# Llama Manager Refactor (Dynamic Model Selection)

## O Problema Atual
O painel `Llama Manager` (`Ctrl + L` na TUI) que permite rodar um modelo local usando o `llama.cpp` foi projetado de forma extremamente acoplada (Hardcoded) ao sistema operacional:
1. Ele lista arquivos físicos da pasta `/var/lib/ml-models/llamacpp/models` via system calls (`fs::read_dir`).
2. Ele tenta iniciar/matar um serviço `systemd` (`llamacpp-swap.service`) e lança binários diretamente via `Command::new("llama-server")`.

Esse design vai contra o resto do Neoland (que é voltado a chamadas HTTP/gRPC). Você mencionou corretamente: *"Deveríamos ter usado uma API, o llamaswap também faz essa troca dinâmica."*

## O SecureLLM Bridge
Na pasta `securellm-bridge/crates/api-server/src/routes/models.rs`, já existe um endpoint `GET /v1/models` perfeitamente compatível com o formato OpenAI. E esse Bridge é feito exatamente para unificar tudo (llamacpp, vllm, ml_ops).

## A Solução Elegante (A Refatoração)
Em vez de lermos arquivos locais de `gguf` e matarmos processos do SO, nós faremos o `Llama Manager` se tornar uma "Interface Cliente" pro **SecureLLM Bridge**.

1. **`scan_models`**: Em vez de `fs::read_dir`, fará uma chamada `GET http://localhost:8080/v1/models`.
2. **Troca de Modelo (Run/Stop)**: Em vez de iniciar o processo na força bruta, enviaremos um comando via API ou confiaremos no roteador dinâmico do `securellm-bridge` ou do `llamaswap` (que aparentemente é um comando que você possui no ambiente ou vai criar) para trocar o contexto na memória.
