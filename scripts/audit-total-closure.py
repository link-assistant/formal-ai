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
            j = i + 1
            while j < len(stripped):
                if stripped[j] == "\\":
                    j += 2
                elif stripped[j] == c:
                    break
                else:
                    j += 1
            if j >= len(stripped):
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


def facet_label_projection(source: str) -> dict[str, str]:
    """Project exact source-owned facet identifiers without losing their spelling."""
    matches = re.findall(r"const FACET_KINDS: &\[&str\] = &\[(.*?)\];", source, re.S)
    if len(matches) != 1:
        raise RuntimeError("facet source declaration is missing or ambiguous")
    body = re.sub(r"//[^\n]*", "", matches[0])
    names = re.findall(r'"([a-z][a-z0-9]*(?:[-_][a-z0-9]+)*)"', body)
    remainder = re.sub(r'"[a-z][a-z0-9]*(?:[-_][a-z0-9]+)*"', "", body)
    if not names or re.sub(r"[\s,]", "", remainder):
        raise RuntimeError("unrecognized facet source initializer")
    projected: dict[str, str] = {}
    for name in names:
        if "-" in name and "_" in name:
            raise RuntimeError("mixed facet identifier spelling")
        label = name.replace("_", "-")
        if label in projected:
            raise RuntimeError("duplicate or colliding facet projection")
        projected[label] = name
    for label, name in projected.items():
        if name.replace("_", "-") != label or projected[label] != name:
            raise RuntimeError("facet projection failed exact source roundtrip")
    return projected


def scoped_shared_references(root: str) -> tuple[set[str], set[tuple[str, int]], dict[tuple[str, int], str]]:
    """Resolve only validated meaning-local shared lexical field references.

    Identity keys remain qualified by source, root and owner. They never enter
    the global meaning/role/cache namespace, and field values remain audited.
    """
    schema_path = os.path.join(root, CLOSURE_SCHEMA)
    with open(schema_path, encoding="utf-8") as handle:
        schema_lines = handle.read().splitlines()
    directive = "  scoped-shared-lexeme-schema data/meta/shared-lexeme-fields-schema.lino"
    if directive not in schema_lines:
        return set(), set(), {}
    technical_schema = os.path.join(root, "data/meta/shared-lexeme-fields-schema.lino")
    with open(technical_schema, encoding="utf-8") as handle:
        contract = handle.read().splitlines()
    required = {
        "shared-lexeme-fields-schema",
        "  declaration shared-lexeme-fields",
        "  reference use-lexeme-fields",
        "  field part-of-speech",
        "  field grammatical-number",
    }
    if not required.issubset(contract):
        raise RuntimeError("unknown shared lexical schema")
    labels = [line[8:] for line in contract if line.startswith("  field ")]
    import hashlib
    facet_rows = [line[15:].split(" ", 1) for line in contract if line.startswith("  facet-source ")]
    facet_path = "rust/src/seed/meanings/parse.rs"
    if len(facet_rows) != 1 or len(facet_rows[0]) != 2 or facet_rows[0][0] != facet_path:
        raise RuntimeError("facet source authority missing")
    with open(os.path.join(root, facet_path), "rb") as handle:
        facet_bytes = handle.read()
    if hashlib.sha256(facet_bytes).hexdigest() != facet_rows[0][1]:
        raise RuntimeError("facet source authority drift")
    field_names = facet_label_projection(facet_bytes.decode("utf-8"))
    if labels != ["part-of-speech", "grammatical-number"] or any(label not in field_names for label in labels):
        raise RuntimeError("shared lexical schema drift")
    fields = [field_names[label] for label in labels]
    import hashlib
    source_rows = [line[17:].split(" ", 1) for line in contract if line.startswith("  runtime-source ")]
    expected_sources = {"js/seed_loader.js", "rust/src/seed/parser.rs", "ts/seed_loader.ts"}
    if {row[0] for row in source_rows} != expected_sources or len(source_rows) != 3:
        raise RuntimeError("shared lexical runtime source authority missing")
    for source, digest in source_rows:
        with open(os.path.join(root, source), "rb") as handle:
            if hashlib.sha256(handle.read()).hexdigest() != digest:
                raise RuntimeError("shared lexical runtime source drift")
    bound: set[str] = set()
    identities: set[tuple[str, int]] = set()
    references: dict[tuple[str, int], str] = {}
    reserved = {"meanings", "lexeme", "surface", "shared-lexeme-fields", "use-lexeme-fields", *fields}
    for path in sorted(glob.glob(os.path.join(root, "data/seed/*.lino"))):
        with open(path, encoding="utf-8") as handle:
            lines = handle.read().splitlines()
        document = {"name": "", "raw": "", "index": -1, "children": []}
        stack = [(-1, document)]
        for index, raw in enumerate(lines):
            stripped = raw.strip()
            if not stripped or stripped.startswith("#"):
                continue
            tokens = _line_tokens(stripped)
            if not tokens:
                continue
            indent = _indent(raw)
            while len(stack) > 1 and stack[-1][0] >= indent:
                stack.pop()
            node = {"name": tokens[0], "raw": stripped, "index": index, "children": []}
            stack[-1][1]["children"].append(node)
            stack.append((indent, node))
        def marker(node: dict) -> bool:
            return node["name"] in {"shared-lexeme-fields", "use-lexeme-fields"}
        def contains(node: dict) -> bool:
            return marker(node) or any(contains(child) for child in node["children"])
        def nodes(node: dict):
            yield node
            for child in node["children"]:
                yield from nodes(child)
        document_valid = not any(re.match(r"^[ \t]*\t", line) for line in lines if line.strip() and not line.lstrip().startswith("#"))
        all_markers = [node for node in nodes(document) if marker(node)]
        for node in all_markers:
            if node["name"] == "use-lexeme-fields":
                references[(path, node["index"])] = "scoped-invalid:" + path + ":" + str(node["index"])
        for container in document["children"]:
            if container["name"] != "meanings":
                continue
            for owner in container["children"]:
                declarations = [node for node in owner["children"] if node["name"] == "shared-lexeme-fields"]
                if not declarations:
                    continue
                prefix = "scoped:" + path + ":" + str(container["index"]) + ":" + str(owner["index"]) + ":"
                valid = document_valid and len(declarations) == 1
                valid = valid and owner["name"] not in reserved
                owner_identity = owner["raw"][8:].strip().strip("\"'") if owner["name"] == "meaning" else owner["name"]
                valid = valid and bool(SLUG.fullmatch(owner_identity))
                declaration = declarations[0]
                match = re.fullmatch(r"shared-lexeme-fields +([a-z][a-z0-9_-]*)(?: +#.*)?", declaration["raw"])
                identity = match[1] if match else ""
                valid = valid and bool(identity)
                declaration_fields = declaration["children"]
                names = [node["name"] for node in declaration_fields]
                valid = valid and bool(names) and len(names) == len(set(names))
                for field in declaration_fields:
                    valid = valid and field["name"] in fields and not field["children"]
                    valid = valid and bool(re.fullmatch(r"[a-z][a-z0-9_-]* +[a-z][a-z0-9_-]*(?: +#.*)?", field["raw"]))
                lexemes = [node for node in owner["children"] if node["name"] == "lexeme"]
                languages = [node["raw"][7:].strip().strip("\"'") for node in lexemes]
                valid = valid and len(lexemes) >= 2 and len(languages) == len(set(languages))
                valid = valid and all(SLUG.fullmatch(language) for language in languages)
                used = []
                for node in owner["children"]:
                    if node is not declaration and node["name"] != "lexeme" and contains(node):
                        valid = False
                for lexeme in lexemes:
                    valid = valid and bool(lexeme["children"])
                    for surface in lexeme["children"]:
                        valid = valid and surface["name"] == "surface"
                        refs = [node for node in surface["children"] if node["name"] == "use-lexeme-fields"]
                        valid = valid and len(refs) == 1
                        for ref in refs:
                            ref_match = re.fullmatch(r"use-lexeme-fields +([a-z][a-z0-9_-]*)(?: +#.*)?", ref["raw"])
                            value = ref_match[1] if ref_match else "invalid"
                            key = prefix + value
                            references[(path, ref["index"])] = key
                            used.append(key)
                            valid = valid and value == identity and not ref["children"]
                        for field in surface["children"]:
                            if field not in refs and (contains(field) or field["name"] in names):
                                valid = False
                recognized = {declaration["index"], *[ref["index"] for lexeme in lexemes for surface in lexeme["children"] for ref in surface["children"] if ref["name"] == "use-lexeme-fields"]}
                valid = valid and all(node["index"] in recognized for node in nodes(owner) if marker(node))
                if valid:
                    identities.add((path, declaration["index"]))
                    bound.update(used)
    return bound, identities, references


class Resolver:
    """Resolution oracle for a single repository root."""

    def __init__(self, root: str, *, include_generated: bool = True) -> None:
        self.root = root
        self.scoped_bindings, _, _ = scoped_shared_references(root)
        self.defined = defined_meaning_slugs(root, include_generated=include_generated)
        self.roles = declared_roles(root)
        self.wiktionary = cached_lemmas(root, "wiktionary")
        self.wordnet = cached_lemmas(root, "wordnet")
        self.ids = grounded_ids(root)

    def resolves(self, token: str) -> bool:
        if token.startswith("scoped:") or token.startswith("scoped-invalid:"):
            return token in self.scoped_bindings
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
    scoped_bindings, scoped_identities, scoped_references = scoped_shared_references(root)
    for path in sorted(glob.glob(os.path.join(root, "data/seed/*.lino"))):
        if not include_generated and os.path.basename(path).startswith(GENERATED_PREFIX):
            continue
        with open(path, encoding="utf-8") as handle:
            lines = handle.read().split("\n")
        significant: list[tuple[int, list[str]]] = []
        for source_index, raw in enumerate(lines):
            stripped = raw.strip()
            if not stripped:
                continue
            if stripped.startswith("#"):
                ignored_full_line_comments += 1
                continue
            toks = _line_tokens(stripped)
            if toks:
                significant.append((source_index, _indent(raw), toks))

        lex_lang: str | None = None
        for index, (source_index, indent, toks) in enumerate(significant):
            head, values = toks[0], toks[1:]
            scoped_key = scoped_references.get((path, source_index))
            if scoped_key is not None:
                counts[scoped_key] += 1
                heads.setdefault(scoped_key, Counter())[head] += 1
                continue
            if (path, source_index) in scoped_identities:
                values = values[1:]
            # The concise lexeme form (R1188-U7, docs/links-notation-style.md):
            # the words after the language, and the words of a `words` line,
            # are the lexeme's surfaces, read as the `text` values they expand to.
            surfaces: list[str] = []
            if head == "lexeme" and values:
                lex_lang = values[0]
                values, surfaces = values[:1], values[1:]
            elif head == "words":
                values, surfaces = [], values
            has_children = (
                index + 1 < len(significant) and significant[index + 1][1] > indent
            )
            if has_children and head in declaration_heads and values:
                declarations.update(_expand_slug_values(values[:1]))
                values = values[1:]
            if head in literal_heads:
                literals.update(_expand_slug_values(values))
                continue
            for value_head, value in [(head, value) for value in _expand_slug_values(values)] + [
                ("text", value) for value in _expand_slug_values(surfaces)
            ]:
                # Non-English surface forms are attested by their grounded
                # parent meaning, not by an English lexical record.
                if value_head in {"text", "phrase"} and lex_lang not in (None, "en"):
                    continue
                counts[value] += 1
                heads.setdefault(value, Counter())[value_head] += 1
    diagnostics: dict[str, object] = {
        "ignored_full_line_comments": ignored_full_line_comments,
        "resolved_scoped_reference_bindings": len(scoped_bindings),
        "source_bound_scoped_identity_declarations": len(scoped_identities),
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
