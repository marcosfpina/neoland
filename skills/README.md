# Sistema de Skills do Neoland

As **Skills** no Neoland são pacotes de conhecimento especializado e ferramentas que expandem as capacidades cognitivas dos agentes. Elas permitem que o sistema atue como um especialista em domínios específicos, como administração de servidores NixOS, segurança cibernética ou design de experiência do usuário (UX).

## 🏗️ Estrutura de uma Skill

Cada skill é organizada em seu próprio diretório dentro de `skills/` e segue uma estrutura padronizada para garantir que os agentes possam consumir o conteúdo de forma eficiente:

*   **`SKILL.md` (O Core):** O documento principal que define a "personalidade" e o conhecimento da skill. Contém:
    *   **Frontmatter:** Metadados como nome, descrição e gatilhos (triggers).
    *   **Core Principles:** A filosofia de atuação do especialista.
    *   **When to Use:** Gatilhos específicos que indicam quando esta skill deve ser ativada.
    *   **Workflow:** Fases de execução (Avaliação, Planejamento, Implementação).
*   **`assets/`:** Recursos visuais, diagramas, logotipos ou dados estáticos complementares.
*   **`references/`:** Documentação técnica, whitepapers, guias de melhores práticas e links externos.
*   **`scripts/`:** Ferramentas auxiliares, scripts de diagnóstico ou geradores de configuração que o agente pode utilizar ou recomendar.

## 🧠 Como as Skills são Utilizadas

As skills são integradas ao ecossistema Neoland através de dois mecanismos principais:

1.  **RAG (Retrieval-Augmented Generation):** O conteúdo das pastas `skills/` é indexado no `VectorStore` (pgvector). Quando um usuário faz uma pergunta relacionada a um domínio (ex: "Como otimizar meu cache NixOS?"), o sistema recupera os trechos mais relevantes da skill correspondente.
2.  **Contexto Sistêmico:** Os agentes (especialmente o Junior e o Senior) são instruídos a buscar e seguir as diretrizes definidas nos arquivos `SKILL.md` quando o tema da tarefa coincide com os gatilhos da skill.

## 🛠️ Criando uma Nova Skill

Para adicionar uma nova especialidade ao Neoland, siga estes passos:

1.  **Crie o diretório:** `skills/minha-nova-skill/`.
2.  **Defina o `SKILL.md`:** Use como base o template de skills existentes (como `skills/Linux_Server_Master/nixos-remote-cache-expert/SKILL.md`).
3.  **Adicione Referências:** Coloque guias técnicos em `references/` para dar profundidade ao conhecimento do agente.
4.  **Codifique Ferramentas:** Se houver automações úteis, coloque-as em `scripts/`.

## 📂 Categorias Atuais

*   **`Linux_Server_Master`:** Especialistas em NixOS, caches remotos e otimização de hardware.
*   **`security-architect`:** Conhecimento profundo em segurança ofensiva e defensiva.
*   **`nix-expert`:** Domínio total de Nix Flakes, linguagens Nix e reprodutibilidade.
*   **`ux-agents`:** Foco em design de interfaces modernas e experiência do usuário em terminal.
*   **`neoland-coding`:** Diretrizes internas para o desenvolvimento do próprio projeto Neoland.
