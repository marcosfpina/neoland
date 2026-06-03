#!/usr/bin/env python3
"""Check that detected features have been integrated (tests, ADR, docs)."""

import argparse
import json
import sys
from pathlib import Path


def check_integration(feature_log: Path, fail_on_unintegrated: bool) -> int:
    if not feature_log.exists():
        print("ℹ  No feature tracking log found — nothing to check")
        return 0

    try:
        entries: list = json.loads(feature_log.read_text())
    except (json.JSONDecodeError, ValueError):
        print("⚠  Feature log is malformed — skipping integration check")
        return 0

    unintegrated = [
        e for e in entries
        if not e.get("integrated", False)
    ]

    if not unintegrated:
        print(f"✓  All {len(entries)} detected features are integrated")
        return 0

    print(f"⚠  {len(unintegrated)} feature(s) pending integration:")
    for e in unintegrated:
        print(f"  [{e.get('feature', '?')}] {e.get('description', '')} "
              f"(commit {str(e.get('commit', '?'))[:8]})")

    if fail_on_unintegrated:
        print("❌  --fail-on-unintegrated set — failing")
        return 1

    return 0


def main() -> int:
    parser = argparse.ArgumentParser(description="Check feature integration status")
    parser.add_argument("--feature-log",          required=True)
    parser.add_argument("--fail-on-unintegrated",  action="store_true")
    args = parser.parse_args()

    return check_integration(Path(args.feature_log), args.fail_on_unintegrated)


if __name__ == "__main__":
    sys.exit(main())
