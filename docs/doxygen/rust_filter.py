#!/usr/bin/env python3
"""Doxygen INPUT_FILTER for Rust — best-effort translation to C++-shaped syntax.

Doxygen has no native Rust parser. This rewrites the most common Rust
patterns into something its C++ parser can pick up:

- strips attributes (`#[...]`, `#![...]`) and visibility (`pub`, `pub(crate)`)
- strips single-line `use ...;` statements (they have no C++ equivalent and
  otherwise get mis-parsed as bogus "members with no name")
- turns `fn`/`async fn` into `auto` so the C++11 trailing-return-type form
  (`auto name(...) -> Ret`) is recognized
- rewrites `impl [Trait for] Type {` into `struct Type {`, reopening the
  same compound so methods attach to the type Doxygen already knows
- appends the trailing `;` C++ requires after a top-level `struct`/`enum`
  body — Rust doesn't have one, and without it Doxygen silently drops the
  whole declaration instead of registering it as a documented compound

Known limitations (accepted trade-off vs. rustdoc): parameter lists
(`name: Type`) and struct fields keep Rust's name-colon-type order instead
of C++'s type-name order, so Doxygen usually finds the function/struct name
and attaches the `///`/`//!` doc comment, but rendered parameter/field
*types* are frequently wrong or missing. Multi-line attributes, multi-line
`use` blocks, and complex generic bounds are not handled — a line that
doesn't match one of the patterns above passes through unchanged.
"""
from __future__ import annotations

import re
import sys

ATTRIBUTE_LINE = re.compile(r"^\s*#!?\[.*\]\s*$")
USE_LINE = re.compile(r"^\s*(?:pub\s+)?use\s+.*;\s*$")
VISIBILITY = re.compile(r"\bpub(\([^)]*\))?\s+")
ASYNC_KEYWORD = re.compile(r"\basync\s+")
FN_KEYWORD = re.compile(r"\bfn\b")
IMPL_HEADER = re.compile(r"^(\s*)impl(?:<[^>]*>)?\s+(?:[\w:]+(?:<[^>]*>)?\s+for\s+)?([\w:]+)")
TOP_LEVEL_TYPE_HEADER = re.compile(r"^\s*(struct|enum)\b")
LONE_CLOSING_BRACE = re.compile(r"^(\s*)\}\s*$")


def translate_line(line: str) -> str:
    if ATTRIBUTE_LINE.match(line) or USE_LINE.match(line):
        return ""

    impl_match = IMPL_HEADER.match(line)
    if impl_match:
        indent, type_name = impl_match.groups()
        return f"{indent}struct {type_name} {{"

    line = VISIBILITY.sub("", line)
    line = ASYNC_KEYWORD.sub("", line)
    line = FN_KEYWORD.sub("auto", line)
    return line


def filter_source(lines: list[str]) -> list[str]:
    out: list[str] = []
    depth = 0
    top_kind: str | None = None  # "type" while inside a depth-0 struct/enum/impl block

    for raw in lines:
        translated = translate_line(raw.rstrip("\n"))

        opening_here = depth == 0 and "{" in translated
        if opening_here:
            is_type = bool(TOP_LEVEL_TYPE_HEADER.match(translated)) or bool(
                re.match(r"^\s*struct\b", translated)
            )
            top_kind = "type" if is_type else "other"

        depth += translated.count("{") - translated.count("}")

        if depth <= 0 and top_kind == "type":
            brace_match = LONE_CLOSING_BRACE.match(translated)
            if brace_match:
                translated = f"{brace_match.group(1)}}};"
            top_kind = None
            depth = max(depth, 0)

        out.append(translated)

    return out


def main() -> int:
    if len(sys.argv) < 2:
        print("usage: rust_filter.py <file>", file=sys.stderr)
        return 1

    with open(sys.argv[1], "r", encoding="utf-8") as fh:
        lines = fh.readlines()

    for line in filter_source(lines):
        sys.stdout.write(line + "\n")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
