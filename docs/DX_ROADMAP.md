# Neoland DX Roadmap

**Última atualização**: 2026-04-28  
**Objetivo**: validar se a experiência real do projeto corresponde ao que o Neoland promete em docs, CLI, TUI e superfícies web.

---

## O Que DX Significa Aqui

Para Neoland, `DX` não é só conforto de desenvolvimento.

É a soma de:

- instalar sem atrito
- entender o que o projeto faz de verdade
- subir os componentes principais sem improviso
- usar os fluxos centrais sem comportamento surpreendente
- confiar que os docs não prometem mais do que o sistema entrega

O problema atual não é ausência total de componentes.  
O problema é que **precisamos verificar se cada componente funciona conforme é prometido**.

---

## Pergunta Norteadora

Para cada fluxo principal, queremos responder:

**“O que o Neoland promete?”**  
**“O que o código realmente entrega?”**  
**“O que já está testado?”**  
**“O que ainda depende de confiança manual?”**

---

## Meta Principal

Sair de:

- componentes aparentemente funcionais
- documentação forte demais
- confiança implícita

Para:

- promessas explícitas
- fluxo principal verificado
- gaps nomeados
- DX honesta para Nix, NixOS e Ubuntu bare metal

---

## Streams De Trabalho

## Stream 1 — Promise Inventory
**Objetivo**: listar as promessas reais do projeto.

Fontes prioritárias:

- [`README.md`](/home/kernelcore/master/neoland/README.md)
- [`QUICKSTART.md`](/home/kernelcore/master/neoland/QUICKSTART.md)
- [`ROADMAP.md`](/home/kernelcore/master/neoland/ROADMAP.md)
- help da CLI
- telas do TUI
- páginas centrais do frontend

**Entregáveis**

- inventário de claims por superfície
- classificação por claim:
  - `verified`
  - `partially verified`
  - `doc-only`
  - `stale`

## Stream 2 — Core Journey Verification
**Objetivo**: validar os caminhos principais de uso.

Journeys prioritárias:

1. entrar no ambiente
2. configurar secrets
3. subir server
4. validar health
5. abrir TUI
6. executar fluxo principal
7. inspecionar sessão/checkpoint
8. usar frontend workbench

**Entregáveis**

- checklist de smoke por jornada
- resultado por jornada:
  - `works as promised`
  - `works with caveats`
  - `broken`
  - `unclear claim`

## Stream 3 — TUI Truthfulness
**Objetivo**: verificar se o TUI é estável e se comunica o estado real do sistema.

Pontos de foco:

- bugs visíveis
- fluxos quebrados
- mensagens de estado enganosas
- teclas/atalhos prometidos versus comportamento real
- ergonomia para primeira execução

**Entregáveis**

- lista priorizada de bugs de DX do TUI
- separação entre:
  - bug funcional
  - bug de feedback
  - bug de onboarding

## Stream 4 — Frontend Truthfulness
**Objetivo**: confirmar se o workbench mostra a verdade do backend.

Pontos de foco:

- SSE já exposto mas não consumido
- sessões reais versus navegação prometida
- dependências residuais de Matrix
- métricas que parecem canônicas sem serem

**Entregáveis**

- matriz página -> contrato real consumido
- lista de cópias e módulos que precisam ser rebaixados ou corrigidos

## Stream 5 — Environment DX
**Objetivo**: suportar bem os dois públicos principais.

Perfis:

- `Nix/NixOS`
- `Ubuntu bare metal`

Pontos de foco:

- setup inicial
- secrets
- comandos principais
- troubleshooting mínimo

**Entregáveis**

- comparação lado a lado dos dois fluxos
- gaps onde um dos ambientes está claramente pior documentado ou menos suportado

## Stream 6 — Release Honesty
**Objetivo**: alinhar claims de readiness ao que já foi verificado.

Pontos de foco:

- percentuais de readiness
- claims do README
- “production ready” versus “pré-release técnico”
- features avançadas ainda não verificadas ponta a ponta

**Entregáveis**

- lista de claims para manter
- lista de claims para reescrever
- lista de claims para remover até validação

---

## Matriz Inicial De Avaliação

| Área | Promessa atual | Estado percebido | Ação |
|------|----------------|------------------|------|
| TUI | moderna, estável, pronta para uso central | parcial | validar fluxo real e fechar bugs |
| Pipeline | multi-agent flow real | bom no core, mas precisa checagem ponta a ponta | smoke + contrato |
| Frontend Pipeline | live view operacional | parcial | consumir SSE real |
| Sessions | inspeção real | parcial | fluxo real existe, browsing é limitado |
| ADR Vault | checkpoints reais | bom | validar corpus e navegação |
| Services | health real | bom | validar smoke em ambiente real |
| Secrets | SOPS preferido, env válido | bom | manter mensagem consistente |
| Quickstart | subir em poucos minutos | parcial | validar em fluxo limpo |
| Production readiness | alto e publicável | inflado | recalibrar por evidência |

---

## Roadmap Em Fases

## Fase A — Inventário E Critérios
**Status**: `next`

- [ ] listar promessas centrais por superfície
- [ ] definir critérios de `verified`, `partial`, `stale`
- [ ] registrar os fluxos de maior risco de frustração

## Fase B — Smoke De Journeys
**Status**: `next`

- [ ] validar fluxo Nix/NixOS
- [ ] validar fluxo Ubuntu bare metal
- [ ] validar TUI principal
- [ ] validar frontend principal
- [ ] validar sessão + ADR

## Fase C — Gap Closure
**Status**: `planned`

- [ ] corrigir bugs de TUI que quebram a jornada principal
- [ ] corrigir claims enganosas no frontend
- [ ] alinhar Quickstart com fluxo realmente suportado
- [ ] reduzir ambiguidade de paths, comandos e estados

## Fase D — DX Verification Loop
**Status**: `planned`

- [ ] transformar smoke checks em rotina repetível
- [ ] adicionar verificação por release
- [ ] impedir que docs voltem a prometer além da evidência

---

## Critério De Conclusão

Consideraremos o DX do Neoland em estado saudável quando:

- os principais fluxos de uso tiverem smoke claro e repetível
- TUI, frontend e docs descreverem o mesmo produto
- Nix e Ubuntu tiverem caminhos compreensíveis e honestos
- claims de readiness forem lastreadas em verificação, não em intenção

---

## Próximo Passo Recomendado

A sequência mais útil agora é:

1. inventariar claims do `README`, `QUICKSTART` e TUI
2. transformar isso em uma matriz `promessa -> evidência -> status`
3. atacar primeiro os bugs e drifts que quebram a jornada principal

Esse roadmap existe para garantir que Neoland não seja só um projeto com partes impressionantes, mas um sistema que cumpre aquilo que promete para quem vai realmente usar, instalar e avaliar.
