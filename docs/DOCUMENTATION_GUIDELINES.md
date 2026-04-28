# Documentation Guidelines

Este documento define como a documentação do Neoland deve comunicar setup, operação e segurança.

## Princípios

- a documentação deve distinguir claramente o que é `obrigatório`, `recomendado` e `opcional`
- a documentação deve separar `Nix/NixOS` de `bare metal Ubuntu` sem tratar o caminho Ubuntu como cidadão de segunda classe
- a documentação não deve vender conveniência de tooling como se fosse requisito arquitetural
- a documentação deve evitar linguagem que infle o estado de readiness além do que o código e o fluxo operacional sustentam
- quando houver múltiplos caminhos válidos, o docs deve dizer qual é o caminho preferido e por quê

## Política De Secrets

### Regra principal

`SOPS` é o caminho **preferido e melhor suportado** para secrets no Neoland, especialmente para usuários de `Nix` e `NixOS`.

### Regra complementar

`SOPS` **não é obrigatório** para todos os ambientes.

Usuários em `bare metal Ubuntu` ou ambientes equivalentes podem operar o projeto com:

- variáveis de ambiente exportadas manualmente
- `.env` local não versionado
- secret managers externos compatíveis com o ambiente

### Como documentar isso

Sempre que um doc mencionar secrets:

- dizer explicitamente quando `SOPS` é `preferido`
- dizer explicitamente quando `SOPS` é `opcional`
- fornecer ao menos um caminho sem `SOPS` para `bare metal Ubuntu`
- nunca insinuar que ausência de `SOPS` impede avaliação, desenvolvimento ou deploy responsável por si só

## Linguagem Recomendada

### Usar

- `preferido para Nix/NixOS`
- `opcional em bare metal`
- `melhor suportado`
- `recomendado para secrets versionados com criptografia`
- `alternativamente, exporte as variáveis no ambiente`

### Evitar

- `required`
- `must use SOPS`
- `the repo requires SOPS`
- `unsupported without SOPS`

Exceto quando um fluxo específico realmente depender disso.

## Estrutura Recomendada Em Docs De Setup

Quando um documento abordar instalação ou execução, preferir esta ordem:

1. objetivo do fluxo
2. caminho recomendado para `Nix/NixOS`
3. caminho alternativo para `Ubuntu/bare metal`
4. política de secrets para cada caminho
5. troubleshooting

## Fonte De Verdade Por Tema

- `README.md`: posicionamento geral e caminhos principais
- `QUICKSTART.md`: entrada rápida de uso
- `docs/SOPS_SETUP.md`: fluxo preferido com SOPS
- `deploy/README.md`: estratégia operacional e deploy
- `secrets/README.md`: convenções do diretório de secrets

## Regra De Compatibilidade

Mudanças em documentação de setup devem preservar estes cenários:

- usuário com `nix develop` consegue usar `neoland-secrets` e wrappers do repo
- usuário Ubuntu consegue iniciar o projeto com variáveis de ambiente sem depender de SOPS
- documentação de produção pode recomendar Vault, SOPS ou secret stores externos sem confundir o fluxo local

## Definição Editorial

Para Neoland, documentação boa é documentação que:

- reduz ambiguidade
- não superestima maturidade
- permite adoção por quem usa Nix
- não exclui quem usa Ubuntu normal
