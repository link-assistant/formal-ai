#!/usr/bin/env python3
"""Wiktionary grounding pipeline for the canonical seed (issue #398, defect: Wiktionary depth).

PR #399 review (deep review of `92a29b0`, open item #1) flags that
`data/cache/wiktionary/` holds a **single** placeholder entry while the seed
already grounds 140+ meanings. A system meant to ground *every* word — its
parts of speech, senses, and pronunciations — needs Wiktionary used heavily,
one cached entry per grounded English surface.

This script closes that gap by *algorithm*, not by hand:

  1. **Discover** candidate lemmas straight from the grounded seed: every
     single-word English `surface / text` belonging to a meaning that already
     carries a `grounded-in <id>` anchor. The word list is therefore derived
     from the data, never hardcoded — re-running it after grounding more
     meanings finds more candidates automatically.
  2. **Fetch** each lemma from the Wiktionary-backed Free Dictionary API
     (`https://api.dictionaryapi.dev/api/v2/entries/en/<word>`, the same source
     and schema as the existing `en/reference.json` snapshot), which serves
     Wiktionary content under CC BY-SA 3.0.
  3. **Verify** the response actually describes the requested lemma (a non-empty
     list whose first `word` matches case-insensitively, with at least one
     `meanings[].definitions[].definition`). Anything else — a 404, a redirect
     to a different lemma, an empty body — is skipped, never cached.
  4. **Cache** the verified entry as pretty multi-line JSON
     (`data/cache/wiktionary/en/<word>.json`) and regenerate the lossless
     `.lino` snapshot beside it with the `wikidata_json_to_lino` cargo example —
     the very codec `wiktionary_cache_is_pretty_printed_and_rebuilds_full_json`
     round-trips in CI.

Idempotent: a word whose `.json` + `.lino` already exist is left untouched, so
re-runs only add what is missing. Network is only needed the first time a lemma
is fetched.

Run with `python3 scripts/ground-wiktionary.py` (requires `curl`, `python3`,
and a built `cargo`). Pass words explicitly to ground a specific batch:
`python3 scripts/ground-wiktionary.py water apple bread`.
"""

import json
import re
import subprocess
import sys
import time
from pathlib import Path

API = "https://api.dictionaryapi.dev/api/v2/entries/en/"
USER_AGENT = "formal-ai-grounding/1.0 (https://github.com/link-assistant/formal-ai)"
CACHE_DIR = Path("data/cache/wiktionary/en")
SEED_DIR = Path("data/seed")
SINGLE_WORD = re.compile(r"[a-z]{2,}")


def grounded_english_surfaces() -> list[str]:
    """Every single-word English surface of a `grounded-in` meaning, sorted."""
    found: set[str] = set()
    for path in sorted(SEED_DIR.glob("meanings-*.lino")):
        lines = path.read_text(encoding="utf-8").split("\n")
        # Split the file into top-level meaning blocks (a slug sits at indent 2).
        blocks: list[list[str]] = []
        current: list[str] | None = None
        for line in lines:
            if re.match(r"^  [^ ]", line):
                if current is not None:
                    blocks.append(current)
                current = [line]
            elif current is not None:
                current.append(line)
        if current is not None:
            blocks.append(current)
        for block in blocks:
            if not any(line.strip().startswith("grounded-in ") for line in block):
                continue
            in_english = False
            for line in block:
                stripped = line.strip()
                if stripped.startswith("lexeme "):
                    in_english = stripped == "lexeme en"
                    continue
                if in_english and stripped.startswith("text "):
                    value = stripped[len("text "):].strip().strip("\"'`")
                    if SINGLE_WORD.fullmatch(value):
                        found.add(value)
    return sorted(found)


def fetch(word: str) -> list | None:
    """Fetch and verify a Wiktionary entry; return the parsed list or None."""
    result = subprocess.run(
        ["curl", "-s", "-m", "30", "-A", USER_AGENT, f"{API}{word}"],
        capture_output=True,
        text=True,
    )
    if result.returncode != 0 or not result.stdout.strip():
        return None
    try:
        data = json.loads(result.stdout)
    except json.JSONDecodeError:
        return None
    if not isinstance(data, list) or not data:
        return None
    first = data[0]
    if not isinstance(first, dict):
        return None
    if str(first.get("word", "")).lower() != word.lower():
        return None
    has_definition = any(
        isinstance(meaning, dict)
        and any(
            isinstance(definition, dict) and definition.get("definition")
            for definition in meaning.get("definitions", [])
        )
        for entry in data
        if isinstance(entry, dict)
        for meaning in entry.get("meanings", [])
    )
    return data if has_definition else None


def lino_path(word: str) -> Path:
    return CACHE_DIR / f"{word}.lino"


def json_path(word: str) -> Path:
    return CACHE_DIR / f"{word}.json"


def write_entry(word: str, data: list) -> None:
    CACHE_DIR.mkdir(parents=True, exist_ok=True)
    json_path(word).write_text(
        json.dumps(data, indent=2, ensure_ascii=False) + "\n", encoding="utf-8"
    )
    result = subprocess.run(
        [
            "cargo",
            "run",
            "--quiet",
            "--example",
            "wikidata_json_to_lino",
            "entry",
            str(json_path(word)),
            str(lino_path(word)),
        ],
        capture_output=True,
        text=True,
    )
    if result.returncode != 0:
        json_path(word).unlink(missing_ok=True)
        raise SystemExit(f"wikidata_json_to_lino failed for {word}: {result.stderr}")


def main(argv: list[str]) -> int:
    words = argv[1:] if len(argv) > 1 else grounded_english_surfaces()
    print(f"grounding {len(words)} candidate lemma(s) against Wiktionary")
    added, skipped, missing = 0, 0, []
    for word in words:
        if json_path(word).is_file() and lino_path(word).is_file():
            skipped += 1
            continue
        data = fetch(word)
        if data is None:
            missing.append(word)
            continue
        write_entry(word, data)
        added += 1
        print(f"  + {word}")
        time.sleep(0.2)
    cached = len(list(CACHE_DIR.glob("*.lino")))
    print(
        f"done: +{added} new, {skipped} already cached, "
        f"{len(missing)} without a Wiktionary entry; {cached} total entries"
    )
    if missing:
        print("  no Wiktionary entry for: " + ", ".join(missing))
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv))
