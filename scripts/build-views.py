#!/usr/bin/env python3
"""Build the multi-source ``data/view/`` merge layer with deterministic M-ids (issue #398, PR #399).

PR #399 review (comment 4668929105) requires a working multi-source ``view``:

  * ``data/view/`` exists with merged entities,
  * ``M-…`` ids generated **deterministically** (same inputs → identical id),
  * a **merge** that carries per-field provenance and respects a merge threshold
    (senses that are the same merge and keep both sources; senses that differ
    stay separate), and
  * CI that fails immediately if any of it is missing or not working.

This script is that builder, preserved in the repo as a re-runnable migration.
For every English lemma cached by *any* lexical source — Open English WordNet
(``data/cache/wordnet/en``) and/or Wiktionary (``data/cache/wiktionary/en``) — it
emits one merged view entity ``data/view/en/<lemma>.lino``:

  view
    id M-<12 hex>            # = "M-" + sha1("view:en:<lemma>")[:12], deterministic
    lemma <lemma>
    language en
    sources (wordnet wiktionary)
    sense
      part_of_speech noun
      gloss "…"
      provenance (wordnet wiktionary)   # every sense names ≥1 source
    …

Merge rule: two senses from different sources merge into one view sense when they
share a normalised part of speech **and** their gloss content-word Jaccard
overlap is ≥ ``MERGE_THRESHOLD``; the merged sense lists both sources as
provenance. Otherwise they stay separate, each with its single source. No view
field is ever emitted without a source.

Usage::

    python3 scripts/build-views.py            # (re)build data/view/en/*.lino
    python3 scripts/build-views.py --check     # verify committed views + merge logic (CI)
    python3 scripts/build-views.py --selftest  # merge-threshold unit checks only

``--check`` rebuilds every entity in memory, confirms the committed files match
byte-for-byte (no drift), reconfirms id determinism, and runs the merge
self-tests; it exits non-zero on any failure and is what the Rust CI gate shells
out to, so the merge logic lives in exactly one place.
"""
from __future__ import annotations

import glob
import hashlib
import json
import os
import re
import sys
from pathlib import Path

WORDNET_DIR = Path("data/cache/wordnet/en")
WIKTIONARY_DIR = Path("data/cache/wiktionary/en")
VIEW_DIR = Path("data/view/en")
MERGE_THRESHOLD = 0.5

POS = {
    "n": "noun", "v": "verb", "a": "adjective", "s": "adjective", "r": "adverb",
    "noun": "noun", "verb": "verb", "adjective": "adjective", "adverb": "adverb",
    "pronoun": "pronoun", "preposition": "preposition", "conjunction": "conjunction",
    "interjection": "interjection", "numeral": "numeral", "determiner": "determiner",
    "article": "article", "exclamation": "interjection",
}
STOP = {
    "a", "an", "the", "of", "to", "in", "on", "or", "and", "for", "with", "as",
    "by", "that", "which", "is", "are", "be", "some", "any", "it", "its", "this",
    "such", "from", "at", "into", "one", "used", "use",
}
WORD = re.compile(r"[a-z]+")


def view_id(lemma: str, language: str = "en") -> str:
    digest = hashlib.sha1(f"view:{language}:{lemma}".encode("utf-8")).hexdigest()
    return "M-" + digest[:12]


def _norm_pos(raw: str) -> str:
    return POS.get((raw or "").lower(), (raw or "unknown").lower())


def _content_words(gloss: str) -> set[str]:
    return {w for w in WORD.findall(gloss.lower()) if w not in STOP}


def _gloss_clean(gloss: str) -> str:
    return " ".join(gloss.replace('"', "'").split()).strip()


def _lino_escape(value: str) -> str:
    """Escape a value for the backslash dialect ``seed::parser`` reads (issue #715).

    Only the backslash needs escaping here, and that is a property of the caller
    rather than of the notation: ``_gloss_clean`` has already collapsed every run
    of whitespace, so no tab, newline or carriage return can reach the slot, and
    it has already replaced every double quote (lossily — see below).

    An unescaped backslash is not safe to emit. ``unescape_value`` decodes ``\\n``,
    ``\\t`` and ``\\r``, so a gloss carrying LaTeX would read back wrong: ``\\rightarrow``
    in ``data/view/en/graph.lino`` decodes to a carriage return followed by
    ``ightarrow``. Doubling it here means the decoder's ``\\\\`` arm returns the one
    backslash the gloss actually holds.

    This is deliberately not a general Links Notation encoder, and it must not
    grow into one. The notation picks a delimiter the value does not contain and
    doubles it only as a fallback, and reproducing that choice correctly in a
    second language is what ``_gloss_clean``'s quote replacement is already
    getting wrong: it rewrites Wiktionary's own quotation marks into apostrophes
    because it cannot escape them, so ``"Hello!" or an equivalent greeting.``
    is stored as ``'Hello!' or an equivalent greeting.``. Fixing that means
    writing these values through the codec in ``src/links_format.rs`` rather than
    adding a tenth hand-rolled escaper; it is tracked separately.
    """
    return value.replace("\\", "\\\\")


def wordnet_senses(lemma: str) -> list[dict]:
    path = WORDNET_DIR / f"{lemma}.json"
    if not path.is_file():
        return []
    record = json.loads(path.read_text(encoding="utf-8"))
    out = []
    for sense in record.get("senses", []):
        gloss = _gloss_clean(sense.get("definition", ""))
        if not gloss:
            continue
        out.append({
            "pos": _norm_pos(sense.get("partOfSpeech", "")),
            "gloss": gloss,
            "sources": ["wordnet"],
        })
    return out


def wiktionary_senses(lemma: str) -> list[dict]:
    path = WIKTIONARY_DIR / f"{lemma}.json"
    if not path.is_file():
        return []
    entries = json.loads(path.read_text(encoding="utf-8"))
    out = []
    for entry in entries:
        for meaning in entry.get("meanings", []):
            pos = _norm_pos(meaning.get("partOfSpeech", ""))
            for definition in meaning.get("definitions", []):
                gloss = _gloss_clean(definition.get("definition", ""))
                if not gloss:
                    continue
                out.append({"pos": pos, "gloss": gloss, "sources": ["wiktionary"]})
    return out


def _similar(a: dict, b: dict) -> bool:
    if a["pos"] != b["pos"]:
        return False
    wa, wb = _content_words(a["gloss"]), _content_words(b["gloss"])
    if not wa or not wb:
        return False
    overlap = len(wa & wb) / len(wa | wb)
    return overlap >= MERGE_THRESHOLD


def merge_senses(primary: list[dict], secondary: list[dict]) -> list[dict]:
    """Merge ``secondary`` senses into ``primary`` by the threshold rule.

    A secondary sense that is similar to an existing sense joins it (its source
    is added to that sense's provenance); otherwise it is appended as its own
    sense. Order is deterministic: primary senses first in input order, then any
    non-merged secondary senses in input order.
    """
    merged = [dict(s, sources=list(s["sources"])) for s in primary]
    for cand in secondary:
        for existing in merged:
            if _similar(existing, cand):
                for src in cand["sources"]:
                    if src not in existing["sources"]:
                        existing["sources"].append(src)
                break
        else:
            merged.append(dict(cand, sources=list(cand["sources"])))
    return merged


def build_entity(lemma: str) -> tuple[str, str]:
    wn = wordnet_senses(lemma)
    wk = wiktionary_senses(lemma)
    senses = merge_senses(wn, wk)
    sources: list[str] = []
    for sense in senses:
        for src in sense["sources"]:
            if src not in sources:
                sources.append(src)
    sources.sort()
    ident = view_id(lemma)
    lines = [
        "view",
        f"  id {ident}",
        f"  lemma {lemma}",
        "  language en",
        f"  sources ({' '.join(sources)})",
    ]
    for sense in senses:
        prov = " ".join(sorted(sense["sources"]))
        lines.append("  sense")
        lines.append(f"    part_of_speech {sense['pos']}")
        lines.append(f'    gloss "{_lino_escape(sense["gloss"])}"')
        lines.append(f"    provenance ({prov})")
    return ident, "\n".join(lines) + "\n"


def all_lemmas() -> list[str]:
    lemmas = {os.path.basename(p)[:-5] for p in glob.glob(str(WORDNET_DIR / "*.json"))}
    lemmas |= {os.path.basename(p)[:-5] for p in glob.glob(str(WIKTIONARY_DIR / "*.json"))}
    lemmas.discard("reference")
    return sorted(lemmas)


def build() -> int:
    VIEW_DIR.mkdir(parents=True, exist_ok=True)
    for old in glob.glob(str(VIEW_DIR / "*.lino")):
        os.remove(old)
    lemmas = all_lemmas()
    multi = 0
    for lemma in lemmas:
        _, text = build_entity(lemma)
        (VIEW_DIR / f"{lemma}.lino").write_text(text, encoding="utf-8")
        if text.count("wordnet") and text.count("wiktionary"):
            multi += 1
    print(f"built {len(lemmas)} view entities ({multi} merged across both sources)")
    return 0


def selftest() -> int:
    """Merge-threshold correctness: same-meaning pairs merge, different ones do not."""
    failures = []
    same_a = {"pos": "noun", "gloss": "a precise rule for solving a problem", "sources": ["wordnet"]}
    same_b = {"pos": "noun", "gloss": "a precise rule for solving a problem quickly", "sources": ["wiktionary"]}
    merged = merge_senses([same_a], [same_b])
    if len(merged) != 1 or sorted(merged[0]["sources"]) != ["wiktionary", "wordnet"]:
        failures.append(f"similar same-pos senses should merge with both sources, got {merged}")

    diff_pos = {"pos": "verb", "gloss": "a precise rule for solving a problem", "sources": ["wiktionary"]}
    merged = merge_senses([same_a], [diff_pos])
    if len(merged) != 2:
        failures.append(f"different-pos senses must stay separate, got {merged}")

    diff_gloss = {"pos": "noun", "gloss": "a young sheep raised for wool", "sources": ["wiktionary"]}
    merged = merge_senses([same_a], [diff_gloss])
    if len(merged) != 2:
        failures.append(f"unrelated glosses must stay separate, got {merged}")

    if view_id("algorithm") != view_id("algorithm"):
        failures.append("id is not deterministic")
    if not re.fullmatch(r"M-[0-9a-f]{12}", view_id("algorithm")):
        failures.append("id does not match M-<hex> shape")

    for sense in merge_senses([same_a], [same_b, diff_gloss]):
        if not sense["sources"]:
            failures.append("a merged sense has empty provenance")

    for msg in failures:
        print(f"SELFTEST FAIL: {msg}")
    print("selftest: " + ("ok" if not failures else f"{len(failures)} failure(s)"))
    return 1 if failures else 0


def check() -> int:
    problems = []
    if selftest() != 0:
        problems.append("merge self-tests failed")
    committed = sorted(glob.glob(str(VIEW_DIR / "*.lino")))
    if not committed:
        problems.append("no committed view entities under data/view/en/")
    expected = {f"{lemma}.lino" for lemma in all_lemmas()}
    found = {os.path.basename(p) for p in committed}
    if expected != found:
        missing = sorted(expected - found)[:5]
        extra = sorted(found - expected)[:5]
        problems.append(f"view set drift: missing {missing} extra {extra}")
    for path in committed:
        lemma = os.path.basename(path)[:-5]
        _, want = build_entity(lemma)
        have = Path(path).read_text(encoding="utf-8")
        if have != want:
            problems.append(f"{path} is stale; rerun scripts/build-views.py")
        if f"id {view_id(lemma)}" not in have:
            problems.append(f"{path} has a non-deterministic id")
        if "provenance ()" in have or "sources ()" in have:
            problems.append(f"{path} has a field with no source")
    for msg in problems:
        print(f"CHECK FAIL: {msg}")
    print(f"check: {len(committed)} entities, " + ("ok" if not problems else f"{len(problems)} problem(s)"))
    return 1 if problems else 0


def main(argv: list[str]) -> int:
    if "--selftest" in argv:
        return selftest()
    if "--check" in argv:
        return check()
    return build()


if __name__ == "__main__":
    raise SystemExit(main(sys.argv[1:]))
