# Neoland LLM Runtime Roadmap

**Última atualização**: 2026-04-28  
**Objetivo**: fechar a arquitetura operacional de inferência do Neoland com uma topologia única, validável e pronta para endurecimento.

---

## Decisão Arquitetural

### Topologia alvo

```text
Neoland
  -> SecureLLM Bridge API
    -> ml-ops-api
      -> llama.cpp
      -> vLLM (opcional)
```

### O que esta decisão significa

- `Neoland` não deve falar com `llama.cpp` diretamente no fluxo principal
- `SecureLLM Bridge` é o gateway/proxy reverso semântico para LLMs
- `ml-ops-api` é a camada de inferência e roteamento GPU/local
- `llama.cpp` e `vLLM` ficam como backends atrás do `ml-ops-api`
- `spider-nix-network` não é gateway principal do produto; ele pode existir como proxy outbound auxiliar quando houver necessidade real

### Fonte de verdade por camada

- `Neoland`: UX, TUI, control plane, jornadas, health do produto
- `securellm-bridge`: gateway LLM, policies, audit, provider routing
- `ml-ops-api`: inferência OpenAI-compatible, health do runtime, escolha de backend
- `llama.cpp` / `vLLM`: execução do modelo

---

## Problema Atual

Hoje o runtime de inferência está funcional em partes, mas com naming e health ainda confusos:

- `ml_api_url` no Neoland ainda carrega semântica mista entre `ml-offload`, `SecureLLM API` e `llama.cpp`
- `doctor` do Neoland ainda verifica um endpoint incompatível com o `securellm-bridge` (`/health` em vez de `/api/health`)
- a documentação mistura `8080`, `8081`, `8083`, `5001` e `9000` sem declarar claramente qual porta pertence a qual camada
- parte do repo ainda descreve `llama.cpp` direto, enquanto a topologia decidida usa gateway + inference bridge

O roadmap desta trilha existe para resolver isso sem improviso.

---

## Meta Principal

Sair de:

- backend de inferência parcialmente funcional
- nomenclatura ambígua
- health checks enganosos
- múltiplas leituras possíveis da mesma stack

Para:

- uma topologia oficial
- um conjunto claro de URLs e portas
- diagnóstico correto por camada
- fluxo repetível de `boot -> health -> task -> fallback -> troubleshooting`

---

## Configuração Alvo

### Single-host dev/staging

| Camada | Papel | URL sugerida |
|--------|-------|--------------|
| `Neoland REST` | control plane | `http://127.0.0.1:3001` |
| `SecureLLM Bridge API` | gateway LLM principal | `http://127.0.0.1:8080` |
| `ml-ops-api` | inference bridge | `http://127.0.0.1:8083` |
| `llama.cpp` | backend local | `http://127.0.0.1:5001` |
| `vLLM` | backend opcional | `http://127.0.0.1:8000` |

### Variáveis esperadas

| Variável | Dono | Valor alvo |
|----------|------|------------|
| `NEOLAND_ML_API_URL` | Neoland | `http://127.0.0.1:8080` |
| `ML_OPS_API_URL` | securellm-bridge | `http://127.0.0.1:8083` |
| `LLAMACPP_URL` | ml-ops-api | `http://127.0.0.1:5001` |
| `VLLM_URL` | ml-ops-api | `http://127.0.0.1:8000` |

### Regra de naming

- `ml_api_url` no Neoland deve ser entendido como: endpoint OpenAI-compatible principal consumido pelo cliente
- na topologia decidida, esse endpoint é o `SecureLLM Bridge API`
- `llama.cpp` e `vLLM` não devem ser tratados como URLs primárias do client do Neoland

---

## Roadmap De Execução

## Fase 0 — Freeze De Topologia
**Status**: `in_progress`

- [ ] declarar explicitamente em docs centrais que o gateway principal é o `securellm-bridge`
- [ ] declarar explicitamente que `ml-ops-api` é upstream de inferência
- [ ] declarar explicitamente que `spider-nix-network` não é o gateway principal do produto
- [ ] parar de usar linguagem ambígua como “ml-offload ou llama.cpp” no mesmo campo sem distinguir camada

**Saída da fase**

- uma topologia oficial, curta e repetível

## Fase 1 — Alinhamento De Naming No Neoland
**Status**: `next`

- [ ] revisar `README.md`, `docs/PROVIDERS.md` e docs de runtime para refletir o gateway real
- [ ] rebaixar ou remover referências que tratem `ml_api_url` como `llama.cpp` direto
- [ ] renomear labels de UX e diagnóstico onde `ml-offload` não descreve mais o runtime real
- [ ] decidir se `ml_api_url` mantém esse nome por compatibilidade ou se ganha alias/documentação mais precisa

**Saída da fase**

- operador entende qual serviço o Neoland realmente consome

## Fase 2 — Health E Doctor Corretos
**Status**: `next`

- [ ] ajustar `neoland doctor` para validar `SecureLLM Bridge API` com o path correto
- [ ] suportar pelo menos um destes modelos de health:
  - `gateway health`
  - `gateway ready`
  - health encadeado do upstream
- [ ] separar no diagnóstico:
  - control plane down
  - gateway down
  - inference bridge down
  - backend local down
- [ ] impedir que warning genérico esconda qual camada realmente falhou

**Saída da fase**

- `doctor` vira fonte de verdade do runtime de inferência

## Fase 3 — Bootstrap Do SecureLLM Bridge
**Status**: `next`

- [ ] validar que o `securellm-api-server` sobe com `ml-ops-api` habilitado como provider
- [ ] configurar `ML_OPS_API_URL` corretamente no ambiente do bridge
- [ ] decidir fallback order oficial entre `ml-ops`, providers externos e local directo
- [ ] garantir que o bridge continue OpenAI-compatible no endpoint que o Neoland consome
- [ ] documentar env mínimo para dev shell, NixOS e Ubuntu bare metal

**Saída da fase**

- gateway pronto para operar como entrypoint do Neoland

## Fase 4 — Bootstrap Do ml-ops-api
**Status**: `next`

- [ ] configurar `LLAMACPP_URL` para upstream do `llama.cpp`
- [ ] configurar `VLLM_URL` quando houver backend disponível
- [ ] validar `/health`, `/api/health` e `/v1/chat/completions`
- [ ] garantir que o roteador interno do `ml-ops-api` enxerga pelo menos um backend funcional
- [ ] documentar claramente quando o Neoland deve falar com `8080` e quando o compose publica `8083`

**Saída da fase**

- camada de inferência pronta e observável

## Fase 5 — Backend Local E Forwarding
**Status**: `planned`

- [ ] fixar porta oficial do `llama.cpp` atrás do `ml-ops-api`
- [ ] validar forwarding `ml-ops-api -> llama.cpp`
- [ ] validar forwarding `securellm-bridge -> ml-ops-api -> llama.cpp`
- [ ] opcional: validar `vLLM` como backend alternativo sem quebrar o gateway principal
- [ ] decidir estratégia default:
  - `llama.cpp` como baseline local
  - `vLLM` como upgrade de throughput/contexto

**Saída da fase**

- backend local roteado sem atalhos informais

## Fase 6 — Validação End-to-End
**Status**: `planned`

- [ ] subir `Neoland + SecureLLM Bridge + ml-ops-api + llama.cpp`
- [ ] rodar `neoland doctor`
- [ ] abrir `neoland client`
- [ ] executar prompt real via endpoint OpenAI-compatible principal
- [ ] validar fallback e mensagens quando uma camada cai
- [ ] validar que latência, erros e backend ativo aparecem de forma honesta

**Saída da fase**

- jornada principal da inferência comprovada

## Fase 7 — Observabilidade E Release Discipline
**Status**: `planned`

- [ ] definir health board por camada
- [ ] padronizar logs e nomes dos componentes
- [ ] garantir troubleshooting mínimo para:
  - gateway indisponível
  - ml-ops indisponível
  - llama.cpp indisponível
  - vLLM indisponível
  - mismatch de portas/envs
- [ ] transformar a validação em checklist de release

**Saída da fase**

- stack de inferência pronta para operar sem adivinhação

---

## Tracking Inicial

| ID | Item | Status | Criticidade |
|----|------|--------|-------------|
| LLM-1 | Congelar topologia `Neoland -> SecureLLM Bridge -> ml-ops-api -> llama.cpp/vLLM` | In progress | Alta |
| LLM-2 | Corrigir semântica de `ml_api_url` no Neoland | Next | Alta |
| LLM-3 | Ajustar `doctor` para health do gateway real | Next | Alta |
| LLM-4 | Subir `securellm-api-server` com `ml-ops` habilitado | Next | Alta |
| LLM-5 | Fixar `LLAMACPP_URL` e forwarding no `ml-ops-api` | Next | Alta |
| LLM-6 | Validar E2E completo com task real | Planned | Alta |
| LLM-7 | Padronizar docs de portas e envs | Planned | Média |
| LLM-8 | Decidir papel operacional do `vLLM` no primeiro corte | Planned | Média |
| LLM-9 | Deixar `spider-nix-network` explícito como opcional e não-core | Planned | Média |

---

## Critério De Conclusão

Consideraremos esta trilha fechada quando:

- `Neoland` apontar para o gateway correto por default
- `doctor` diagnosticar corretamente cada camada da stack
- `securellm-bridge` estiver operacionalmente posicionado como proxy/gateway principal
- `ml-ops-api` estiver roteando para `llama.cpp` com health real
- as portas e variáveis oficiais não gerarem ambiguidade documental
- a stack completa puder ser explicada e validada em menos de 5 minutos

