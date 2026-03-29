# Teste de UX Granular - Neoland

## Recursos Implementados

### 1. Parâmetros Avançados de Geração (Proto)
- ✅ `temperature` (0.0-2.0)
- ✅ `top_p` (0.0-1.0)
- ✅ `max_tokens` (1-2000)
- ✅ `repetition_penalty` (1.0-2.0)
- ✅ `typical_p` (0.0-1.0) - Adicionado pelo usuário
- ✅ `epsilon_cutoff` (0.0-1.0) - Adicionado pelo usuário
- ✅ `eta_cutoff` (0.0-1.0) - Adicionado pelo usuário
- ✅ `tail_free_sampling` (0.0-1.0) - Adicionado pelo usuário
- ✅ `top_a` (0.0-1.0) - Adicionado pelo usuário

### 2. Controle de Contexto RAG
- ✅ `context_top_k` - Número de documentos a buscar
- ✅ `context_similarity_threshold` - Score mínimo de similaridade
- ✅ `disable_context` - Toggle para desabilitar RAG

### 3. System Prompt Customizado
- ✅ Campo de texto expansível para editar prompt do sistema
- ✅ Sobrescreve prompt padrão quando preenchido

### 4. Controle de Comandos
- ✅ Toggle para habilitar/desabilitar execução de comandos
- ✅ Whitelist de comandos permitidos (separados por vírgula)
- ✅ Filtragem server-side de comandos

### 5. Metadata e Transparência
- ✅ ResponseMetadata com parâmetros utilizados
- ✅ Contagem de documentos de contexto injetados
- ✅ IDs dos documentos utilizados
- ✅ Display no cliente dos metadados

## Interface GTK4

### Painel Lateral (Sidebar)
```
🔌 CONECTADO AO SERVIDOR
━━━━━━━━━━━━━━━━━━━━━━

⚙️ Parâmetros de Geração
├─ Temperature [slider] 0.00-2.00
├─ Top P [slider] 0.00-1.00
├─ Max Tokens [spinner] 1-2000
├─ Repetition Penalty [slider] 1.00-2.00
├─ Typical P [slider] 0.00-1.00
├─ Epsilon Cutoff [slider] 0.00-1.00
├─ Eta Cutoff [slider] 0.00-1.00
├─ Tail Free Sampling [slider] 0.00-1.00
└─ Top A [slider] 0.00-1.00

📚 Controle de Contexto
├─ Documentos (Top K) [spinner] 0-10
├─ Similaridade Mínima [slider] 0.00-1.00
└─ ☐ Desabilitar Contexto

🖊️ System Prompt Customizado [Expander]
└─ [TextArea com scroll]

⚡ Execução de Comandos
├─ ☑ Habilitar Comandos
└─ Comandos Permitidos: [Entry]
   (ex: move_ws,scratchpad)
```

## Casos de Uso

### 1. Máxima Criatividade
```
Temperature: 1.5
Top P: 0.95
Typical P: 0.95
Context: Desabilitado
```

### 2. Respostas Focadas e Precisas
```
Temperature: 0.3
Top P: 0.7
Repetition Penalty: 1.3
Context Top K: 5
Similarity Threshold: 0.4
```

### 3. Contexto RAG Profundo
```
Context Top K: 8
Similarity Threshold: 0.2
Temperature: 0.7 (padrão)
```

### 4. Modo Seguro (Sem Comandos)
```
☐ Desabilitar Comandos
Commands Whitelist: (vazio)
```

### 5. Comandos Restritos
```
☑ Habilitar Comandos
Commands Whitelist: move_ws
(Apenas movimento de workspace permitido)
```

## Integrações

### SecureLLM Bridge
- Audit logging ao adicionar documentos
- Preparado para rate limiting e validação

### Phantom (IntelAgent Core)
- TaskId generation para tracking
- QualityGate preparado para validação de qualidade

## Arquitetura

```
[GTK4 Client] --gRPC--> [Server]
                           |
                           +-> [LocalEngine] (Qwen 1.8B)
                           |     |
                           |     +-> GenerationConfig
                           |     +-> LogitsProcessor
                           |
                           +-> [VectorStore] (MiniLM)
                           |     |
                           |     +-> Embeddings
                           |     +-> Semantic Search
                           |
                           +-> [SecureLLM] (Audit)
                           |
                           +-> [Phantom] (TaskId)
```

## Como Testar

1. **Iniciar Servidor:**
   ```bash
   ./target/release/llamachat-server
   ```

2. **Iniciar Cliente:**
   ```bash
   ./target/release/llamachat-client
   ```

3. **Testar Controles:**
   - Ajustar sliders e ver valores atualizando
   - Modificar system prompt
   - Testar whitelist de comandos
   - Observar metadata nas respostas

4. **Testar Integração:**
   - Verificar logs de SecureLLM no servidor
   - Confirmar TaskId do Phantom na inicialização
   - Testar RAG com diferentes Top K valores

## Melhorias Futuras
- [ ] Session management (conversation history)
- [ ] Preset profiles (salvar configurações)
- [ ] Export/import de configurações
- [ ] Visualização de embeddings
- [ ] Histórico de metadata
- [ ] Gráficos de parâmetros vs qualidade
