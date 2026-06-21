#!/usr/bin/env python3
"""Parse ADR markdown files into various output formats for the knowledge base."""

import argparse
import json
import re
import sys
from pathlib import Path


def parse_frontmatter(text: str) -> tuple[dict, str]:
    """Extract YAML-style frontmatter (key: value lines before first blank line or ##)."""
    meta: dict = {}
    lines = text.splitlines()
    body_start = 0

    # Strip leading ---
    if lines and lines[0].strip() == "---":
        for i, line in enumerate(lines[1:], 1):
            if line.strip() == "---":
                body_start = i + 1
                break
            m = re.match(r'^(\w+):\s*"?([^"]*)"?\s*$', line)
            if m:
                meta[m.group(1)] = m.group(2).strip()
    else:
        for i, line in enumerate(lines):
            m = re.match(r'^(\w+):\s*"?([^"]*)"?\s*$', line)
            if m:
                meta[m.group(1)] = m.group(2).strip()
                body_start = i + 1
            elif line.strip() == "" and meta:
                body_start = i + 1
                break

    body = "\n".join(lines[body_start:])
    return meta, body


def load_adrs(adr_dir: Path) -> list[dict]:
    adrs = []
    for f in sorted(adr_dir.glob("*.md")):
        text = f.read_text()
        meta, body = parse_frontmatter(text)
        adrs.append({
            "id":     meta.get("id", f.stem),
            "title":  meta.get("title", ""),
            "status": meta.get("status", ""),
            "date":   meta.get("date", ""),
            "file":   str(f),
            "body":   body,
        })
    return adrs


def fmt_knowledge(adrs: list[dict]) -> dict:
    return {
        "version": "1.0",
        "adrs": [
            {"id": a["id"], "title": a["title"], "status": a["status"], "date": a["date"]}
            for a in adrs
        ],
    }


def fmt_graph(adrs: list[dict]) -> dict:
    nodes = [{"id": a["id"], "label": a["title"]} for a in adrs]
    return {"nodes": nodes, "edges": []}


def fmt_spectre(adrs: list[dict]) -> dict:
    return {
        "corpus": [
            {"id": a["id"], "text": f"{a['title']}\n{a['body'][:500]}"}
            for a in adrs
        ]
    }


def fmt_phantom(adrs: list[dict]) -> dict:
    return {
        "training_data": [
            {"prompt": f"ADR {a['id']}: {a['title']}", "completion": a["body"][:300]}
            for a in adrs
        ]
    }


def fmt_index(adrs: list[dict]) -> dict:
    return {
        "index": {a["id"]: {"title": a["title"], "status": a["status"]} for a in adrs}
    }


FORMATTERS = {
    "knowledge": fmt_knowledge,
    "graph":     fmt_graph,
    "spectre":   fmt_spectre,
    "phantom":   fmt_phantom,
    "index":     fmt_index,
}


def main() -> int:
    parser = argparse.ArgumentParser(description="Parse ADRs into knowledge formats")
    parser.add_argument("adr_dir",         help="Directory containing accepted ADR .md files")
    parser.add_argument("--format",        required=True, choices=FORMATTERS)
    parser.add_argument("--pretty",        action="store_true")
    parser.add_argument("-o", "--output",  help="Output file (default: stdout)")
    args = parser.parse_args()

    adr_dir = Path(args.adr_dir)
    if not adr_dir.exists():
        print(f"⚠  ADR directory not found: {adr_dir} — writing empty output", file=sys.stderr)
        adrs: list[dict] = []
    else:
        adrs = load_adrs(adr_dir)

    result = FORMATTERS[args.format](adrs)
    indent = 2 if args.pretty else None
    text   = json.dumps(result, indent=indent, ensure_ascii=False)

    if args.output:
        out = Path(args.output)
        out.parent.mkdir(parents=True, exist_ok=True)
        out.write_text(text + "\n")
        print(f"✓ Wrote {args.format} → {out} ({len(adrs)} ADRs)")
    else:
        print(text)

    return 0


if __name__ == "__main__":
    sys.exit(main())
