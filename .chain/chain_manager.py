#!/usr/bin/env python3
"""Chain integrity manager — verify Merkle-style ADR signing chain."""

import hashlib
import json
import sys
from pathlib import Path

CHAIN_FILE = Path(__file__).parent / "chain.json"


def verify() -> int:
    if not CHAIN_FILE.exists():
        print("ℹ  No chain file found — skipping verification")
        return 0

    try:
        data = json.loads(CHAIN_FILE.read_text())
    except (json.JSONDecodeError, ValueError) as exc:
        print(f"❌ Invalid JSON in chain file: {exc}")
        return 1

    if "chain" not in data:
        print("❌ Invalid chain structure: missing 'chain' key")
        return 1

    chain = data["chain"]
    errors: list[str] = []

    for i, block in enumerate(chain):
        # Genesis block (block_number == 0) is a sentinel — skip hash check
        if block.get("block_number", i) == 0:
            continue

        adr_id     = block.get("adr_id", "")
        chain_hash = block.get("content_hash", "")

        if not adr_id or not chain_hash:
            continue

        # Locate the ADR file
        candidates = list(Path("adr/accepted").glob(f"{adr_id}*.md"))
        if not candidates:
            # ADR may have been superseded/removed — warn but don't fail
            print(f"⚠  Block {i}: {adr_id} not found in adr/accepted/ — skipped")
            continue

        h = hashlib.sha256()
        with open(candidates[0], "rb") as f:
            for chunk in iter(lambda: f.read(8192), b""):
                h.update(chunk)
        actual = h.hexdigest()

        if actual != chain_hash:
            errors.append(
                f"TAMPERED: {adr_id}\n"
                f"  chain:   {chain_hash[:32]}...\n"
                f"  current: {actual[:32]}...\n"
                f"  Fix: bash scripts/chain-resign.sh {adr_id}"
            )

    if errors:
        for e in errors:
            print(f"❌ {e}")
        return 1

    print(f"✓ Chain verified ({len(chain) - 1} blocks checked)")
    return 0


def main() -> int:
    cmd = sys.argv[1] if len(sys.argv) > 1 else ""
    if cmd == "verify":
        return verify()
    print("Usage: chain_manager.py verify", file=sys.stderr)
    return 1


if __name__ == "__main__":
    sys.exit(main())
