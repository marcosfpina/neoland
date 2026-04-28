# Azure DevOps Setup Guide

**Objetivo**: configurar o Neoland no Azure DevOps com CI/CD automatizado e obter um primeiro diagnóstico útil já na execução inicial.

---

## O Que Vamos Configurar

1. pipeline YAML apontando para [`azure-pipelines.yml`](/home/kernelcore/master/neoland/azure-pipelines.yml)
2. gatilho de CI em `main`
3. validação de PR via branch policy
4. execução noturna de validação estendida
5. secrets opcionais
6. leitura do primeiro diagnóstico

---

## Pré-Requisitos

Você precisa ter:

- projeto criado no Azure DevOps
- repositório importado para `Azure Repos Git` ou conectado a `GitHub`
- permissão para criar pipelines
- permissão para editar branch policies de `main`

Opcional, mas recomendado:

- token do Cachix para acelerar `nix`

---

## Passo 1 — Criar O Pipeline

No Azure DevOps:

1. vá em `Pipelines`
2. clique em `New pipeline`
3. escolha a origem do repositório
4. selecione o repositório do Neoland
5. escolha `Existing Azure Pipelines YAML file`
6. selecione:

```text
/azure-pipelines.yml
```

7. salve e rode

---

## Passo 2 — Configurar Variables / Secrets

No pipeline, abra:

`Edit` -> `Variables`

Adicione:

### Opcional

- `CACHIX_AUTH_TOKEN`
  - marque como `secret`

Se vocês ainda não tiverem Cachix, deixem isso em branco. O pipeline continua funcionando.

### Não adicionar agora

Para a primeira fase, não é necessário adicionar:

- `LLM_API_KEY`
- secrets de produção
- Vault tokens

O objetivo inicial é estabilizar feedback e diagnóstico do core.

---

## Passo 3 — Branch Policy Em `main`

Se estiverem usando `Azure Repos Git`, façam isso:

1. `Repos`
2. `Branches`
3. no branch `main`, clique em `Branch policies`
4. em `Build validation`, clique em `+`
5. selecione o pipeline criado

Configuração recomendada:

- `Trigger`: `Automatic`
- `Policy requirement`: `Required`
- `Build expiration`: `Immediately when <branch> is updated`
- `Display name`: `Neoland CI Validation`

Isso garante validação real de PR.

Se o repo estiver conectado via `GitHub`, o bloco `pr:` do YAML já ajuda, mas ainda vale configurar branch protection do lado do GitHub se quiserem gate forte.

---

## Passo 4 — Rodar O Primeiro Build

Execute manualmente o pipeline uma vez.

Na primeira rodada, o esperado é:

- mais lenta por bootstrap do Nix
- cache ainda frio
- possível exposição de gaps reais de ambiente

Isso é normal. O objetivo dessa primeira run é **diagnóstico**, não performance máxima.

---

## Passo 5 — Como Ler O Primeiro Diagnóstico

Na primeira execução, olhem estes jobs:

### 1. `rust_lint`

Confirma:

- `nix` sobe no agent
- `cargo fmt`
- `cargo clippy`
- `cargo check`

Se falhar aqui:

- o problema é bootstrap/toolchain ou qualidade básica de código

### 2. `rust_tests`

Confirma:

- suíte Rust principal executa no CI
- cache de `cargo/target` está ajudando

Se falhar aqui:

- temos regressão real ou diferença entre ambiente local e CI

### 3. `agents_contract`

Confirma:

- Poetry + Python dos agents está saudável
- schemas e IPC contracts continuam coerentes

Se falhar aqui:

- drift entre pipeline Python e control plane ou ambiente Python

### 4. `frontend_checks`

Confirma:

- `npm ci`
- `lint`
- `build`

Se falhar aqui:

- temos problema direto de DX no frontend

### 5. `dx_validation`

Esse é o diagnóstico consolidado mais útil no começo.

Artefato esperado:

- `dx-validation`

Ele publica a saída do script:

- [`scripts/validate-production-readiness.sh`](/home/kernelcore/master/neoland/scripts/validate-production-readiness.sh)

Esse relatório vai te dizer rapidamente:

- o que passou
- o que falhou
- o que ainda está pendente

---

## Passo 6 — O Que Fazer Se A Primeira Run Falhar

Use esta ordem:

1. `Nix install` falhou
2. `cargo/poetry/npm` falhou
3. testes falharam
4. build frontend falhou
5. `dx_validation` expôs claim inflada ou caminho incompleto

Não tentem resolver tudo de uma vez.  
O primeiro objetivo é deixar o pipeline verde no caminho rápido.

---

## Meta Da Primeira Semana

Meta realista:

- pipeline criado
- branch policy ligada
- primeira run manual concluída
- pelo menos `fast_feedback` estabilizado
- diagnóstico inicial publicado como artifact

Ainda não é meta da primeira semana:

- full E2E com LLM real
- self-hosted agents
- cache perfeito
- release automation completa

---

## Próximos Ajustes Depois Da Primeira Run

Depois que o primeiro diagnóstico sair:

1. corrigir as falhas do `fast_feedback`
2. medir quais jobs estão mais lentos
3. decidir se vale:
   - habilitar Cachix
   - usar self-hosted runner
   - introduzir `cargo-nextest`
4. transformar findings do diagnóstico em tickets do roadmap de DX

---

## Checklist Curto

- [ ] pipeline criado com `/azure-pipelines.yml`
- [ ] variável opcional `CACHIX_AUTH_TOKEN` cadastrada
- [ ] branch policy em `main` configurada
- [ ] primeira execução manual realizada
- [ ] artifact `dx-validation` publicado
- [ ] jobs do `fast_feedback` analisados
- [ ] falhas transformadas em backlog objetivo

---

## Resultado Esperado

Ao final dessa configuração, vocês já devem ter:

- automação básica rodando no Azure DevOps
- feedback rápido por PR
- uma trilha noturna de validação estendida
- um diagnóstico concreto do estado atual do Neoland

Esse é o ponto em que o CI deixa de ser “mais uma integração” e passa a ser uma ferramenta real de verdade do projeto.
