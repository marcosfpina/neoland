#!/usr/bin/env python3
"""Detect features introduced by a commit based on changed files."""

import argparse
import json
import re
import sys
from pathlib import Path

FEATURE_PATTERNS = [
    (r"src/tui/",          "tui",         "TUI layer change"),
    (r"src/server/",       "server",      "Server layer change"),
    (r"src/matrix/",       "matrix",      "Matrix client change"),
    (r"agents/",           "agents",      "Agent system change"),
    (r"adr/accepted/",     "adr",         "ADR accepted"),
    (r"\.github/workflows/","ci",          "CI/CD change"),
    (r"Cargo\.toml",       "deps",        "Dependency change"),
    (r"flake\.nix",        "nix",         "Nix build change"),
]


def detect(commit: str, files: list[str], min_confidence: float) -> list[dict]:
    detected = []
    seen_features: set[str] = set()

    for path in files:
        for pattern, feature, description in FEATURE_PATTERNS:
            if re.search(pattern, path) and feature not in seen_features:
                seen_features.add(feature)
                detected.append({
                    "commit": commit,
                    "feature": feature,
                    "description": description,
                    "trigger_file": path,
                    "confidence": 0.85,
                })

    return [d for d in detected if d["confidence"] >= min_confidence]


def main() -> int:
    parser = argparse.ArgumentParser(description="Detect features in a commit")
    parser.add_argument("--commit",         required=True)
    parser.add_argument("--files",          required=True)
    parser.add_argument("--min-confidence", type=float, default=0.7)
    parser.add_argument("--output",         default=".feature_tracking.json")
    args = parser.parse_args()

    files = [f for f in args.files.split() if f]
    features = detect(args.commit, files, args.min_confidence)

    output_path = Path(args.output)
    existing: list = []
    if output_path.exists():
        try:
            existing = json.loads(output_path.read_text())
        except (json.JSONDecodeError, ValueError):
            existing = []

    existing.extend(features)
    output_path.write_text(json.dumps(existing, indent=2))

    if features:
        print(f"Detected {len(features)} feature(s) in commit {args.commit[:8]}:")
        for f in features:
            print(f"  [{f['feature']}] {f['description']} ({f['confidence']:.0%})")
        return 1  # signal to GHA: feature_detected=true

    print(f"No significant features detected in commit {args.commit[:8]}")
    return 0


if __name__ == "__main__":
    sys.exit(main())
