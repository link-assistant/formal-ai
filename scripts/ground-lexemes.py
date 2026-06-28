#!/usr/bin/env python3
"""Lexeme grounding pipeline for the canonical seed (issue #398, defect #6).

PR #399 review (deep review of f1f78dc, defect 6 / CI check 6) requires that
every grounded word expose its **parts of speech and all its forms sourced from
Wikidata**, not hand-authored. This migration enriches a curated, *verified*
batch of grounded English nouns with the rich lexeme notation already used by
`data/seed/meanings-links-root.lino`:

    source-lexeme <L-id>
      language <Q-id>
      lexical-category <Q-id>          # part of speech, from the source
      form <L-id-F#>                   # every form the lexeme records
        feature <Q-id>                 # grammatical feature (singular/plural/…)
      sense <L-id-S#>
    surface <L-id-F#>                  # canonical surface references a real form
      text <representation>            # text comes from the lexeme, not hand-typed
      language <lang>
      sense <L-id-S#>

For each `(slug, lexeme-id, expected-lemma, sense-id)` entry it:

  1. Fetches the FULL `Special:EntityData/<L-id>.json` (lexemes are cached
     untrimmed — claims, forms and senses are the whole point here), wraps it in
     the `{entities:{…}, success:1}` cache convention, and writes pretty
     multi-line JSON to `data/cache/wikidata/lexeme/<L-id>.json` when missing.
  2. Generates the lossless `.lino` snapshot via the `wikidata_json_to_lino`
     cargo example (the same codec the cache is built with).
  3. **Verifies** the lexeme actually is the expected English lemma, a noun
     (`lexicalCategory` Q1084) in English (`language` Q1860), and that the named
     sense exists — refusing to ground on any mismatch (the wrong-id guard).
  4. Reads the lexeme's real forms/features and rewrites the meaning's English
     `lexeme en / surface / text` block into the rich `source-lexeme` + `surface`
     notation, sourcing the part of speech and every form from the cache.
     Idempotent: a meaning already carrying `source-lexeme <L-id>` is untouched.

Run with `python3 scripts/ground-lexemes.py` (requires `curl`, `python3`, and a
built `cargo`). Network is only needed the first time a lexeme is fetched.
"""

import json
import subprocess
import sys
from collections import OrderedDict
from pathlib import Path

USER_AGENT = "formal-ai-grounding/1.0 (https://github.com/link-assistant/formal-ai)"
LANG_ENGLISH = "Q1860"
CATEGORY_NOUN = "Q1084"
CACHE_DIR = Path("data/cache/wikidata/lexeme")
SEED_DIR = Path("data/seed")

# `(meaning slug, lexeme id, expected English lemma, sense id)`. Every lemma,
# part of speech and sense was confirmed against the live lexeme before being
# added; the verification step re-checks them on every run.
LEXEMES = [
    ("apple", "L3257", "apple", "L3257-S1"),
    ("water", "L3302", "water", "L3302-S1"),
    ("bread", "L3865", "bread", "L3865-S1"),
    ("potato", "L3784", "potato", "L3784-S1"),
    ("tomato", "L7993", "tomato", "L7993-S1"),
]


def fetch_lexeme(lid):
    """Fetch the full lexeme and cache it under the `{entities,success}` wrapper."""
    json_path = CACHE_DIR / f"{lid}.json"
    if not json_path.exists():
        url = f"https://www.wikidata.org/wiki/Special:EntityData/{lid}.json"
        raw = subprocess.run(
            ["curl", "-sfL", "-A", USER_AGENT, url],
            capture_output=True, check=True,
        ).stdout
        doc = json.loads(raw)
        # Special:EntityData omits the `success` flag the cache convention keeps.
        doc.setdefault("success", 1)
        CACHE_DIR.mkdir(parents=True, exist_ok=True)
        with json_path.open("w", encoding="utf-8") as handle:
            json.dump(doc, handle, ensure_ascii=False, indent=2)
            handle.write("\n")
    lino_path = CACHE_DIR / f"{lid}.lino"
    if not lino_path.exists():
        subprocess.run(
            ["cargo", "run", "--quiet", "--example", "wikidata_json_to_lino",
             "--", lid, str(json_path), str(lino_path)],
            check=True,
        )
    return json.loads(json_path.read_text(encoding="utf-8"))["entities"][lid]


def verify(entity, lemma, sense_id):
    """The wrong-id guard: refuse anything that is not the expected English noun."""
    got = entity.get("lemmas", {}).get("en", {}).get("value")
    if got != lemma:
        raise ValueError(f"lemma mismatch: expected {lemma!r}, got {got!r}")
    if entity.get("language") != LANG_ENGLISH:
        raise ValueError(f"not English: language {entity.get('language')}")
    if entity.get("lexicalCategory") != CATEGORY_NOUN:
        raise ValueError(f"not a noun: category {entity.get('lexicalCategory')}")
    if not any(s["id"] == sense_id for s in entity.get("senses", [])):
        raise ValueError(f"sense {sense_id} not present")


def build_block(entity, lid, sense_id):
    """Render the rich `source-lexeme` + `surface` lines from the real lexeme."""
    forms = entity["forms"]
    lines = [
        f"    source-lexeme {lid} # wikidata english source lexeme",
        f"      language {LANG_ENGLISH} # wikidata language english",
        f"      lexical-category {CATEGORY_NOUN} # wikidata category noun",
    ]
    for form in forms:
        rep = form["representations"].get("en", {}).get("value", "")
        lines.append(f"      form {form['id']} # wikidata form {rep}")
        for feature in form.get("grammaticalFeatures", []):
            lines.append(f"        feature {feature} # wikidata grammatical feature")
    lines.append(f"      sense {sense_id} # wikidata grounded sense")
    # The canonical surface references the singular form (Q110786) when present.
    canonical = next(
        (f for f in forms if "Q110786" in f.get("grammaticalFeatures", [])),
        forms[0],
    )
    text = canonical["representations"].get("en", {}).get("value", "")
    lines += [
        f"    surface {canonical['id']} # wikidata english surface",
        f"      text {text}",
        "      language en",
        f"      sense {sense_id} # wikidata grounded sense",
    ]
    return lines


def rewrite_seed(slug, lid, lemma, block_lines):
    """Replace the meaning's plain `lexeme en / surface / text` with the block."""
    for path in sorted(SEED_DIR.glob("meanings*.lino")):
        lines = path.read_text(encoding="utf-8").split("\n")
        try:
            header = lines.index(f"  {slug}")
        except ValueError:
            continue
        end = header + 1
        while end < len(lines):
            stripped = lines[end].lstrip(" ")
            indent = len(lines[end]) - len(stripped)
            if stripped and indent <= 2:
                break
            end += 1
        body = lines[header:end]
        if any(line.strip() == f"source-lexeme {lid}" or
               line.strip().startswith(f"source-lexeme {lid} ") for line in body):
            return "already"
        target = [
            "    lexeme en",
            "      surface",
            f"        text {lemma}",
        ]
        for i in range(header, end - len(target) + 1):
            if lines[i:i + len(target)] == target:
                lines[i:i + len(target)] = block_lines
                path.write_text("\n".join(lines), encoding="utf-8")
                return "inserted"
        return "no-plain-en-block"
    return "slug-missing"


def main():
    grounded, skipped = 0, []
    for slug, lid, lemma, sense_id in LEXEMES:
        entity = fetch_lexeme(lid)
        try:
            verify(entity, lemma, sense_id)
        except ValueError as error:
            skipped.append(f"{slug} ({lid}): {error}")
            continue
        block = build_block(entity, lid, sense_id)
        outcome = rewrite_seed(slug, lid, lemma, block)
        if outcome == "inserted":
            grounded += 1
        elif outcome != "already":
            skipped.append(f"{slug} ({lid}): {outcome}")
    print(f"grounded {grounded} word(s) with sourced part of speech and forms")
    for entry in skipped:
        print(f"  - skipped {entry}")
    return 1 if any("token" in s or "mismatch" in s for s in skipped) else 0


if __name__ == "__main__":
    sys.exit(main())
