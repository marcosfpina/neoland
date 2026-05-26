# AI Assistant Hub - Integration Guide

**Status**: Frontend and Backend now live in the same monorepo workspace.

> Current status note (2026-05-17): this guide describes the inherited Matrix
> frontend/backend integration. The active Neoland workbench path is
> `matrix/frontend`, with the Rust control plane at `:3001` and DSPy agents at
> `:8001`. Use `docs/neoland-project-snapshot.md` for the current Neoland map.

---

## 🏗️ Architecture Overview

The system consists of two independent services communicating via REST API:

1. **Frontend (The Qualia)**
   - **Path**: `apps/frontend`
   - **Tech**: Next.js 15, React 18, TypeScript, TailwindCSS
   - **Port**: `3000` (Proxy to Backend)
   - **LLM**: Llama.cpp (via `llama-server` on port `8081`)
   - **Responsibilities**: UI/UX, Workflow Orchestration, Human-in-the-Loop
2. **Backend (The Core)**
   - **Path**: `apps/backend`
   - **Tech**: FastAPI, Python 3.13, TimescaleDB, Prometheus
   - **Port**: `8000`
   - **Responsibilities**: ML Ranking, STF Validation, Observability

---

## Quick Start

You need two terminal windows running in parallel.

### **Terminal 1: Backend**

```bash
nix develop --command zsh -lc 'cd apps/backend && export PYTHONPATH="$PWD/src${PYTHONPATH:+:$PYTHONPATH}" && uvicorn src.ranking.main:app --reload --host 0.0.0.0 --port 8000'
# → Running on http://localhost:8000
```

### **Terminal 2: Frontend**

```bash
nix develop --command zsh -lc 'cd apps/frontend && npm install && npm run dev'
# → Running on http://localhost:3000
```

---

## 🔗 Connection Details

- **Frontend → Backend**: The Frontend Proxy is configured in `next.config.mjs`:
  - `/api/rank` → `http://localhost:8000/rank`
  - `/api/health` → `http://localhost:8000/health`
  - `/api/metrics` → `http://localhost:8000/metrics`
- **Shared Types**:
  To ensure type safety, the Frontend types in `lib/api/types.ts` must be kept in sync with Backend Pydantic models.
  *Roadmap: Automate via OpenAPI generation.*

---

## 🧪 Testing Integration

### 1. Verify Backend Health

```bash
curl http://localhost:8000/health
# Expected: {"status": "ok", ...}
```

### 2. Verify Frontend Connection

Go to  (if enabled) or use the **Console** in the dashboard.

### 3. Agent Orchestra

Navigate to **/agent-orchestra** run a "Complete Code Review". The frontend will:

1. Send `DecisionRequest` to Backend
2. Backend validates against STF Protocol
3. Backend calculates ML Score
4. If score < 0.9, Frontend shows **Approval Dialog**

---

## ⚠️ Troubleshooting

**Backend won't start?**

- Ensure Port 8000 is free: `lsof -i :8000`
- Check Nix dependencies: `nix develop`
- Check database: `just db-status`

**Frontend can't connect?**

- Check CORS settings in Backend (`ALLOWED_ORIGINS` env var)
- Check `next.config.mjs` rewrites

---

## 📦 Deployment Strategy

- **Backend**: Deploy as Docker Container (Cloud Run / VPS)
- **Frontend**: Deploy as Next.js App (Vercel / Netlify)
- **Database**: Managed PostgreSQL (TimescaleDB cloud or self-hosted)

See `deployment/` folder in respective repos for details.
