#!/usr/bin/env python3
"""Ground English lemmas against Open English WordNet 2024 (issue #398, PR #399).

PR #399 review (comment 4668929105) requires a working multi-source ``view``
whose first new source is WordNet: "archived OEWN 2024 mirrored in-repo (raw +
``.lino``), reachable by meanings." This script is that importer.

Unlike the live Wikidata/Wiktionary APIs, WordNet ships as a single downloadable
lexicon, so one download grounds the whole English content vocabulary offline —
no per-word network round-trips. The pipeline:

  1. **Discover** the work-list: by default, every still-undefined English lemma
     reported by ``scripts/audit-total-closure.py --candidates`` (so the list is
     derived from the data, never hardcoded). Pass words explicitly to ground a
     specific batch: ``python3 scripts/ground-wordnet.py algorithm ability``.
  2. **Look up** each lemma in OEWN 2024 via the ``wn`` library, downloading the
     lexicon into ``WN_DATA_DIR`` (default ``~/.formal-ai-wndata``) the first
     time. Only the compact per-lemma projection is mirrored in-repo; the multi-
     megabyte source database stays out of git, matching R398-09 (small repo,
     on-demand expansion).
  3. **Cache** each grounded lemma as pretty JSON
     (``data/cache/wordnet/en/<lemma>.json``) plus the lossless Links-Notation
     projection beside it (``<lemma>.lino``), regenerated with the same
     ``wikidata_json_to_lino`` codec the Wiktionary/Wikidata caches use, so the
     round-trip CI gate covers it too.

Idempotent: a lemma whose ``.json`` + ``.lino`` already exist is skipped, so
re-runs only add what is missing.

License: Open English WordNet is published under CC BY 4.0
(https://github.com/globalwordnet/english-wordnet), recorded per entry and in
``data/seed/sources-registry.lino``.
"""
from __future__ import annotations

import json
import os
import re
import subprocess
import sys
from pathlib import Path

LEXICON = "oewn:2024"
LICENSE = {
    "name": "CC BY 4.0",
    "url": "https://creativecommons.org/licenses/by/4.0/",
}
CACHE_DIR = Path("data/cache/wordnet/en")
WN_DATA_DIR = os.environ.get(
    "WN_DATA_DIR", str(Path.home() / ".formal-ai-wndata")
)
WORD = re.compile(r"^[a-z][a-z-]*[a-z]$|^[a-z]$")
# Reuse the already-built example binary when present (≈100× faster than a fresh
# `cargo run` per lemma); fall back to `cargo run` otherwise.
FAST_BIN = Path("target/debug/examples/wikidata_json_to_lino")


def load_wordnet():
    import wn

    wn.config.data_directory = WN_DATA_DIR
    if not any(lex.id == "oewn" for lex in wn.lexicons()):
        print(f"downloading {LEXICON} into {WN_DATA_DIR} (one-time) …")
        wn.download(LEXICON)
    return wn


def candidate_lemmas() -> list[str]:
    out = subprocess.check_output(
        [sys.executable, "scripts/audit-total-closure.py", "--candidates"]
    ).decode()
    return [line.strip() for line in out.splitlines() if line.strip()]


def build_record(wn, lemma: str) -> dict | None:
    """Project a lemma's OEWN synsets into a compact, lossless cache record."""
    synsets = wn.synsets(lemma, lexicon=LEXICON)
    if not synsets:
        return None
    senses = []
    for synset in synsets:
        words = [w for w in synset.lemmas() if w.lower() != lemma.lower()]
        record = {
            "id": synset.id,
            "partOfSpeech": synset.pos,
            "definition": synset.definition() or "",
        }
        examples = synset.examples()
        if examples:
            record["examples"] = examples
        if words:
            record["synonyms"] = words
        senses.append(record)
    return {
        "lemma": lemma,
        "language": "en",
        "source": LEXICON,
        "license": LICENSE,
        "senses": senses,
    }


def write_entry(lemma: str, record: dict) -> None:
    CACHE_DIR.mkdir(parents=True, exist_ok=True)
    json_path = CACHE_DIR / f"{lemma}.json"
    lino_path = CACHE_DIR / f"{lemma}.lino"
    json_path.write_text(
        json.dumps(record, indent=2, ensure_ascii=False) + "\n", encoding="utf-8"
    )
    if FAST_BIN.is_file():
        cmd = [str(FAST_BIN), "entry", str(json_path), str(lino_path)]
    else:
        cmd = [
            "cargo", "run", "--quiet", "--example", "wikidata_json_to_lino",
            "entry", str(json_path), str(lino_path),
        ]
    result = subprocess.run(cmd, capture_output=True, text=True)
    if result.returncode != 0:
        json_path.unlink(missing_ok=True)
        raise SystemExit(f"json_to_lino failed for {lemma}: {result.stderr}")


def main(argv: list[str]) -> int:
    explicit = [a for a in argv[1:] if not a.startswith("--")]
    lemmas = explicit or [w for w in candidate_lemmas() if WORD.match(w)]
    print(f"grounding {len(lemmas)} lemma(s) against {LEXICON}")
    wn = load_wordnet()
    added, skipped, missing = 0, 0, []
    for lemma in lemmas:
        if (CACHE_DIR / f"{lemma}.json").is_file() and (CACHE_DIR / f"{lemma}.lino").is_file():
            skipped += 1
            continue
        record = build_record(wn, lemma)
        if record is None:
            missing.append(lemma)
            continue
        write_entry(lemma, record)
        added += 1
        if added % 50 == 0:
            print(f"  … {added} grounded")
    cached = len(list(CACHE_DIR.glob("*.lino")))
    print(
        f"done: +{added} new, {skipped} already cached, "
        f"{len(missing)} not in WordNet; {cached} total entries"
    )
    if missing:
        print("  not in WordNet (need Wiktionary/define-as-meaning): "
              + ", ".join(missing))
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv))
