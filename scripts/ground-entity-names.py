#!/usr/bin/env python3
"""Ground `data/seed/entity-names.lino` in the Wikidata cache (issue #699).

Batch 2 of the handler migration replaced the `who_is` handler's hardcoded table
of eight people (and their hand-written misspellings) with nearest-surface
search over remembered names. The names themselves live in
`data/seed/entity-names.lino`, and this migration keeps that file *derived*
rather than hand-typed: every surface is the checked-in Wikidata label for the
entity's `grounded-in` id, in each supported response language.

For every `entity <slug> / grounded-in <Q-id>` record it:

  1. Fetches `Special:EntityData/<Q-id>.json` when the cache lacks it, trims the
     document to the labels/descriptions/aliases of the supported languages —
     the same shape the checked-in entity records already use — and writes
     `data/cache/wikidata/entity/<Q-id>.json`.
  2. Generates the lossless `.lino` snapshot with the `wikidata_json_to_lino`
     cargo example, the same codec the rest of the cache is built with.
  3. Rewrites the record's `lexeme`/`surface`/`text` blocks from the cached
     labels, so the seed cannot drift from its grounding and no spelling in it
     is a human guess.

Run `python3 scripts/ground-entity-names.py` (add `--check` to fail instead of
writing, which is what CI wants). Network is only needed the first time an id
is fetched.
"""
from __future__ import annotations

import json
import subprocess
import sys
from pathlib import Path

USER_AGENT = "formal-ai-grounding/1.0 (https://github.com/link-assistant/formal-ai)"
LANGUAGES = ["en", "ru", "hi", "zh"]
SEED = Path("data/seed/entity-names.lino")
CACHE_DIR = Path("data/cache/wikidata/entity")


def fetch_entity(qid: str) -> dict:
    """Return the cached entity document, fetching and trimming it when absent."""
    json_path = CACHE_DIR / f"{qid}.json"
    if not json_path.exists():
        url = f"https://www.wikidata.org/wiki/Special:EntityData/{qid}.json"
        raw = subprocess.run(
            ["curl", "-sfL", "-A", USER_AGENT, url], capture_output=True, check=True
        ).stdout
        entity = json.loads(raw)["entities"][qid]
        trimmed = {
            "type": entity.get("type", "item"),
            "id": qid,
            "labels": {
                lang: entity["labels"][lang]
                for lang in LANGUAGES
                if lang in entity.get("labels", {})
            },
            "descriptions": {
                lang: entity["descriptions"][lang]
                for lang in LANGUAGES
                if lang in entity.get("descriptions", {})
            },
            "aliases": {
                lang: entity["aliases"][lang]
                for lang in LANGUAGES
                if lang in entity.get("aliases", {})
            },
        }
        CACHE_DIR.mkdir(parents=True, exist_ok=True)
        with json_path.open("w", encoding="utf-8") as handle:
            json.dump({"entities": {qid: trimmed}, "success": 1}, handle,
                      ensure_ascii=False, indent=2)
            handle.write("\n")
    lino_path = CACHE_DIR / f"{qid}.lino"
    if not lino_path.exists():
        subprocess.run(
            ["cargo", "run", "--quiet", "--example", "wikidata_json_to_lino",
             "--", qid, str(json_path), str(lino_path)],
            check=True,
        )
    return json.loads(json_path.read_text(encoding="utf-8"))["entities"][qid]


def render(records: list[tuple[str, str, dict]]) -> str:
    """Render the whole registry from the cached labels."""
    lines = ["entity_names"]
    for slug, qid, entity in records:
        lines.append(f"  entity {slug}")
        lines.append(f"    grounded-in {qid}")
        for language in LANGUAGES:
            label = entity.get("labels", {}).get(language, {}).get("value")
            if not label:
                continue
            lines.append(f"    lexeme {language}")
            lines.append("      surface")
            lines.append(f'        text "{label}"')
    return "\n".join(lines) + "\n"


def parse_records(text: str) -> list[tuple[str, str]]:
    records = []
    slug = None
    for line in text.split("\n"):
        stripped = line.strip()
        if stripped.startswith("entity ") and line.startswith("  entity "):
            slug = stripped.split(" ", 1)[1]
        elif stripped.startswith("grounded-in ") and slug:
            records.append((slug, stripped.split(" ", 1)[1]))
            slug = None
    return records


def main(argv: list[str]) -> int:
    check = "--check" in argv[1:]
    text = SEED.read_text(encoding="utf-8")
    records = [(slug, qid, fetch_entity(qid)) for slug, qid in parse_records(text)]
    rendered = render(records)
    if rendered == text:
        print(f"{SEED}: {len(records)} entities, already grounded")
        return 0
    if check:
        print(f"{SEED} does not match its Wikidata grounding; run "
              "python3 scripts/ground-entity-names.py", file=sys.stderr)
        return 1
    SEED.write_text(rendered, encoding="utf-8")
    print(f"{SEED}: rewrote {len(records)} entities from the Wikidata cache")
    return 0


if __name__ == "__main__":
    raise SystemExit(main(sys.argv))
