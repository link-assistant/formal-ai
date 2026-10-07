# Issue #1164 — Code Examples as Formal Knowledge

Issue [#1164](https://github.com/link-assistant/formal-ai/issues/1164) (E129,
part of the #1183 umbrella): code found on the internet was either rendered
verbatim (the Rosetta Code path in `rust/src/coding/rosetta_request.rs`, and
only for tasks in the seeded alias list) or not used at all. Nothing parsed a
retrieved example into the meta language, gave its parts meanings, or
recombined those parts for a new requirement, although meta-language already
ships tree-sitter grammars for most Hello World languages.

The raw issue, its (empty) comment list and a projection of its timeline are
under `raw-data/`. The requirement table with per-row status is the shard
`docs/requirements/issue-1164-code-node-decomposition.md`.

## Timeline

| When (UTC) | What |
| --- | --- |
| 2026-09-29 09:16 | #1164 opened with requirements `R1164-1` to `R1164-11`, a design, a test plan and a definition of done; linked to the #1183 umbrella at 09:21. |
| 2026-09-29 09:34–10:48 | Cross-referenced from #1170, #1177, #1167, #1163, #1165 and #1166. |
| 2026-09-30 | `5d76720ae` adds `rust/src/code_example_knowledge.rs`, the seeds `code-example-parts.lino` and `code-node-decomposition.lino`, five Hello World page fixtures and `rust/tests/unit/issue_1164_code_example_knowledge.rs`. |
| 2026-10-06 | Formatting and lint passes only. |
| 2026-10-07 | `7d4f110d2` fixes the #1163 HTML walker that the internet-path test reads its code blocks through. |
| 2026-10-07 | PR #1188 (branch `qa-reasoning-coding-bulk-fixes`) cross-references the issue. |

## Requirements

The issue's own `R1164-1` to `R1164-11` keep their IDs. The "Tests" and
"Definition of done" sections add four obligations, `R1164-12` to
`R1164-15`: captured fixtures, the gated Pascal online test, case-study data
in Links Notation, and the changelog fragment.

Status on this branch, from the shard:

| Status | IDs |
| --- | --- |
| Implemented, with a test | R1164-1, R1164-3, R1164-4, R1164-5, R1164-10 |
| Partial | R1164-2, R1164-6, R1164-7, R1164-8, R1164-9, R1164-12, R1164-14 |
| Not started | R1164-11, R1164-13, R1164-15 |

## Root causes

1. **Examples were opaque text.** The coding path could render a stored or
   fetched program but had no representation of what its parts mean, so it
   could neither generalize across languages nor bind a new literal.
2. **Meanings had nowhere to live except Rust.** Without a data vocabulary, a
   decomposer would need a per-language table in code, which `R1164-3`
   forbids. The fix records output calls, entry names, node-kind hints, prose
   relations and program shapes in `data/seed/code-example-parts.lino`.
3. **Code blocks had no source.** Decomposition depends on #1163 delivering
   code blocks from fetched pages; before that, there was no path from a page
   to a decomposable block. `decompose_reads_the_code_block_of_a_captured_page`
   now walks that path end to end on a captured Kotlin page.

## Findings that limit the current slice

- **Recomposition fills a stored template.** `recompose_for_requirement`
  substitutes the bound literal into the target's `program_shape` template
  from the seed. The aligned parts contribute their source URLs, but not the
  program text, and the generalized `output_call` parameter is unused. This is
  why `R1164-6` and `R1164-8` are partial.
- **The held-out language is not held out.** `R1164-9` requires that no
  Pascal program be stored anywhere, yet the seed now carries a Pascal
  `program_shape` (`program HelloWorld; … writeln('{literal}') … end.`).
  Pascal also has no grammar row, so decomposition refuses it. Before Pascal
  can count as discovered from the Free Pascal documentation, that template
  has to be removed and the grammar row added.
- **CST parts carry no URL.** `decompose_code_node(source, slug, prose)` is
  never told where the code came from, so only prose-supplied build and run
  commands carry a source URL.
- **Adoption is not wired.** `procedure_step_records` produces what the
  approval gate consumes, but `adopt_decomposed_procedure` does not exist and
  nothing calls the gate (`R1164-7`).

## Prior art

- **Inside this repository.** `coding/cst.rs` (slug to meta-language label
  through `program-cst-grammars.lino`), `coding/rosetta_request.rs`,
  `coding_research_learning.rs` (`adopt_extracted_procedure` and its license
  and approval gate), and `formalization/procedures.rs`
  (`ProcedureStepRecord`).
- **Parsing.** tree-sitter grammars as shipped through meta-language; srcML
  and similar markup of source as a tree of named constructs.
- **Generalization by alignment.** Anti-unification (Plotkin, Reynolds), the
  standard way to find the most specific common structure of two terms with
  the differing positions as variables; tools that learn fix or edit patterns
  from examples use it the same way `generalize_examples` uses shared part
  kinds and per-language parameters.
- **Template-based recomposition.** Structural search-and-replace tools such
  as Comby rewrite code through holes in a template, which is what the
  `program_shape` substitution amounts to today.

## The fix on this branch

- `rust/src/code_example_knowledge.rs`: `decompose_code_node`,
  `generalize_examples`, `recompose_for_requirement`,
  `GeneralizedProcedure::procedure_step_records`, and the three Links Notation
  renderers.
- `data/seed/code-example-parts.lino` and
  `data/seed/code-node-decomposition.lino`, mirrored byte-for-byte under
  `rust/embedded/data/seed/`.
- Fixtures: `rust/tests/fixtures/coding-discovery/issue-1164/` (Rust, Go,
  Kotlin, Swift, Scala Hello World pages, trimmed to the parts the tests read).

## Verification

- Offline, in CI: `rust/tests/unit/issue_1164_code_example_knowledge.rs`
  (fourteen tests). One of them reads the code block out of a captured page
  through the #1163 formalizer, so it also guards the HTML walker fix.
- The schema seed carries one worked generalized procedure in Links Notation.
  `decomposed-example/hello-world-run.lino` (`R1164-14`) is the verbatim output
  of an actual run on 2026-10-07: the branch's browser engine formalized the
  byte-for-byte captures under
  `rust/tests/fixtures/coding-discovery/captured/` (SHA-256 in the file),
  decomposed each page's Hello World block (`decomposeCodeExample`), generalized
  the five nodes (`generalizeCodeExamples`) and rendered both through
  `decomposedCodeExampleNotation` / `generalizedCodeExampleNotation`. The run
  shows two limits: the HTML walker drops code indentation, and the Kotlin and
  Swift program bodies keep a trailing comment that repeats the original
  literal (`// Hello, world!`), so a recomposition with another literal would
  carry a stale comment. Swift's guided-tour block has no entry point, so the
  shared structure is `output_operation` and `string_literal` only.
- Byte-for-byte captures (`R1164-12`): the five Hello World pages (Kotlin tour,
  Rust Book, go.dev, Swift book source, Scala 3 book) are pinned by SHA-256,
  language tags and decomposition in
  `rust/tests/unit/issue_1163_1164_captured_pages.rs` and
  `rust/tests/web/issue-1163-1164-captured-pages.test.mjs`.
- Not yet verified: compiling or running any recomposed program (`R1164-8`),
  the Pascal online path (`R1164-13`), and JS/TS parity (`R1164-11`).
