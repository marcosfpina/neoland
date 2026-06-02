#!/usr/bin/env bash
# Secret / credential leak scan.
# Uses gitleaks if available; falls back to a regex sweep over staged/tracked files.
set -euo pipefail

TARGET="${1:-.}"

echo "Secret scan: $TARGET"

if command -v gitleaks >/dev/null 2>&1; then
  gitleaks detect --source "$TARGET" --no-git --redact --exit-code 1
  echo "  ✓  gitleaks: no secrets found"
  exit 0
fi

# Fallback: grep for high-signal patterns (no gitleaks installed)
PATTERNS=(
  'AKIA[0-9A-Z]{16}'                  # AWS access key
  'sk-[a-zA-Z0-9]{32,}'              # OpenAI / generic sk- key
  'ghp_[a-zA-Z0-9]{36}'              # GitHub PAT
  'glpat-[a-zA-Z0-9_-]{20}'         # GitLab PAT
  'ENC\[AES256_GCM'                   # SOPS encrypted (allowed — not plaintext)
)

FAIL=0
for pattern in "${PATTERNS[@]}"; do
  # SOPS-encrypted values are fine; skip them
  [[ "$pattern" == *"SOPS"* ]] && continue
  if git -C "$TARGET" grep -rE "$pattern" \
       --untracked \
       -- ':!*.sops.*' ':!*.enc.*' ':!flake.lock' \
       2>/dev/null | grep -v "ENC\["; then
    echo "  ✗  Possible secret matched pattern: $pattern"
    FAIL=$((FAIL+1))
  fi
done

if [ "$FAIL" -gt 0 ]; then
  echo "Secret scan FAILED — $FAIL pattern(s) matched."
  exit 1
fi

echo "  ✓  No plaintext secrets detected (fallback scan)"
