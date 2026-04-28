# Azure DevOps CI/CD Strategy

Este documento define como usar Azure DevOps para acelerar testes e validação do Neoland.

## Objetivo

A estratégia não é apenas “ter CI”.

Ela existe para:

- reduzir tempo de feedback em PR
- validar claims centrais do projeto com mais frequência
- separar feedback rápido de validação mais pesada
- dar visibilidade para gaps reais de DX e readiness

## Arquivo Principal

- [`azure-pipelines.yml`](/home/kernelcore/master/neoland/azure-pipelines.yml)

## Nota Sobre Triggers

Se o repositório estiver em `Azure Repos Git`, a validação de pull requests deve
ser conectada também por `branch policies`.

Se o repositório estiver em `GitHub` usando Azure Pipelines, o bloco `pr:` do
YAML cobre esse caso diretamente.

## Modelo De Execução

### Fast Feedback

Executa em `push` para `main` e em `pull requests`:

- Rust lint: `fmt`, `clippy`, `check`
- Rust tests
- Python contract tests do pipeline DSPy
- frontend lint + build

Objetivo:

- falhar cedo
- paralelizar o que já pode rodar isoladamente
- evitar que PR espere por validações mais pesadas

### Extended Validation

Executa fora de PR, em `main`, manual ou schedule:

- build de release
- cobertura Rust
- auditoria de DX via `scripts/validate-production-readiness.sh`

Objetivo:

- manter o feedback de PR rápido
- ainda assim rodar verificações mais profundas com regularidade

## Como Isso Acelera

### 1. Paralelismo por job

Em vez de um job único e longo, Azure roda trilhas independentes em paralelo.

### 2. Cache do Azure Pipelines

Usamos `Cache@2` para:

- `cargo`
- `target`
- `npm`

Isso reduz recompilação e reinstalação desnecessárias entre runs.

### 3. Nightly DX validation

Validações mais caras não bloqueiam toda interação de PR.

### 4. Artefatos úteis

O pipeline publica:

- binário de release
- relatório de coverage
- relatório de validação DX

## Segredos Recomendados

### Opcionais

- `CACHIX_AUTH_TOKEN`

Se configurado, permite usar Cachix para acelerar ainda mais o fluxo Nix.

### Não necessários no caminho rápido

O caminho rápido foi desenhado para não depender de:

- `LLM_API_KEY`
- segredos de produção
- Vault

Isso mantém a validação central mais barata e previsível.

## Limites Da Primeira Versão

Esta primeira pipeline prioriza estabilidade e adoção rápida.

Ainda não fecha tudo:

- Rust test reporting rico no tab de testes
- smoke end-to-end completo com serviços vivos
- self-hosted agents com cache de Nix store
- validação full integration com LLM real

## Próximas Melhorias Recomendadas

### Curto prazo

- introduzir `cargo-nextest` para acelerar Rust tests e exportar JUnit
- transformar `DX roadmap` em checks mais explícitos por jornada
- publicar resumo markdown consolidado por run

### Médio prazo

- usar self-hosted agent para Nix store persistente
- integrar Cachix como padrão
- adicionar smoke stack real com Rust server + DSPy + frontend

### Longo prazo

- separar pipeline de release público de pipeline de contribuição
- promover gates de DX para critérios formais de release

## Relação Com Os Roadmaps

- [`docs/DX_ROADMAP.md`](/home/kernelcore/master/neoland/docs/DX_ROADMAP.md)
- [`ROADMAP.md`](/home/kernelcore/master/neoland/ROADMAP.md)

Azure DevOps aqui não substitui esses roadmaps.  
Ele serve para acelerar a verificação prática deles.
