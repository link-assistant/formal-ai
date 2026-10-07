# Pull request #1188: bulk QA, reasoning and coding fixes

Pull request: <https://github.com/link-assistant/formal-ai/pull/1188>

Branch: `qa-reasoning-coding-bulk-fixes` (opened 2026-09-30, 191 commits ahead
of `main` at the time of writing).

Per-issue investigations live in each issue's case study
([#1161](../issue-1161/), [#1172](../issue-1172/), [#1173](../issue-1173/),
[#1174](../issue-1174/), [#1175](../issue-1175/), [#1176](../issue-1176/),
[#1177](../issue-1177/), [#1181](../issue-1181/)). This directory records what
is specific to the pull request: its scope, its CI history, the root causes
found while getting the full suite green, the decisions taken, and the
constraints the work ran under (CONTRIBUTING rule 13).

## Raw data

| File | Contents |
| --- | --- |
| [`raw-data/pull-request.json`](raw-data/pull-request.json) | `gh pr view 1188 --json title,body,comments,reviews,commits,createdAt,headRefName`, commits trimmed to short SHA, headline and date (GitHub lists the newest 100). No conversation comments or reviews existed. |
| [`raw-data/ci-cd-pipeline-runs.json`](raw-data/ci-cd-pipeline-runs.json) | Every "CI/CD Pipeline" run on the branch (51), one per line: run id, created, head SHA, status, conclusion. |
| [`raw-data/ci-cd-failed-jobs.json`](raw-data/ci-cd-failed-jobs.json) | For each failed run (20), the names of the jobs that failed. |
| [`raw-data/commits.txt`](raw-data/commits.txt) | `git log --reverse origin/main..HEAD`: short SHA, date, subject. |

## Scope

The description closes eight issues with GitHub closing keywords
(CONTRIBUTING rule 12):

| Issue | Scope |
| --- | --- |
| #1161 | clients registry `config_env` with JSON/CLI projection; release CLI binaries |
| #1172 | factual QA word-boundary subject match |
| #1173 | the fallback executes the search it offers; no canned "Web search requested" paragraph |
| #1174 | summarization and text-transform handler family |
| #1175 | routing by formalization: software-project phrase vocabulary, a 359-probe routing corpus with a coverage budget |
| #1176 | calendar offsets, statistics, unit conversion, word problems |
| #1177 | nine code-task handlers (review, debugging, explanation, refactoring, tests, regex/SQL synthesis, format conversion, shell compose) |
| #1181 | shared `cli_env` boolean parser and install scripts |

It also carries fixes for #1154, #1155, #1163, #1164, #1165, #1166 and #1184
found on the way, and four standing doctrines stated by the project owner
while the pull request was open:

- JavaScript first, full parity, then translate (R997-R1000);
- the recursive meta algorithm is the main path (R1001-R1012);
- the JavaScript server has full parity with the Rust server, guaranteed by
  CI (R1013-R1015), and no maintained file exceeds 1500 lines (R1016);
- work is delegated to Formal AI itself (R1017).

## Timeline

The full list is [`raw-data/commits.txt`](raw-data/commits.txt).

| Date | Commits | What landed |
| --- | --- | --- |
| 2026-09-30 | `7239c69bc` .. `05620d936` (78) | The bulk issue batch: handlers behind data-driven cue tables, registries, ledgers re-measured. The branch did not compile. |
| 2026-10-06 | `9e2f73b14` .. | Build restored (missing `version_resolution` module, lifetimes, the sha2 0.11 hex helper, 189 duplicated test-module declarations); js -> ts translator in JavaScript (`bc0986862`); 35 native-only handler rows ported to JavaScript; recursive meta reasoner. |
| 2026-10-07 early | `5a56705b5`, `8f1d3aaa2`, `77b7c86d2` | JavaScript server at Rust parity; 1500-line ceiling on every file (stylesheet, app, specs, requirements, changelog split); local web tests serialized. |
| 2026-10-07 | `ca1e24848`, `e00b3baa5`, `3836ea751` | Ladder rungs and small delegated tasks derive by general rules (R1009, R1017); coding-ladder floor corrected 24 -> 23. |
| 2026-10-07 | `46d45c9e1` .. `7688c93db` | Full-suite repairs, 234 test-target clippy fixes, workflow hardening. |
| 2026-10-07 | `7d4f110d2`, `6872307e4`, `a0e1786ee`, `49447ab5c` | HTML walker, role registry, agentic call arguments, REST issue URLs, quoted-literal masking. |
| 2026-10-07 | `2e1d6cb62` | Lean and Rocq keep the negation of a "No ..." statement. |
| 2026-10-07 | requirement-drafting batch | Every planned open row drafted: #1163/#1164 JS twins and working memory, #1172 subject-verified facts, #1176 word relations and Wikidata-checked unit factors, #1177 GROUP BY, #1180 history store, #1184 rule stage and JS twin, #1185 honest repair stops, #1186 deformalization and probe set, #1187 optional credentials. |
| 2026-10-07 | `a3340286e`, `065cb49af` and the next commit | #1177 breadth. The code debugger gets two more defect classes, a loop bound past the end of an indexed collection and an assignment inside an `if`/`while` condition (R1). The SQL composer reads a threshold after the grouping as HAVING and no longer emits a bogus `WHERE with > 5` (R3). The shell composer gets a literal `sed -i` substitution from seeded cues (R4). |

## CI history

51 "CI/CD Pipeline" runs: 20 failed, 30 were cancelled by a newer push
(the workflow's concurrency group), one in progress at the time of writing.

| Phase | Failed runs | Failing jobs per run |
| --- | --- | --- |
| Opened; branch did not build | 36707093714, 36742616682 (2026-09-30) | 4, then 14: build, lint, both test shards, all seven box-image projects, both E2E suites |
| Build being restored | 37470366082 .. 37513029093 (2026-10-06) | 12-14 while the lib-test build failed; 5-7 in the runs where it built (37485467245, 37489943444, 37491306927) |
| Full unit suite runs | 37516756207 .. 37541942966 | 5-7: lint, both test shards, both E2E suites, Docker, server parity once |
| Narrowing | 37549001684 .. 37570930929 (2026-10-07) | 4, 3, 2, 3, 3: only `Lint and Format Check` and `Test (ubuntu-latest / full)` remain |
| Current | 37574144432 (`28443684c`) | in progress |

Inside the test jobs, the first complete unit run reported **113 test
failures plus 7 failing lint gates** (test counts as recorded in the pull request
description); after the full-suite repairs (`46d45c9e1` and the
lint follow-ups) the next complete run reported **18**. The latest completed
run, 37570930929 on `6872307e4`, still failed `Lint and Format Check` and
`Test (ubuntu-latest / full)`; those failures are addressed by `7d4f110d2`,
`6872307e4`, `a0e1786ee`, `49447ab5c` and `2e1d6cb62`, which the in-progress
run covers.

The failed-job sets per run are in
[`raw-data/ci-cd-failed-jobs.json`](raw-data/ci-cd-failed-jobs.json).

## Root causes found and decisions taken

Every failure was traced to a root cause; no test was relaxed to pass.

1. **Coding-ladder floor 24 -> 23, with a "corrected overcount" note**
   (`3836ea751`, gate in `46d45c9e1`). `multi.doc_and_requirement` printed
   the same open impasse as when the floor was recorded; its verify step
   grepped for `coding-task` in `REQUIREMENTS.md`, which the pre-split file
   happened to contain. After the 1500-line split the word was gone and the
   spurious pass disappeared. Decision: record the honest floor of 23, and
   make the gate accept a falling floor only when a "corrected overcount"
   note names the old floor, so a real regression can never be relabelled.
2. **Worker modules under the 1500-line limit** (`46d45c9e1`, R995/R999,
   R1016). `formal_ai_worker_20.js` grew with the JavaScript twins; its
   helpers moved into separate modules so it sits under the 1400-line warning
   band (1331 lines now), and `prompt_variations` was split. A worker module's budget shard may rise only
   when its rationale names the handler keys it ports.
3. **HTML walker** (`7d4f110d2`, #1163, #1164). `web_formalize` jumped from
   `<html>` to `</html>` instead of stepping into container tags, did not skip
   comments and opaque tags, and matched close tags by prefix, so `</p`
   closed a `<pre>`. Close tags now match by full name; the comment delimiters
   are named seed data so the literal ratchet holds (`a0e1786ee`).
4. **Role registry from `MEANING_FILES`** (`7d4f110d2`). The registry was
   generated from a different file list than the one the lexicon loads, so the
   roles of `software-project-phrases` were undeclared. It is now generated
   from exactly the files the lexicon loads.
5. **JSON-object call arguments** (`6872307e4`, #1154). A replayed tool call
   whose `arguments` is a JSON object, not a string, lost its command. Fixed
   in Rust and in the JS server.
6. **REST `/issues` URL** (`6872307e4`, #1155). `gh api` and `curl` reads of
   an issue map to `/issues/{n}`, the URL the plan targets; the test reads the
   two-call `gh api` template.
7. **Quoted-literal masking** (`6872307e4`, #1166). Sentence and clause
   splitting masks quoted literals, so an output literal like
   `"Hello, World!"` survives decomposition. Rust and JS.
8. **Lean and Rocq negation** (`2e1d6cb62`). The conjunctive-clause templates
   hardcoded an existential instead of reading `{quantifier}`, so "No cat
   that sleeps hunts" rendered positive. Lean reads the quantifier; Rocq gains
   a `clause_negative` template that parenthesizes the existential under `~`.
   Rust and JS twins.
9. **WASM worker refreshed from a CI artifact** (`7d4f110d2`). Without a local
   Rust toolchain the committed `js/formal_ai_worker.wasm` cannot be rebuilt on
   the workstation. The release workflow now uploads the CI-built WASM worker
   as an artifact, and the committed file is refreshed from that artifact
   rather than from a local build, so it matches the exact toolchain CI
   checks it with.
10. **Seed loaders one level too shallow** (2026-10-06). Ten handlers read
    `.lino` fields at the child level while seed files nest one wrapper deeper,
    so their cues silently never matched; the worker registry's permutation
    assertion threw inside a silent catch and emptied the lexicon. All ten
    loaders descend one level; the precedence seed lists every key.
11. **Rust follows JavaScript.** Porting the 35 native-only handlers exposed
    Rust handlers that misread their own seeds (TRIZ by record name, the
    formalization seed nesting, creative-composition roots, a double `≈` in
    unit conversion, code_debugging's whitespace scan); each was fixed to
    match the JS twin.
12. **Stale pins hidden behind earlier failures.** The full suite of run
    37574144432 failed only on the issue #918 metadata-gap pin (737, while the
    shards held 770): every earlier leg had died at the census-freshness step
    before the unit tests ran, so 33 reviewed-data gap rows from the seed
    growth of #1163 to #1187 accumulated unpinned. The pin now records the
    recount, including the eight meanings this batch added.
13. **Rules reach the derivation by data.** Instead of teaching
    `derivation.rs` the event names of the text-transform handlers, the
    derivation schema seed declares which event kinds a `rule` stage collects;
    a new handler joins `formal-ai explain` by naming its kind there (#1174 R9).
14. **A stopped loop says why.** The repair loop's ladder-exhausted and
    unresolved-need notes and its attempt chain existed but never reached an
    answer; the executor now appends them to the failure report in both roots
    (#1185 R4 to R6).

## Constraints

- **JavaScript first, full parity.** Every requirement lands in JavaScript
  first (or together with Rust); `scripts/check-js-parity.mjs` holds the
  native-only row count at 0, and `scripts/check-server-parity.mjs` compares
  the two servers on 147 requests with a falling ceiling of content gaps.
- **No local cargo.** Rust is built, linted and tested only in CI on push; the
  js -> ts translator was ported to JavaScript so `ts/` regenerates without a
  crate build, and the WASM worker comes from a CI artifact.
- **One node test at a time.** Local verification runs single test files
  serially (`node --test --test-concurrency=1`; `npm run test:web` is
  serialized, `77b7c86d2`) to keep CPU load down on the workstation; the full
  suite belongs to CI.
- **At most two or three sub-agents at once**, each making code changes only.
- **Files at most 1500 lines** (Rust 1000), including this case study and its
  raw data.

## Review checklist

- The description closes #1161, #1172-#1177 and #1181 with `Fixes`.
- Each ratchet (JS parity 0, worker literals 1106, server parity 5, routing
  misroutes js 22 / rust 54, closure gap 1045, ladder floor 23) is measured
  by CI, not asserted.
- `CI/CD Pipeline` is green on the final head SHA before the draft is marked
  ready.
