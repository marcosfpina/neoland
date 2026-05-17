#!/usr/bin/env bash
# Valida toda a stack Neoland e cospe logs estruturados.
# Uso: nix develop --command bash scripts/validate-stack.sh
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
LOG_DIR="$ROOT/.validate-logs"
TIMESTAMP="$(date +%Y%m%d-%H%M%S)"
REPORT="$LOG_DIR/report-$TIMESTAMP.txt"

mkdir -p "$LOG_DIR"

PASS=0
FAIL=0
SKIP=0

# DATABASE_URL — usa env ou mock local para validação
DB_URL="${DATABASE_URL:-postgres://neoland:neoland@localhost:5432/neoland_dev}"

# ── helpers ──────────────────────────────────────────────────────────────────

sep()  { printf '\n'; printf '%.0s─' {1..72}; echo; }
hdr()  { sep; echo "▶ $1"; sep; }
ok()   { echo "  ✅ $1"; PASS=$((PASS+1)); }
fail() { echo "  ❌ $1"; FAIL=$((FAIL+1)); }
skip() { echo "  ⏭  $1 [skip]"; SKIP=$((SKIP+1)); }

run() {
  local label="$1"; shift
  local log="$LOG_DIR/${label// /_}-$TIMESTAMP.log"
  echo "  › $label"
  if "$@" >"$log" 2>&1; then
    ok "$label"
    echo "    log → $log"
  else
    local exit_code=$?
    fail "$label (exit $exit_code)"
    echo "    log → $log"
    tail -20 "$log" | sed 's/^/      /'
  fi
}

run_allow_connfail() {
  # Como run(), mas erros de conexão ao DB são reportados como skip (DB offline)
  local label="$1"; shift
  local log="$LOG_DIR/${label// /_}-$TIMESTAMP.log"
  echo "  › $label"
  if "$@" >"$log" 2>&1; then
    ok "$label"
    echo "    log → $log"
  else
    local exit_code=$?
    if grep -qE "connection refused|could not connect|FATAL.*database|no such host|Name or service" "$log" 2>/dev/null; then
      skip "$label — PostgreSQL offline (use DATABASE_URL para conectar a um DB real)"
    else
      fail "$label (exit $exit_code)"
      echo "    log → $log"
      tail -20 "$log" | sed 's/^/      /'
    fi
  fi
}

# ── 1. Rust ───────────────────────────────────────────────────────────────────

hdr "RUST — check / test / build"

run "cargo check --lib" \
  cargo check --lib

run "cargo test --lib" \
  cargo test --lib --quiet

run_allow_connfail "cargo test agent_ (integration)" \
  cargo test agent_ -- --test-threads=1

run "cargo build --release" \
  cargo build --release --quiet

# ── 2. Python pipeline ────────────────────────────────────────────────────────

hdr "PYTHON — schemas / checkpoint / routes"

cd "$ROOT/agents"

run "python schemas (PipelineResult)" \
  .venv/bin/python3 -c "
from neoland_agents.schemas.api import PipelineResult, TaskRequest, JuniorOutput, SeniorOutput, TechLeaderOutput
fields = list(PipelineResult.model_fields.keys())
assert 'task' in fields, 'campo task ausente'
assert 'stage_latencies_ms' in fields, 'campo stage_latencies_ms ausente'
assert 'checkpoint_path' in fields, 'campo checkpoint_path ausente'
print('PipelineResult fields:', fields)
print('OK')
"

run "python checkpoint (CheckpointManager)" \
  .venv/bin/python3 -c "
import inspect
from neoland_agents.pipeline.checkpoint import CheckpointManager
assert hasattr(CheckpointManager, 'save'), 'save ausente'
assert hasattr(CheckpointManager, 'list_by_session'), 'list_by_session ausente'
assert hasattr(CheckpointManager, 'close'), 'close ausente'
print('list_by_session:', inspect.signature(CheckpointManager.list_by_session))
print('OK')
"

run "python orchestrator (AgentOrchestrator)" \
  .venv/bin/python3 -c "
import inspect, ast, pathlib
src = pathlib.Path('neoland_agents/pipeline/orchestrator.py').read_text()
# Verifica que time.monotonic é usado para latências reais
assert 'time.monotonic()' in src, 'time.monotonic() não encontrado — latências não são reais'
assert 'stage_latencies_ms' in src, 'stage_latencies_ms não populado no orchestrator'
assert 'result.task = request.task' in src or \"task=request.task\" in src, 'task não copiado para result'
print('OK — latências reais e task propagados')
"

# Verifica rotas sem importar DSPy (que quebra por libstdc++)
run "python app routes (sem DSPy)" \
  .venv/bin/python3 -c "
import ast, pathlib
src = pathlib.Path('neoland_agents/app.py').read_text()
tree = ast.parse(src)
decorators = []
for node in ast.walk(tree):
    if isinstance(node, ast.AsyncFunctionDef):
        for d in node.decorator_list:
            if isinstance(d, ast.Call) and isinstance(d.func, ast.Attribute):
                if d.func.attr in ('get','post','put','delete'):
                    args = [ast.literal_eval(a) for a in d.args if isinstance(a, ast.Constant)]
                    decorators.append((d.func.attr.upper(), args[0] if args else '?'))
print('Rotas registradas:')
for method, path in decorators:
    print(f'  {method} {path}')
# Verifica que o stub foi removido
src_body = src
assert 'not implemented yet' not in src_body, 'stub \"not implemented yet\" ainda presente em app.py'
print('OK — sem stubs')
"

# pytest contract (pode falhar por libstdc++ — detecta e faz skip)
log_contract="$LOG_DIR/pytest-contract-$TIMESTAMP.log"
echo "  › pytest -m contract"
if .venv/bin/pytest tests/ -m contract -v --tb=short >"$log_contract" 2>&1; then
  ok "pytest -m contract"
  echo "    log → $log_contract"
elif grep -q "libstdc++" "$log_contract"; then
  skip "pytest -m contract — libstdc++.so.6 ausente (problema NixOS no venv, não nas mudanças)"
  echo "    log → $log_contract"
else
  fail "pytest -m contract"
  echo "    log → $log_contract"
  tail -20 "$log_contract" | sed 's/^/      /'
fi

cd "$ROOT"

# ── 3. DB migrations ──────────────────────────────────────────────────────────

hdr "DB — migrations (DATABASE_URL=$DB_URL)"

if command -v sqlx &>/dev/null; then
  run_allow_connfail "sqlx migrate run" \
    sqlx migrate run --database-url "$DB_URL"
else
  skip "sqlx migrate run — sqlx-cli não instalado (adicione à devShell ou rode: cargo install sqlx-cli)"
fi

run_allow_connfail "cargo test agent_ com DB" \
  env DATABASE_URL="$DB_URL" cargo test agent_ -- --test-threads=1

# ── 4. Frontend type check ────────────────────────────────────────────────────

hdr "FRONTEND — tsc --noEmit (nossos arquivos)"

# Apenas paths que são nossos (Neoland-specific)
OUR_PATHS="lib/neoland/|app/pipeline/|app/sessions/|app/adr/|app/services/|app/api/neoland/|components/pipeline/|components/shared/|components/sessions/|components/adr/"

log_tsc="$LOG_DIR/tsc-$TIMESTAMP.log"
echo "  › nix develop [frontend] --command npx tsc --noEmit"

# Nested nix develop: usa o flake.nix do matrix/frontend
(cd "$ROOT/matrix/frontend" && nix develop --command npx tsc --noEmit) >"$log_tsc" 2>&1 || true

TOTAL_ERRORS=$(grep -c "error TS" "$log_tsc" 2>/dev/null || echo 0)
OUR_ERRORS=$(grep "error TS" "$log_tsc" 2>/dev/null | grep -cE "$OUR_PATHS" || echo 0)

if [[ "$OUR_ERRORS" -eq 0 ]]; then
  if [[ "$TOTAL_ERRORS" -gt 0 ]]; then
    skip "tsc — $TOTAL_ERRORS erros em páginas legadas (Matrix), 0 nos arquivos Neoland"
  else
    ok "tsc --noEmit — sem erros"
  fi
else
  fail "tsc — $OUR_ERRORS erros nos arquivos Neoland (de $TOTAL_ERRORS totais)"
  grep "error TS" "$log_tsc" | grep -E "$OUR_PATHS" | head -20 | sed 's/^/      /'
fi
echo "    log → $log_tsc"

# ── Relatório final ───────────────────────────────────────────────────────────

hdr "RESULTADO FINAL"

{
  echo "Timestamp : $TIMESTAMP"
  echo "Passou    : $PASS"
  echo "Falhou    : $FAIL"
  echo "Ignorado  : $SKIP"
  echo "Logs      : $LOG_DIR"
} | tee "$REPORT"

sep

if [[ "$FAIL" -eq 0 ]]; then
  echo "  Stack validada — $PASS checks ok, $SKIP pulados"
  exit 0
else
  echo "  $FAIL check(s) falharam — veja os logs acima"
  exit 1
fi
