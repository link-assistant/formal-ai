#!/usr/bin/env python3
"""Derive `data/meta/seed-registry.lino` from the hand-maintained seed lists.

Issue #991: `src/seed/embedded.rs` (27 manual conflict resolutions) and
`src/web/seed_loader.js` (14) each carried the same seed inventory as a
hand-ordered list. This one-shot extractor reads the three Rust lists and the
browser list and writes the single registry that now generates all of them, so
the migration is a measurement of the old files rather than a retyping of them.

Run once from the repository root:

    python3 experiments/issue_991_build_seed_registry.py
"""

import os
import re

RUST = "src/seed/embedded.rs"
WEB = "src/web/seed_loader.js"
OUT = "data/meta/seed-registry.lino"

HEADER = '''# The one inventory of `data/seed/*.lino`, shared by every production path.
#
# Issue #991: `src/seed/embedded.rs` and `src/web/seed_loader.js` each held this
# inventory as a hand-ordered list, and between them needed 41 manual conflict
# resolutions -- two branches adding a seed file always appended to the same
# lines. Both files are now generated from this one, which is `merge=union` and
# sorted by name: two branches adding a seed file produce two `seed` blocks in
# any order, the union keeps both, and
# `rust-script scripts/generate-seed-registry.rs --write` restores the order and
# rewrites the generated files.
#
# Flags, all optional:
#   bundle true       the file is embedded in the binary and joins the merged
#                     seed bundle returned by `seed_files()`
#   lexicon meaning   the file joins `MEANING_FILES` (the meaning lexicon)
#   lexicon response  the file joins `RESPONSE_FILES` (multilingual responses)
#   web true          the browser worker fetches the file at startup
#
# A file with `bundle` or `lexicon` gets an `include_str!` constant; a file with
# only `web` is fetched by the browser and never embedded.
'''

UNREGISTERED = [
    (
        "closure-generated-*",
        "python3 scripts/close-total.py",
        "Derived from the other seed files by the total-closure pass, so embedding "
        "a snapshot of them would embed the same statements twice.",
    ),
    (
        "google-trends-*",
        "src/google_trends_catalog.rs",
        "The Google Trends catalog embeds its own prompts and snapshot; the "
        "capability owns the data rather than the shared bundle.",
    ),
    (
        "question-generation-lexicon",
        "src/question_generation.rs",
        "Question generation embeds its own lexicon so the words it may use are "
        "scoped to the one module that may use them.",
    ),
    (
        "roles",
        "scripts/generate-role-registry.py",
        "The reserved-role registry is generated from the other seed files and "
        "read from disk by the audits, never bundled.",
    ),
    (
        "search-fusion-language-grammar",
        "src/search_fusion_grammar.rs",
        "The per-language query grammar is embedded by the search-fusion module "
        "that parses it.",
    ),
]


def read(path):
    with open(path, encoding="utf-8") as handle:
        return handle.read()


def main():
    rust = read(RUST)
    consts = dict(
        (name, stem)
        for name, stem in re.findall(
            r'pub const (\w+): &str =\s*include_str!\("\.\./\.\./data/seed/([^"]+)\.lino"\)',
            rust,
        )
    )
    bundle_block = rust[rust.index("pub fn seed_files") : rust.index("pub const RESPONSE_FILES")]
    bundle = set(re.findall(r'"data/seed/([^"]+)\.lino"', bundle_block))
    response_block = rust[rust.index("pub const RESPONSE_FILES") :]
    response_block = response_block[: response_block.index("];")]
    response = {consts[name] for name in re.findall(r"^\s+(\w+),$", response_block, re.M)}
    meaning_block = rust[rust.index("pub const MEANING_FILES") :]
    meaning = {consts[name] for name in re.findall(r"^\s+(\w+),$", meaning_block, re.M)}
    web = set(re.findall(r'"seed/([^"]+)\.lino"', read(WEB)))

    names = sorted(set(consts.values()) | web)
    lines = [HEADER, "seed_registry"]
    for name in names:
        lines.append(f"  seed {name}")
        if name in bundle:
            lines.append("    bundle true")
        if name in meaning:
            lines.append("    lexicon meaning")
        if name in response:
            lines.append("    lexicon response")
        if name in web:
            lines.append("    web true")
    for pattern, owner, reason in UNREGISTERED:
        lines.append(f"  unregistered {pattern}")
        lines.append(f'    owner "{owner}"')
        lines.append(f'    reason "{reason}"')

    with open(OUT, "w", encoding="utf-8") as handle:
        handle.write("\n".join(lines) + "\n")

    on_disk = {
        entry[:-5] for entry in os.listdir("data/seed") if entry.endswith(".lino")
    }
    print(f"{len(names)} seed file(s) registered, {len(on_disk)} on disk")
    print(f"wrote {OUT}")


if __name__ == "__main__":
    main()
