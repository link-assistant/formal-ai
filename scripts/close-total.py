#!/usr/bin/env python3
"""Drive total reference-closure to zero by defining every dangling token (issue #398, PR #399).

PR #399 review (comment 4668929105) requires *total* closure: every non-keyword,
non-quoted value token anywhere in ``data/seed/**.lino`` must resolve to a defined
meaning, a grounded source id with a cache record, or an override — and the build
fails the instant one does not. WordNet/Wiktionary grounding (``ground-wordnet.py``,
``ground-wiktionary.py``) closes the plain-English dictionary words. What remains is
the *internal* vocabulary: intent names, task names, prompt-pattern ids, source
kinds, programming-language tags, and similar snake_case / hyphenated identifiers
that name concepts of this system itself. The reviewer's standard is explicit:
"Every word used in a description/intent/definition is either a defined meaning, a
grounded lexeme/sense, or it must be **made** one." This migration *makes* them.

It is a re-runnable migration, preserved in the repo per the review's "use
automated scripts for mass actions … preserve them" instruction:

  1. Compute the unresolved value tokens over the **base** seed — every
     ``data/seed/*.lino`` file *except* this script's own generated output
     (``closure-generated-*.lino``). Reading the base only makes the migration
     idempotent: the input never changes when the generated files are rewritten,
     so re-running produces byte-identical output.
  2. For each unresolved token, derive a parent meaning from the predicate (head)
     it most often appears under — ``intent`` → the ``intent`` concept, ``task`` →
     ``task``, ``pattern`` → ``pattern``, ``language`` → ``programming_language``,
     and so on — building a real two-level taxonomy rather than a flat dump. Any
     parent concept that does not already resolve is defined here too (rooted at
     ``concept``), so the generated layer is itself closed.
  3. Emit each token as a first-class meaning nested under ``meanings``:
     ``defined-by <parent>`` plus an English ``lexeme`` whose surface is the
     identifier with separators turned to spaces (quoted, so it adds no new
     tokens). Output is sharded into a fixed set of ``closure-generated-NN.lino``
     files, each well under the 1500-line data-file limit.

Run ``python3 scripts/close-total.py`` then ``python3 scripts/audit-total-closure.py``;
the unresolved count must be 0.

Sharding is **content-addressed, not sequential** (PR #965 review). Filling shards
in sorted order up to a line cap makes every shard depend on the *size* of every
block before it: two branches that each define one new token rewrite the tail of
every shard from the insertion point onward, so `data/seed` conflicted in almost
every pull request. Instead each block is placed by a stable digest of its own
slug — ``sha256(slug) % SHARD_COUNT`` — over a shard count that is fixed rather
than derived from the corpus size. A token's shard therefore depends only on the
token, so adding, removing or reparenting one touches exactly one file and leaves
the other ``SHARD_COUNT - 1`` byte-identical. Two branches conflict only when
their new tokens hash to the same shard *and* land adjacent within it.

``SHARD_COUNT`` is deliberately generous: it is the one number whose change
reshuffles everything, so it is raised rarely and never automatically. Bump it
(and re-run) only when ``check-file-size`` starts warning on a shard.
"""
from __future__ import annotations

import glob
import hashlib
import importlib.util
import os
import re
from collections import Counter, defaultdict
from pathlib import Path

SEED_DIR = Path("data/seed")
GENERATED_PREFIX = "closure-generated-"
MAX_LINES = 1400

#: Number of content-addressed shards. Fixed on purpose — see the module docstring.
#: At ~5 lines per meaning this leaves each shard room for ~280 meanings before it
#: approaches ``MAX_LINES``; raising it reshuffles every shard exactly once.
SHARD_COUNT = 16


def shard_for(slug: str) -> int:
    """Return the 1-based shard index that owns ``slug``.

    Keyed on the slug alone so a block's file never depends on what else exists.
    """
    digest = hashlib.sha256(slug.encode("utf-8")).digest()
    return int.from_bytes(digest[:8], "big") % SHARD_COUNT + 1

SLUG = re.compile(r"^[a-z][a-z0-9_-]*$")
QID = re.compile(r"^[QLP][0-9]+$")
LANG_CODES = {"en", "ru", "hi", "zh"}

# Predicate (head) -> parent meaning for the tokens that appear under it. The
# parent names a real category; tokens become children of that category.
PARENT_BY_HEAD = {
    "intent": "intent",
    "expected_intent": "intent",
    "pattern": "prompt_pattern",
    "source_kind": "source_kind",
    "task": "task",
    "response": "response_template",
    "name": "name",
    "display_name": "name",
    "language": "programming_language",
    "code_fence": "programming_language",
    "category": "category",
    "kind": "kind",
    "tool": "tool",
    "operation": "operation",
    "isolation": "isolation_mode",
    "field": "field",
    "flow": "flow",
    "rule": "rule",
    "context": "context",
    "context_links": "context",
    "environment": "environment",
    "value": "value",
    "term": "term",
    "topic": "topic",
    "org": "organization",
    "slug": "concept",
    "token": "token",
    "coverage_group": "coverage_group",
    "text": "word_surface",
    "phrase": "phrase",
    "trace_prefix": "identifier",
    "source": "source",
    "timeline": "release_timeline",
    "release_timeline": "release_timeline",
}
DEFAULT_PARENT = "concept"

# How to root any parent category that does not already resolve in the base seed.
PARENT_DEFINED_BY = {
    "intent": "concept",
    "release_timeline": "concept",
    "prompt_pattern": "concept",
    "source_kind": "source",
    "task": "concept",
    "response_template": "response",
    "programming_language": "language",
    "category": "concept",
    "operation": "action",
    "isolation_mode": "concept",
    "field": "concept",
    "flow": "concept",
    "rule": "concept",
    "context": "concept",
    "environment": "concept",
    "term": "concept",
    "organization": "entity",
    "coverage_group": "concept",
}


def _load_audit():
    spec = importlib.util.spec_from_file_location(
        "audit_total_closure", "scripts/audit-total-closure.py"
    )
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


def base_files() -> list[str]:
    return [
        p
        for p in sorted(glob.glob(str(SEED_DIR / "*.lino")))
        if not os.path.basename(p).startswith(GENERATED_PREFIX)
    ]


def base_defined_slugs(audit, files: list[str]) -> set[str]:
    """Slugs defined (either syntax) in the given base files."""
    defined: set[str] = set()
    for path in files:
        lines = Path(path).read_text(encoding="utf-8").split("\n")
        stack: list[tuple[int, str]] = []
        for raw in lines:
            if not raw.strip():
                continue
            ind = audit._indent(raw)
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


def base_tokens(audit, files: list[str]):
    """Return (counts, dominant_head) for value tokens in the base files."""
    counts: Counter[str] = Counter()
    heads: dict[str, Counter] = defaultdict(Counter)
    for path in files:
        lines = Path(path).read_text(encoding="utf-8").split("\n")
        lex_lang = None
        for raw in lines:
            if not raw.strip():
                continue
            toks = audit._line_tokens(raw.strip())
            if not toks:
                continue
            head, values = toks[0], toks[1:]
            if head == "lexeme" and values:
                lex_lang = values[0]
            expanded: list[str] = []
            for value in values:
                expanded.extend(value.split("+"))
            for value in expanded:
                if not SLUG.match(value):
                    continue
                if head in {"text", "phrase"} and lex_lang not in (None, "en"):
                    continue
                counts[value] += 1
                heads[value][head] += 1
    dominant = {t: c.most_common(1)[0][0] for t, c in heads.items()}
    return counts, dominant


def surface_for(token: str) -> str:
    return token.replace("_", " ").replace("-", " ").strip()


def emit_meaning(token: str, parent: str) -> list[str]:
    lines = [
        f"  {token}",
        f"    defined-by {parent}",
        "    lexeme en",
        "      surface",
        f'        text "{surface_for(token)}"',
    ]
    return lines


def main() -> int:
    audit = _load_audit()
    files = base_files()
    resolver = audit.Resolver(".")  # caches/roles/ids are unaffected by generated files
    defined_base = base_defined_slugs(audit, files)
    counts, dominant = base_tokens(audit, files)

    def resolves_base(token: str) -> bool:
        if token in LANG_CODES:
            return True
        if QID.match(token):
            return token in resolver.ids
        if token in defined_base or token in resolver.roles:
            return True
        if token in resolver.wiktionary or token in resolver.wordnet:
            return True
        return False

    unresolved = sorted(t for t in counts if not resolves_base(t))
    print(f"unresolved tokens to define: {len(unresolved)}")

    # Determine the parent for each token, and which parents need defining.
    token_parent: dict[str, str] = {}
    needed_parents: set[str] = set()
    for token in unresolved:
        parent = PARENT_BY_HEAD.get(dominant.get(token, ""), DEFAULT_PARENT)
        # A token cannot be its own parent; fall back to the default.
        if parent == token:
            parent = DEFAULT_PARENT
        token_parent[token] = parent
        if not resolves_base(parent):
            needed_parents.add(parent)

    # Parents are defined first (and may themselves need a parent definition).
    parent_defs: dict[str, str] = {}
    for parent in sorted(needed_parents):
        parent_defs[parent] = PARENT_DEFINED_BY.get(parent, "concept")

    # Build the full ordered list of (slug, parent) blocks: parents first so a
    # reader meets a category before its members.
    blocks: list[tuple[str, str]] = []
    for parent in sorted(parent_defs):
        blocks.append((parent, parent_defs[parent]))
    for token in unresolved:
        if token in parent_defs:
            continue  # already emitted as a parent
        blocks.append((token, token_parent[token]))

    # Remove any stale generated shards before rewriting.
    for old in glob.glob(str(SEED_DIR / f"{GENERATED_PREFIX}*.lino")):
        os.remove(old)

    # Shard into files by a stable digest of each slug, so a block's file is a
    # function of the block alone. `blocks` is already in a deterministic order
    # (parents first, then members, each sorted), and bucketing preserves it, so
    # a new token is inserted at one place in one shard.
    buckets: dict[int, list[str]] = defaultdict(list)
    for slug, parent in blocks:
        buckets[shard_for(slug)].extend(emit_meaning(slug, parent))

    oversized: list[str] = []
    for shard_idx in range(1, SHARD_COUNT + 1):
        body = buckets.get(shard_idx, [])
        path = SEED_DIR / f"{GENERATED_PREFIX}{shard_idx:02d}.lino"
        # Every shard is written even when empty, so the file set is stable too:
        # a shard emptying out must not delete a file other branches still edit.
        path.write_text("meanings\n" + "\n".join(body) + "\n", encoding="utf-8")
        lines = len(body) + 1
        print(f"  wrote {path} ({lines} lines)")
        if lines > MAX_LINES:
            oversized.append(f"{path.name} ({lines} lines)")

    if oversized:
        print(
            "ERROR: shards exceed the "
            f"{MAX_LINES}-line budget: {', '.join(oversized)}\n"
            "       raise SHARD_COUNT in scripts/close-total.py and re-run."
        )
        return 1

    print(
        f"defined {len(parent_defs)} parent categories + "
        f"{len(blocks) - len(parent_defs)} member meanings"
    )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
