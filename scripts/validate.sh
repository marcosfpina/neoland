#!/usr/bin/env bash
# scripts/validate.sh <subsystem>
# Dispatches to the appropriate validation script.
set -euo pipefail

SUBSYSTEM="${1:-}"
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

case "$SUBSYSTEM" in
  adr)
    exec bash "$SCRIPT_DIR/validate-adr.sh"
    ;;
  *)
    echo "Usage: $0 <adr>" >&2
    exit 1
    ;;
esac
