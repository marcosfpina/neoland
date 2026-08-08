#!/usr/bin/env python3
"""Detect features introduced by a commit based on changed files."""

import argparse
import json
import re
import subprocess
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

# Features de infra são validadas por jobs dedicados do CI
# (nix-check, adr-validation, release) — integradas por definição.
SELF_INTEGRATED = {"ci", "deps", "nix", "adr"}

# Features de código contam como integradas quando o mesmo commit
# carrega evidência de integração (testes ou ADR).
INTEGRATION_EVIDENCE = [
    r"^tests/",
    r"^agents/tests/",
    r"^adr/(proposed|accepted)/",
]

# Tipos de commit (conventional commits) que não introduzem comportamento
# novo — limpeza, refactor sem mudança semântica, docs, testes, infra.
# Não faz sentido exigir evidência de integração deles (ex.: remover código
# morto em src/server/ não tem o que testar). feat/fix continuam exigindo.
NON_FEATURE_TYPES = {"chore", "refactor", "docs", "test", "tests", "style", "ci", "build", "revert"}


def has_integration_evidence(files: list[str]) -> bool:
    return any(
        re.search(pattern, path)
        for path in files
        for pattern in INTEGRATION_EVIDENCE
    )


def commit_subject(commit: str) -> str:
    try:
        return subprocess.run(
            ["git", "log", "-1", "--format=%s", commit],
            capture_output=True, text=True, check=True,
        ).stdout.strip()
    except (subprocess.CalledProcessError, FileNotFoundError, OSError):
        return ""


def is_non_feature_commit(subject: str) -> bool:
    match = re.match(r"^(\w+)[(!:]", subject)
    return bool(match) and match.group(1).lower() in NON_FEATURE_TYPES


def detect(commit: str, files: list[str], min_confidence: float) -> list[dict]:
    detected = []
    seen_features: set[str] = set()
    evidence = has_integration_evidence(files) or is_non_feature_commit(commit_subject(commit))

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
                    "integrated": feature in SELF_INTEGRATED or evidence,
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
