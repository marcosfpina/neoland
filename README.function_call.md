# Entendendo Function Call e Contratos de Ferramentas

## O Problema Original
As Large Language Models (LLMs) foram treinadas para prever texto, não para apertar botões ou executar scripts bash. Quando você pede a um agente: "crie um arquivo src/main.rs com Hello World", o LLM naturalmente responde *com o texto do código*, mas não consegue salvar no seu HD.

## A Solução: Function Calling (Tool Use)
Function Calling é a capacidade de ensinar ao LLM que ele não precisa apenas *falar*, ele pode emitir um *comando estruturado*.

Para isso funcionar de forma segura (e não o agente deletar seu sistema), o **Rust Backend** e o **Agente Python** firmam um **Contrato Rigoroso**.

### 1. O Contrato (O Schema)
O Servidor Rust (via API `/v1/agents/tools`) diz ao Python (DSPy):
> "Eu tenho uma ferramenta chamada `run_shell_command`. Se você quiser usar ela, você **precisa** me enviar um JSON exatamente com esse formato:
> `{"command": "string"}`"

### 2. O Raciocínio (O Agente)
O Agente Python recebe essa lista de ferramentas do Rust e injeta no prompt do Llama/DeepSeek.
Quando o Agente entende o que você pediu na TUI, ele pensa:
> "O usuário quer apagar a pasta /tmp. A ferramenta `run_shell_command` faz isso."
> Em vez de responder em português, ele cospe o JSON: `{"tool": "run_shell_command", "args": {"command": "rm -rf /tmp"}}`

### 3. A Execução (A Interceptação do Rust)
O Python recebe esse JSON do LLM e fala: "Opa, não sou eu que rodo shell, é o Rust."
O Python faz um POST de volta pro Rust (`/v1/agents/tools/call`):
> Python: "Ei Rust, executa a tool `run_shell_command` com args `{"command": "rm -rf /tmp"}`."

### 4. O Breakpoint (A Mágica que fizemos hoje)
O Servidor Rust recebe o POST do Python. Em vez de executar cego, ele pausa a requisição HTTP (o Python fica dormindo esperando).
O Rust acende o seu TUI em laranja: **"Aprovar rm -rf /tmp?"**
Se você apertar `[Enter]`, o Rust executa no sistema operacional e devolve o texto do bash (STDOUT) pro Python!

### Resumo Visual
`TUI (Você) -> Backend Rust -> DSPy Python -> LLM -> (Decide usar Tool) -> DSPy Python -> Backend Rust -> TUI (Aprova?) -> Rust Executa -> Retorna pro Python.`
