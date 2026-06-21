#!/usr/bin/env bash
# scripts/chain-resign.sh <adr-id>
# Re-signs a tampered ADR in the chain after authorized content changes.
set -euo pipefail

ADR_ID="${1:?Usage: chain-resign.sh <adr-id>}"
ADR_ROOT="${ADR_ROOT:-$(git rev-parse --show-toplevel 2>/dev/null || pwd)}"
CHAIN_FILE="$ADR_ROOT/.chain/chain.json"

if [ ! -f "$CHAIN_FILE" ]; then
  echo "❌ No chain file found at $CHAIN_FILE"
  exit 1
fi

ADR_FILE=$(find "$ADR_ROOT/adr/accepted" -name "${ADR_ID}*.md" 2>/dev/null | head -1)
if [ -z "$ADR_FILE" ]; then
  echo "❌ Accepted ADR '$ADR_ID' not found"
  exit 1
fi

python3 - "$CHAIN_FILE" "$ADR_ID" "$ADR_FILE" <<'EOF'
import json, hashlib, sys
from pathlib import Path

chain_path = Path(sys.argv[1])
adr_id     = sys.argv[2]
adr_file   = Path(sys.argv[3])

data = json.loads(chain_path.read_text())
chain = data.get("chain", [])

h = hashlib.sha256()
with open(adr_file, "rb") as f:
    for chunk in iter(lambda: f.read(8192), b""):
        h.update(chunk)
new_hash = h.hexdigest()

updated = False
for block in chain:
    if block.get("adr_id") == adr_id:
        block["content_hash"] = new_hash
        updated = True
        break

if not updated:
    print(f"⚠ '{adr_id}' not found in chain — nothing to resign")
    sys.exit(0)

chain_path.write_text(json.dumps(data, indent=2))
print(f"✓ Re-signed {adr_id} → {new_hash[:16]}...")
EOF

echo "✓ Chain updated for $ADR_ID"
