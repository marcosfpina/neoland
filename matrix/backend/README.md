# AI Assistant Backend

Backend system for observability and ranking of AI Agents, implementing the **STF (Supreme Technical Directive)** philosophy where the developer is the ultimate fallback.

**Part of the AI Assistant Ecosystem**: This backend works together with [AI Assistant Frontend](../frontend) inside the same monorepo workspace.

## 🧠 Philosophy

> **"Entropy is inevitable, but Governance is mandatory."**

Agents operate with high entropy (creativity), but critical decisions are gated by a Ranking System that enforces a "Human-in-the-loop" protocol when certainty drops below `0.9`.

## 🏗 Architecture

### 1. Observability (`src/observability`)
- **Prometheus**: Scrapes agent metrics (latency, decisions/sec).
- **ELK/Loki**: Logs detailed decision contexts.

### 2. Ranking API (`src/ranking`)
- **Endpoint**: `POST /rank`
- **Logic**: ML-based scoring + Heuristic penalties.
- **Policy**: `score > 0.9 ? AUTO : HUMAN_REVIEW`

### 3. Feedback Loop (`src/feedback`)
- Collects outcomes of decisions.
- Re-trains scoring models via MLflow.

## 🛡 Nix Integration

This hub is a first-class Nix citizen.

### DevShell
```bash
nix develop
# Python, Prometheus, Grafana available
```

### NixOS Module
Include in your system configuration:
```nix
imports = [ ./modules/ai-agent-hub.nix ];

services.ai-agent-hub = {
  enable = true;
  observability.prometheus = true;
};
```

## 🚨 Critical Protocols

1. **Feature Detection**: Agents must not invent features.
2. **Fallback**: If `approved=False`, the Agent MUST pause and request developer confirmation.
