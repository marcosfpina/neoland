#!/usr/bin/env bash
# ADR structural validation — checks frontmatter, status dirs, and ID uniqueness.
# Used by CI (copilot-setup-steps.yml) and local dev.
set -euo pipefail

PASS=0
FAIL=0

ok()   { echo "  ✓  $*"; PASS=$((PASS+1)); }
fail() { echo "  ✗  $*" >&2; FAIL=$((FAIL+1)); }

ADR_ROOT="${ADR_ROOT:-$(git rev-parse --show-toplevel 2>/dev/null || pwd)}"

# ── Frontmatter completeness ──────────────────────────────────────────────────
for status_dir in proposed accepted superseded rejected; do
  dir="$ADR_ROOT/adr/$status_dir"
  [ -d "$dir" ] || continue
  for file in "$dir"/*.md; do
    [ -f "$file" ] || continue
    grep -q "^id:"     "$file" || { fail "Missing 'id' in $file";     continue; }
    grep -q "^title:"  "$file" || { fail "Missing 'title' in $file";  continue; }
    grep -q "^status:" "$file" || { fail "Missing 'status' in $file"; continue; }
    ok "$file"
  done
done

# ── Status ↔ directory alignment ─────────────────────────────────────────────
for status_dir in proposed accepted superseded rejected; do
  dir="$ADR_ROOT/adr/$status_dir"
  [ -d "$dir" ] || continue
  for file in "$dir"/*.md; do
    [ -f "$file" ] || continue
    file_status=$(grep "^status:" "$file" | awk '{print $2}' | tr -d '"')
    if [ "$file_status" != "$status_dir" ]; then
      fail "Status mismatch: '$file_status' in '$status_dir/' — $file"
    fi
  done
done
ok "Status ↔ directory alignment"

# ── ID uniqueness ─────────────────────────────────────────────────────────────
python3 - "$ADR_ROOT" << 'EOF'
import re, sys
from pathlib import Path
root = Path(sys.argv[1])
seen = {}
for status_dir in ["proposed", "accepted", "superseded", "rejected"]:
    for f in (root / "adr" / status_dir).glob("*.md"):
        m = re.search(r'^id:\s+"?([^"\n]+)"?', f.read_text(), re.M)
        if not m:
            continue
        adr_id = m.group(1).strip()
        if adr_id in seen:
            print(f"CONFLICT: {adr_id} in {seen[adr_id]} AND {f}", file=sys.stderr)
            sys.exit(1)
        seen[adr_id] = str(f)
print(f"  ✓  No ID conflicts ({len(seen)} ADRs checked)")
EOF

echo ""
echo "ADR validation: PASS=$PASS FAIL=$FAIL"
[ "$FAIL" -eq 0 ] || exit 1
