#!/usr/bin/env python3
"""Migrate seed LiNo `codepoints N N N` byte-dumps back to readable values.

Issue #398 review (comment 4660584608) requires that seed data be human
readable rather than obfuscated as codepoint byte-dumps. The seed and canonical
parsers already accept bare references and double-quoted strings, so every
`key codepoints <ints>` line is decoded to its text and re-emitted as either:

* a bare reference, when the text is a safe bare token (no whitespace and none
  of the parser-significant characters), or
* a double-quoted string with JSON-compatible escaping otherwise.

Runtime values are preserved exactly: the seed/web/e2e parsers decode quoted
strings to the same string the codepoints decoded to.
"""

from __future__ import annotations

import os
import re
import sys

SEED_DIR = os.path.join(os.path.dirname(__file__), "..", "data", "seed")

_COMMENT = r"(?:\s+#.*)?"
VALUE_RE = re.compile(r"^(\s*)(\S+)\s+codepoints(?:\s+([0-9 ]+?))?" + _COMMENT + r"\s*$")
# A bare `codepoints <ints>` leaf node carries a lexeme surface string; it is
# re-emitted under the readable `text` key consumed by `surface_text`. The old
# `# surface text` annotation is dropped because the value is now readable.
LEAF_RE = re.compile(r"^(\s*)codepoints(?:\s+([0-9 ]+?))?" + _COMMENT + r"\s*$")
UNSAFE = set(" \t\r\n():\"'|#`\\")


def decode(nums: str) -> str:
    return "".join(chr(int(token)) for token in nums.split() if token)


def safe_bare(text: str) -> bool:
    if not text:
        return False
    if any(ch in UNSAFE for ch in text):
        return False
    if any(ord(ch) < 0x20 for ch in text):
        return False
    return True


def quote(text: str) -> str:
    # Pick a delimiter that does not occur in the text so the inner quote never
    # needs backslash-escaping. The canonical Links Notation parser mishandles a
    # `\"` escape immediately followed by `)`, so we must avoid emitting `\"`.
    if '"' not in text:
        delimiter = '"'
    elif "'" not in text:
        delimiter = "'"
    elif "`" not in text:
        delimiter = "`"
    else:  # pragma: no cover - no seed value uses all three quote styles
        delimiter = '"'
    escaped = text.replace("\\", "\\\\").replace("\n", "\\n").replace("\r", "\\r")
    escaped = escaped.replace(delimiter, "\\" + delimiter)
    return f"{delimiter}{escaped}{delimiter}"


def render(indent: str, key: str, text: str) -> str:
    if text == "":
        return f"{indent}{key}"
    if safe_bare(text):
        return f"{indent}{key} {text}"
    return f"{indent}{key} {quote(text)}"


def migrate_line(line: str) -> str:
    leaf = LEAF_RE.match(line)
    if leaf:
        indent, nums = leaf.group(1), leaf.group(2) or ""
        return render(indent, "text", decode(nums))
    match = VALUE_RE.match(line)
    if not match:
        return line
    indent, key, nums = match.group(1), match.group(2), match.group(3) or ""
    return render(indent, key, decode(nums))


def main() -> int:
    changed = 0
    for name in sorted(os.listdir(SEED_DIR)):
        if not name.endswith(".lino"):
            continue
        path = os.path.join(SEED_DIR, name)
        with open(path, encoding="utf-8") as handle:
            lines = handle.read().split("\n")
        new_lines = [migrate_line(line) for line in lines]
        if new_lines != lines:
            changed += 1
            with open(path, "w", encoding="utf-8") as handle:
                handle.write("\n".join(new_lines))
    print(f"migrated {changed} seed files")
    return 0


if __name__ == "__main__":
    sys.exit(main())
