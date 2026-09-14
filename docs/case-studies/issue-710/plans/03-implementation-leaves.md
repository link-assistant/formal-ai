# Plan 03 — Implementation leaves

Each leaf is small enough to be finished, tested and committed on its own, so
an interruption between two leaves loses nothing. A leaf names the files it
touches, the test that proves it, and the gates it must keep green. The order
is the order of work; a leaf is not started before the one above it is
committed, except where marked *parallel*.

Standing gates for every leaf (run before each commit):

```bash
export RUSTUP_TOOLCHAIN=1.98.1
cargo fmt --check && cargo clippy --lib --bins --tests --all-features -- -D warnings
rust-script scripts/check-hardcoded-language.rs      # R379: no prose literals
rust-script scripts/check-minimal-core-boundary.rs   # handler files may not grow past the ledger
rust-script scripts/check-file-size.rs               # 1000 lines per Rust file
rust-script scripts/normalize-ordered-lists.rs       # mod lists canonical
cargo run --example regenerate_self_ast_census       # after any src/ change
rust-script scripts/run-ci-gates.rs --stage rust     # everything the rust stage runs
```

Conventions that bite: no `#[cfg(test)]` under `src/`; tests live in
`tests/unit/<name>.rs` and are registered in `tests/unit/mod.rs` in canonical
order; a test that reads `.answer` asserts the exact string; every new seed
lexeme ships in en, ru, hi, zh and es; Links Notation values carry no
backslash escapes; commit with `git commit -F`.

## L0 — CI recovery (plan 00)

- [x] L0.1 `src/web/app.js` rebuilt with bun 1.4.0
- [x] L0.2 E2E slowdown measured, fixed, regression test added
- [x] L0.3 the two 2026-08-01 commits carry `Formal-AI-Model`; branch force-pushed with lease
- [ ] L0.4 CI green on the pushed head (all workflows)

## L1 — The ratchet reads history correctly

**Files.** `src/external_benchmarks/ratchet.rs`, `tests/unit/specification/external_benchmarks.rs`.

**Change.** `violations` compares each dated result row with the floor that
was in force *when that row was recorded* (the `minimum_pass_count` recorded
before that date, or the suite's first floor), not with today's floor. A
row that scores below the floor in force at its date is a regression; a row
that predates a later raise is history. `regressions` (base-ref comparison) is
unchanged: a PR may still never lower a recorded pass count or a floor.

**Test.** A ledger with `humaneval` rows `0, 0, 1` and a floor raised to 1 by
the third row has no violation; the same ledger with a fourth row `0` has one;
a PR that rewrites the floor from 1 to 0 is a regression.

- [x] implemented and pinned; the 2026-09-14 ledger text (copied into the test)
      produces no violation

## L2 — `CodingTaskSpec`: one spec from three prompt shapes

**Files.** `src/coding/task_spec.rs` (new), `src/coding/python_signature.rs`
(reuse), `data/seed/meanings-coding-request.lino` (new; roles
`coding_request_verb` — write/implement/complete/define/create, and
`coding_request_object` — function/method/program, in five languages),
`tests/unit/coding_discovery/task_spec.rs`.

**Shapes.** (a) HumanEval: leading prose line optional, imports, `def
name(params) -> ret:`, docstring with sentences and `>>>` examples; (b) MBPP:
task sentence(s), `Reply with … It must pass these tests:`, `assert
name(args) == expected` lines (the name and arity come from the asserts, the
parameters are named `arg1…argN` unless the sentence names them); (c)
conversational: "write a Python function `name` that … / напиши функцию на
Python …", with or without examples.

**Output.** `CodingTaskSpec { language, name, parameters: Vec<Parameter>,
return_annotation, imports, requirement_sentences, examples: Vec<Example>,
prose_language }` and `to_links_notation()`.

**Tests.** The three shapes in en; shape (c) in ru, hi, zh, es with the same
spec identity; a prompt that is not a coding task (a definition question, an
arithmetic question) yields `None`.

- [x] implemented; five-language tests green; `check-hardcoded-language` clean

## L3 — A coding task is recognised before any lexical route

**Files.** `src/solver_dispatch.rs` / `data/seed/handler-precedence.lino`,
`src/intent_formalization.rs` (`route_for_prompt`), `tests/unit/coding_discovery/routing.rs`.

**Change.** When `task_spec::recognise(prompt)` returns a spec, the problem
frame's domain is `program_synthesis` and the synthesis handler runs first;
arithmetic, concept lookup and definition merge do not see the prompt. The
recognition is structural (a signature with a docstring; task text with
asserts; a request verb + object + language name in any seeded language), not
a keyword.

**Tests.** The HumanEval/8 misroute (`sum_product` → *Color*) is pinned as a
regression: the answer is either a verified function or the named gap, never a
concept lookup; the same for the MBPP shape; an arithmetic question that
mentions "function" in prose ("what is the value of the function f(x)=2x at
3") still routes to arithmetic.

- [x] implemented; routing regressions green; `handler-precedence.lino` rationale line added

## L4 — Wikifunctions as a live source of function parts

**Files.** `src/coding/function_catalog/mod.rs`, `src/coding/function_catalog/wikifunctions.rs`
(new), `data/seed/sources-registry.lino` (the `wikifunctions` entry gains
`use function_parts`, the `wikilambdasearch_labels` and `wikilambda_fetch`
API templates, and both licenses as separate fields),
`tests/fixtures/coding-discovery/wikifunctions/` (captured JSON for a handful
of functions with a `capture-manifest.lino`: timestamp, sha256, bytes,
license), `tests/unit/coding_discovery/wikifunctions.rs`.

**API.** `search_functions(client, phrase, language) -> Vec<FunctionMatch>`
(`page_title` Zid, `label`, `match_rate`, `match_lang`); `fetch_function(client,
zid) -> FunctionPart` (argument type Zids, return type, implementation Zids,
tester Zids, labels per language); `fetch_implementations(client, zids) ->
Vec<Implementation>` (language `Z610` → `python`, `Z600` → `javascript`, code,
license `Apache-2.0`); `fetch_testers` → calls and expected results where the
tester is a plain call with literal arguments. Every fetch is a
`CachedSourceClient::fetch` of a `wikilambda_fetch` URL, so offline replay is
the same code path.

**Links.** `FunctionPart::to_links_notation()` emits `function_part` records
(§1 step 8 of plan 02).

**Tests.** Offline over the fixtures: search "greatest common divisor" finds
`Z13612` first; its Python implementations are `Z14857` and `Z13642`; the
JavaScript ones are not offered for a Python task; license and sha256 are on
every record; a search with no match returns an empty list, not an error; with
the transport disabled no network call is made (the existing
`SourceTransport` fake).

- [x] fixtures captured live and recorded in the manifest (`FORMAL_AI_LIVE_FETCH=1` once)
- [x] client and links implemented; offline tests green

## L5 — The standard-library documentation index

**Files.** `src/coding/function_catalog/python_docs.rs` (new),
`tests/fixtures/coding-discovery/python-docs/` (captured `functions.html`,
`stdtypes.html`, `math.html`, `itertools.html`, `heapq.html`, `collections.html`
from the pinned 3.12 documentation, with manifest), `tests/unit/coding_discovery/python_docs.rs`.

**Index.** From each page: symbol, signature, first sentence of the description,
anchor URL. Stored as `stdlib_part` links (symbol, module, description words,
source URL, license `PSF-2.0`). Offline fallback when a page is not cached:
`python3 -c 'import <module>; print(<symbol>.__doc__)'` for a fixed list of
symbols, recorded with provenance `python3.<minor>:<module>.<symbol>.__doc__`
so the answer says where the words came from.

**Matching.** `parts_for_phrase(phrase) -> Vec<StdlibPart>` ranks by
description-token overlap after lemmatisation through the existing lexicon
(`sum` ↔ "sum", "total"; `max` ↔ "largest", "maximum"; `sorted` ↔ "sorted",
"ascending"; `str.count` ↔ "how many times … non-overlapping").

**Tests.** "sum of all the integers" → `sum`; "product of all the integers" →
`math.prod`; "largest" → `max`; "how many times a substring can be found …
overlapping" ranks `str.count` **below** the structural `count_overlapping`
meaning because the description says non-overlapping (the negation is a
token, so the test pins that the composer sees it).

- [x] fixtures captured; index and matching implemented; tests green

## L6 — Concept discovery over requirement sentences

**Files.** `src/coding/concept_discovery.rs` (new), `data/seed/meanings-coding-structure.lino`
(new; the structural meanings of plan 02 §4 with five-language lexemes,
idiom, grounding URL), `tests/unit/coding_discovery/concepts.rs`.

**Algorithm.** For each requirement sentence: (1) canonicalise through
`operation_vocabulary`; (2) mark structural meanings whose cues occur (with
their spans); (3) the remaining noun/verb phrases (split at the structural
cues, conjunctions and punctuation) are concept phrases; (4) for each phrase:
local links first (seed meanings, discovered-procedure ledger), then the
standard-library index, then Wikifunctions label search in the prose language
and in English; (5) any label or description word not in the lexicon is looked
up through the Wiktionary/Wikipedia chain (`definition_merge` already does the
lookup) to attach a meaning, bounded by `max_depth 2`, `max_pages 8`,
`max_seconds` from `SolverConfig`; (6) output a `ConceptMap` (per need: phrase
→ candidate parts with source, arity, license) as links, and a `need_ledger`
row `blocked` for a phrase that found nothing.

**Tests.** Over fixtures: "Return a greatest common divisor of two integers a
and b" → one need with parts [`Z14857`, `Z13642`, `math.gcd`]; "return a tuple
consisting of a sum and a product of all the integers in a list" → structural
`tuple_of` over needs [sum → `sum`; product → `math.prod`, Wikifunctions
"product of list"]; "are any two numbers closer to each other than given
threshold" → `quantifier_any` over `pairwise_distinct` over
`predicate_abs_diff_lt`; the same three sentences in ru/hi/zh/es paraphrase
produce the same `ConceptMap` identity; an unknown word triggers exactly one
bounded lookup and is recorded.

- [x] seed meanings written in five languages; total-closure and multilingual tests green
- [x] discovery implemented; tests green; `need_ledger` rows emitted

## L7 — Composition, drafts, selection

**Files.** `src/coding/composition.rs` (new), `src/coding/python_render.rs`
(new: render a composition tree to Python through the CST validator),
`tests/unit/coding_discovery/composition.rs`.

**Algorithm.** Plan 02 §4 steps 1–5, using `draft_portfolio::PortfolioLeaf`
for k drafts and `AgentWorkspace` for verification; parts are wrapped as inner
helpers renamed from their Zid to a name derived from their label; imports
come from the part (`import math`) plus the spec's own imports.

**Tests.** Over fixtures, without network: `greatest_common_divisor` passes by
direct wrap of a Wikifunctions part and the comparison records `math.gcd` as
the least-action winner; `sum_product` passes by `tuple_of(sum, math.prod)`
with the `[] → (0, 1)` example deciding; `has_close_elements` passes by the
structural composition (no catalog part exists); a task whose parts all fail
the examples returns `None` with a `research_trail` naming the phrases, parts
and the failing example; the parity fixture `count_vowels` case passes by
`reduce_count` over `filter_only` over membership (no literal body).

- [x] implemented; the three previously memorized tasks pass by derivation
- [x] `tests/unit/issue_1085_upstream_prompt_transfer.rs` expectations updated to the derived answers (exact strings)

## L8 — Rewire the synthesis handler; delete the memorized bodies

**Files.** `src/solver_handlers/program_synthesis.rs` (becomes: recognise →
spec → ledger lookup → discovery → composition → answer or gap; the three
`if task.slug == …` blocks and `PythonCandidate` literals are deleted),
`src/program_skill_gap.rs` (the gap message gains the research trail),
`data/seed/multilingual-responses-synthesis.lino` (five-language `Sources:`
and trail lines), `data/seed/meanings-program-synthesis.lino` (the
`has_close_elements`/`similar_elements`/`count_vowels` task meanings are
removed — they are memorized specifics), `data/meta/core-boundary-ledger.lino`
(the handler file shrinks; ratchet down), `data/parity/cross-runtime-synthesis.json`
+ `src/web/worker/formal_ai_worker_06.js` / `_13.js` (the JavaScript bodies are
removed; the browser answers with the formalized spec and discovered parts
marked unverified — plan 02 §7), `tests/e2e/tests/issue-327.spec.js`,
`tests/unit/specification/benchmarks.rs` (derivation marker unchanged:
`synthesis:verification tests_passed`).

**Tests.** The curated slice stays 13/13; `issue_1085_upstream_prompt_transfer`
green with derived answers; the web tests and the Playwright parity spec green
with the browser boundary answer; the gap answer for an undiscoverable task
is exact and names its trail.

- [x] handler rewired; benchmark-specific bodies gone
- [x] curated slice, transfer tests, web tests, parity spec green
- [x] core-boundary ledger ratcheted down with rationale

## L9 — Discovered-procedure ledger: remember, forget, rediscover

**Files.** `src/coding/discovered_procedures.rs` (new), the cache directory
file `discovered-procedures.lino`, `tests/unit/coding_discovery/ledger.rs`.

**Tests.** A solved spec is appended with every provenance field; the same
prompt again is a `cache_hit` and regenerates identical code; deleting the
file and solving again over the same captures yields the same content id
(`forgotten_procedures_are_rediscovered_from_the_same_sources`); a tampered
entry (sha256 mismatch) is ignored and re-derived.

- [x] implemented; tests green

## L10 — The harness discovers online; honest numbers recorded

**Files.** `src/external_benchmarks/mod.rs` (`benchmark_solver_with(online:
bool)`), `src/cli_benchmark.rs` (`--online`, default off; `FORMAL_AI_LIVE_FETCH`
respected), `.github/workflows/external-benchmarks.yml` (the scheduled and
dispatch runs pass `--online`; the PR-time `ratchet` step is unchanged and
runs no suite), `tests/unit/ci-cd/` workflow contract for the new flag,
`docs/benchmarks.md` "Running it".

**Local measurement** (recorded in plan README log and the PR body, dated,
labelled local):

```bash
FORMAL_AI_LIVE_FETCH=1 cargo run --release -- benchmark run --suite humaneval --slice 20 --online
FORMAL_AI_LIVE_FETCH=1 cargo run --release -- benchmark run --suite mbpp --slice 20 --online
```

- [x] flag and workflow change landed; contract test green
- [x] local HumanEval and MBPP 20-slice numbers recorded: 3/20 and 1/20

## L11 — No-memorization gate

**Files.** `tests/unit/coding_discovery/no_memorization.rs`, reading the
cached upstream slices (`target/formal-ai-benchmarks/humaneval.jsonl`,
`mbpp.jsonl`; skipped with a printed reason when not cached, never a false
pass) and asserting that no `entry_point`, no docstring sentence longer than
four words, and no MBPP task sentence appears in any file under `src/` or
`data/seed/`, with a narrow allowlist for words that also occur as ordinary
prose (`sum`, `max`, and the comparative `longest`; `intersperse` is not
allowlisted).

- [x] gate green against the cached upstream slices; documented in `CONTRIBUTING.md` beside the
      other honesty gates

## L12 — Conversational coding requests in five languages; count to N

**Files.** `data/benchmarks/coding-discovery-paraphrases.lino` (held-out
requests, five languages, families: gcd, sum-and-product, any-two-closer,
count-distinct, count-to-N), `tests/unit/coding_discovery/multilingual.rs`,
`src/coding/catalog/tasks.rs` (`count_to_three` stays; a parametric
`count_to_n` is *derived*, not templated — the request's number binds
`range_inclusive`).

**Tests.** Every paraphrase produces the same spec identity as its English
sibling and a verified answer over fixtures; "count to 100" / "посчитай до 100"
/ "100 तक गिनो" / "数到 100" / "cuenta hasta 100" produce a program whose output
is `1 … 100` (verified by execution); none of the paraphrase sentences occurs
in `data/seed/` (asserted).

- [x] implemented; five-language tests green; #1071 partially delivered and
      the issue comment says exactly which part

## L13 — Rosetta Code task pages (#862, #863)

**Files.** `src/coding/function_catalog/rosetta_code.rs` (new; MediaWiki
`action=parse` for a task page, section per language, code blocks with the
page's GFDL-1.2 attribution), `data/seed/meanings-coding-request.lino`
(`example_request` and `execute_url_request` roles, five languages),
`src/solver_handlers/program_synthesis.rs` (an example request answers with the
attributed example and a note that it was not generated; an execute request
runs the Rust example in the bounded workspace when `rustc` is present),
`tests/fixtures/coding-discovery/rosetta/Copy_stdin_to_stdout.json`,
`tests/unit/coding_discovery/rosetta.rs`.

**Tests.** The two issues' exact prompts (and their four other-language
paraphrases) produce the example / the execution result; the URL is never
turned into a shell command; the license line is present.

- [x] implemented; #862 and #863 are named for closure in the drafted PR body

## L14 — Documents, requirements, traceability, changelog

- [x] `docs/requirements/issue-0710-dynamic-coding-discovery.md` (new shard:
      R710-D1 … one row per plan-01 B-row, status with test names); wording
      updates to the #919 and #412 shards (plan 01 §D); `rust-script
      scripts/assemble-requirements.rs --write`
- [x] `docs/requirements-traceability.md` rows for the new R-ids
- [x] `docs/benchmarks.md` "Honest current numbers" refreshed from the ledger;
      the doc-pin test reads the ledger's latest row per suite instead of a
      literal table (`tests/unit/docs_requirements/benchmarks.rs`)
- [x] `VISION.md` "Current Direction" upstream sentence refreshed the same way
- [x] `docs/meta-algorithm.md`: "The coding discovery meta-algorithm (issue
      #710 continuation)" section with the grounded step list, pinned by a
      grounding test like the other recipes
- [x] `data/meta/coding-discovery-recipe.lino` (the recipe as data, each step
      citing its live function) and `data/meta/coding-research-learning-contract.lino`
      gaining the real source formats
- [x] `changelog.d/20260915_020000_coding_discovery.md` (`bump: minor`)
- [x] `docs/case-studies/issue-710/README.md`: a dated section pointing at
      `plans/` and stating the before/after numbers
- [x] self-AST census regenerated; seed registry regenerated after seed changes
      added (`rust-script scripts/generate-seed-registry.rs --write`)

## L15 — Formal AI authors one leaf

- [x] one leaf of this batch (the coding-discovery recipe data of L14) is
      authored through
      `scripts/author-change-with-formal-ai.sh` against the served branch
      binary; the commit carries `Formal-AI-Session`, `Formal-AI-Evidence`,
      `Formal-AI-Model`; the evidence goes to the gist store, not the repo

## L16 — Final preparation

- [x] `rust-script scripts/run-ci-gates.rs --stage rust` green; `npm run
      test:web` green; Playwright parity spec green locally
- [x] PR body: summary, before/after numbers (upstream local measurements,
      curated slice, ladder unchanged), `Fixes #710`, `Closes #862`, `Closes
      #863`, partial note on #1071, links to the plans
- [ ] one CI wait; every workflow green; then stop
