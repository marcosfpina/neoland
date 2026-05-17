# Especificação da Feature: Llama.cpp TUI Manager

## 1. Visão Geral
Esta feature adicionará uma nova aba/módulo ao **Neoland TUI** para gerenciamento interativo, visualização e depuração de modelos do `llama.cpp` (LLMs locais). O objetivo é substituir scripts Bash improvisados por uma interface rica e integrada ao ecossistema Neoland.

## 2. Requisitos de Negócio e Contexto do Sistema
*   **Caminho Alvo Obrigatório:** O diretório fonte da verdade para os modelos GGUF é **`/var/lib/ml-models/llamacpp/models/`**. Este caminho é fixado pela raiz do Flake do NixOS.
*   **Permissões:** Os arquivos operam sob o usuário e grupo `llamacpp` ou `llamacpp-swap`. O Neoland TUI, ao gerenciar serviços de sistema (ex: parar o serviço ativo para liberar porta), precisará de elevação de privilégios (`sudo` ou `polkit`) ou permissão via configuração do NixOS para operar chamadas do `systemctl`.
*   **Serviço Conflitante:** O serviço padrão gerenciado pelo NixOS é o `llamacpp-swap.service`. Antes de inicializar um modelo no modo interativo (debug), a TUI deve garantir que este serviço seja parado, evitando o erro `Address already in use` (porta 8081 ocupada).

## 3. Funcionalidades Principais (TUI)

### 3.1. Painel de Descoberta (Model Browser)
*   **Listagem Dinâmica:** A TUI deve usar `tokio::fs` para escanear de forma assíncrona o diretório alvo e listar todos os arquivos terminados em `.gguf` e `.bin`.
*   **Navegação:** O usuário deve conseguir navegar pela lista com as setas do teclado (`Up`/`Down`).
*   **Extração de Metadados (GGUF):** Ao selecionar um modelo na lista, o painel direito deve exibir metadados básicos (Tamanho do arquivo, Nome, versão da quantização - se extraível do nome ou via crate como `gguf-rs`).

### 3.2. Painel de Configuração (Flags & Toggles)
*   Menu de opções interativo (Checkboxes) renderizado via Ratatui para ativar/desativar as seguintes flags do `llama-server`:
    *   `[x] --flash-attn` (Padrão: ON)
    *   `[x] --no-kv-offload` (Padrão: ON)
    *   `[x] --cont-batching` (Padrão: ON)
    *   `[x] --embeddings` (Padrão: ON)
    *   `[x] --metrics` (Padrão: ON)
*   Configurações numéricas editáveis (Inputs de texto):
    *   `--ctx-size` (Padrão: 8192)
    *   `--gpu-layers` (Padrão: 40)
    *   `--threads` (Padrão: 12)
    *   `--port` (Padrão: 8081)

### 3.3. Painel de Execução e Logs (Debug Mode)
*   **Botão Virtual "Run/Debug"**: Ao ser pressionado (ex: tecla `R` ou `Enter`), a TUI deve:
    1.  Executar `sudo systemctl stop llamacpp-swap.service`.
    2.  Fazer um spawn (spawn assíncrono via `tokio::process::Command`) do binário do `llama-server` passando o caminho do modelo e todas as flags selecionadas.
*   **Captura de Logs (Stdout/Stderr):** A aba inferior deve capturar o output da thread do `llama-server` em tempo real e pintar os logs na tela (com suporte opcional a ANSI colors ou parsers de nível de log INFO/WARN/ERROR).
*   **Botão "Stop/Restore"**: Tecla para enviar sinal SIGINT (`Ctrl+C` programático) ao subprocesso, matando o servidor de debug e, em seguida, restaurando o serviço nativo (`sudo systemctl start llamacpp-swap.service`).

## 4. Arquitetura Proposta (Rust / Ratatui)

*   **Estado Global (`AppState`):** Deve conter uma nova aba/estado `LlamaManager`, com sub-estados: `Browsing`, `Configuring`, `Running`.
*   **Estrutura de Dados:**
    ```rust
    pub struct LlamaModelState {
        pub available_models: Vec<String>,
        pub selected_index: usize,
        pub flags: LlamaFlags,
        pub server_process: Option<tokio::process::Child>,
        pub logs: Vec<String>,
    }
    ```

## 5. Passos para Implementação (Runbook)
1.  **Módulo UI:** Criar os componentes (List, Paragraph para metadados, e um Block de Logs) dentro do diretório de views do Neoland.
2.  **Lógica de Arquivos:** Implementar o scan do diretório `/var/lib/ml-models/llamacpp/models/`.
3.  **Engine de Execução:** Criar a máquina de estado que cuida de parar o `systemctl`, lançar o processo `tokio::process::Command` e gerenciar o tempo de vida (lifecycle) do processo filho, roteando os logs de volta para a interface gráfica sem bloquear o loop de renderização do `crossterm`.
4.  **Testes e Refinamento:** Validar a captura de erros quando o diretório estiver inacessível devido a permissões.