#!/usr/bin/env python3
"""Canonical total-closure audit for the formal-ai seed (issue #398, PR #399).

PR #399 review (comment 4668929105) requires *total* closure, not just the
meaning-graph backbone: **every semantic reference** used as a value anywhere
in ``data/seed/**.lino`` must resolve to one of

  * a defined meaning slug (either the ``slug:`` colon-head form or the
    nested-under-``meanings`` indent form),
  * a declared role in ``data/seed/roles.lino`` (a reserved predicate name),
  * an external grounded id (``Q…``/``L…``/``P…``) with a checked-in Wikidata
    cache record,
  * a lexical cache record (``data/cache/wiktionary/en/<lemma>.json`` or
    ``data/cache/wordnet/en/<lemma>.json``), optionally via an override, or
  * a structural language code (``en``/``ru``/``hi``/``zh``).

Anything else is *undefined* and must be grounded, defined, or overridden — the
build fails the instant a value token resolves to nothing.

This module is the single source of truth shared by

  * the grounding scripts (``--candidates`` emits the work-list), and
  * the Rust CI gate ``tests/unit/total_closure.rs`` (which shells out to
    ``--json`` so the resolver logic lives in exactly one place).

Usage::

    python3 scripts/audit-total-closure.py [ROOT]           # human report
    python3 scripts/audit-total-closure.py --json [ROOT]     # machine report
    python3 scripts/audit-total-closure.py --candidates en   # English lemmas
                                                              # still undefined

A *semantic reference* is a whitespace-separated value token after the first
(head) token of a line, with quoted scalar spans and comments removed and
``+``-joined combos split into parts. The LiNo schema distinguishes references
from two other kinds of bare values: the first value of an identity-bearing
declaration (for example ``response response_id``), and literal operands read
by matchers (for example ``cue earlier``). Those are deliberately not closure
edges. Only ``[a-z][a-z0-9_-]*`` tokens are considered references; numbers, ids
of other shapes, and punctuation are ignored. Surfaces under a non-English
``lexeme`` block are attested by their grounded parent meaning, so they are not
required to carry an English lexical record here (their meaning still must be
grounded by the backbone gate).
"""
from __future__ import annotations

import glob
import json
import os
import re
import sys
from collections import Counter

SLUG = re.compile(r"^[a-z][a-z0-9_-]*$")
QID = re.compile(r"^[QLP][0-9]+$")
WORD = re.compile(r"^[a-z][a-z-]*[a-z]$|^[a-z]$")
LANG_CODES = {"en", "ru", "hi", "zh"}
CLOSURE_SCHEMA = "data/meta/total-closure-schema.lino"


def _indent(line: str) -> int:
    return len(line) - len(line.lstrip(" "))


def _line_tokens(stripped: str) -> list[str]:
    """Tokenize a line, dropping quoted scalar spans and trailing comments."""
    out: list[str] = []
    i = 0
    buf = ""
    while i < len(stripped):
        c = stripped[i]
        if c in "\"'`":
            j = stripped.find(c, i + 1)
            if j == -1:
                break
            i = j + 1
            buf += " "  # keep head/value boundary intact
            continue
        if c == "#" and (i == 0 or stripped[i - 1].isspace()):
            break
        buf += c
        i += 1
    return buf.split()


def closure_schema(root: str) -> tuple[set[str], set[str]]:
    """Load identity-bearing and literal-operand heads from authored data."""
    path = os.path.join(root, CLOSURE_SCHEMA)
    declaration_heads: set[str] = set()
    literal_heads: set[str] = set()
    try:
        handle = open(path, encoding="utf-8")
    except OSError as error:
        raise RuntimeError(f"{CLOSURE_SCHEMA} is required: {error}") from error
    with handle:
        for raw in handle:
            toks = _line_tokens(raw.strip())
            if len(toks) < 2 or not SLUG.match(toks[1]):
                continue
            if toks[0] == "declaration_identity_head":
                declaration_heads.add(toks[1])
            elif toks[0] == "literal_operand_head":
                literal_heads.add(toks[1])
    if not declaration_heads or not literal_heads:
        raise RuntimeError(
            f"{CLOSURE_SCHEMA} must declare both identity and literal heads"
        )
    overlap = declaration_heads & literal_heads
    if overlap:
        raise RuntimeError(
            f"{CLOSURE_SCHEMA} classifies heads both ways: {sorted(overlap)}"
        )
    return declaration_heads, literal_heads


#: Prefix of the files ``scripts/close-total.py`` writes.
#:
#: Issue #1138 B9, plan 09 leaf 6. Counting these as definitions made the metric
#: self-satisfying: the generator wrote a gloss for every unresolved token and
#: the audit then read its own output back as grounding, so the reported gap was
#: zero by construction while thousands of English-only glosses no runtime loads
#: stood in for it. The honest measurement excludes them; the generator may
#: propose work, never satisfy the gate.
GENERATED_PREFIX = "closure-generated-"


def defined_meaning_slugs(root: str, *, include_generated: bool = True) -> set[str]:
    """Slugs defined by either syntax across every ``data/seed/*.lino`` file.

    With ``include_generated=False`` the files ``scripts/close-total.py`` writes
    are skipped, which is the honest definition set (see ``GENERATED_PREFIX``).
    """
    defined: set[str] = set()
    for path in sorted(glob.glob(os.path.join(root, "data/seed/*.lino"))):
        if not include_generated and os.path.basename(path).startswith(GENERATED_PREFIX):
            continue
        with open(path, encoding="utf-8") as handle:
            lines = handle.read().split("\n")
        stack: list[tuple[int, str]] = []
        for raw in lines:
            if not raw.strip():
                continue
            ind = _indent(raw)
            stripped = raw.strip()
            while stack and stack[-1][0] >= ind:
                stack.pop()
            head = stripped.split(" ", 1)[0]
            if head.endswith(":"):
                head = head[:-1]
            if stack and stack[-1][1] == "meanings":
                defined.add(head)
            stack.append((ind, head))
    return defined


def declared_roles(root: str) -> set[str]:
    roles: set[str] = set()
    path = os.path.join(root, "data/seed/roles.lino")
    if not os.path.isfile(path):
        return roles
    for line in open(path, encoding="utf-8"):
        if _indent(line) == 2:
            name = line.strip()
            if name and name != "roles":
                roles.add(name)
    return roles


def cached_lemmas(root: str, source: str) -> set[str]:
    base = os.path.join(root, "data", "cache", source)
    found = {
        os.path.splitext(os.path.basename(path))[0]
        for path in glob.glob(os.path.join(base, "*", "*.json"))
    }
    found.discard("reference")
    return found


def grounded_ids(root: str) -> set[str]:
    return {
        os.path.splitext(os.path.basename(path))[0]
        for path in glob.glob(os.path.join(root, "data/cache/wikidata", "*", "*.json"))
    }


class Resolver:
    """Resolution oracle for a single repository root."""

    def __init__(self, root: str, *, include_generated: bool = True) -> None:
        self.root = root
        self.defined = defined_meaning_slugs(root, include_generated=include_generated)
        self.roles = declared_roles(root)
        self.wiktionary = cached_lemmas(root, "wiktionary")
        self.wordnet = cached_lemmas(root, "wordnet")
        self.ids = grounded_ids(root)

    def resolves(self, token: str) -> bool:
        if token in LANG_CODES:
            return True
        if QID.match(token):
            return token in self.ids
        if token in self.defined or token in self.roles:
            return True
        if token in self.wiktionary or token in self.wordnet:
            return True
        return False


def _expand_slug_values(values: list[str]) -> list[str]:
    """Return the slug-shaped parts of possibly ``+``-joined values."""
    expanded: list[str] = []
    for value in values:
        expanded.extend(part for part in value.split("+") if SLUG.match(part))
    return expanded


def semantic_reference_inventory(
    root: str, *, include_generated: bool = False
) -> tuple[Counter[str], dict[str, str], dict[str, object]]:
    """References, dominant predicates, and schema-exclusion diagnostics.

    Grounding-work-list producers use this function too, so declaration and
    literal classification cannot drift between the reported metric and the
    proposed remediation.
    """
    counts: Counter[str] = Counter()
    heads: dict[str, Counter[str]] = {}
    declarations: Counter[str] = Counter()
    literals: Counter[str] = Counter()
    ignored_full_line_comments = 0
    declaration_heads, literal_heads = closure_schema(root)
    for path in sorted(glob.glob(os.path.join(root, "data/seed/*.lino"))):
        if not include_generated and os.path.basename(path).startswith(GENERATED_PREFIX):
            continue
        with open(path, encoding="utf-8") as handle:
            lines = handle.read().split("\n")
        significant: list[tuple[int, list[str]]] = []
        for raw in lines:
            stripped = raw.strip()
            if not stripped:
                continue
            if stripped.startswith("#"):
                ignored_full_line_comments += 1
                continue
            toks = _line_tokens(stripped)
            if toks:
                significant.append((_indent(raw), toks))

        lex_lang: str | None = None
        for index, (indent, toks) in enumerate(significant):
            head, values = toks[0], toks[1:]
            if head == "lexeme" and values:
                lex_lang = values[0]
            has_children = (
                index + 1 < len(significant) and significant[index + 1][0] > indent
            )
            if has_children and head in declaration_heads and values:
                declarations.update(_expand_slug_values(values[:1]))
                values = values[1:]
            if head in literal_heads:
                literals.update(_expand_slug_values(values))
                continue
            for value in _expand_slug_values(values):
                # Non-English surface forms are attested by their grounded
                # parent meaning, not by an English lexical record.
                if head in {"text", "phrase"} and lex_lang not in (None, "en"):
                    continue
                counts[value] += 1
                heads.setdefault(value, Counter())[head] += 1
    diagnostics: dict[str, object] = {
        "ignored_full_line_comments": ignored_full_line_comments,
        "excluded_declaration_identities": len(declarations),
        "excluded_declaration_identity_occurrences": sum(declarations.values()),
        "excluded_literal_operands": len(literals),
        "excluded_literal_operand_occurrences": sum(literals.values()),
        "schema_declaration_identity_heads": len(declaration_heads),
        "schema_literal_operand_heads": len(literal_heads),
    }
    dominant_heads = {
        token: predicates.most_common(1)[0][0] for token, predicates in heads.items()
    }
    return counts, dominant_heads, diagnostics


def value_tokens(root: str) -> Counter[str]:
    """Every word-like semantic reference and its occurrence count."""
    return semantic_reference_inventory(root)[0]


def audit(root: str) -> dict:
    """Both numbers: the honest gap, and the gap the generator's output hides.

    ``unresolved`` and its two counts are the **honest** measurement — the
    generated shards are not in the definition set — because that is the number
    a grounding work list and the ratchet must read. The
    ``*_with_generated`` keys preserve the historical measurement so the two can
    be compared in one run and the difference is visible rather than asserted.
    """
    honest = Resolver(root, include_generated=False)
    with_generated = Resolver(root)
    counts, _, diagnostics = semantic_reference_inventory(root)
    unresolved = {t: c for t, c in counts.items() if not honest.resolves(t)}
    generated_included = {t: c for t, c in counts.items() if not with_generated.resolves(t)}
    return {
        "defined": len(with_generated.defined),
        "defined_honest": len(honest.defined),
        "roles": len(honest.roles),
        "wiktionary": len(honest.wiktionary),
        "wordnet": len(honest.wordnet),
        "ids": len(honest.ids),
        "distinct_value_tokens": len(counts),
        "unresolved": dict(sorted(unresolved.items(), key=lambda kv: (-kv[1], kv[0]))),
        "unresolved_distinct": len(unresolved),
        "unresolved_distinct_honest": len(unresolved),
        "unresolved_occurrences": sum(unresolved.values()),
        "unresolved_occurrences_honest": sum(unresolved.values()),
        "unresolved_distinct_with_generated": len(generated_included),
        "unresolved_occurrences_with_generated": sum(generated_included.values()),
        **diagnostics,
    }


def english_candidates(root: str) -> list[str]:
    """Undefined tokens that look like plain English lemmas (groundable)."""
    result = audit(root)
    return sorted(t for t in result["unresolved"] if WORD.match(t))


def main(argv: list[str]) -> int:
    args = [a for a in argv[1:] if not a.startswith("--")]
    flags = {a for a in argv[1:] if a.startswith("--")}
    root = args[0] if args and "--candidates" not in flags else (args[1] if len(args) > 1 else ".")
    if "--candidates" in flags:
        for lemma in english_candidates("."):
            print(lemma)
        return 0
    result = audit(root)
    if "--json" in flags:
        print(json.dumps(result, ensure_ascii=False, indent=2))
        return 0
    print(
        f"defined meanings: {result['defined']}  "
        f"(honest, generated shards excluded: {result['defined_honest']})  "
        f"roles: {result['roles']}"
    )
    print(
        f"wiktionary: {result['wiktionary']}  wordnet: {result['wordnet']}  "
        f"wikidata ids: {result['ids']}"
    )
    print(f"distinct value tokens: {result['distinct_value_tokens']}")
    print(
        "schema exclusions: "
        f"{result['excluded_declaration_identities']} declaration identities / "
        f"{result['excluded_literal_operands']} literal operands; "
        f"ignored full-line comments: {result['ignored_full_line_comments']}"
    )
    print(
        f"UNRESOLVED distinct (honest): {result['unresolved_distinct_honest']}  "
        f"occurrences: {result['unresolved_occurrences_honest']}"
    )
    print(
        f"UNRESOLVED distinct (counting the generated shards as definitions): "
        f"{result['unresolved_distinct_with_generated']}  "
        f"occurrences: {result['unresolved_occurrences_with_generated']}"
    )
    print("\n=== unresolved (top 60) ===")
    for token, count in list(result["unresolved"].items())[:60]:
        print(f"{count:4d}  {token}")
    # The exit code keeps the historical meaning -- "does anything resolve to
    # nothing at all, generated glosses included" -- so a step that runs this
    # script as a pass/fail verification does not silently flip red on the day
    # the honest number is first reported. The honest number is ratcheted by
    # `tests/unit/total_closure.rs::seed_closure_gap_only_shrinks`, which is the
    # gate plan 09 leaf 7 gives it.
    return 1 if result["unresolved_distinct_with_generated"] else 0


if __name__ == "__main__":
    raise SystemExit(main(sys.argv))
