# Conversion best practices: what meta-language and relative-meta-logic do, and what Formal AI adopted (R1024)

The owner asked (2026-10-06 13:17) to "give priority to the JavaScript
version … first implement all requirements in JavaScript, and only after that
use automated translation; for that use
https://github.com/link-foundation/meta-language/pull/196 and
github.com/link-foundation/relative-meta-logic — take a copy of best
practices we can apply here, so we have an easy way to convert between
JavaScript, Meta Language and Rust, so we can do more with less effort."
This document records what each upstream does, what this repository copied,
what it did not copy and why. The requirement row is R1024 in
[`docs/requirements/doctrine-standing-doctrine-workstation-resources-and-continuous-delivery-2026-10-07.md`](../../requirements/doctrine-standing-doctrine-workstation-resources-and-continuous-delivery-2026-10-07.md).

Both upstreams are released into the public domain under the Unlicense, so
their code and formats may be copied without restriction. The copies here
still name their source in every file header.

## What was studied

- **meta-language PR #196** (branch `issue-195-cc10e17ab860`, open draft,
  read 2026-10-08): `docs/self-translation.md`, `js/src/self-translation.js`,
  `parity/self-translation/` (cases, sources, expected outputs, item lists,
  the hand-written comparison), `docs/decorators.md`,
  `docs/four-language-contracts.md`, `docs/translation-rules.md` and
  `docs/grammar/translation.md`.
- **relative-meta-logic `main`**: `test-corpus/` (README, `expected.lino`),
  `scripts/check-corpus-parity.mjs` and its own test,
  `.github/workflows/parity.yml`, `js/tests/shared-test-corpus.test.mjs` and
  `rust/tests/shared_test_corpus.rs`.
- **relative-meta-logic PR #184** (branch `issue-183-7fedfddffe9c`, open):
  `docs/case-studies/issue-183/portable-natural-translation.md` and
  `meta-language-integration.md`.

Nothing upstream was built or run. The clones were sparse, read-only and kept
in the session scratch directory.

## What meta-language PR #196 does

Self-translation ports meta-language's own modules between JavaScript,
TypeScript and Rust with the same API in both runtimes
(`selfTranslate(source, from, to)`, `meta-language translate --to rust`).
The practices that make it work:

1. **A provenance header.** Every output starts with
   `// meta-language:self-translation:v1 source=… target=… sha256=… bytes=…`,
   so a translation states what it was made from.
2. **Translated or carried, never dropped.** Each top-level item becomes one
   block. A *translated* block keeps its source above the code as `// |`
   lines, with the SHA-256 of the code it emitted. A *carried* block keeps
   only the source lines and the reason the translator could not express the
   item (`(no definition)`, `(unsupported)`, `(type)`). Comments travel with
   the item they document, and a comment group no item follows is copied only
   when it fits both languages. Helpers and the Rust `#![allow(...)]` line sit
   once in a prelude.
3. **Lossless round trips.** Translating back restores every block whose code
   still matches its hash. An edited block loses its provenance and is
   translated again, so edits made in the target language carry over. When
   the header's hash equals the restored text, the result is the original
   source byte for byte.
4. **One corpus in Links Notation.** `parity/self-translation/cases.lino`
   lists `(case …)` rows (source, languages, expected output) and `(call …)`
   rows (a function, its arguments and its result in both languages). Each
   case has an `.items.lino` with one `(item start end term status "reason")`
   link per item. Both runtimes check every case, and the Rust test compiles
   the translated Rust with clippy at `-D warnings` and runs every call.
5. **A check/generate pair.** `npm run check:self-translation` fails when the
   expected files differ from the translator's output, and
   `generate-self-translation-cases.mjs` rewrites them after an intended
   change. Nobody edits expected output by hand.
6. **A translated-against-hand-written report.**
   `generate-self-translation-report.mjs` translates every module and measures
   it against its hand-written Rust: items by status, functions written, how
   many the hand-written Rust defines under the same name, and how many are
   identical up to whitespace. CI runs it in eight shards.
7. **Decorators.** A data-defined set of edits at ten pipeline levels brings a
   generic translation to match hand-written code. Removing a decorator
   restores the generic output, and the provenance hash is taken after
   decoration.
8. **Honest contracts.** The four-language contract tables separate "implemented"
   from "required". Each row names its evidence, and surface syntax is never
   described as resolved semantics.
9. **Declarative translation rules** (`TranslationRuleSet`: a link query plus
   per-language templates, serialized as Links Notation). Formal AI already
   adopted these for its grammar projection legs (plan 16 L8,
   `data/seed/grammar-projection-rules.lino`).

## What relative-meta-logic does

1. **One contract, two runtimes.** `test-corpus/*.lino` are inputs shared by
   the JavaScript and Rust suites. `test-corpus/expected.lino` holds the one
   expected result list per input, and each suite walks the corpus and
   compares its output against that single contract.
2. **A cross-runtime parity runner.** `scripts/check-corpus-parity.mjs` runs
   every corpus file through the JavaScript CLI and the built Rust CLI. It
   fails when exit status, stdout or stderr differ, and the runner has its own
   test (`check-corpus-parity.test.mjs`). A path-filtered `parity.yml`
   workflow runs it.
3. **A named fragment with an explicit contract** (PR #184,
   `portable-natural-v1`). The source grammar of each language is spelled
   out. Values have a stated domain, and names exclude the keyword vocabulary
   of every language. The program is encoded as a Links Notation network
   (`(function name (parameters …) (body …))`), and emission reads only that
   network, never the source text.
4. **Refusal with the source preserved.** Unsupported input returns its
   unchanged source with a code, a description, a stage and an offset. A
   negative corpus pins the refusals, and the full-language refusal is never
   a substitute for the fragment's real translators.
5. **Reimport equality.** Translating source to target and reading it back
   must give the same normalized links network.
6. **An explicit trust boundary.** The preservation argument is stated as an
   argument and a tested contract, not a machine-checked theorem. Native
   receipts record compiler versions and source hashes.

## What Formal AI had before this change

- `scripts/translate-es.mjs` renders `js/` into `ts/`. The rendering is a
  canonical token re-spacing pinned byte for byte to
  `rust/src/es_tokenizer.rs` and `es_meta.rs`. It is a faithful js → ts leg,
  but it carries no types and no provenance.
- `rust/src/meta_translate.rs` (`formal-ai translate`) holds every directed
  leg between rust, js, ts and meta. The rust ↔ ES legs are the plan 16 L8
  grammar projection: token-level `TranslationRuleSet` templates.
- 118 hand-written JavaScript twin modules in `js/agentic/crate/`, and more
  in `js/worker/`, port Rust functions. Each cites its original in a comment,
  "Mirrors \`fn x\` in rust/src/y.rs", and nothing checked that the citation
  still held.
- Behaviour parity was pinned case by case: 27
  `rust/tests/web/*-parity.test.mjs` files, each with its own fixture shape.

None of these gave a JavaScript function a Rust translation that a
human could commit, read back losslessly, and run in both runtimes against
one shared set of observations.

## What was adopted

| Practice | Upstream source | Here | Pinned by |
| --- | --- | --- | --- |
| Provenance header, translated/carried blocks, `// \|` source lines, code hash, prelude | meta-language #196 `self-translation.js` | `scripts/self-translation/envelope.mjs`, header `// formal-ai:self-translation:v1 …` | `rust/tests/web/self-translation.test.mjs` (exact output of a small module) |
| Lossless round trip; edited blocks re-translated | meta-language #196 | `selfTranslate(code, 'Rust', 'JavaScript')` restores by hash | corpus cases `ratings-back-to-javascript`, `geometry-back-to-rust` (expected file is the source itself), `ratings-edited-to-javascript` |
| One Links Notation corpus with `(case …)` and `(call …)` rows, `.items.lino` per case | meta-language #196 `parity/self-translation/`, relative-meta-logic `test-corpus/` | `rust/tests/fixtures/self-translation/cases.lino`: 7 cases and 30 calls | JS side: `self-translation.test.mjs`. Rust side: `rust/tests/unit/issue_1188_self_translation_corpus.rs`, which compiles the committed translation and the Rust source as modules, so CI's build, clippy (pedantic and nursery, `-D warnings`) and rustfmt check them |
| Check/generate pair; expected files never hand-edited | meta-language #196 | `node scripts/self-translate.mjs --check` and `--write` | the `check_self_translation` gate (`data/meta/ci-gates/check-self-translation.lino`) |
| A named fragment with a stated contract, links IR, names outside both keyword vocabularies | relative-meta-logic #184 `portable-natural-v1` | `portable-pure-v1` (`scripts/self-translation/ir.mjs`): pure functions and literal constants over number (`f64`), boolean and string (`&str` in, `String` out). `--to meta` prints the IR as Links Notation, and `--from meta` emits from it | the meta leg cases `ratings-to-meta`, `ratings-from-meta` |
| Refusal names its reason; the source is kept | relative-meta-logic #184, meta-language #196 carried blocks | every refusal is a `(carried slug "reason")` row of `data/meta/self-translation/constructs.lino`, and the marker repeats it | the corpus exercises all 19 carried rows, and a test fails when a reason is not a row |
| Per-construct mapping table as data | meta-language four-language contracts, `TranslationRuleSet` | `constructs.lino`: types, operators (with precedence), unary operators, builtins and string methods. The emitters spell code only from it | a test fails when a row has no evidence in the corpus |
| Reimport equality through the pivot | relative-meta-logic #184 | the IR read from the JS source equals the IR read back from the emitted Rust, and the reverse for the Rust source | `the meta language is the pivot` test |
| Translated-against-hand-written report | meta-language #196 `generate-self-translation-report.mjs` | `node scripts/self-translate.mjs --report` over `js/agentic/crate` | `the report measures the twins` test |
| Cross-runtime drift gate | relative-meta-logic `check-corpus-parity.mjs` | `scripts/check-twin-citations.mjs`: every "Mirrors \`symbol\` in rust/src/…" citation names a definition that still exists | the `check_twin_citations` gate; on its first run it found seven stale citations in `js/worker/formal_ai_worker_code_examples.js`, left by the `discovery_production` split, and they are fixed |

Two adaptations go beyond a copy:

- **The prelude is computed, not blanket.** meta-language emits
  `#![allow(unused, unreachable_patterns, non_snake_case, …)]` and
  parenthesized tails such as `(a + b)`. Formal AI builds with clippy
  pedantic and nursery at `-D warnings`, so the emitter writes `#[must_use]`
  on every function, borrows strings as parameters, parenthesizes only where
  precedence needs it, and puts only the lints the emitted constructs trigger
  (`float_cmp`, `suboptimal_flops`, `imprecise_flops`,
  `missing_const_for_fn`) in the `#![allow(...)]` line. It also writes
  rustfmt's own layout. The canonical IR never tests a negation in a two-way
  branch: the branches are swapped instead, which is the form `if_not_else`
  asks for.
- **JSDoc carries the types.** The JavaScript root already documents every
  twin's parameters with JSDoc. The frontend reads `@param {number}` and
  `@returns {string}` instead of inferring types, and the JavaScript emitter
  writes the same tags back, so a Rust → JS → Rust trip keeps them.

## Measured state (2026-10-08)

- The corpus: `ratings.mjs` (10 translated functions and 2 constants,
  beside 17 carried items and one carried comment group, one for each
  refusal row) and `geometry.rs` (4 hand-written Rust functions, plus the
  attribute that exercises the 19th row, a Rust item outside the fragment). Both round-trip byte for byte, and all 30 calls return the
  same printed result in JavaScript and in Rust.
- The report over the 118 modules of `js/agentic/crate`: 2,057 top-level
  items, 136 of them translated. 68 of the translated items are defined under
  the same name in the Rust module the twin cites, and 46 of those are
  identical to the hand-written Rust up to whitespace. The 22 that differ are
  listed under decorators below. The carried items are
  mostly untyped for the fragment (1,179 items whose JSDoc type is an array,
  object or record), imports (378), and constructs outside it (246).
- Twin citations: all 713 resolvable citations name an existing Rust
  definition. 17 more describe behaviour in prose and are counted, not
  checked.

The report is the honest measure of how much of the JavaScript root converts
mechanically today. Widening `portable-pure-v1` is the work that grows the
136: arrays to slices and `Vec`, JSDoc record types to structs, and imports
to `use`. Each step is a construct-map row, a corpus case and its calls.
That widening is the three-root parity of R992-R996; this row only records
that the practices are in place to carry it.

## What was not adopted, and why

- **Decorators** (meta-language #196). Their purpose is to bring a
  translation to match existing hand-written Rust. The report found 22
  translated definitions that differ from a hand-written one of the same name:
  - Most are counts and limits. The Rust types them `usize`, `u8`, `u32` or
    `LinkAddress`, and the fragment spells JavaScript's `number` as `f64`.
    That is a missing type: an integer row in the construct map, read from a
    JSDoc typedef. A per-module decoration is the wrong fix for it.
  - Two are placeholder strings that the Rust spells with `concat!` to keep
    clippy's `literal_string_with_formatting_args` quiet.
  - One embeds a seed file with `include_str!` where the JavaScript holds the
    file's path.

  Only those last three are what decorators exist for, and three definitions
  do not yet justify a ten-level decorator engine. The report counts them, so
  it shows when they do.
- **A TypeScript target in self-translation.** `ts/` is the canonical token
  rendering of `scripts/translate-es.mjs`, pinned byte for byte to the Rust
  `es_meta` renderer and checked in CI. Adding a provenance header there
  would break that pin for no gain, because TypeScript is read as the
  JavaScript family anyway.
- **Lean and Rocq targets and native receipts** (both upstreams). They are not
  roots of this repository (rust, js, ts, meta).
- **Running a built Rust CLI from Node**, as `check-corpus-parity.mjs` does.
  R1020 forbids local Rust builds, and the Rust side here is a `cargo test`
  over the same corpus that CI runs. The two runtimes still consume one
  contract, which is the point of the practice.
- **The tree-sitter-backed lossless parser** that meta-language uses to cut
  items. The fragment needs only a lexer and a top-level item splitter, both
  in JavaScript (`scripts/self-translation/lexer.mjs`). The Rust side of
  `formal-ai translate` already uses the meta-language crate for its CST legs.
- **A blanket `#![allow(unused, …)]` prelude.** It would hide dead code. The
  computed allow-line above replaces it, and the corpus test requires every
  translated function to have a call, so `dead_code` stays live.

## Upstream gap

The meta-language self-translation output is not clippy-pedantic clean:
missing `#[must_use]`, `String` parameters where `&str` is enough,
parenthesized tails, and no `missing_const_for_fn` entry in the prelude.
A consumer that builds with pedantic lints cannot commit the output as it
is. This is drafted as
[upstream-issue-drafts/07-meta-language.md](upstream-issue-drafts/07-meta-language.md).

## How to use it

```bash
node scripts/self-translate.mjs --to rust js/agentic/crate/module.mjs   # JS → Rust, with provenance
node scripts/self-translate.mjs --to js translated.rs                    # back: restores unedited blocks
node scripts/self-translate.mjs --to meta module.mjs                     # the links IR
node scripts/self-translate.mjs --items --to rust module.mjs             # one (item …) link per item
node scripts/self-translate.mjs --check                                  # the corpus gate
node scripts/self-translate.mjs --report                                 # twins vs their Rust
node scripts/check-twin-citations.mjs                                    # twin citations resolve
```
