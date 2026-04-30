# Agentic Breakpoints & Live Steering Roadmap

## Visão Geral
A funcionalidade de **Agentic Breakpoints** visa transformar a execução de agentes autônomos em um processo colaborativo e seguro. Em vez de o agente executar um script longo e destrutivo sem supervisão, o sistema pausa antes de ações críticas (ex: comandos de shell, modificação de arquivos core, chamadas a APIs externas), pede aprovação do operador humano via TUI, e permite correção de rota (Live Steering) no meio do raciocínio.

## Arquitetura Atual
1. **TUI (Rust)**: Interface de usuário.
2. **Server (Rust)**: Orquestrador central e gateway.
3. **Pipeline (Python DSPy)**: Motor de raciocínio.
4. **SecureLLM MCP (TypeScript)**: Provedor de ferramentas (ex: `execute_in_sandbox`, manipulação de arquivos).

## O Desafio
Atualmente, o Rust delega as ferramentas para o Python, ou o Python roda suas próprias ferramentas. Para que o Breakpoint funcione de forma centralizada e bloqueante (sem sobrecarregar o Python com websockets de UI), a interrupção precisa ocorrer no **Server (Rust)** antes que a ferramenta seja efetivamente chamada.

## Roadmap de Implementação

### Fase 1: Fundações de Comunicação (Rust Backend & TUI)
- [ ] **Eventos de Domínio:** Expandir `AgentEvent` e `AgentStreamEvent` no `src/agents/events.rs` para incluir:
  - `BreakpointHit { tool: String, args: String, session_id: Uuid }`
  - `BreakpointResolved { resolution: String }` (Approve, Reject, Steer)
- [ ] **State Machine na TUI (`src/tui/app.rs`)**:
  - Adicionar estado `waiting_for_breakpoint`.
  - Quando `BreakpointHit` chegar via SSE, travar o `input_buffer`, mudar o layout para modo de aprovação (cor Laranja/Roxa).
  - Mapear teclas `[Enter]` (Approve), `[Esc]` (Reject) e `[Tab]` (Mudar para modo Steer text).
- [ ] **Nova Rota de API (`src/server/mod.rs`)**:
  - Criar o endpoint `POST /v1/agents/session/:id/breakpoint/resolve`.
  - Essa rota receberá a decisão do TUI e a enviará por um channel assíncrono para a task pausada.

### Fase 2: O Bloqueio Assíncrono (Orchestrator)
- [ ] **Ponte de Controle (`src/agents/orchestrator.rs`)**:
  - Criar um mecanismo de registro de canais (ex: `DashMap<Uuid, mpsc::Sender<BreakpointResolution>>`) para correlacionar uma sessão ativa com sua espera.
  - Implementar a função `async fn request_human_approval(tool, args) -> BreakpointResolution`.
  - Essa função: 
    1. Emite `AgentEvent::BreakpointHit`.
    2. Dá `.await` no channel de resolução.
    3. Retorna a decisão (Approve, Reject, Steer).

### Fase 3: Roteamento de Ferramentas (O Pulo do Gato)
- *Decisão de Design Arquitetural necessária:* Atualmente, o Python chama as tools diretamente ou ele pede pro Rust chamar?
- **Se o Python chama as tools (DSPy nativo):** Precisamos injetar uma "Mock Tool" no Python que, quando acionada, faz uma chamada HTTP síncrona/bloqueante de volta pro Rust (`POST /internal/mcp_proxy`), acionando o `request_human_approval` do Rust. O Rust segura a resposta HTTP até o humano clicar na TUI.
- **Se o Rust controla o MCP e passa pro Python:** O Rust intercepta o `mcp.call()`, aciona o `request_human_approval`, e dependendo da resposta, ele prossegue com a chamada MCP real ou retorna um erro de "Acesso Negado pelo Humano" pro Python.
- [ ] **Implementar o interceptador de chamadas de Tool no fluxo escolhido.**

### Fase 4: O "Live Steer" como Rejeição Corretiva
- [ ] Quando o humano escolhe `Steer` num Breakpoint, o sistema não apenas "falha" a ferramenta. Ele deve injetar a string de Steer no erro retornado para a Tool (Ex: `ToolError: Human rejected with instruction: "Não use rm -rf, use trash-cli"`). 
- [ ] O pipeline DSPy (Sênior/TechLeader) deve ser capaz de receber esse erro de Tool, entender a correção de rota, e planejar um novo passo sem quebrar a sessão inteira.
