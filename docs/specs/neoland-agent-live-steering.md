# Especificação da Feature: Agent Live-Steering (Direcionamento em Tempo Real)

## 1. Visão Geral (O Problema)
Atualmente, a execução de Agentes de IA em tarefas longas (ex: 30+ minutos de análise de código, refatoração em lote, pesquisa profunda) funciona como uma **caixa preta e inflexível**. O desenvolvedor envia o prompt inicial e o Agente executa sua percepção da tarefa até o fim (ou falha). 

Se o desenvolvedor (que possui o contexto absoluto do sistema) perceber no minuto 5 que o Agente está seguindo um caminho arquitetural errado, a única opção atual é **abortar, perder todo o trabalho já feito e recomeçar** com um prompt corrigido.

## 2. A Solução (Live-Steering via Natural Language)
Esta feature introduz o conceito de **"Direcionamento em Tempo Real" (Live-Steering)** no Neoland TUI. 

O desenvolvedor pode abrir um canal de comunicação bidirecional com um ou múltiplos Agentes *enquanto eles estão executando* suas rotinas principais, enviando comandos em **Linguagem Natural**.

### Casos de Uso (Exemplos de Intervenção)
*   **Correção de Rota (Micro-Steering):** "Ei, pare de tentar usar o módulo `fs`, eu esqueci de avisar que a gente migrou pra `tokio::fs` ontem. Refaça a partir dessa premissa."
*   **Adição de Restrições:** "Não modifique o arquivo `Cargo.lock` nesta rodada, apenas os `.rs`."
*   **Sincronização de Conhecimento:** "Acabei de rodar um script externo que corrigiu o banco de dados. Pode prosseguir assumindo que a tabela `users` está limpa."
*   **Inspeção de Estado (Dump):** "Me explique o que você entendeu do problema até agora antes de começar a escrever o código."

## 3. Arquitetura Técnica (Rust + Tokio + LLM)

A base técnica requer a transição de um modelo de execução sequencial (Bloqueante) para um **Modelo de Atores Orientado a Eventos**.

### 3.1. O Loop do Agente (Actor Loop)
Em vez de um Agente executar um loop `while(has_tasks) { execute() }` que ignora o mundo exterior, o loop principal do Agente deve ser um `tokio::select!`.

```rust
// Pseudocódigo da Arquitetura do Agente
loop {
    tokio::select! {
        // 1. O Trabalho Principal (Progresso da Task)
        step_result = agent.execute_next_step() => {
            handle_progress(step_result);
        },
        
        // 2. A "Orelha" do Agente (Canal de Intervenção Humana)
        Some(user_message) = steering_rx.recv() => {
            // O Agente pausa o loop principal.
            // Envia o `user_message` (Linguagem Natural) para o LLM pedindo para 
            // analisar como essa nova informação altera o Plano de Execução atual.
            let updated_plan = agent.replan_with_new_context(user_message).await;
            
            // O Agente retoma a execução (loop) com as novas diretrizes.
            agent.apply_new_plan(updated_plan);
        },
        
        // 3. Sinais de Sistema (Kill, Pause)
        _ = shutdown_signal.recv() => break,
    }
}
```

### 3.2. Interface do Usuário (Neoland TUI)
Na interface Ratatui, a abstração será semelhante a um "Chat sobreposto à Barra de Progresso".

*   **Painel Central:** Exibe a árvore de raciocínio atual do Agente (Thought -> Action -> Observation).
*   **Caixa de Input (Bottom):** Um campo de texto sempre ativo (ex: pressionando `i` para intervir). O usuário digita a correção em linguagem natural e pressiona `Enter`.
*   **Feedback Visual:** A interface deve indicar quando um Agente recebeu o sinal de "Intervenção" (Mudando de cor, ex: Amarelo/Pausado) enquanto ele processa o novo contexto e recalcula seu plano de ação.

## 4. Desafios e Considerações de Design
1.  **Custo de Contexto (Context Window):** Ao alterar a rota, o Agente precisa do histórico do que já fez + a nova instrução do usuário. É crucial gerenciar o tamanho do contexto enviado ao LLM no momento do *replan* (Replanejamento).
2.  **Estado Parcialmente Executado:** Se o usuário interrompe o Agente no meio de uma edição de arquivo (ex: um `replace` via ferramenta MCP), o Agente precisa ter mecanismos de "Rollback" do passo atual ou capacidade de continuar a edição com segurança sob a nova premissa.
3.  **Broadcast vs Unicast:** O usuário deve poder escolher se a intervenção ("O banco de dados mudou") é para um Agente específico (Unicast) ou para toda a "Frota" de Agentes rodando (Broadcast).