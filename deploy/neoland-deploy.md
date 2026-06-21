# Neoland Infrastructure & Deployment

Este diretório centraliza a configuração de infraestrutura, monitoramento e estratégias de deploy do Neoland. O projeto adota uma abordagem *Infrastructure as Code* (IaC) e preza pela reprodutibilidade total do ambiente.

## ❄️ Nix: Ambiente e Build

O Neoland utiliza **Nix Flakes** para gerenciar todo o ciclo de vida do desenvolvimento. O arquivo `flake.nix` na raiz do projeto define:

* **Toolchains:** Rust (Stable), Node.js (v24), Python (3.13), Bun.
* **Wrappers de Conveniência:** Comandos como `neoland`, `nsrv`, `ncli`, `nfdev` e `agents-start` que abstraem a complexidade de rodar cada componente.
* **Shell de Desenvolvimento:** Garante que todos os desenvolvedores utilizem exatamente as mesmas versões de bibliotecas de sistema (OpenSSL, Protobuf, etc.).

Para entrar no ambiente:

```bash
nix develop
```

## 📊 Monitoramento e Alerta (`deploy/prometheus/`)

O sistema de observabilidade é baseado na stack Prometheus:

* **Prometheus (`prometheus.yml`):** Coleta métricas de performance, uso de LLM, latência de gRPC/REST e saúde do sistema.
* **AlertManager (`alertmanager.yml`):** Gerencia o roteamento de alertas para Slack e PagerDuty.
* **Alert Rules (`alerts.yml`):** Mais de 60 alertas pré-configurados em categorias como:
  * **Disponibilidade:** Queda de serviço, loops de reinicialização.
  * **Performance:** Latência p99 > 2s, inferência lenta.
  * **Segurança:** Detecção de Brute Force, excesso de Rate Limit.
  * **Recursos:** Uso crítico de CPU/Memória.

## 🔐 Gestão de Segredos

O Neoland utiliza uma estratégia de camadas para segredos:

1. **SOPS (`secrets/neoland.sops.env`):** Caminho preferido, especialmente para usuários Nix/NixOS. Segredos versionados em Git, criptografados com `age` ou chaves de nuvem.
2. **HashiCorp Vault:** Integração em tempo de execução para segredos dinâmicos e rotação de chaves.
3. **Environment variables / external secret stores:** Caminho válido para Ubuntu bare metal e ambientes sem workflow SOPS.
4. **Configuração:** O comando `neoland-secrets` permite editar segredos de forma segura.

`SOPS` é recomendado, mas não obrigatório para operar o projeto de forma responsável.

## 🚀 Estratégia de Deploy

### Validação de Prontidão

Antes de qualquer deploy em produção, o script de validação deve ser executado: # TODO: Não execute scripts sem saber o que é.

```bash
./scripts/validate-production-readiness.sh
```

Atualmente, o projeto está com **93% de Production Readiness**.

### Scripts de Operação (`scripts/`)

* **`neoland-run.sh`:** Wrapper principal usado pelo Nix.
* **`backup/backup.sh`:** Scripts automatizados para backup de bancos de dados PostgreSQL e estados do Vector Store.
* **`setup-hooks.sh`:** Configuração de Git Hooks para garantir linting e testes antes do commit.

## 📈 Observabilidade Adicional

Os serviços expõem métricas e health checks compatíveis com Kubernetes:

* `/metrics`: Métricas no formato Prometheus.
* `/health`: Status completo dos componentes.
* `/ready` / `/live`: Probes padrão de orquestração.
