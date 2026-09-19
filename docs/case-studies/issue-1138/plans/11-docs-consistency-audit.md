# Plan 11 — Docs consistency audit (bottleneck B11 of #1138)

Bottleneck B11 of issue #1138: *"The record of truth disagrees with itself, which
starves self-improvement of honest input."* This plan is the exhaustive list of
every outdated, contradictory or unsupported statement the project's documents
carry, measured against `main` at `be8fd3174` (the merge of PR #888,
2026-09-16), so a later implementation pass can make every document agree with
the latest requirements and with measured reality.

Nothing is fixed here. This plan is the input to the fix.

## Issues addressed

*(Added by the 2026-09-16 reconciliation. This plan was the only one of 01-12
without this section; plan 13's coverage table names it as the deliverer of six
issues, and a plan that does not list them cannot be cross-checked against that
table — plan 00 §8.)*

- **#1138 B11** — "the record of truth disagrees with itself, which starves
  self-improvement of honest input." This plan is the exhaustive finding list and
  the mechanism that stops the findings recurring.
- **#957 (E105)** — "Traceability protocol: delivered / tested / confirmed
  columns, CI-enforced on every requirement row." Delivered by L1-L3:
  `data/meta/requirement-status-ledger.lino` plus
  `scripts/check-requirement-status.rs`, with
  `docs/requirements-traceability.md` becoming a generated projection. The
  table's own header admits today that "CI enforcement of this table's freshness
  is tracked by E105 (#957); this is the initial data population, not an
  automated generator wired into CI yet".
- **#958 (E106)** — "report the upstream benchmark score wherever the curated
  number is cited." Delivered by L5 (the curated-beside-upstream assertion) and
  L33 (the rendered README block). `NON-GOALS.md:55` already states the rule and
  D23 records the two sites that violate it.
- **#1089 (E111)** — "collapse the gate ecosystem: render status tables from
  `data/meta`; at most 5 `docs_` tests." Delivered by L2 (one generated surface)
  and **L76** (the collapse itself, added by the reconciliation). The count is 49
  today and rose since #1089 was filed (D149).
- **#1090 (E112)** — "finish or retire the traceability manual-confirmation
  column." This plan measures it (743 of 805 unconfirmed) and offers both
  branches; **the choice between them is a maintainer decision this plan records
  rather than makes** (risk 2), so #1090 stays open — see plan 13's
  "will not close" list.
- **#949 (E97)** — "close the en/ru/hi/zh parity gap; add a parity lint."
  Delivered by **L75** (the lint, added by the reconciliation) plus every plan's
  held-out five-language corpus, `es` included. D49/D61 record that Spanish is
  `status partial` in three places and absent from a fourth.
- **#955 (E103)** — the 49-check runtime hand-check suite. The checklist and its
  ledger land here (L50); a live device, a deployed demo and an upstream tracker
  need the maintainer, so #955 stays open.
- **#651** — "refresh vision and roadmap with status tracking." The docs half of
  the epic lands here; the epic stays open while any child does.
- **#1085 D5.4 / R1085-16** — one gate per justification, and the status render
  #1085 recorded as an open sub-issue.

## Root causes

*(Added by the 2026-09-16 reconciliation. The findings table below is the
evidence; these are the four mechanisms it is evidence of.)*

1. **A number copied beside the thing it counts is a number that drifts.**
   Every "stale number" finding — D27, D28, D30, D36-D38, D41, D42, D45,
   D51-D53, D63, D81-D84, D101, D127-D130, D141, D148, D149 — is one defect
   repeated: a count lives in prose in six documents and in a ledger in
   `data/meta/`, and only the ledger is checked. `tests/unit/docs_requirements_issue_1021.rs:13-21`
   already states the principle and applies it in exactly one place.
2. **The pins are backwards.** 49 `docs_*` files carry 360 assertions that pin
   *phrases*, and almost nothing pins *numbers*. That is why every phrase-level
   claim has held and every numeric one has rotted.
3. **Four documents each claim to be the single source of truth** (D26:
   `ROADMAP.md:3`, `REQUIREMENTS.md:1524-1525`, `ARCHITECTURE.md:11`,
   `docs/USER-JOURNEYS.md:10`), so a contradiction between them has no
   adjudicator and no gate.
4. **A citation to an issue is never checked against the issue's state.** 34
   "tracked in #N" citations across the twelve documents name closed issues
   (D154), because nothing compares a citation to a committed issue-state
   snapshot.

## Tests first

*(Added by the 2026-09-16 reconciliation. The gates below are written and
observed failing before the leaves that make them pass; they are listed here
rather than only inside "Generation and validation" so this plan matches the
shape of plans 01-10 and 12.)*

| Test / gate | Written before | Asserts |
| --- | --- | --- |
| `scripts/check-requirement-status.rs` | L1 | every ID in `REQUIREMENTS.md` has a ledger entry and a generated traceability row; every ledger entry names an existing shard; a verdict of `implemented` names a test that exists on disk. Fails today on **225** IDs |
| `scripts/render-status.rs --check` | L2 | `docs/status.md` and the two pinned regions are byte-current against the seven ledger inputs. Fails today, because the file does not exist |
| `scripts/check-issue-citations.rs` | L4 | every "tracked in #N" names an issue open in `data/meta/issue-state.lino`. Fails today on **34** citations |
| `tests/unit/docs_requirements/benchmarks.rs` (widened) | L5 | any line containing a curated pass ratio contains an upstream ratio in the same paragraph, across five documents rather than two. Fails today on `ROADMAP.md:144` and `ARCHITECTURE.md:1351` |
| `scripts/check-language-parity.rs` (new) | **L75** | every meaning that declares a `lexeme` for one of en/ru/hi/zh/es declares one for all five, or carries an explicit dated `uncovered_behavior` row naming the gap. Fails today; the measured first value is the ceiling |
| `tests/unit/docs_requirements/count.rs` (new) | **L76** | `ls tests/unit \| grep -c '^docs_'` is at or below the ceiling recorded in `data/meta/debt-ratchet.lino`, strictly downward, target 5 (#1089). Fails today at 49 |
| `tests/unit/specification/status_render.rs` (new) | L2 | deleting `docs/status.md` and regenerating reproduces its content id — the forget-and-rediscover rule applied to the status surface itself |

## Method

Read in full: `VISION.md`, `GOALS.md`, `NON-GOALS.md`, `ROADMAP.md`,
`ARCHITECTURE.md`, `README.md`, `CONTRIBUTING.md`, `REQUIREMENTS.md`,
`docs/requirements-traceability.md`, `docs/benchmarks.md`,
`docs/meta-algorithm.md`, `docs/philosophy.md`, `docs/USER-JOURNEYS.md`,
`docs/architect-notes/*`, and the READMEs of `docs/case-studies/issue-710/plans/`
and `docs/case-studies/issue-1085/plans/`.

Ground truth, all read at `be8fd3174`:

| Measure | Value | Where it was read |
| --- | --- | --- |
| HumanEval upstream, first 20 | 20/20, 2026-09-15, v0.349.2 | `data/benchmarks/external-results.lino:862-872` |
| MBPP upstream, first 20 | 20/20 with `--online`, 2026-09-15, v0.349.2 | `data/benchmarks/external-results.lino:873-883` |
| GSM8K / MATH / object counting / CoEdIT | 2/20, 0/20, 0/20, 0/20, 2026-09-07, v0.347.0 | same ledger |
| SWE-bench Lite | 0/1, 2026-09-07 | `data/benchmarks/external-results.lino:851-861` |
| egg rewrite laws / Ascent closure | 20/20, 5/5, 2026-09-07 | same ledger |
| Latest release self-hosting share | 171 bp release, 389 bp trailing (v0.350.0) | last `release` block of `data/meta/self-hosting-ledger.lino` |
| Handler files under `src/solver_handlers/` | 36 (excluding `mod.rs`, `modules.rs`); ceiling 42 | `data/meta/debt-ratchet.lino` ceiling `handler_files` |
| Handler migrations pending | 40 (16 migrated) | `data/meta/handler-migration-ledger.lino`; `debt-ratchet.lino` `handler_migration_pending` |
| Literal-predicate ceiling | 549 | `data/meta/debt-ratchet.lino` ceiling `literal_predicates` |
| Hardcoded prose allowlist rows | 1,286 | `scripts/hardcoded-language-allowlist.txt`; `debt-ratchet.lino` ceiling `hardcoded_language_rows` |
| Outside-core boundary | 45 source files / 18,466 lines | `data/meta/core-boundary-ledger.lino:4-7` |
| `src/**/*.rs` | 541 files, 187,765 lines (185,639 excluding `src/web/`) | `find src -name '*.rs' -exec cat {} + \| wc -l` |
| `src/web/worker/*.js` | 27 files, 28,807 lines | same method |
| `src/web/wasm-worker/src/*.rs` | 2,126 lines | same method |
| Self-AST census vs source | 541 of 541 | `data/meta/self-ast/` vs `src/**/*.rs` |
| Agent-CLI ladder | 15 of 32 leaves, deepest passing level `none`, recorded 2026-09-08 | `data/meta/ladder-ratchet.lino` |
| `record_external_search` | appends `search:external`, then `policy:no_fetch_capability`, and returns; no retrieval, no `example.org` | `src/solver.rs:874-884` |
| `try_http_fetch_with_offline` | performs a real cached fetch through `CachedSourceClient` / `CurlSourceTransport` | `src/solver_handlers/web_requests.rs:39-98` |
| Occurrences of `example.org` in `src/` | 1, in a fact-checking denylist | `src/fact_checking.rs:591` |
| REQUIREMENTS shards | 111 | `ls docs/requirements/*.md \| wc -l` |
| Requirement IDs in `REQUIREMENTS.md` | 1,030 | `grep -oE '^\| (R[0-9]+[A-Za-z0-9-]*) \|' REQUIREMENTS.md \| sort -u` |
| Rows in `docs/requirements-traceability.md` | 805 | `grep -c '^\| R'` |
| IDs with **no** traceability row | **225** | `comm -23` of the two ID sets |
| Traceability `not yet confirmed` | 743 of 805 (92.3 %) | `grep -c 'not yet confirmed'` |
| Traceability `manually confirmed` | 19 | `grep -c 'manually confirmed'` |
| `docs_*` pin-test entries under `tests/unit/` | 49 files, 360 assertions | `ls tests/unit \| grep -c '^docs_'` |

Issue state was read once in bulk with
`gh api --paginate 'repos/link-assistant/formal-ai/issues?state=all'` and
cross-checked per number. 61 issues and 5 pull requests are open. Every
"tracked in #N" in the twelve documents was matched against that list; the
closed ones are findings below.

Categories: **stale number**, **contradiction**, **missing row**, **closed
tracker**, **unsupported claim**, **vision-requirements drift**, **dangling
reference**.

For every `REQUIREMENTS.md` finding the shard is named, because `REQUIREMENTS.md`
carries the banner "Generated by `rust-script scripts/assemble-requirements.rs
--write` from `docs/requirements/`. Edit the shard for your issue, never this
file" (`REQUIREMENTS.md:1`) and a hand edit is overwritten by the next
regeneration.

## Findings table

### VISION.md

| ID | file:line | Quoted statement | What is true now | Category | Proposed replacement text | Tests that pin it |
| --- | --- | --- | --- | --- | --- | --- |
| D1 | `VISION.md:265` | "the desktop wrapper is tracked by issue [#280]" | #280 closed 2026-05-26 by PR #289; the Electron shell ships in `desktop/` | closed tracker | "the desktop wrapper shipped in `desktop/` (issue #280, PR #289); packaging follow-ups live in ROADMAP pillar 16" | none |
| D2 | `VISION.md:373-375` | "with implementation tracked by issue [#702]" | #702 closed 2026-07-27 by PR #818; `src/world_model.rs` implements contexts and STRIPS-style actions (`ROADMAP.md:424`) | closed tracker | "implemented by `src/world_model.rs` (issue #702, PR #818), covered by `tests/unit/issue_649_world_model.rs`" | none |
| D3 | `VISION.md:377-386` | "The self-evolution frontier is explicit and benchmark-gated … (issues #656, #701); the share … is measured honestly from 0% upward (issue #657); the solver should try multiple candidate drafts … (issue #704)" | #656 closed 2026-07-16, #701 closed 2026-07-30, #657 closed 2026-07-17, #704 closed 2026-07-31; `src/draft_portfolio.rs` exists | closed tracker | restate each as a delivered mechanism with its closing PR, and name the one still-open item (#705) separately | none |
| D4 | `VISION.md:389-394` | "Language breadth grows by data alone through the meta language (issue #706), and computer-use tasks are handled as verified algorithmic plans … (issue #707)" | both closed 2026-07-31 (PRs #880, #882); `ROADMAP.md:429-430` already records them as Done | closed tracker | state as delivered; point at ROADMAP rows 429-430 | none |
| D5 | `VISION.md:341` | "progress is measured against an imported industry benchmark slice (HumanEval, MBPP, GSM8K, MATH, BIG-bench)" | that is the *curated* slice; the upstream harness (#698, `src/external_benchmarks/`) is the measurement that matters and is not named here | unsupported claim | append "and, since issue #698, against the unmodified upstream suites recorded in `data/benchmarks/external-results.lino`" | `tests/unit/docs_requirements/benchmarks.rs:57-60` |
| D6 | `VISION.md:343` | "computes the GSM8K (`18`), MATH (`11`), and BIG-bench object-counting (`3`) answers" | those are three curated cases; the upstream rows of the same suites are 2/20, 0/20 and 0/20 (stated later in the same paragraph) | unsupported claim | "computes the three curated GSM8K / MATH / object-counting cases; the upstream slices of the same suites score 2/20, 0/20 and 0/20" | `tests/unit/docs_requirements/benchmarks.rs:57-60` |
| D7 | `VISION.md:343` | "Issue [#1085] requires the upstream row to stand beside the curated one wherever it is cited." | #1085 closed 2026-09-10; the rule is now standing doctrine at `NON-GOALS.md:55`, and it is violated at `ROADMAP.md:144` and `ARCHITECTURE.md:1351` | closed tracker | "`NON-GOALS.md` makes this standing: a curated benchmark number is never cited without the upstream number beside it (originally issue #1085 D5.4)" | `tests/unit/docs_requirements/benchmarks.rs` |
| D8 | `VISION.md:326-328` | "Closing that gap is the current direction: the entire source is translated to links / meta language and back again (issue #558)" | #558 closed 2026-07-05 by PR #637; the live gap is that the lossless round-trip test is `#[ignore]`d (architect note `2026-09-11-lino-and-src-one-to-one.md:10-14`) | closed tracker | name the live gap instead of a closed issue: "the lossless round-trip exists in `self_source_links.rs` but is `#[ignore]`d, so the representation it proves is never committed" | `tests/unit/architect_notes.rs:46-68` pins the architect's clauses, not this sentence |
| D9 | `VISION.md:318-345` ("Current Direction") | the whole section | no paragraph for PR #888 / issue #710: dynamic coding discovery, `concept_discovery`, `data/meta/coding-discovery-recipe.lino`, or the fact that the universal loop still performs no retrieval (`src/solver.rs:874-884`) | missing row | add a dated paragraph stating what #888 delivered and the B1 limit plainly | `tests/unit/docs_requirements/benchmarks.rs:63-69` requires VISION to contain "run of &lt;latest ledger date&gt;" |
| D10 | `VISION.md:231` | "Treating the internet (Wikipedia, Wikidata, Wiktionary, Wikifunctions, Rosetta Code, public APIs) as a public database and the local doublets store as a cache for that database substitutes deterministic reasoning over reviewable links for opaque GPU-backed inference." | present tense for target behaviour: the universal loop's external-search step performs no retrieval (`src/solver.rs:874-884`); retrieval exists only inside specific handlers (how-to, coding catalogs, research learning) | vision-requirements drift | keep the thesis, add: "Today this holds inside the how-to, coding-discovery and research handlers. The universal loop's own external-search step records `policy:no_fetch_capability`; closing that is bottleneck B1 of issue #1138." | none |
| D11 | `VISION.md:171` | "Search local links first, then external sources if local data is insufficient." | same drift as D10 | vision-requirements drift | mark step 3 as target behaviour with the same one-line current-state note | none |
| D12 | `VISION.md:219` | "each verb phrase is mapped to a Wikidata **P-id** (property), each noun phrase to a Wikidata **Q-id** (item), with a fallback chain to Wikipedia article links and Wiktionary entries" | the only `UnknownConceptLookup` implementation in the tree is `NoLookup` (`src/coding/concept_discovery.rs:87`), and production `discover()` (`:189`) calls it; plan 01 row B8 records that nothing turns a requirement phrase into concepts by lookup | vision-requirements drift | state the chain as the target and name `NoLookup` as today's implementation | none |
| D13 | `VISION.md:304` | the seed inventory sentence ("multilingual responses, the concept table, the tool registry … and the **intent-routing rule book**") | omits `data/seed/sources-registry.lino`, the 13 trusted source kinds with licenses, APIs and cache paths that the how-to and coding-discovery paths consume | missing row | add the sources registry to the inventory | none |
| D14 | `VISION.md:41-123` ("The Goal Is The Meta Algorithm") | quotes the 2026-09-11 and 2026-09-12 notes | the 2026-09-14 maintainer instruction that PR #888 answers ("the goal is not to know everything in advance, the goal to know how to get know anything when it is needed") appears only in `docs/case-studies/issue-710/plans/README.md:14-36`, not in `VISION.md` or `docs/architect-notes/` | missing row | add `docs/architect-notes/2026-09-14-know-how-to-get-to-know-anything.md` and a VISION subsection quoting it | `tests/unit/architect_notes.rs:85-126` requires every note file to carry an ISO-date prefix and be listed in `docs/architect-notes/README.md` |

### GOALS.md

| ID | file:line | Quoted statement | What is true now | Category | Proposed replacement text | Tests that pin it |
| --- | --- | --- | --- | --- | --- | --- |
| D15 | `GOALS.md:109` | "Measure the share of each release authored by Formal AI itself, starting honestly at 0% and ratcheting upward." | the last ledger row (v0.350.0) records `percentage_basis_points "171"` and `trailing_percentage_basis_points "389"`, while `REQUIREMENTS.md:2397` still asserts the ledger "reads `0.00% self-authored`" | contradiction | keep the goal; make the *number* live only in the ledger and have every document cite it by reference | `tests/unit/specification/self_hosting_metric.rs` pins the metric, not the prose |
| D16 | `GOALS.md:96` | "Complete the self-coding chain: Formal AI codes itself via Agent CLI, directed by Hive Mind, with every change landing as a reviewed pull request." | `ROADMAP.md:437` calls the chain "Done for the recurring release loop" while `REQUIREMENTS.md:2397` says "**Not delivered by a `solve` run**" and `:2405` says "**Not achieved.**" | contradiction | GOALS keeps the target; ROADMAP must read "Partial: the release loop is enforced; a pull request opened by a real `solve` run (R1021-22) is not achieved" | `tests/unit/docs_requirements_issue_1021.rs:147-160` requires the R1021-14/22 traceability rows to keep reading "not delivered" / "not achieved" |
| D17 | `GOALS.md:112` | "a `.lino` per file under `src/`, one to one on every merged pull request, **from which the source can be reconstructed** (issue #558)" | 541-of-541 file correspondence holds, but architect note `2026-09-11-lino-and-src-one-to-one.md:10-14` records "Full representation does **not** [hold]: the committed census is a signature, not the source", and the round-trip test is `#[ignore]`d | unsupported claim | "one `.lino` per file holds today (541 of 541); the committed census is a signature, and the lossless round-trip that would make reconstruction real is `#[ignore]`d in `self_source_links.rs`" | none on this clause |
| D18 | `GOALS.md:29-30` | "Search local associative knowledge first and external sources only when local knowledge is insufficient." / "Cache external source access with provenance and refresh policy" | same B1 drift as D10 | vision-requirements drift | add the same one-line current-state note used in VISION | none |
| D19 | `GOALS.md:42` | "When no reusable part exists, combine reasoning, random search, and evolutionary search according to the available compute budget instead of giving up." | `src/solver_search.rs::try_budget_search` recognises arithmetic reachability only (plan 01 row A5: "done for one domain") | vision-requirements drift | append "; today implemented for arithmetic reachability (`src/solver_search.rs`), a target for every other domain" | `tests/unit/specification/budget_search_meta_algorithm.rs` |
| D20 | `GOALS.md:105-106` | "frontier detection … → candidate knowledge/rules with generated tests → benchmark-gated promotion as reviewed seed edits" | every loop is proposal-only (`docs/meta-algorithm.md:137-142`, `:760-768`); #1138 B7 records that nothing learned changes a later answer outside the single #701 opener class | vision-requirements drift | keep the goal; add "no loop currently closes without a human `--apply --confirm`; the #701 opener class is the only proven adoption" | none |
| D21 | `GOALS.md:129` | "including the failing tracked requirement tests under `tests/unit/specification/`, and graduate them out of `#[ignore]` as the implementation lands" | zero `#[ignore = "tracked requirement: …"]` tests remain (`ROADMAP.md:51-52`; verified by grep across `tests/unit/specification/`) | stale number | retire the clause, or replace it with the current `#[ignore]` inventory (network- and Docker-gated tests) | none |
| D22 | `GOALS.md` (whole file) | — | no goal states the 2026-09-14 doctrine: understand each unknown word by live lookup from trusted sources, then reconstruct the procedure from what was retrieved | missing row | add a Reasoning Goal for live concept lookup through `data/seed/sources-registry.lino`, bounded by evidence rather than by a budget | `tests/unit/architect_notes.rs:160-170` pins the "No task is rated hard, complex or ambitious before it is split" clause only |

### NON-GOALS.md

| ID | file:line | Quoted statement | What is true now | Category | Proposed replacement text | Tests that pin it |
| --- | --- | --- | --- | --- | --- | --- |
| D23 | `NON-GOALS.md:55` | "A curated benchmark number is never cited without the upstream number beside it." | violated at `ROADMAP.md:144` ("The suite now reports **13/13 passing**") and `ARCHITECTURE.md:1351` ("13 cases / 13-floor today"), while `REQUIREMENTS.md:2466` records R1085-10 as "Delivered" | contradiction | keep the rule; fix the two sites (D29, D55) and widen the pin test past `docs/benchmarks.md` and `VISION.md` | `tests/unit/docs_requirements/benchmarks.rs:31-61` covers only those two files |
| D24 | `NON-GOALS.md:18` | "Each `impulse:`, `search:local`, `search:external`, … link must correspond to a real recorded event." | `src/solver.rs:879` appends `search:external` for a search that is never executed; `:880-883` immediately appends `policy:no_fetch_capability` | vision-requirements drift | make the boundary explicit — "a `search:external` link with no retrieval must carry `policy:no_fetch_capability` in the same trace" — which the code already satisfies | `tests/unit/specification/source_cache.rs` |
| D25 | `NON-GOALS.md:7` | "The only sanctioned exception is the experimental, strictly opt-in small-model formalization fallback of issue [#483]" | #483 is open and its implementing PR #644 has been open and untouched since 2026-07-09 | unsupported claim | add "PR #644 carries the implementation and has been stale since 2026-07-09; the exception is declared, not delivered" | none |

### ROADMAP.md

| ID | file:line | Quoted statement | What is true now | Category | Proposed replacement text | Tests that pin it |
| --- | --- | --- | --- | --- | --- | --- |
| D26 | `ROADMAP.md:3` | "This file is the single source of truth for how much of `VISION.md` is actually built." | three other documents claim the same authority: `REQUIREMENTS.md:1524-1525` ("this table is the compact current requirement-status authority"), `ARCHITECTURE.md:11` ("the single source of truth for the design"), `docs/USER-JOURNEYS.md:10` ("the single place that enumerates every user journey we support today") | contradiction | one authority per dimension, stated in each header: ledgers own numbers, ROADMAP owns pillar status, REQUIREMENTS owns per-requirement status, traceability owns delivery + test + manual confirmation | none |
| D27 | `ROADMAP.md:492` | "industry (13/13; the upstream HumanEval/MBPP slices score 0/20, see `data/benchmarks/external-results.lino`)" | the ledger's latest rows are 20/20 and 20/20 (2026-09-15, v0.349.2) | stale number | "industry (13/13; the upstream HumanEval/MBPP slices scored 0/20 when #922 landed on 2026-08-14 and are 20/20 as of 2026-09-15)" | none — ROADMAP is **not** covered by `docs_requirements/benchmarks.rs`, which is why it went stale |
| D28 | `ROADMAP.md:570` | "upstream coding scores are 0/20 and flat" | 20/20 on both coding suites since 2026-09-15 | stale number | restate as the dated #1085 diagnosis: "were 0/20 and flat through 2026-09-07" | none |
| D29 | `ROADMAP.md:144` | "The suite now reports **13/13 passing** with a `minimum_pass_count` ratchet so progress cannot silently regress." | a curated number with no upstream number beside it; violates `NON-GOALS.md:55` | contradiction | append "; the upstream first-20 rows of the same suites are HumanEval 20/20 and MBPP 20/20 (2026-09-15)" | extend `tests/unit/docs_requirements/benchmarks.rs` to ROADMAP |
| D30 | `ROADMAP.md:137` | "`src/web/worker/*.js` still carries ~27,700 lines of mirrored solver logic" | 28,807 lines across 27 files | stale number | render from the shards under `data/meta/worker-line-budget/` instead of writing a literal | `scripts/check-worker-line-budget.rs` enforces per-module ceilings; nothing pins the prose |
| D31 | `ROADMAP.md:137` | "Absorbing the remaining worker logic into Rust→WASM is tracked by [#658] (R380)" | #658 closed 2026-07-18 by PR #691 | closed tracker | name the live mechanism: the 3,000-line end-state target in `scripts/check-worker-line-budget.rs:35` and the per-module shrink ratchet | none |
| D32 | `ROADMAP.md:436` | "Not done for PWA, npm engine, VS Code Marketplace, debugger, WebVM, cloud sync — [#665]-[#670]; shareable packages (#658) closed" | shareable packages is **#668 (E49)** and it is **open**; #658 is the WASM-absorption issue and is closed. The row names the wrong issue and asserts a closure that did not happen | contradiction | "Not done for PWA/npm (#665), Marketplace (#666), debugger (#667), shareable packages (#668), cloud sync (#669), WebVM (#670); all six remain open" | none |
| D33 | `ROADMAP.md:133` | "Trigger/response generalized by [#283] (PR #293). General substitution rules tracked separately as E24 [#301]." | #283 and #301 both closed 2026-05-26 | closed tracker | drop "tracked separately"; these are delivery citations | none |
| D34 | `ROADMAP.md:129` | "Native physical-store default is tracked separately in [#278]." | #278 closed 2026-05-26 by PR #285 | closed tracker | "delivered by #278 (PR #285)" | none |
| D35 | `ROADMAP.md:122` | "Future ranking improvements feed into [#279]." | #279 closed 2026-05-26 by PR #287 | closed tracker | remove, or re-file as a live issue | none |
| D36 | `ROADMAP.md:362` | "ratcheting the tree at 37 handler files / 48 `try_*` registry entries" | eighth-pass (2026-07-14) historical row; today 36 handler files, 40 pending ledger entries, 46 `try_` occurrences in `src/solver_handlers/mod.rs` | stale number | leave the historical row, but prefix the table with "As of 2026-07-14:" so no reader takes it as current — line 401 says so only *after* the table | none |
| D37 | `ROADMAP.md:421` | "#918 recursively classifies all 46 mixed handler sources as migration debt and ratchets their 19,543 outside-core lines" | this is the **current** (ninth-pass) table; `data/meta/core-boundary-ledger.lino:4-7` records 45 files / 18,466 lines, and only 36 files sit under `src/solver_handlers/` | stale number | render the pair from `core-boundary-ledger.lino` | `scripts/check-minimal-core-boundary.rs` enforces the ledger; nothing pins the prose |
| D38 | `ROADMAP.md:478-481` | "A recursive ledger now covers 46 recursive handler sources. Every current handler is mixed, so all 46 files and 19,543 outside-core lines remain explicit migration debt" | same as D37 | stale number | same | none |
| D39 | `ROADMAP.md:437` | "Self-coding chain \| Done for the recurring release loop: #673 census, #656 gated promotion, #657 release metric, and the self-development loop (delivered by #924) with a non-decreasing target" | contradicts `REQUIREMENTS.md:2397`, `:2405` and `docs/requirements-traceability.md:792` | contradiction | "Partial: release loop, census, gated promotion and metric are delivered; the chain's definition of done — a pull request opened by a real `solve` run (R1021-22) — is not achieved" | `tests/unit/docs_requirements_issue_1021.rs:140-160` |
| D40 | `ROADMAP.md:434` | "the last focused gaps, [#990] and [#991], are credited only after their current production-path regressions passed" | both closed (#990 2026-08-11 PR #994, #991 2026-08-14 PR #995); the wording reads as if they are still gaps | closed tracker | "the last focused gaps #990 and #991 closed on 2026-08-11 and 2026-08-14; their production regressions are pinned" | `tests/unit/docs_requirements_issue_710.rs:122` pins the exact substring "31 work now and 1 is superseded" in this row — a rewrite must keep that clause |
| D41 | `ROADMAP.md:425` | "the #848 coding ladder (2 of 13 rungs, zero write effects) exposed the new defect cluster [#902]-[#909]" | #848 closed 2026-08-02; `REQUIREMENTS.md:1738` measures **130** tasks; plan 01 row A13 and issue #1138 both record **65/130** at v0.320.0; #902 and #909 are closed | stale number | "the #848 coding ladder measured 65 of 130 tasks at v0.320.0 (`data/meta/ladder-ratchet.lino`, `experiments/issue_1028_agent_cli_ladder`)" | none |
| D42 | `ROADMAP.md:438` | "the #848 ladder passes 2 of 13 rungs with zero successful write effects; E69 is the blocker epic" | same stale number; E69 (#916) closed 2026-08-05 by PR #966 | stale number + closed tracker | restate with 65/130 and record that E69 closed | none |
| D43 | `ROADMAP.md:443-444` | "**Open planning batch E69-E77** ([#916]-[#924])." | every issue #916 … #924 is closed (2026-08-05 … 2026-08-18) | closed tracker | "Planning batch E69-E77 (#916-#924), all closed by merged PRs #966, #984, #986, #992, #1003, #1004, #1005, #1006, #1007" | none |
| D44 | `ROADMAP.md:564` | "## Issue #1085 The Links Network Is Not The System That Reasons (PR in progress)" | #1085 closed 2026-09-10; PR #1086 merged the same day | closed tracker | "(PR #1086, merged 2026-09-10)" | none |
| D45 | `ROADMAP.md:578` | "(ledger pending 51 to 40, handler files 43 to 42, literal predicates 529 to 504)" | `data/meta/debt-ratchet.lino` records `literal_predicates` value **549**, not 504; the other two match | stale number | render the triple from `debt-ratchet.lino` | `scripts/check-debt-ratchet.rs` enforces the ledger; nothing pins the prose |
| D46 | `ROADMAP.md:580` | "D5.4 upstream numbers beside every curated citation \| Delivered \| this file, `VISION.md`" | contradicted 436 lines above by `:144`, 88 lines below by `:492`, and by `ARCHITECTURE.md:1351` | contradiction | mark Delivered only once D29, D27 and D55 are fixed and the pin test covers all four documents | `tests/unit/docs_requirements/benchmarks.rs` |
| D47 | `ROADMAP.md` (no such section) | — | the file has no section for issue #710 / PR #888, although its own Verification Contract (`:591-594`) and every other closed issue get one, and #888 is the merge that produced today's 20/20 | missing row | add "## Issue #710 Dynamic Coding Discovery (PR #888, merged 2026-09-16)" with before/after upstream numbers and the honest open items from plans 06 and 07 | `tests/unit/docs_requirements_issue_710.rs:5` `include_str!`s `ROADMAP.md` |
| D48 | `ROADMAP.md:591-594` | "the PR should update the corresponding rows in `REQUIREMENTS.md`, the architecture status table, and this file" | the contract omits `docs/requirements-traceability.md`, which is the mechanical reason 225 requirement IDs have no traceability row | missing row | add the traceability table, and require the edit to land in the *shard* rather than in `REQUIREMENTS.md` | none |
| D49 | `ROADMAP.md:21` and `:217` | "With E1-E34 all merged, **no vision-planning epic remains open** for issue #244." | true only for #244; `ROADMAP.md:380-389` and `ARCHITECTURE.md:1379-1382` scope it explicitly, but the header statement at line 21 does not | contradiction | add "for issue #244 specifically" at both lines, matching `ARCHITECTURE.md:1379` | none |
| D50 | `ROADMAP.md:145` (pillar 26) | "The 2026-09-15 upstream rows are HumanEval 20/20 and MBPP 20/20 (empty source cache: 20/20 and 18/20); other latest rows remain GSM8K 2/20, MATH 0/20, CoEdIT 0/20, and SWE-bench Lite 0/1." | **correct and current** — this row is the model the rest of the file should follow. It omits only BIG-bench object counting 0/20 | stale number (minor) | add the object-counting row so the list matches the ledger's nine suites | `tests/unit/docs_requirements/benchmarks.rs` should be extended to cover it |

### ARCHITECTURE.md

| ID | file:line | Quoted statement | What is true now | Category | Proposed replacement text | Tests that pin it |
| --- | --- | --- | --- | --- | --- | --- |
| D51 | `ARCHITECTURE.md:1254` | "The WASM worker crate (`src/web/wasm-worker/src/`) is roughly 1,700 lines" | 2,126 lines | stale number | render from the tree, or drop the figure | `scripts/check-wasm-worker-size.rs` bounds the built artifact, not the source lines |
| D52 | `ARCHITECTURE.md:1255` | "`src/web/worker/*.js` still carries roughly 27,700 lines of solver logic" | 28,807 lines across 27 files | stale number | render from `data/meta/worker-line-budget/` | `scripts/check-worker-line-budget.rs` |
| D53 | `ARCHITECTURE.md:1256` | "mirroring the ~90,000-line Rust core" | 187,765 lines under `src/` (185,639 excluding `src/web/`) — more than double | stale number | either render the count or delete it: `VISION.md:64-66` says a Rust line count measures an output and "No release condition may be a Rust line count" | `tests/unit/architect_notes.rs:137-155` forbids any document from demanding that Rust shrink; a *count* is still allowed |
| D54 | `ARCHITECTURE.md:1262` | "is tracked by issue [#658] (R380)" | #658 closed 2026-07-18 | closed tracker | same replacement as D31 | none |
| D55 | `ARCHITECTURE.md:1351` | "the imported benchmark suite grew to a 10-case slice that passed **10/10** with a `minimum_pass_count` ratchet (13 cases / 13-floor today …)" | curated numbers with no upstream row beside them; violates `NON-GOALS.md:55` | contradiction | append the current upstream pair and a pointer to `docs/benchmarks.md` "Honest current numbers" | extend `tests/unit/docs_requirements/benchmarks.rs` |
| D56 | `ARCHITECTURE.md:1380-1388` | "two later batches are open: **E37-E55** ([#656]-[#674]) … **E56-E68** ([#698]-[#710])" | of #656-#674 only #665-#670 are open; of #698-#710 only #710 is open; #651 is the open parent | closed tracker | replace with the live open set (#651, #665-#670, #700, #705, #710) and add the batches opened since (E69-E77 all closed, E78-E117 mixed) | none |
| D57 | `ARCHITECTURE.md:1390-1399` | "The largest architectural gaps those batches own are: the #559 mandate to retire the specialized handlers (#663, #699), real upstream benchmark execution (#698), absorbing the JavaScript worker into WASM (#658), symbolic world-model behaviors (#702), and driving external agent CLIs as an orchestrator (#703)." | **all six issues are closed** (#663 2026-07-19, #699 2026-07-31, #698 2026-07-29, #658 2026-07-18, #702 2026-07-27, #703 2026-07-30) | closed tracker | replace with today's largest gaps, naming their live trackers: live concept lookup (B1, no tracker yet), handler-migration ratchet (#959), the gate/status collapse (#1089), the frontier queue (#1087), the traceability column (#1090) | none |
| D58 | `ARCHITECTURE.md:1327` | "## 16. Open Questions" | the section contains no open question; it is a 2026-05-29 audit narrative | contradiction | rename to "16. Audit history and current gaps", or restore real open questions | none |
| D59 | `ARCHITECTURE.md:148-158` | the step table, every row "Implemented", step 4 "Formalization \| `src/concepts.rs` plus `src/translation/formalization.rs` … \| Implemented" | plan 07 `:26-28` records "the live memory-contract formalizer preserves source sentences, but produces no concepts or procedures. Deep formalization therefore remains an open prerequisite itself" (#1138 B4) | unsupported claim | change step 4 to "Implemented for surface-form anchoring; concept and procedure extraction is open (issue #1138 B4)" and add a row for the coding-discovery path (`src/coding/`) | `tests/unit/docs_requirements_issue_540.rs:64-66` and `docs_requirements.rs:122-124` read `ARCHITECTURE.md` |
| D60 | `ARCHITECTURE.md:1433` | "requires every source to pass … for 1,440 of 1,440 passing checks" | a curated number with no upstream number beside it; the upstream analogue is CoEdIT 0/20 | contradiction | append "the upstream instructed-editing analogue, CoEdIT, scores 0/20" | none |
| D61 | `ARCHITECTURE.md:1462` | "`REQUIREMENTS.md` — issue-by-issue implementation matrix (R1 … R558, plus per-issue blocks such as R499-1…R499-8 and R914-1…R914-15)." | R558 is still the highest plain ID, but the per-issue examples stop at R914; the document now carries R1021-1…32, R1085-1…17, R1137-1…3, R710-01…32, R710-D1…D16 and R710-R1…R10 | stale number | update the examples, or render the ranges | none |
| D62 | `ARCHITECTURE.md:11` | "this document as the single source of truth for the design" | see D26 | contradiction | scope it to design, not status | none |

### README.md

| ID | file:line | Quoted statement | What is true now | Category | Proposed replacement text | Tests that pin it |
| --- | --- | --- | --- | --- | --- | --- |
| D63 | `README.md:1024-1037` ("Self-Development Share") | "How much of each release the formal-ai model authored is recorded in `data/meta/self-hosting-ledger.lino` … A figure of 0.00% is an honest figure." | no figure is published. `REQUIREMENTS.md:2463` (R1085-7) requires "publish the figure", and issue #1085 D3.4 required README to show the figure and its trend "even at 0.00 %". The latest row is 171 bp / 389 bp trailing | missing row | add the rendered current figure and trend (generated, not typed) | `tests/unit/specification/self_hosting_metric.rs` pins the metric only |
| D64 | `README.md` (whole file) | — | the README cites no benchmark number at all, curated or upstream. Issue #958 (open) asks precisely for the upstream scores to be reported | missing row | add a short "Measured today" block rendered from `data/benchmarks/external-results.lino` | extend `tests/unit/docs_requirements/benchmarks.rs` to README |
| D65 | `README.md:102-104` | "what is planned — including the open industry-benchmark coverage gap — is tracked in [ROADMAP.md]" | "the open industry-benchmark coverage gap" no longer names anything: the curated suite is 13/13, the coding upstream suites are 20/20, and the real gaps are GSM8K/MATH/object-counting/CoEdIT/SWE-bench | stale number | name the four floored suites and SWE-bench Lite explicitly | none |
| D66 | `README.md` (whole file) | — | zero mentions of `docs/requirements-traceability.md`, although `REQUIREMENTS.md:7-12` calls it a standing requirement that every row have one | missing row | add it to the documentation map at `:28` | none |
| D67 | `README.md:28` | "Where any other document contradicts it [VISION.md], that document is wrong and must be fixed." | this audit lists about a hundred such contradictions; no gate enforces the rule | unsupported claim | keep the rule and add the enforcing gate (see "Generation and validation") | `tests/unit/architect_notes.rs:73-79` only checks that README mentions `VISION.md` |

### CONTRIBUTING.md

| ID | file:line | Quoted statement | What is true now | Category | Proposed replacement text | Tests that pin it |
| --- | --- | --- | --- | --- | --- | --- |
| D68 | `CONTRIBUTING.md:966` | "a requirement \| write `docs/requirements/issue-NNNN-*.md` and run `rust-script scripts/assemble-requirements.rs --write` \| append a section to `REQUIREMENTS.md`" | the row never mentions `docs/requirements-traceability.md`. The file has **zero** occurrences of "traceab". This is the mechanical root cause of the 225 requirement IDs with no traceability row | missing row | "…and add one row per new ID to `docs/requirements-traceability.md` with delivery date, the automated test, and an honest `not yet confirmed`" | `tests/unit/docs_requirements_issue_1021.rs:58-67` enforces this for R1021-* only |
| D69 | `CONTRIBUTING.md:742` | "under the compiled-logic doctrine (REQUIREMENTS.md R536), prefer absorbing" | R536 (`REQUIREMENTS.md:2495`) cites the closed #658 and the stale ~27,700 figure | closed tracker | fix R536's shard first (D82), then this reference stands | none |
| D70 | `CONTRIBUTING.md:1085-1099` | the repository map | omits `docs/requirements-traceability.md`, `docs/benchmarks.md`, `docs/meta-algorithm.md` and `docs/architect-notes/` | missing row | add all four | `tests/unit/architect_notes.rs:73-79` checks only that CONTRIBUTING mentions `VISION.md` |
| D71 | `CONTRIBUTING.md` (whole file) | — | no rule tells a contributor that 49 `docs_*` test files with 360 prose assertions will fail if a document changes, nor which one covers which document | missing row | add a "Documents that are pinned by tests" table | none |

### REQUIREMENTS.md (each row names its shard)

| ID | file:line | Quoted statement | What is true now | Category | Proposed replacement text (apply to the shard) | Tests that pin it |
| --- | --- | --- | --- | --- | --- | --- |
| D72 | `REQUIREMENTS.md:114` (R67) — shard `docs/requirements/issue-0012-holistic-vision-requirements.md` | "Online mode currently synthesizes the provenance it reports: `record_external_search` … emits `source:http https://example.org/{hash-of-prompt} fetched_at=1970-01-01T00:00:00Z sha256={hash-of-prompt}`" | `record_external_search` (`src/solver.rs:874-884`) appends `search:external` and `policy:no_fetch_capability` and returns. No `example.org` exists in `src/` except a fact-checking denylist entry (`src/fact_checking.rs:591`). Issue #843 closed 2026-07-29 (PR #853) and removed the fabrication | stale number / unsupported claim | "Partially implemented: the evidence vocabulary exists and is honest. `record_external_search` (`src/solver.rs:874-884`) records `search:external` followed by `policy:no_fetch_capability` and performs no retrieval; the fabricated `example.org` provenance was removed by #843 (PR #853, 2026-07-29). Retrieval exists only inside specific handlers; giving the universal loop a real lookup is bottleneck B1 of #1138." | `docs/requirements-traceability.md:94` cites `tests/unit/specification/source_cache.rs` for R67 |
| D73 | `REQUIREMENTS.md:114` (R67) — same shard | "`record_external_search` (`src/solver.rs:886-900`)" | the function occupies `src/solver.rs:874-884` | stale number | update the span, or drop line numbers from requirement prose | none |
| D74 | `REQUIREMENTS.md:114` (R67) — same shard | "`try_http_fetch` (`src/solver_handlers/web_requests.rs:23-50`) resolves a curated static registry and otherwise returns prose" | `try_http_fetch_with_offline` (`src/solver_handlers/web_requests.rs:39-98`) tries the curated fetch, then performs a **real** cached HTTP fetch through `CachedSourceClient::new(cache_dir, CurlSourceTransport).with_online(true)` (`:64-67`), records the capture (`:85`) and returns up to 8,000 bytes of the body | stale number | rewrite to describe the live path and its offline branch (`:50-63`) | none |
| D75 | `REQUIREMENTS.md:114` (R67) — same shard | "Tracked by [#843]; conflicts with `NON-GOALS.md` ('Faking the evidence trail is not acceptable')." | #843 closed 2026-07-29; the conflict it names no longer exists | closed tracker | "Closed by #843 (PR #853). The remaining gap is retrieval in the universal loop, not honesty of the trail." | none |
| D76 | `REQUIREMENTS.md:1945` (R914-8) — shard `issue-0914-vision-implementation-planning-coding-first.md` | "Implemented as a growing general mechanism: E72/#919 supplies research-to-verified-procedure learning … PR #888 adds bounded Python documentation, Wikifunctions, Rosetta Code, and official OEIS adapters" | `docs/requirements-traceability.md:741` reads "not delivered — tracked in #873 \| none — tracked in #873 \| not yet confirmed", and #873 closed 2026-08-09 | contradiction + closed tracker | keep the REQUIREMENTS wording; rewrite the traceability row to "delivered 2026-09-15; PR #888 (issue #710) \| `tests/unit/coding_discovery/*` \| not yet confirmed" | `tests/unit/docs_requirements_issue_914.rs` |
| D77 | `REQUIREMENTS.md:1946` (R914-9) — same shard | "Implemented with measured boundaries: … the issue-1021 action ladder passes 16/16 … PR #888 … raises the first-20 HumanEval/MBPP upstream rows to 20/20" | `docs/requirements-traceability.md:742` reads "not delivered — tracked in #848"; #848 closed 2026-08-02 | contradiction + closed tracker | same shape as D76, citing the PR #888 discovery suites | `tests/unit/docs_requirements_issue_914.rs` |
| D78 | `docs/requirements-traceability.md:743` (R914-10) | "not delivered — tracked in #527" | #527 closed 2026-07-07 by PR #638; the live owner is E73 question-necessity, delivered by #920 (closed 2026-08-14, PR #1003) | closed tracker | "delivered 2026-08-14; PR #1003 (issue #920) \| `tests/unit/issue_920_question_necessity.rs` \| not yet confirmed" | none |
| D79 | `docs/requirements-traceability.md:738-739` (R914-5, R914-6) | "not delivered — untracked \| none — untracked" | R914-5 is owned by E70 (#917, closed) and E75 (#922, closed); R914-6 by #918 (closed) and #959 (open) — none is untracked | contradiction | name the owners | none |
| D80 | `docs/requirements-traceability.md:744` (R914-11) | "not delivered — untracked" | E74 (#921) closed 2026-08-18 by PR #1004; `ROADMAP.md:426` records the gate as Done | contradiction | "delivered 2026-08-18; PR #1004 (issue #921)" | none |
| D81 | `REQUIREMENTS.md:1943` (R914-6) — same shard | "43 specialized handlers remain migration debt" | 36 handler files under `src/solver_handlers/`; 40 pending ledger entries; ceiling 42 (`data/meta/debt-ratchet.lino`) | stale number | render from `debt-ratchet.lino` and `handler-migration-ledger.lino` | `scripts/check-debt-ratchet.rs` |
| D82 | `REQUIREMENTS.md:2495` (R536) — shard `doctrine-standing-doctrine-compiled-logic-interfacing-only-javascript-2026-08-04.md` | "`src/web/worker/*.js` still carries ~27,700 lines of mirrored solver logic … Full absorption … is tracked by [#658] (R380)" | 28,807 lines; #658 closed 2026-07-18 | stale number + closed tracker | render the line count; replace the tracker with the `check-worker-line-budget` 3,000-line target | `scripts/check-worker-line-budget.rs` |
| D83 | `REQUIREMENTS.md:2397` (R1021-14) — shard `issue-1021-full-range-coding-and-contribution-artifacts.md` | "**Not delivered by a `solve` run**; `data/meta/self-hosting-ledger.lino` still reads `0.00% self-authored`." | the "not delivered" verdict is correct; the *number* is stale — the last ledger row records 171 bp release and 389 bp trailing | stale number | "**Not delivered by a `solve` run.** The ledger's self-authored share is non-zero (Agent-CLI-authored leaves), but no qualifying run is a `solve` run." | `tests/unit/docs_requirements_issue_1021.rs:147-160` requires the traceability row to keep "not delivered"; the number is unpinned |
| D84 | `REQUIREMENTS.md:2465` (R1085-9) — shard `issue-1085-the-links-network-is-not-the-system-that-reasons.md` | "The seventeen leaves that fail are three mechanisms, measured and filed as #1095 and #1096" | three mechanisms are named but only two issues are listed, and both closed 2026-09-10 by PR #1086; `data/meta/ladder-ratchet.lino` still records `leaf_nodes_passing 15`, `deepest_passing_level none`, `recorded_on "2026-09-08"` | stale number + closed tracker | name the third mechanism and its issue, record that #1095/#1096 closed, and either re-measure the ladder or say the record predates their fixes | `data/meta/ladder-ratchet.lino` is compared by the ladder workflow, not by a unit test |
| D85 | `REQUIREMENTS.md:2466` (R1085-10) — same shard | "Cite the upstream benchmark row beside every curated 13/13 citation. \| Delivered: `VISION.md`, `ROADMAP.md` (three sites)." | `ROADMAP.md:144` and `ARCHITECTURE.md:1351` cite curated numbers with no upstream row; `ROADMAP.md:492` cites a *stale* upstream row | contradiction | downgrade to "Partial" until D29, D27 and D55 land and the pin test covers all four documents | `tests/unit/docs_requirements_issue_922.rs` is cited by the traceability row as "13/13 token retained" |
| D86 | `REQUIREMENTS.md:2470-2473` (R1085-14 … R1085-17) — same shard | "Sub-issue of #1085 (D6)." … "(D9)." | the four sub-issues are #1087, #1088, #1089 and #1090 and all four are open, but none is named in the row, and none of the four IDs has a traceability row | missing row | name the issue number in each row and add four traceability rows reading "not delivered — tracked in #1087/#1088/#1089/#1090" | `tests/unit/docs_requirements_issue_1021.rs:298-360` verifies that any test a traceability row cites exists |
| D87 | `REQUIREMENTS.md:1531-1562` (R710-01 … R710-32) — shard `issue-0710-dropped-requirements-re-verification.md` | 32 verdict rows | **none of the 32 has a traceability row** | missing row | add 32 rows; the verdict text already names the pinning suites | `tests/unit/docs_requirements_issue_710.rs:91-121` counts these rows in `REQUIREMENTS.md` only and never looks at the traceability table |
| D88 | `REQUIREMENTS.md:1600-1609` (R710-R1 … R710-R10) — shard `issue-0710-repository-and-retention-continuation.md` | ten rows, several honest "Partial"/"remains open" | **none of the ten has a traceability row** | missing row | add ten rows carrying the partial verdicts verbatim | none |
| D89 | `REQUIREMENTS.md:1602` (R710-R3) — same shard | "Partial: recipe replay resumes after a bound successful retry … Automatic prerequisite discovery/setup remains open" | correct and honest; but `ROADMAP.md` carries no row for it, so the roadmap reads as if #710 is fully re-verified | missing row | add the open items to the ROADMAP section proposed in D47 | none |
| D90 | `REQUIREMENTS.md:1607` (R710-R8) — same shard | "The open-ended refactor only read its input … Kotlin recovery, semantic formalization and open-ended regression authorship remain open." | correct; contradicted by `ROADMAP.md:437` "Self-coding chain \| Done for the recurring release loop" | contradiction | see D39 | none |
| D91 | `REQUIREMENTS.md:1608` (R710-R9) — same shard | "complete obligation-ledger execution and runtime verification feedback remain open" | correct; this is #1138 B5 | missing row | cite #1138 B5 as the live tracker once it is filed | none |
| D92 | `REQUIREMENTS.md:1588` (R710-D15) — shard `issue-0710-dynamic-coding-discovery.md` | "Published current benchmark numbers must be derived from the latest committed ledger rows … Implemented in `docs/benchmarks.md` and `VISION.md`; pinned by `docs_requirements::benchmarks::latest_external_rows_are_published_from_the_ledger`." | true for those two files; `ROADMAP.md:144`, `:492`, `:570` and `ARCHITECTURE.md:1351` publish numbers that are not derived from the ledger and went stale | contradiction | extend the requirement (and the test) to ROADMAP, ARCHITECTURE and README | `tests/unit/docs_requirements/benchmarks.rs:12-70` |
| D93 | `REQUIREMENTS.md:1997-1998` (R919-2, R919-3) — shard `issue-0919-research-driven-coding-procedures.md` | dated status prose gained by plan 01 §D | correct, but **none of R919-1 … R919-6 has a traceability row** | missing row | add six rows | none |
| D94 | `REQUIREMENTS.md:2034` (R922-3) — shard `issue-0922-method-learning-from-experience.md` | "cleared the 4/4 coding, 13/13 industry, and 12/12 unit floors" | curated numbers with no upstream row; `ROADMAP.md:492` describes the same run with a stale upstream parenthetical | contradiction | add the dated upstream pair, as `ROADMAP.md:145` does | none |
| D95 | `REQUIREMENTS.md` (222 further IDs) | — | 225 IDs in total have no traceability row, spread over 23 shards: R491-C1…C4 (4), R710-01…32 + R710-R1…R10 (42), R873-1…10 (10), R895-1…5 (5), R917-1…7 (7), R918-1…6 (6), R919-1…6 (6), R921-1…8 (8), R922-1…6 (6), R923-1…5 (5), R924-1…11 (11), R931-1…12 (12), R932-1…13 (13), R933-1…14 (14), R936-1…7 (7), R960-1…6 (6), R961-1…6 (6), R982-1…11 (11), R991-1…9 (9), R1012-1…9 (9), R1014-1…9 (9), R1017-1…12 (12), R1085-2/3/11/14/15/16/17 (7) | missing row | add all 225 rows; see the generation decision below for how to stop the gap reopening | `tests/unit/docs_requirements_issue_1021.rs:58-67` enforces the rule for R1021-* only; `docs_requirements_issue_909.rs:126` for R909-* |

### docs/requirements-traceability.md

| ID | file:line | Quoted statement | What is true now | Category | Proposed replacement text | Tests that pin it |
| --- | --- | --- | --- | --- | --- | --- |
| D96 | `docs/requirements-traceability.md:15-17` | "CI enforcement of this table's freshness is tracked by E105 (#957); this is the initial data population, not an automated generator wired into CI yet." | #957 is still open; there is still no generator (the only script that mentions the file is `scripts/assemble-requirements.rs:519-522`, and only to rewrite relative links); 225 IDs are missing | unsupported claim | keep the honesty note; add the current gap size ("225 of 1,030 requirement IDs have no row") so the number itself is the pressure | none |
| D97 | `docs/requirements-traceability.md:22-24` | "The `Line` column still records the REQUIREMENTS.md line each row was audited at in 2026-08-04 and is therefore historical, not live." | correct disclosure, but all 805 values are now wrong — e.g. the R1085-13 row records line 2342 while `REQUIREMENTS.md` carries R1085-13 at line 2469 | stale number | drop the column, or replace it with the shard file name, which is stable and is what an editor actually needs | none |
| D98 | `docs/requirements-traceability.md:11-12` | "`not delivered — tracked in #NNN` rows point at the batch #930-#962 trackers or pre-existing open issues" | of #930-#962, most are closed; the open ones are #930, #934, #935, #937, #939-#942, #948-#955, #957-#959 | unsupported claim | re-verify each of the 9 `not delivered` rows against the live issue list; three of them (D76, D77, D78) point at closed issues | none |
| D99 | `docs/requirements-traceability.md:94` (R67) | "`tests/unit/specification/source_cache.rs`" as R67's automated test | `REQUIREMENTS.md:114` states that that test's own header says "the implementation does not yet hit external sources" — the row claims a test pins a behaviour the test explicitly does not exercise | unsupported claim | "covers the evidence-link shape only; no test pins external retrieval, which does not exist in the universal loop" | `tests/unit/docs_requirements_issue_1021.rs:298-360` checks only that a cited test *exists* |
| D100 | `docs/requirements-traceability.md:376` (R314) | "manually confirmed 2026-08-04 (audit): … (see audit finding: falls back to the seeded fairy-tale KB rather than reflecting custom `--task`)" | still true (`docs/meta-algorithm.md:209-212` documents the canonical-synopsis fallback), but the row records a *defect* inside a "manually confirmed" cell | contradiction | split the finding out: confirmation and defect are different columns | none |
| D101 | `docs/requirements-traceability.md` (whole file) | 743 of 805 rows read "not yet confirmed" (92.3 %); 234 read "none recorded"; 19 are manually confirmed | issue #1085 D9 asked for "unconfirmed rows < 100, or the header carries the status" and recorded 716 of 776 at the time; the count has **risen** to 743 of 805 | stale number | either finish the column or mark it aspirational in the header, per #1090 (open) | none |
| D102 | `docs/requirements-traceability.md` (whole file) | — | no rows for R1137-1…3 or R1085-14…17, so the two most recent requirement blocks are invisible to the table | missing row | add them | none |
| D103 | `docs/requirements-traceability.md:830` (R1085-10) | "`tests/unit/docs_requirements_issue_922.rs` (13/13 token retained)" | that test pins a token in a *different* issue's documentation; it does not verify that an upstream row stands beside any curated citation | unsupported claim | cite `tests/unit/docs_requirements/benchmarks.rs::latest_external_rows_are_published_from_the_ledger` once it is widened | `tests/unit/docs_requirements_issue_1021.rs:298-360` |

### docs/benchmarks.md

| ID | file:line | Quoted statement | What is true now | Category | Proposed replacement text | Tests that pin it |
| --- | --- | --- | --- | --- | --- | --- |
| D104 | `docs/benchmarks.md:277-312` ("Honest current numbers") | the nine-row table plus the cold-cache control | **correct and current**, and derived from the ledger by test. This is the model every other status table should follow | — | none; cite it as the pattern | `tests/unit/docs_requirements/benchmarks.rs:12-70` |
| D105 | `docs/benchmarks.md:18` | "Permissive industry slice \| #304, #317 \| … \| `minimum_pass_count` 13" | a curated floor listed 241 lines before the upstream table; `NON-GOALS.md:55` asks for the upstream number *beside* it | contradiction (minor) | add a cross-reference column or a one-line pointer to "Honest current numbers" | none |
| D106 | `docs/benchmarks.md:24` | "External (upstream) harness \| #698, #923 \| … \| `external_benchmarks::recorded_upstream_pass_count_may_never_regress`" | the test exists at `tests/unit/specification/external_benchmarks.rs:267` | — | no change | that test |
| D107 | `docs/benchmarks.md:144-156` | the seven recorded `benchmark_limitation` rows (irrational roots, complex roots, degenerate, identity, units, named unknowns, `Find x:` misrouted) | **corrected 2026-09-16: `data/benchmarks/equation-type-corpus.lino:891-980` holds ten `benchmark_limitation` records, not seven, verified by `grep -c 'record_type benchmark_limitation'`. The document collapses the three named-unknown routing gaps (`What is x if …`, `Calculate x for …`, `Find x: …`) into one row and omits `malformed_expression`. Plan 08 §Current state and carry-over C63 both count ten.** The last of the three is a routing bug, not a math one: `solver_terminal::try_terminal_command` claims the prompt because `Find` is a shell command name | stale number | list all ten rows, and mark `named_unknown_colon_clause` as a routing defect rather than an upstream calculator limitation | `tests/unit/docs_requirements_issue_891.rs:60-62` — **which must be widened to assert the document's row count equals the corpus's, or it will pass on seven again** |
| D108 | `docs/benchmarks.md` (whole file) | — | never states which test pins "Honest current numbers", so a reader who edits the table does not know what will fail | missing row | name `docs_requirements::benchmarks::latest_external_rows_are_published_from_the_ledger` in the section | itself |

### docs/meta-algorithm.md

| ID | file:line | Quoted statement | What is true now | Category | Proposed replacement text | Tests that pin it |
| --- | --- | --- | --- | --- | --- | --- |
| D109 | `docs/meta-algorithm.md:14-17` | "Fourteen recipes are grounded today. The **recursive core** (issue #559) is the general algorithm every prompt walks; the other **twelve** encode a topic handler…" | the table at `:21-34` has 14 rows; 14 minus the recursive core is **thirteen** | stale number | "the other thirteen" — or derive the count from `data/meta/*-recipe.lino` | `tests/unit/specification/meta_algorithm.rs:270-280` reads this file |
| D110 | `docs/meta-algorithm.md:219-224` | "1. **Recognise the agentic task** … against a small closed keyword set … 2. **Pin the canonical plan as named constants** (`SEARCH_QUERY`, `CANONICAL_SOURCE_URL`, `KB_PATH`) so the recipe is data, not scattered literals." | these are the hard-coded constants issue #1138 B4 names; `docs/requirements-traceability.md:376` records that the loop "falls back to the seeded fairy-tale KB rather than reflecting custom `--task`" | unsupported claim | keep the description, add: "These three constants are the loop's current limit: a task outside the closed keyword set yields `None`, and a custom `--task` still reaches the seeded corpus rather than its own sources (issue #1138 B4)." | `tests/unit/specification/agentic_meta_algorithm.rs:328-337` asserts this file contains specific needles |
| D111 | `docs/meta-algorithm.md:209-212` | "the formalizer falls back to the canonical synopsis so the loop still completes with a stable, all-nine-primitive knowledge base" | presented as robustness; in practice it is the fairy-tale-KB fallback of D110 | unsupported claim | say what the fallback costs, not only that it completes | same as D110 |
| D112 | `docs/meta-algorithm.md:137-142` | "gated `off` by default, never writing the recipe back, so adoption stays a human review step (R340)" | accurate; but nothing in the document records that no learning loop has yet changed a later answer outside the single #701 class (#1138 B7) | missing row | add a "What none of these loops does yet" paragraph | none |
| D113 | `docs/meta-algorithm.md:874-901` (coding-discovery section) | "2. **Discover** — `concept_discovery::discover` maps each requirement to seeded structural meanings and licensed parts from Python documentation, Wikifunctions, or a previously verified procedure." | the only `UnknownConceptLookup` implementation is `NoLookup` (`src/coding/concept_discovery.rs:87`) and `discover()` (`:189`) calls it, so a word outside the 82 seeded structural meanings cannot be resolved | vision-requirements drift | add "A word outside `data/seed/meanings-coding-structure.lino` reaches `NoLookup` and is recorded as a blocked need; live lookup is bottleneck B1 of issue #1138." | `tests/unit/specification/coding_discovery_meta_algorithm.rs` |
| D114 | `docs/meta-algorithm.md:36-38` | "The other `data/meta/*.lino` files are catalogues, lexicons, and ledgers (cue sets, route/method aliases, repair cases, the self-AST census, …)" | omits the four ratchets a reader needs — `debt-ratchet.lino`, `core-boundary-ledger.lino`, `ladder-ratchet.lino`, `self-hosting-ledger.lino` — which are where the project's live numbers actually are | missing row | name the four ledgers and say they are the numeric authority | none |

### docs/philosophy.md

| ID | file:line | Quoted statement | What is true now | Category | Proposed replacement text | Tests that pin it |
| --- | --- | --- | --- | --- | --- | --- |
| D115 | `docs/philosophy.md:99-107` | "Implemented today: link-native storage … Still a direction: representing every algorithm uniformly as substitutable link networks, generally synthesizing those networks from arbitrary requirements" | **accurate and the most honest boundary statement in the repository.** `VISION.md:5` asserts "The associative network is the AI" with no such qualification, and reaches the honest version only at `:318-335` | contradiction (by omission) | link this boundary from `VISION.md:5`, not only from `:7-11` | `tests/issue_885_docs.rs:132`, `:150-151` require README and VISION to link `docs/philosophy.md` |
| D116 | `docs/philosophy.md` (whole file) | — | no entry for the 2026-09-14 doctrine ("the goal is not to know everything in advance, the goal to know how to get know anything when it is needed"), which is the strongest present claim about what learning means here | missing row | add a "Knowing how to get to know" section beside "Learning is controlled self-modification" | `tests/issue_885_docs.rs:132` |

### docs/USER-JOURNEYS.md

| ID | file:line | Quoted statement | What is true now | Category | Proposed replacement text | Tests that pin it |
| --- | --- | --- | --- | --- | --- | --- |
| D117 | `docs/USER-JOURNEYS.md:370-372` | future journey F4, "tracked by [#662] … and parallel [#704]" | both closed (#662 2026-07-19 PR #695, #704 2026-07-31 PR #878) and `ROADMAP.md:427` records F4 as "Done for #662/#704" — yet the journey is still listed under "Potential future" | contradiction | move F4 to "Currently Supported Journeys" with `src/draft_portfolio.rs` and `src/solver_search.rs` as evidence, or state the residual gap | none |
| D118 | `docs/USER-JOURNEYS.md:317` | "Supported today for freely phrased procedures through [#674]" | #674 closed 2026-07-28 by PR #815 | closed tracker (benign) | "delivered by #674 (PR #815)" | none |
| D119 | `docs/USER-JOURNEYS.md:10` | "It is the single place that enumerates **every user journey we support today**" | see D26; and there is no journey for coding-by-discovery — the capability PR #888 delivered — nor for agentic self-coding | contradiction + missing row | add a journey for "ask for a program whose method the assistant does not know" and one for "Formal AI authors a change in its own repository" | none |
| D120 | `docs/USER-JOURNEYS.md:406-407` | "● = supported today, ○ = potential future on that surface" | the matrix has no row for the coding-discovery journey (D119) or for benchmark measurement | missing row | extend the matrix with the new journeys | none |

### docs/architect-notes/

| ID | file:line | Quoted statement | What is true now | Category | Proposed replacement text | Tests that pin it |
| --- | --- | --- | --- | --- | --- | --- |
| D121 | `docs/architect-notes/2026-09-11-lino-and-src-one-to-one.md:10` | "1-to-1 file correspondence holds (516 of 516)." | 541 of 541 today | stale number (historical) | keep the number; the file already heads it "## State when this was written", so add the same heading discipline to every note and a line in `README.md` saying note bodies are frozen at their date | `tests/unit/architect_notes.rs:85-126` |
| D122 | `docs/architect-notes/README.md:23-32` | the eight-row notes index | the 2026-09-14 instruction (quoted at `docs/case-studies/issue-710/plans/README.md:14-36`) and the 2026-09-15 continuation (quoted at `REQUIREMENTS.md:1593-1596`) are not recorded as notes, although `README.md:11-19` calls this folder the chronological record and `VISION.md:126-130` says VISION is kept up to date from it | missing row | add two note files and index rows | `tests/unit/architect_notes.rs:85-126` requires an ISO-date prefix and an index entry for every note |
| D123 | `docs/architect-notes/README.md:16-19` | "Where a document, gate, requirement or plan contradicts the latest note, the document is wrong and must be fixed." | the same unenforced rule as D67 | unsupported claim | keep the rule; add the enforcing gate | `tests/unit/architect_notes.rs:137-155` enforces only two specific forbidden phrases |

### docs/case-studies/issue-710/plans/

| ID | file:line | Quoted statement | What is true now | Category | Proposed replacement text | Tests that pin it |
| --- | --- | --- | --- | --- | --- | --- |
| D124 | `issue-710/plans/README.md:47-50` | "Current work is [Plan 06] … Resume Plan 06's unchecked leaves" | PR #888 merged as `be8fd3174` on 2026-09-16; #710 is still open, so the work continues, but not on that branch | contradiction | "PR #888 merged on 2026-09-16 as `be8fd3174`. Issue #710 remains open; plans 06 and 07 carry its unfinished leaves and are now worked on `main` (see issue #1138 B3 and B6)." | none |
| D125 | `issue-710/plans/README.md:52-73` | "The PR branch is checked out in a worktree of the main clone. `cd /tmp/wt888` # branch merge-888-into-main → origin/issue-710-14da90b08a12" | that branch is merged; the resume recipe cannot be followed | dangling reference | replace with a `main`-based recipe | none |
| D126 | `issue-710/plans/README.md:83`, `:86`, `:87` | status cells "CI confirmation pending", "local gates green; CI confirmation pending", "final-head CI observation pending" | PR #888 merged with CI green | stale number | change all three to "merged in PR #888 (2026-09-16)" | none |
| D127 | `issue-710/plans/README.md:186` | "hardcoded-language debt fell from 1,278 to 1,274 entries" | `scripts/hardcoded-language-allowlist.txt` has 1,286 rows and `data/meta/debt-ratchet.lino` records the ceiling at 1,286 — the debt **rose by 12** after this dated entry | stale number (append-only log) | leave the dated entry; add a closing log line recording the merge-time value | `scripts/check-debt-ratchet.rs` |
| D128 | `issue-710/plans/README.md:187` | "the reviewed outside-core ceiling fell from 18,854 to 18,469 lines" | `data/meta/core-boundary-ledger.lino:6-7` records 18,466 | stale number | same treatment as D127 | `scripts/check-minimal-core-boundary.rs` |
| D129 | `issue-710/plans/README.md:232` | "The all-features unit target accounted for 3,501 tests" | plan 07 `:379` records 3,542 for the same target after later commits | stale number (append-only log) | leave both; the log is dated | none |
| D130 | `issue-710/plans/01-…md:14` | "`REQUIREMENTS.md` (assembled from 106 shards in `docs/requirements/`)" | 111 shards today; plan 07 `:404` says 110 | stale number | leave the dated audit input; add the current figure to the closing log | `scripts/assemble-requirements.rs` |
| D131 | `issue-710/plans/01-…md:61` (row B9) | "\| **partial** \| `composition` builds and ranks candidate trees from seed-declared structural idioms/runtime templates" | `issue-710/plans/03-…md:212` (L7) records "[x] implemented; the three previously memorized tasks pass by derivation" — the two plans give different verdicts for the same work | contradiction | reconcile to "partial by design: composition is implemented over *seeded* idioms; composing from retrieved parts is bottleneck B2 of #1138" | none |
| D132 | `issue-710/plans/01-…md:47` (row A16) | "done; the quoted numbers are those of 2026-09-07 and go stale with each run (C4)" | the warning came true: `ROADMAP.md:492` and `:570` still carry the 2026-09-07 zeros | contradiction | mark A16 partial and point at D27/D28 | none |
| D133 | `issue-710/plans/01-…md:58` (row B6) | "`src/knowledge.rs` embeds 25 reviewed snippets (Rosetta/Wikifunctions/Hello World/SO) as a static oracle" with final column "`function_catalog::{wikifunctions,rosetta_code}` now use `CachedSourceClient`" | the final column does not say whether the 25 static snippets were removed; plan 02 §2 calls them "a cache with no way to be refilled" and #1138 B2 records them as still present | unsupported claim | state explicitly that the 25 snippets remain and are B2's target | none |
| D134 | `issue-710/plans/01-…md:83` (row C6) | "The frontier user prompts #720, #721, #722, #724, #869, #1063 (#1087 E109)" | all six are open (correct), but `REQUIREMENTS.md:2470` (R1085-14) includes **#447** in the same frontier and this row omits it | missing row | add #447 | none |
| D135 | `issue-710/plans/03-…md:34` (L0.4) and `:362` (L16) | "- [ ] CI green on the pushed head (all workflows)" / "- [ ] one CI wait; every workflow green; then stop" | PR #888 merged; both are satisfied | stale number | tick both with the merge SHA | none |
| D136 | `issue-710/plans/04-…md:393` | "making the current tally 31/0/1/0" | still true for R710-01…32, but plans 06 and 07 subsequently opened R710-R1…R10 with several "Partial"/"remains open" verdicts, and the tally was never amended to say the audit grew | contradiction | "R710-01…32: 31 works-now / 0 still-broken / 1 superseded / 0 blocked-upstream. The 2026-09-15 continuation adds R710-R1…R10, of which R710-R3, R710-R8 and R710-R9 are explicitly partial." | `tests/unit/docs_requirements_issue_710.rs:103-121` counts exactly 32 R710-NN rows with 31 `works-now` — the amendment must not touch those |
| D137 | `issue-710/plans/04-…md:452-454` (L26) and `:513` (L27) | four unchecked boxes ("normal fast-forward delivery after every CI-derived repair", "every required and informational PR workflow finishes without failure", "remote head equals local HEAD; PR is mergeable", "observe every workflow again on the resulting exact head") | PR #888 merged | stale number | tick with the merge SHA | none |
| D138 | `issue-710/plans/04-…md:460-461` | "The strict local metric passed after the append-only retraction: 0.15%, with 58 of 38,373 behavior-changing lines attributed" | this is the *PR-range* measurement; the *release* ledger's v0.350.0 row reads 171 bp release / 389 bp trailing. Both are true and neither says which it is | contradiction (by ambiguity) | label the measurement "over `origin/main..HEAD` for PR #888" so it cannot be read as the release share | `tests/unit/specification/self_hosting_metric.rs` |
| D139 | `issue-710/plans/06-…md:35` | "PR #888 baseline: 77 checks, 68 success, 9 intended skips; mergeable and clean." | the PR merged; the baseline commit `2f7a381a2` is history | stale number | date the sentence | none |
| D140 | `issue-710/plans/06-…md` and `07-…md` headers | "Status: active" | correct — these are the live open work — but no issue tracks them; #710 is open and #1138 B3/B6 restate them | missing row | name #710 and #1138 as their trackers in both headers | none |
| D141 | `issue-710/plans/07-…md:404` | "requirements aggregate has 110 shards" | 111 today | stale number | date it | `scripts/assemble-requirements.rs` |
| D142 | `issue-710/plans/07-…md:351-352` | "unchanged at 1,286, and the minimal-core boundary passes at 45 handler sources / 18,466 outside-core lines" | **matches the ledgers exactly** — this is the most current numeric statement in the plans and contradicts `ROADMAP.md:421`/`:478` (46 / 19,543) | contradiction | no change here; fix ROADMAP (D37, D38) | `scripts/check-minimal-core-boundary.rs` |

### docs/case-studies/issue-1085/plans/

| ID | file:line | Quoted statement | What is true now | Category | Proposed replacement text | Tests that pin it |
| --- | --- | --- | --- | --- | --- | --- |
| D143 | `issue-1085/plans/README.md:41` | "#959 (E107) handler ledger ratchet \| already ratcheted by `kernel-ratchet.lino`; the seed migration is E108's D1" | `data/meta/kernel-ratchet.lino` **does not exist**; neither does `scripts/check-kernel-ratchet.rs`. The ratchet was renamed on 2026-09-12 to `data/meta/debt-ratchet.lino` + `scripts/check-debt-ratchet.rs` (`data/meta/debt-ratchet.lino:5` note) | dangling reference | "already ratcheted by `data/meta/debt-ratchet.lino` (renamed from `kernel-ratchet.lino` on 2026-09-12)" | `scripts/check-debt-ratchet.rs` |
| D144 | `issue-1085/plans/06-on-topic-issue-triage.md:38` | "the ratchet landed (`kernel-ratchet.lino`)" | same dangling reference | dangling reference | same | none |
| D145 | `docs/case-studies/issue-1085/solution-plan.md:20` | "**D1.4.** `kernel-ratchet.lino` and `check-kernel-ratchet.rs`, registered as…" | same | dangling reference | add a withdrawal note: D1.4 was withdrawn with R1085-1 (`REQUIREMENTS.md:2457`) | `tests/unit/architect_notes.rs:175-201` requires R1085-1 to say "superseded" and "kernel" in both `REQUIREMENTS.md` and its shard |
| D146 | `docs/case-studies/issue-1085/README.md:56-57` | "`rust-script --test scripts/check-kernel-ratchet.rs && rust-script scripts/check-kernel-ratchet.rs` — the five ceilings hold at the values…" | the script is gone and there are now **four** ceilings, not five | dangling reference + stale number | replace with the `check-debt-ratchet.rs` command and four ceilings | `scripts/check-debt-ratchet.rs` |
| D147 | `data/meta/handler-migration-ledger.lino:4` | "the measured, lower-only ceilings now live in `data/meta/kernel-ratchet.lino`" | same dangling reference, in a **data file** that the ratchet script itself reads | dangling reference | point at `data/meta/debt-ratchet.lino` | `scripts/check-debt-ratchet.rs:42` reads this file |
| D148 | `issue-1085/plans/README.md:40` | "#1090 (E112) traceability column \| 716 rows of manual confirmation" | 743 of 805 today | stale number | render from the table | none |
| D149 | `issue-1085/plans/README.md:39` | "#1089 (E111) collapse the gates \| a 48 -> 5 test-file consolidation" | 49 `docs_*` entries under `tests/unit/` today (48 top-level `.rs` plus `docs_requirements/benchmarks.rs`) — the count **rose** | stale number | render from the tree; note the direction | none |
| D150 | `issue-1085/plans/README.md:1` | "# Plans for the remaining #1085 work (PR #1086)" | #1085 closed 2026-09-10 and PR #1086 merged the same day; the README has no closing log and the "Order of work" (`:44-51`) reads as pending | stale number | add a closing log line and mark the order complete | none |
| D151 | `issue-1085/plans/README.md:33-42` | "Sub-issues of #1085 that stay open, with the reason" | accurate: #1087, #1088, #1089, #1090, #959, #1101 — of these #1101 closed 2026-09-10 by PR #1086 | closed tracker | remove #1101 from the stay-open table | none |

### Cross-cutting

| ID | file:line | Quoted statement | What is true now | Category | Proposed replacement text | Tests that pin it |
| --- | --- | --- | --- | --- | --- | --- |
| D152 | `issue-710/plans/01-…md:32`, `:53`, `:61`, `:78`, `:79` | "(#324-3, #331-6, #440)", "(R395-1)", "(R395-6)", "(R412-2, deferred)", "(R423-2, deferred)", "(R8-1/2/4…)", "(R331-5…)" | eleven requirement IDs cited in the plans — **R1-4, R1-5, R8-1, R324-3, R331-5, R331-6, R395-1, R395-6, R412-2, R423-1, R423-2** — do not exist in `REQUIREMENTS.md`. They come from the issue #957 audit NDJSON namespace (`docs/case-studies/issue-957/raw-data/req-chunk-*.ndjson`), which uses `R<issue>-<index>` | dangling reference | either prefix them (`#957-audit R395-1`) or promote them into real shards; a reader today cannot look any of them up | none |
| D153 | repository-wide | — | there is **no** gate asserting that every `REQUIREMENTS.md` ID has a traceability row. The only enforcement is per-issue: `tests/unit/docs_requirements_issue_1021.rs:58-67` (R1021-*), `docs_requirements_issue_909.rs:126` (R909-*), `docs_requirements_issue_891.rs:28-30`, `docs_requirements_issue_893.rs:29-32`, `docs_requirements_issue_894.rs:227-229` | missing row | one global gate (see below) | — |
| D154 | repository-wide | — | there is no gate asserting that a "tracked in #N" / "tracked by #N" citation names an **open** issue. 34 such citations across the twelve documents name closed issues (D1-D4, D8, D31, D33-D35, D40, D42-D44, D54, D57, D69, D74, D76-D78, D82, D84, D86, D118, D151) | missing row | a `scripts/check-issue-citations.rs` gate over a committed issue-state snapshot | — |
| D155 | repository-wide | — | numbers live in prose in six documents and in ledgers in `data/meta/`. Every stale-number finding above (D27, D28, D30, D36-D38, D41, D42, D45, D51-D53, D63, D81-D84, D101, D127-D130, D141, D148, D149) is the same defect: a number copied beside the thing it counts. `tests/unit/docs_requirements_issue_1021.rs:13-21` already states the principle — "A count copied beside the thing it counts is a count that drifts" — and applies it in exactly one place | contradiction | make the principle general (see below) | `tests/unit/docs_requirements_issue_1021.rs:22-43` |

## Issue #1138 plan doc replacements (D156-D275)

Every "Docs to update" entry of plans 01-10 and 12, moved here verbatim on
2026-09-16 so that one document owns the docs audit and no statement is
described in two places (plan 00 §8). Each plan now carries a pointer to its
row ids instead of a body.

These rows differ from D1-D155 in one way that matters: D1-D155 are statements
that are **wrong today**, found by reading the documents. D156-D275 are
statements that **become wrong when a plan lands**, found by reading the plans.
Both are applied by this plan's leaves, and both are subject to plan 00 §6.4 —
a replacement may not state a number before the run that produces it.

Where a row's target already has a D1-D155 finding, the older row is the
authority on what is wrong now and the newer row is the authority on what it
becomes; the two are applied in one commit. The overlaps are: D5/D6/D9 with
D156 (VISION's benchmark sentences), D27/D28/D29 with the ROADMAP rows, D104 with
the `docs/benchmarks.md` rows, and D109-D114 with the `docs/meta-algorithm.md`
rows.

### Index

| row | plan | document |
| --- | --- | --- |
| D156 | plan 01 | `docs/meta-algorithm.md` |
| D157 | plan 01 | `VISION.md` |
| D158 | plan 01 | `ROADMAP.md` |
| D159 | plan 01 | `docs/requirements/issue-1138-live-concept-lookup.md` |
| D160 | plan 02 | `VISION.md:343` |
| D161 | plan 02 | `ROADMAP.md:145` |
| D162 | plan 02 | `ROADMAP.md:492` |
| D163 | plan 02 | `docs/benchmarks.md:284-295` |
| D164 | plan 02 | `docs/benchmarks.md:348-368` |
| D165 | plan 02 | `docs/benchmarks.md:16-33` |
| D166 | plan 02 | `docs/requirements/issue-0710-dynamic-coding-discovery.md` |
| D167 | plan 02 | New shard `docs/requirements/issue-1138-composition-from-sources.md` |
| D168 | plan 02 | `docs/requirements-traceability.md` |
| D169 | plan 02 | `docs/meta-algorithm.md` |
| D170 | plan 03 | `docs/benchmarks.md:311` |
| D171 | plan 03 | `docs/benchmarks.md:314-320` |
| D172 | plan 03 | `docs/meta-algorithm.md:185-262` |
| D173 | plan 03 | `ROADMAP.md:145` |
| D174 | plan 03 | `VISION.md:343` |
| D175 | plan 03 | `GOALS.md:96` |
| D176 | plan 03 | `GOALS.md:111` |
| D177 | plan 03 | `docs/requirements/issue-1085-the-links-network-is-not-the-system-that-reasons.md` |
| D178 | plan 03 | `docs/requirements/issue-1021-full-range-coding-and-contribution-artifacts.md` |
| D179 | plan 03 | New shard `docs/requirements/issue-1138-repository-workspace-protocol.md` |
| D180 | plan 03 | `docs/requirements-traceability.md` |
| D181 | plan 04 | `docs/meta-algorithm.md` |
| D182 | plan 04 | `VISION.md` |
| D183 | plan 04 | `ROADMAP.md` |
| D184 | plan 04 | `docs/requirements/issue-1138-formalization-depth.md` |
| D185 | plan 05 | `docs/requirements/issue-0559-general-meta-algorithm.md:18-26` |
| D186 | plan 05 | `docs/requirements/issue-0710-repository-and-retention-continuation.md:18` |
| D187 | plan 05 | `docs/requirements/issue-0710-repository-and-retention-continuation.md:13` |
| D188 | plan 05 | `docs/meta-algorithm.md:144` |
| D189 | plan 05 | `docs/meta-algorithm.md:172-173` |
| D190 | plan 05 | `docs/meta-algorithm.md:152-153` |
| D191 | plan 05 | `data/meta/recursive-core-recipe.lino:5` |
| D192 | plan 05 | `data/meta/recursive-core-recipe.lino:6` |
| D193 | plan 05 | `data/meta/recursive-core-recipe.lino:35` |
| D194 | plan 05 | `docs/requirements-traceability.md:406` |
| D195 | plan 05 | New shard `docs/requirements/issue-1138-bottleneck-audit.md` |
| D196 | plan 05 | `VISION.md:192` |
| D197 | plan 05 | `ROADMAP.md` |
| D198 | plan 06 | `VISION.md:163` |
| D199 | plan 06 | `VISION.md:269` |
| D200 | plan 06 | `ROADMAP.md:126` |
| D201 | plan 06 | `GOALS.md:96` |
| D202 | plan 06 | `GOALS.md`, Self-Evolution list |
| D203 | plan 06 | `docs/meta-algorithm.md:185-262` |
| D204 | plan 06 | `docs/benchmarks.md:356-358` |
| D205 | plan 06 | `docs/requirements/issue-0008-telegram-bot-requirements.md` |
| D206 | plan 06 | `docs/requirements/issue-0195-docker-in-docker-telegram-runtime.md` |
| D207 | plan 06 | New shard `docs/requirements/issue-1138-prerequisite-discovery.md` |
| D208 | plan 06 | `docs/requirements-traceability.md` |
| D209 | plan 06 | `docs/case-studies/issue-710/plans/07-prerequisite-discovery-bridge.md:65-81` |
| D210 | plan 07 | `docs/meta-algorithm.md:772-775` |
| D211 | plan 07 | `docs/meta-algorithm.md:761-764` |
| D212 | plan 07 | `docs/meta-algorithm.md:671-678` |
| D213 | plan 07 | `docs/meta-algorithm.md:137-142` |
| D214 | plan 07 | `docs/requirements/issue-0922-method-learning-from-experience.md:13` |
| D215 | plan 07 | `docs/requirements/issue-0922-method-learning-from-experience.md:16` |
| D216 | plan 07 | `docs/requirements/issue-0559-general-meta-algorithm.md:39` |
| D217 | plan 07 | `docs/requirements/issue-0710-repository-and-retention-continuation.md:16` |
| D218 | plan 07 | `docs/requirements/issue-0656-benchmark-gated-promotion-protocol.md:26` |
| D219 | plan 07 | `ROADMAP.md:369` and `:428` |
| D220 | plan 07 | `ROADMAP.md:440` |
| D221 | plan 07 | `ROADMAP.md:364` and `:423` |
| D222 | plan 07 | `VISION.md:377-382` |
| D223 | plan 07 | New shard `docs/requirements/issue-0705-anticipatory-dreaming.md` |
| D224 | plan 07 | `docs/requirements/issue-1138-bottleneck-audit.md` |
| D225 | plan 07 | `docs/requirements-traceability.md` |
| D226 | plan 08 | `VISION.md:343` |
| D227 | plan 08 | `ROADMAP.md:145` |
| D228 | plan 08 | `ROADMAP.md:179` |
| D229 | plan 08 | `docs/benchmarks.md:284-295` |
| D230 | plan 08 | `docs/benchmarks.md:307-312` |
| D231 | plan 08 | `docs/benchmarks.md:16-33` |
| D232 | plan 08 | `docs/benchmarks.md:31` |
| D233 | plan 08 | `docs/requirements/issue-0891-equation-corpus-ratchet.md` |
| D234 | plan 08 | `docs/requirements/issue-0698-real-external-benchmark-harness.md` |
| D235 | plan 08 | New shard `docs/requirements/issue-1138-verifiable-task-routing.md` |
| D236 | plan 08 | `docs/requirements-traceability.md` |
| D237 | plan 08 | `docs/meta-algorithm.md:144-167` |
| D238 | plan 08 | `docs/meta-algorithm.md:214-218` |
| D239 | plan 09 | `VISION.md:320-326` |
| D240 | plan 09 | `ROADMAP.md:362` |
| D241 | plan 09 | `ROADMAP.md:421` |
| D242 | plan 09 | `ARCHITECTURE.md:181-185` |
| D243 | plan 09 | `ARCHITECTURE.md:162-165` |
| D244 | plan 09 | `ARCHITECTURE.md:660-673` |
| D245 | plan 09 | `docs/requirements/issue-0559-general-meta-algorithm.md:15-16` |
| D246 | plan 09 | `docs/requirements/issue-0918-*.md` |
| D247 | plan 09 | `docs/requirements-traceability.md` |
| D248 | plan 09 | `data/README.md` |
| D249 | plan 10 | `ROADMAP.md` |
| D250 | plan 10 | `VISION.md:337` |
| D251 | plan 10 | `ARCHITECTURE.md:160-165` |
| D252 | plan 10 | `docs/requirements/issue-0745-intent-routing-generalization.md` |
| D253 | plan 10 | `docs/requirements-traceability.md` |
| D254 | plan 10 | `experiments/issue_840_task_ladder/README.txt` |
| D255 | plan 10 | `tests/fixtures/routing-parity.lino` |
| D256 | plan 10 | `data/meta/learning-frontier-language-gap.lino` and
`data/meta/language-adoption-ledger.lino` |
| D257 | plan 12 | `docs/requirements/issue-0491-least-action-continuation.md:10` |
| D258 | plan 12 | `docs/requirements/issue-0491-least-action-continuation.md:12` |
| D259 | plan 12 | `data/meta/draft-portfolio-recipe.lino:61` |
| D260 | plan 12 | `data/meta/task-decomposition-invariant.lino` |
| D261 | plan 12 | `docs/meta-algorithm.md:150-151` |
| D262 | plan 12 | `docs/meta-algorithm.md`, new section after the budget-search section (`:790-871`) |
| D263 | plan 12 | `VISION.md:189` |
| D264 | plan 12 | `VISION.md:99-100` |
| D265 | plan 12 | `VISION.md:192` |
| D266 | plan 12 | `ROADMAP.md` |
| D267 | plan 12 | New shard `docs/requirements/issue-0901-triz-contradictions.md` |
| D268 | plan 12 | New shard `docs/requirements/issue-0802-hypothesis-search.md` |
| D269 | plan 12 | New shard `docs/requirements/issue-0453-moonshot-splitting.md` |
| D270 | plan 12 | `docs/requirements-traceability.md` |
| D271 | plan 12 | `docs/requirements/issue-1138-bottleneck-audit.md` |
| D272 | plan 01 | `docs/requirements-traceability.md` |
| D273 | plan 01 | `docs/benchmarks.md` |
| D274 | plan 04 | `docs/requirements-traceability.md` |
| D275 | plan 04 | `docs/benchmarks.md` |
| D276 | plan 02 | `ROADMAP.md:146` (pillar 25) |

### Plan 01 — B1 live concept lookup

#### D156 — `docs/meta-algorithm.md`

**`docs/meta-algorithm.md`** — the "coding-discovery meta-algorithm" list at
`:882-891` currently reads:

> 2. **Discover** — `concept_discovery::discover` maps each requirement to seeded
>    structural meanings and licensed parts from Python documentation,
>    Wikifunctions, or a previously verified procedure.

Replace with:

> 2. **Understand** — `concept_lookup::lookup_surface` resolves every surface of the
>    requirement that no seeded meaning accounts for, by walking the
>    `need_kinds`-declaring sources of `data/seed/sources-registry.lino`
>    (dictionary → lexicon → encyclopedia → technical) through the one bounded
>    capture walk in `src/source_walk.rs`. A resolved sense is quoted with its
>    license and digest and is used to reach a seeded structural meaning; it is never
>    inlined into generated code. A surface no source defines stays `blocked`, with
>    every consulted source and its observed outcome reported.
> 3. **Discover** — `concept_discovery::discover_with_lookup` maps each requirement
>    to seeded structural meanings, to the senses step 2 retrieved, and to licensed
>    parts from Python documentation, Wikifunctions, or a previously verified
>    procedure.

(subsequent items renumbered 4–6), and a new `## The concept-lookup meta-algorithm
(issue #1138 B1)` section in the shape of the existing recipe sections, naming
`data/meta/concept-lookup-recipe.lino` and
`tests/unit/specification/concept_lookup_meta_algorithm.rs`.

#### D157 — `VISION.md`

**`VISION.md`** — `:337` currently says:

> Every interface now reads its multilingual responses, concept table, tool
> registry, language-detection rules, prompt patterns, and intent-routing rule book
> from the shared `data/seed/` directory through `src/seed.rs` (Rust) and
> `src/web/seed_loader.js` (browser).

Append, in the same paragraph:

> Since issue #1138 the same seed directory also declares *which* trusted sources may
> be consulted for the meaning of a word it does not know (`need_kinds` in
> `data/seed/sources-registry.lino`), and both the universal loop and the coding path
> consult them through one bounded, content-addressed walk. The assistant's
> vocabulary is therefore no longer bounded by the seed; what is bounded is which
> sources it will trust.

#### D158 — `ROADMAP.md`

**`ROADMAP.md`** — `:145` (row 26) currently ends:

> `task_spec`, source-backed concept discovery, structural composition, bounded
> verification, and the procedure ledger derive Python programs without benchmark
> identifiers or canonical answers in production data.

Replace the clause "source-backed concept discovery" with "live concept lookup
through the sources registry (issue #1138 B1) followed by source-backed concept
discovery", and append: "A requirement word absent from every seed file is resolved
by retrieval, not by a seed edit; the held-out `isogram`/`lipogram` corpus in
`data/benchmarks/concept-lookup-paraphrases.lino` records the honest five-language
number."

#### D159 — `docs/requirements/issue-1138-live-concept-lookup.md`

**`docs/requirements/issue-1138-live-concept-lookup.md`** — new shard (assembled into
`REQUIREMENTS.md` by `rust-script scripts/assemble-requirements.rs --write`;
`REQUIREMENTS.md` itself is never edited by hand):

```markdown
## Issue #1138 B1 Live Concept Lookup

| ID | Requirement | Status / Evidence |
| --- | --- | --- |
| R1138-B1-1 | One `UnknownConceptLookup` implementation walks the sources registry and is used by the universal loop and the coding path. | … |
| R1138-B1-2 | The lookup is bounded by declared depth, pages, services and capture age, never by a time or token budget. | … |
| R1138-B1-3 | Every retrieved sense carries source id, exact URL, sha256, fetch time, license and depth; an offline run replays committed captures byte-identically. | … |
| R1138-B1-4 | A retrieved gloss is quoted with attribution and never inlined into generated code. | … |
| R1138-B1-5 | A word no source defines is reported as unresolved with every consulted source and its outcome; no floor, no guess. | … |
| R1138-B1-6 | Settings opt-outs are authoritative for the lexical tier as they are for the procedural tier. | … |
| R1138-B1-7 | A held-out word absent from every seed file resolves by lookup in en, ru, hi, zh and es, or is honestly reported unresolved per language. | … |
| R1138-B1-8 | Deleting the sense ledger loses nothing: the same captures rediscover the same content ids. | … |
| R1138-B1-9 | The native and browser runtimes execute one walk contract, held to one recorded expectation. | … |
```

#### D272 — `docs/requirements-traceability.md`

**`docs/requirements-traceability.md`** — nine new rows, one per R1138-B1-*, each
naming its test file and `not yet confirmed` for manual confirmation until a run is
recorded. Also correct `:708`, which today reads

> | R710-D8 | 1558 | delivered 2026-09-15; PR #888 (issue #710) | tests/unit/coding_discovery/concepts.rs | not yet confirmed |

to

> | R710-D8 | 1558 | delivered 2026-09-15; PR #888 (issue #710); lookup extension point unimplemented until #1138 B1 | tests/unit/coding_discovery/concepts.rs (lookup exercised only by a test-local fake before #1138) | not yet confirmed |

#### D273 — `docs/benchmarks.md`

**`docs/benchmarks.md`** — add a `### Unknown-word concept lookup — issue #1138 B1`
subsection under "Sources by suite", declaring the corpus path, the five languages,
the two held-out words, the sources consulted with their licenses, and the honest
current score. In `### Honest current numbers` (`:277-284`), append after the table:

> The 2026-09-15 coding rows predate live concept lookup: a requirement whose
> vocabulary fell outside the 82 seeded structural meanings could not be understood
> at all. The unknown-word corpus records that number separately and does not alter
> any upstream row.

### Plan 02 — B2 composition from retrieved sources


#### D160 — `VISION.md:343`

**`VISION.md:343`** currently reads, in part:

> "The latest committed-row summary (run of 2026-09-15) is: HumanEval 20/20; MBPP 20/20; GSM8K 2/20; MATH 0/20; BIG-bench object counting 0/20; CoEdIT 0/20; egg rewrite laws 20/20; Ascent closure assertions 5/5; and SWE-bench Lite 0/1."

Replace with a sentence that names the slice beside every number and adds the
full-suite rows:

> "The latest committed-row summary is, per suite and per slice: HumanEval `<passed>/20` and `<passed>/164`; MBPP `<passed>/20` and `<passed>/500`; GSM8K `<passed>/20`; MATH `<passed>/20`; BIG-bench object counting `<passed>/20`; CoEdIT `<passed>/20`; egg rewrite laws `<passed>/20`; Ascent closure assertions `<passed>/5`; SWE-bench Lite `<passed>/1`. A first-20 score is not a suite score and is never cited without its slice."

#### D161 — `ROADMAP.md:145`

**`ROADMAP.md:145`** (pillar 26) currently reads, in part:

> "The 2026-09-15 upstream rows are HumanEval 20/20 and MBPP 20/20 (empty source cache: 20/20 and 18/20) … `task_spec`, source-backed concept discovery, structural composition, bounded verification, and the procedure ledger derive Python programs without benchmark identifiers or canonical answers in production data."

Replace with:

> "Upstream rows are recorded per slice: HumanEval `<n>/20` and `<n>/164`, MBPP `<n>/20` and `<n>/500` (empty source cache: `<n>` and `<n>`). Composition no longer enumerates shapes in Rust: retrieved procedure text becomes an ordered step list, the step list becomes a language-neutral `ProgramIr`, and per-language lowerings render it. The seeded idiom catalog is a deletable bootstrap with a forget → rediscover → identical-content-id proof."

#### D162 — `ROADMAP.md:492`

**`ROADMAP.md:492`** currently reads:

> "cleared fresh canonical coding (4/4), industry (13/13; the upstream HumanEval/MBPP slices score 0/20, see `data/benchmarks/external-results.lino`), and unit (12/12)"

Replace the parenthetical with the current ledger values and the slice, or — the
better fix, and the one #1089 asks for — replace the literal numbers with a
pointer: "see the generated table in `docs/benchmarks.md`, rendered from
`data/benchmarks/external-results.lino`". Same treatment for **`ROADMAP.md:570`**:

> "were never compiled; upstream coding scores are 0/20 and flat; and most effort"

which is stale on its face and must become "upstream coding scores are published
per slice in `docs/benchmarks.md`".

#### D163 — `docs/benchmarks.md:284-295`

**`docs/benchmarks.md:284-295`** — the "Honest current numbers" table gains a
`Slice` column and two rows (HumanEval @164, MBPP @500), and the preamble at
`docs/benchmarks.md:279-282` ("The latest committed rows are dated `2026-09-15`
for the coding suites") is restated to name each row's slice. The sentence at
`docs/benchmarks.md:297-305` about the empty-source-cache control must be
re-measured at the full slice or explicitly scoped to slice 20.

#### D164 — `docs/benchmarks.md:348-368`

**`docs/benchmarks.md:348-368`** — the "Running it" block gains the full-suite
commands and the forget/rediscover round trip from the Tests-first section.

#### D165 — `docs/benchmarks.md:16-33`

**`docs/benchmarks.md:16-33`** — the "Suites at a glance" table gains a row:

> `| Composition from retrieved sources | #1138 B2 | `coding-composition-from-sources.lino` | `coding_discovery::multilingual` | 25 |`

#### D166 — `docs/requirements/issue-0710-dynamic-coding-discovery.md`

**`docs/requirements/issue-0710-dynamic-coding-discovery.md`** — R710-D2 and
R710-D3 currently read "The first 20 HumanEval cases must be run honestly …" and
"The first 20 MBPP cases …". Replace "first 20" with "full upstream suite (164 /
500), with the first-20 slice retained as a regression control", and update the
Status column to cite the new rows. R710-D10 ("A verified coding procedure must
be content-addressed, provenance-bearing, tamper-detecting, forgettable, and
rediscoverable") gains a sibling: "R710-D17 — the *bootstrap idiom catalog* must
be deletable and rediscoverable to the same content id."

#### D167 — New shard `docs/requirements/issue-1138-composition-from-sources.md`

**New shard `docs/requirements/issue-1138-composition-from-sources.md`**, with
IDs R1138-B2-1 … R1138-B2-8 covering: retrieval-to-step-list, the IR, the
cross-language lowering, the deletable bootstrap, the forget/rediscover hash,
full-suite measurement, the seed-shape gate, and registry declaration of OEIS and
Python docs. `REQUIREMENTS.md` is generated from the shards by
`scripts/assemble-requirements.rs`, so no manual edit there.

#### D168 — `docs/requirements-traceability.md`

**`docs/requirements-traceability.md`** — add rows for every new R1138-B2-*; and
amend the R710-D2/D3 rows at `docs/requirements-traceability.md:702-703`, which
today cite "local upstream run recorded in docs/case-studies/issue-710/README.md"
and "not yet confirmed", to cite the full-suite ledger rows.

#### D169 — `docs/meta-algorithm.md`

**`docs/meta-algorithm.md`** — the coding-discovery recipe must gain the
procedure-text and IR stages. Concretely, the twelve-step recursive core at
`docs/meta-algorithm.md:144-167` keeps its shape, but step 7 ("Construct the
answer back up the tree") is the one this plan implements for coding, and the
document must say so with a pointer to `src/coding/program_ir.rs`. The agentic
recipe's step 2 at `docs/meta-algorithm.md:214-218`:

> "**Pin the canonical plan as named constants** (`SEARCH_QUERY`, `CANONICAL_SOURCE_URL`, `KB_PATH`) so the recipe is data, not scattered literals."

is flatly inconsistent with B2's doctrine and must be rewritten to:

> "**Derive the plan from the task**: the search phrase comes from the formalized need, the source is selected from `data/seed/sources-registry.lino` by `coding_role`, and the knowledge-base path is derived from the task's content id. No constant names a query, a URL or a path."

**reconciled: was "owned jointly with B4"; now owned solely by plan 04 L12,
which owns `src/agentic_coding/formalization_recipe.rs`. This plan cites
row D181 below and does not restate the replacement, because two plans rewriting one
paragraph is how a document acquires two versions of itself (plan 00 §9 X9).**

#### D276 — `ROADMAP.md:146` (pillar 25)

**`ROADMAP.md:146`** (pillar 25, added 2026-09-17 when the full-suite rows
landed and the sentence went stale on that commit) currently reads, in part:

> "the latest committed upstream comparison is HumanEval 20/20 and MBPP 20/20, with every suite row generated in `docs/status.md`."

Replace with:

> "upstream comparisons are recorded per slice — HumanEval 20/20 and 9/164, MBPP 20/20 and `<passed>/500` — in the generated table in `docs/benchmarks.md`, with every suite row generated in `docs/status.md`."

A row that asserts "latest" while citing superseded numbers fails the same
traceability rule this audit enforces everywhere else; it is recorded here
rather than left to silently rot (plan 00 §8).

---

### Plan 03 — B3 repository workspace protocol

#### D170 — `docs/benchmarks.md:311`

**`docs/benchmarks.md:311`** — currently:

> `2 / 20` on GSM8K, `0 / 20` on the other scored core suites, and `0 / 1` on SWE-bench Lite. The ratchet makes every number a floor that may never fall below.

Replace with: *"`2 / 20` on GSM8K, `0 / 20` on the other scored core suites, and `<passed> / 23` on SWE-bench Lite, measured over the whole dev split through the repository workspace protocol (issue #1138 B3). Before that protocol the row was `0 / 1` and was structural: the case was one prompt with no clone, so the empty-patch criterion closed it before the evaluator ran. The ratchet makes every number a floor that may never fall below."*

#### D171 — `docs/benchmarks.md:314-320`

**`docs/benchmarks.md:314-320`** — currently states the evaluator applies "a candidate patch" and that an evaluator/Docker/parquet failure becomes `benchmark_unavailable`. Add one sentence: *"The candidate patch is now produced by cloning the instance at its `base_commit` and diffing the edited tree; Docker is used for the instance tests when it is present and is not required for the patch to exist."*

#### D172 — `docs/meta-algorithm.md:185-262`

**`docs/meta-algorithm.md:185-262`** — the agentic-coding recipe section. Its step 2 currently reads:

> **Pin the canonical plan as named constants** (`SEARCH_QUERY`, `CANONICAL_SOURCE_URL`, `KB_PATH`) so the recipe is data, not scattered literals.

Add, immediately after the eight-step list, a new subsection *"The repository workspace protocol (issue #1138)"* recording the six steps (clone, locate, read, edit, verify, diff), their `source_file`s, and the grounding test — mirroring the table at `docs/meta-algorithm.md:255-266`. Also amend the sentence at `:212-215`:

> ```text
> web_search → web_fetch → write_file(formalize) → run_command(verify) → final
> ```

to note: *"A repository task substitutes the workspace protocol for the middle three stages: the tree replaces the fetched page as the ground truth, and `run_command` runs the named tests rather than a conformance script."*

#### D173 — `ROADMAP.md:145`

**`ROADMAP.md:145`** (row 26) — currently:

> other latest rows remain GSM8K 2/20, MATH 0/20, CoEdIT 0/20, and SWE-bench Lite 0/1.

Replace the SWE-bench clause with the measured full-split number and a pointer to #1138 B3.

#### D174 — `VISION.md:343`

**`VISION.md:343`** — currently ends:

> … and SWE-bench Lite 0/1. MBPP explicitly records `--online`.

Replace `SWE-bench Lite 0/1` with the new measured row and add: *"SWE-bench is now run through the same repository workspace protocol the self-coding path uses, so the number measures repository capability rather than the absence of a clone."*

#### D175 — `GOALS.md:96`

**`GOALS.md:96`** — currently:

> - Complete the self-coding chain: Formal AI codes itself via Agent CLI, directed by Hive Mind, with every change landing as a reviewed pull request.

Replace with: *"- Complete the self-coding chain: `formal-ai solve --model formal-ai` clones at a base commit, locates the files a requirement names, edits, runs the named tests and produces the diff — the same protocol SWE-bench and both ladders use — with every change landing as a reviewed pull request carrying the four self-hosting trailers. An Agent CLI and Hive Mind drive that entry point; they do not own it."*

#### D176 — `GOALS.md:111`

**`GOALS.md:111`** (the "Formal AI codes itself" definition) — currently ends "Seed edits are the first rung; source edits follow through the same rule engine." Append: *"A source edit counts only when it was located from the requirement rather than from a pre-authored rule, and only when the named tests were observed to run."*

#### D177 — `docs/requirements/issue-1085-the-links-network-is-not-the-system-that-reasons.md`

**`docs/requirements/issue-1085-the-links-network-is-not-the-system-that-reasons.md`** — the R1085-9 row (rendered at `REQUIREMENTS.md:2465`) says the ratchet "records how many of the 32 leaves Formal AI actually changed (15, may only rise)". Append: *"Issue #1138 B3 adds `leaf_nodes_passing_without_authored_rules`, measured with `experiments/issue_1028_agent_cli_ladder/rules/` disabled, because a committed per-leaf rule is a memoized answer and the original number measures rule authorship as much as capability."*

#### D178 — `docs/requirements/issue-1021-full-range-coding-and-contribution-artifacts.md`

**`docs/requirements/issue-1021-full-range-coding-and-contribution-artifacts.md`** — the R1021-22 row (rendered at `REQUIREMENTS.md:2405`) says **Not achieved.** Replace only when L17 lands, with the pull-request URL, the session id and the evidence path; until then leave it as written.

#### D179 — New shard `docs/requirements/issue-1138-repository-workspace-protocol.md`

**New shard `docs/requirements/issue-1138-repository-workspace-protocol.md`** with R1138-3-1 … R1138-3-9:

| ID | Requirement |
| --- | --- |
| R1138-3-1 | A repository task carries an origin and an exact base commit; a branch name is refused. |
| R1138-3-2 | The files a requirement names are located without the requirement naming them, by census for Rust trees and by literal or path occurrence otherwise; ambiguity resolves to nothing. |
| R1138-3-3 | Named tests are executed and their command, exit code and output recorded before any obligation may be satisfied. |
| R1138-3-4 | A unified diff is computed from the tree, applies cleanly to the base commit, and is the only thing offered as a patch. |
| R1138-3-5 | SWE-bench, the #848 ladder and self-coding use one protocol document; its steps are data and are grounded against the source. |
| R1138-3-6 | Every command is default-deny; the allowlist is seed data scoped to program plus subcommand plus argument shape. |
| R1138-3-7 | `formal-ai solve --model formal-ai` is an authoring path: it refuses to commit by default, and when it commits it emits all four self-hosting trailers with an evidence bundle naming the exact model. |
| R1138-3-8 | A missing prerequisite is reported as an unsatisfied need with the observed exit code, never as a pass or a skip. |
| R1138-3-9 | The protocol document is forgettable and rediscoverable: deleting and regenerating it reproduces the committed content id. |

#### D180 — `docs/requirements-traceability.md`

**`docs/requirements-traceability.md`** — add one row per R1138-3-x with delivered-in, automated test and `not yet confirmed` for manual, following the honesty rules at `docs/requirements-traceability.md:9-18`.

### Plan 04 — B4 formalization depth

#### D181 — `docs/meta-algorithm.md`

**`docs/meta-algorithm.md`** — the agentic-coding steps at `:214-218` currently read:

> 1. **Recognise the agentic task** from the latest user turn against a small
>    closed keyword set — a non-match yields `None`, so agentic coding stays
>    strictly opt-in and ordinary chat is untouched.
> 2. **Pin the canonical plan as named constants** (`SEARCH_QUERY`,
>    `CANONICAL_SOURCE_URL`, `KB_PATH`) so the recipe is data, not scattered
>    literals.

Replace with:

> 1. **Recognise the agentic task** by role from the seeded lexicon
>    (`ROLE_AGENT_ACTION_FORMALIZE_VERB`) — a non-match yields `None`, so agentic
>    coding stays strictly opt-in and ordinary chat is untouched.
> 2. **Derive the plan from the task's own unresolved needs.** The source text is the
>    one the task quotes or names; the search query is built from the surfaces
>    `ConceptGraph::unresolved()` reports, not from a pinned literal. `KB_PATH`
>    remains a named constant because it is an output path, not a knowledge claim;
>    `SEARCH_QUERY` and `CANONICAL_SOURCE_URL` remain only as the regression
>    fixture's source for the canonical tale.

And the nine-primitive claim implicit in the `meta_primitive` row at `:250`:

> | `meta_primitive` | 9 | each appears in `PRIMITIVE_KINDS` in `src/agentic_coding/formalize.rs`; ordering 1..9 contiguous |

gains a sentence beneath the table:

> Nine kinds are *declared*; how many are *observed* depends on the document, and the
> report states the observed number (`data/seed/meanings-formalization-report.lino`).
> Since issue #1138 B4 the report also states how many of the needs the formalizer
> raised were grounded, so a document cannot be reported as covered while a need is
> unresolved.

Plus a new `## The deep-formalization meta-algorithm (issue #1138 B4)` section naming
`data/meta/formalization-depth-recipe.lino` and
`tests/unit/specification/formalization_depth_meta_algorithm.rs`.

#### D182 — `VISION.md`

**`VISION.md`** — `:339` currently reads:

> The next step is to keep the implemented surfaces small while moving more of the
> assistant's behavior into explicit links: requirements, source facts, traces,
> prompts, handlers, permissions, tests, and reusable problem-solving procedures.

Replace with:

> The next step is to keep the implemented surfaces small while moving more of the
> assistant's behavior into explicit links: requirements, source facts, traces,
> prompts, handlers, permissions, tests, and reusable problem-solving procedures.
> Since issue #1138 B4 a formalized document is a concept graph rather than a set of
> preserved sentences: every surface the formalizer cannot ground becomes an explicit
> need, needs are satisfied by retrieval from the trusted sources the registry
> declares, and a need the sources cannot ground is reported as unresolved with its
> exact source span. Preserving a sentence is recorded as preservation, never as
> understanding.

#### D183 — `ROADMAP.md`

**`ROADMAP.md`** — `:145` (row 26) currently contains:

> `task_spec`, source-backed concept discovery, structural composition, bounded
> verification, and the procedure ledger derive Python programs without benchmark
> identifiers or canonical answers in production data.

Append:

> Since issue #1138 B4 the requirement is first formalized to a concept graph whose
> unresolved surfaces are retrieved rather than assumed; the graph's identity is
> asserted equal across en, ru, hi, zh and es on the held-out corpus in
> `data/benchmarks/formalization-depth-requirements.lino`, and the honest grounded
> ratio is recorded per language.

#### D184 — `docs/requirements/issue-1138-formalization-depth.md`

**`docs/requirements/issue-1138-formalization-depth.md`** — new shard (assembled into
`REQUIREMENTS.md` by `rust-script scripts/assemble-requirements.rs --write`;
`REQUIREMENTS.md` is never hand-edited):

```markdown
## Issue #1138 B4 Formalization Depth

| ID | Requirement | Status / Evidence |
| --- | --- | --- |
| R1138-B4-1 | The formalizer emits an explicit need, with an exact source span and an origin, for every surface, relation and procedure it cannot ground. | … |
| R1138-B4-2 | A need is satisfied by the issue #1138 B1 registry lookup; the retrieved gloss is itself formalized, bounded by a declared concept depth. | … |
| R1138-B4-3 | A grounded result is a concept, predicate, entity or procedure link with source id, URL, sha256 and license — never a stored sentence. | … |
| R1138-B4-4 | Preserving a source sentence is recorded as preservation and can no longer satisfy the assertion primitive; a document with an unresolved need is never reported as covered. | … |
| R1138-B4-5 | An extracted procedure enters the procedure ledger only through the existing bounded execution and named review gate, with its source license honoured. | … |
| R1138-B4-6 | The same unfamiliar requirement in en, ru, hi, zh and es produces one concept-graph identity, or reports per-language why it could not. | … |
| R1138-B4-7 | Sentence segmentation is script-aware and every segment span selects exactly its own text. | … |
| R1138-B4-8 | A custom agentic task is formalized instead of the seeded fairy tale; the tale remains a regression corpus. | … |
| R1138-B4-9 | One need type and one need-status vocabulary serve the universal loop, the coding path and the formalizer. | … |
| R1138-B4-10 | An offline run replays committed captures and reproduces the same graph identity; deleting the graph ledger loses nothing the captures cannot rebuild. | … |
```

#### D274 — `docs/requirements-traceability.md`

**`docs/requirements-traceability.md`** — ten new rows, one per R1138-B4-*, each
naming its test file and `not yet confirmed` until a run is recorded. Also amend
`:376`, which today reads

> | R314 | 867 | PR #469 (issue #468) | tests/unit/agentic_coding.rs; tests/unit/agentic_surfaces.rs | manually confirmed 2026-08-04 (audit): `formal-ai agent --help` run; offline `agent --silent --task ...` exit 0 (see audit finding: falls back to seeded fairy-tale KB rather than reflecting custom --task) |

to

> | R314 | 867 | PR #469 (issue #468); custom-task fallback repaired by #1138 B4 | tests/unit/agentic_coding.rs::a_custom_task_is_formalized_instead_of_the_seeded_fairy_tale; tests/unit/agentic_surfaces.rs | manually confirmed 2026-08-04 (audit): `formal-ai agent --help` run; offline `agent --silent --task ...` exit 0 — the 2026-08-04 finding "falls back to seeded fairy-tale KB rather than reflecting custom --task" is now pinned as a regression |

#### D275 — `docs/benchmarks.md`

**`docs/benchmarks.md`** — add a `### Deep formalization of unfamiliar requirements
— issue #1138 B4` subsection under "Sources by suite", declaring the corpus path,
the five languages, the two families, the sources consulted with their licenses, and
the honest grounded ratio and observed-primitive count per language. In
`### Honest current numbers` (`:277-284`), append after the existing paragraph:

> The observed-primitive count for an unfamiliar document is recorded separately from
> the nine declared kinds. Before issue #1138 B4 it was 2 of 9 in every language,
> because an unrecognised sentence became a preserved span and nothing else; the
> deep-formalization corpus records what it is now, per language, including the
> languages where no source served a definition.

### Plan 05 — B5 obligation execution evidence


#### D185 — `docs/requirements/issue-0559-general-meta-algorithm.md:18-26`

**`docs/requirements/issue-0559-general-meta-algorithm.md:18-26`** — replace:

> the shared ledger runs before dispatch, so a selected method is **planned**, not
> **satisfied**. A connected planning chain accounts for a detected need but does not prove
> its execution. … Runtime per-need verification feedback is still open; the implemented
> artifact rows below are not a claim that every detected obligation executes successfully.

with:

> the planning ledger runs before dispatch, so a selected method is **planned**, not
> **satisfied**. A connected planning chain accounts for a detected need but does not prove
> its execution. A second, append-only pass (`src/obligation_ledger.rs`, recipe step 14)
> supplies the runtime per-need feedback: an execution row reaches **satisfied** through
> `need_ledger_with_execution` only when its input outcome carries a matching
> `Evidence`. That record names the command, retains an exit code or an explicit none,
> and hashes the exact observed bytes with SHA-256. The shared
> `need_status_with_observation` function is the only constructor of the terminal status;
> other domains may call it only after their own observed state or successful re-probe.
> A clause with no derivable expectation is split rather than discarded, and a clause that
> cannot be split is reported as an unsatisfied gap with its byte span.

#### D186 — `docs/requirements/issue-0710-repository-and-retention-continuation.md:18`

**`docs/requirements/issue-0710-repository-and-retention-continuation.md:18`** (R710-R9) —
replace the tail:

> Preserving unknown text does not interpret or execute it; complete obligation-ledger
> execution and runtime verification feedback remain open.

with:

> Preserving unknown text still does not interpret it, but it is no longer discarded: an
> unrecognized clause becomes an `Underivable` obligation node that is split and, failing
> that, reported as a named gap with its byte span. Obligation-ledger execution and runtime
> verification feedback are delivered by `src/obligation_ledger.rs`.

#### D187 — `docs/requirements/issue-0710-repository-and-retention-continuation.md:13`

**`docs/requirements/issue-0710-repository-and-retention-continuation.md:13`** (R710-R4) —
append after the existing text:

> The same binding now applies outside the recipe path: `ObligationLedger::observe` returns
> `None` for a record whose command or path answers no expectation, so an unrelated result
> cannot clear any obligation on any surface.

#### D188 — `docs/meta-algorithm.md:144`

**`docs/meta-algorithm.md:144`** — the heading `### The twelve steps` is already wrong: the
recipe carries thirteen `meta_step` records (including `audit_reasoning_standard` at
`data/meta/recursive-core-recipe.lino:100-107`), and `docs/meta-algorithm.md:21` already
says "13 steps, 29 pinned functions" while `:172-173` say `meta_step | 12` and
`meta_function | 25`. Replace the heading with `### The fourteen steps` and append after
item 12 (`:165-166`):

> 13. **Audit the pass against the reasoning standard, unconditionally** (R1073).
> 14. **Discharge every obligation against an execution record** — an execution-projected
>     need reaches *satisfied* only with a matching command, an exit code or an explicit
>     none, and a hash of the exact observed bytes behind it; an unsatisfied node is split,
>     and a node that cannot be split is reported as a gap.

#### D189 — `docs/meta-algorithm.md:172-173`

**`docs/meta-algorithm.md:172-173`** — replace
`| meta_step | 12 | ordering 1..12 is contiguous …` with
`| meta_step | 14 | ordering 1..14 is contiguous …`, and raise the `meta_function` count
from 25 to include `ObligationNode::build`, `ObligationLedger::observe`,
`need_ledger_with_execution` and `record_obligation_ledger`.

#### D190 — `docs/meta-algorithm.md:152-153`

**`docs/meta-algorithm.md:152-153`** — step 4 currently reads "Account for every need in a
satisfaction ledger, so a need with no method is recorded as blocked rather than silently
dropped." Append: "Selection is not satisfaction: this ledger records planning only, and
step 14 is what can mark a need satisfied."

#### D191 — `data/meta/recursive-core-recipe.lino:5`

**`data/meta/recursive-core-recipe.lino:5`** — remove from `summary`:

> Selected methods are planned, not satisfied; missing methods are blocked. Recording a
> plan neither executes each leaf nor validates its result.

replace with:

> Selected methods are planned; a method becomes satisfied only when step 14 binds an
> execution record to the obligation it discharges. Missing methods are blocked.

#### D192 — `data/meta/recursive-core-recipe.lino:6`

**`data/meta/recursive-core-recipe.lino:6`** — remove from `generalization`:

> The current executable recipe reproduces the native planning trace; runtime per-need
> evidence feedback and general prerequisite recovery remain open.

replace with:

> The executable recipe reproduces the native planning trace and its execution pass;
> general prerequisite recovery remains open (plan 06).

#### D193 — `data/meta/recursive-core-recipe.lino:35`

**`data/meta/recursive-core-recipe.lino:35`** — after "Runtime checks must supply per-need
evidence before satisfaction." append: "Step 14 is that runtime check."

#### D194 — `docs/requirements-traceability.md:406`

**`docs/requirements-traceability.md:406`** — the R344 row's `Automated test` column gains
`tests/unit/specification/obligation_ledger.rs`. Add three new rows for the new shard:

```
| R1138-B5-1 | n/a | PR for #1138 | tests/unit/specification/execution_evidence.rs | not yet confirmed |
| R1138-B5-2 | n/a | PR for #1138 | tests/unit/specification/obligation_ledger.rs | not yet confirmed |
| R1138-B5-3 | n/a | PR for #1138 | tests/unit/issue_1138_obligation_evidence.rs | not yet confirmed |
```

#### D195 — New shard `docs/requirements/issue-1138-bottleneck-audit.md`

**New shard `docs/requirements/issue-1138-bottleneck-audit.md`** (shared with plans 07 and
12; this plan owns the B5 rows):

```
| R1138-B5-1 | Every satisfied obligation node must carry a matching execution record — command, exit code or an explicit none, and a SHA-256 of the exact observed bytes. | … |
| R1138-B5-2 | An observation may discharge only the node whose expectation names its command or path; an unrelated result clears nothing. | … |
| R1138-B5-3 | A clause with no derivable expectation is split, not discarded; a clause that cannot be split is reported as an unsatisfied gap with its byte span, never as completion prose. | … |
```

#### D196 — `VISION.md:192`

**`VISION.md:192`** (Universal Problem-Solving Algorithm, step 9) currently reads:

> **Verification and recursive recovery**: run the composed solution against the whole-task
> test. On failure, record `trace:execution_failure`, descend to smaller tasks, and retry
> upward after their tests pass.

Append:

> An obligation is discharged only by a matching observation — a command, its exit status
> or an explicit none, and the hash of the exact bytes it produced. A node that cannot be
> observed is split; a node that cannot be split is reported as a gap, never as a completed
> step.

#### D197 — `ROADMAP.md`

**`ROADMAP.md`** — neither `obligation` nor `need ledger` appears anywhere in the file
today, so nothing is retracted. Add one row to the status table at `ROADMAP.md:423`:
`| Obligation execution evidence | Delivered for #1138 B5: satisfaction requires a command, exit code and observed-output hash; unsatisfied nodes decompose | ratcheted by data/meta/obligation-evidence-ratchet.lino |`.

### Plan 06 — B6 prerequisite and environment discovery

#### D198 — `VISION.md:163`

**`VISION.md:163`** — currently:

> - Explicit agent autonomy: agent mode should expose actions and run them in an isolated environment such as a Docker image, a server sandbox, or a browser VM where practical.

Replace with: *"- Explicit agent autonomy: agent mode exposes actions and runs them in an isolated environment — the allowlisted host sandbox, a `link-foundation/box` container, a per-conversation detached container, or a browser runtime — chosen by an observed probe rather than declared. Where no environment is available the answer says so and shows no unobserved output (issues #8, #930, #937, #1138 B6)."*

#### D199 — `VISION.md:269`

**`VISION.md:269`** — currently:

> Code-generation tasks should be a first focus area. The assistant should generate algorithms in popular languages, compile or run generated code when the environment supports it, report execution limits honestly, and preserve logs for failed reasoning or failed execution. Browser-only mode can start with JavaScript evaluation and later experiment with WebVM.

Replace the middle clause: *"… compile or run generated code when the environment supports it — and when it does not, discover the missing toolchain from its trusted publisher, install it under the workspace, and retry — report execution limits honestly from a probe rather than from a constant, and preserve logs for failed reasoning or failed execution. Browser-only mode starts with JavaScript evaluation and offers a lazily fetched Python runtime; WebVM remains an open option."*

#### D200 — `ROADMAP.md:126`

**`ROADMAP.md:126`** (row 7) — currently ends:

> #938 unifies the coding-task handler family behind one executable meta-builder | Generalizing the shared builder beyond coding tasks remains tracked work.

Append to the notes column: *"Toolchain availability is probed, not declared (#1138 B6); the fourteen hard-coded `setup_hint` strings and five hard-coded `environment` strings are seed rows with retrieved provenance."*

#### D201 — `GOALS.md:96`

**`GOALS.md:96`** area — add one bullet after the agent-orchestration list: *"- Treat a missing prerequisite as a requirement: observe the failure, name the program, find its procedure at the trusted publisher, install it under the workspace and never system-wide, retry the original step, and keep only the recipe — so the toolchain can be forgotten and rediscovered."*

#### D202 — `GOALS.md`, Self-Evolution list

**`GOALS.md`, Self-Evolution list** — add: *"- Never present unobserved output as observed. Every surface states its execution limit from a probe, in every supported language."*

#### D203 — `docs/meta-algorithm.md:185-262`

**`docs/meta-algorithm.md:185-262`** — the agentic-coding recipe section. After the eight-step list, add *"The prerequisite-discovery meta-algorithm (issue #1138 B6)"* recording the eight recovery steps, their `source_file`s and the grounding table, in the same shape as `docs/meta-algorithm.md:255-266`. Also amend the sentence at `:209-211`:

> The loop is a pure, deterministic function of the conversation so far — no sampling, no hidden state, no neural inference (a NON-GOAL).

to add: *"A step may observe that a program it needs is absent; that observation is a need, not an error, and the recovery sequence that follows is the same deterministic function of the conversation plus the observed exit code."*

#### D204 — `docs/benchmarks.md:356-358`

**`docs/benchmarks.md:356-358`** — currently:

> ```sh
> # Refresh every suite locally. SWE-bench additionally needs the pinned official
> # Python harness and Docker; scheduled CI bounds it separately to one case.
> ```

Replace the comment with: *"# Refresh every suite locally. SWE-bench's pinned official Python harness and Docker are prerequisites the run now discovers and, with `--allow-install`, installs under the workspace; without a grant the run reports the missing prerequisite instead of recording a solver failure."*

#### D205 — `docs/requirements/issue-0008-telegram-bot-requirements.md`

**`docs/requirements/issue-0008-telegram-bot-requirements.md`** — the R8 rows that #930 merges (R8-1, R8-2, R8-4). Each currently records the docker pipeline as deferred. Replace the status column with the implemented mechanism and cite `src/execution_box/`, or, where a leaf did not land, state precisely which.

#### D206 — `docs/requirements/issue-0195-docker-in-docker-telegram-runtime.md`

**`docs/requirements/issue-0195-docker-in-docker-telegram-runtime.md`** — add a row: *"The image's `FORMAL_AI_START_ISOLATION` and `FORMAL_AI_START_RUNNER` (Dockerfile:66-67) are read by the runtime, not only asserted by `scripts/verify-docker-runtime.sh`."*

#### D207 — New shard `docs/requirements/issue-1138-prerequisite-discovery.md`

**New shard `docs/requirements/issue-1138-prerequisite-discovery.md`** with R1138-6-1 … R1138-6-12:

| ID | Requirement |
| --- | --- |
| R1138-6-1 | Toolchain availability is an observation. A status that was never probed is `NotProbed`, never `Unavailable`. |
| R1138-6-2 | `check_command` is executed before an answer claims an output was observed; an unobserved output is labelled as such in every supported language. |
| R1138-6-3 | A missing program is classified from the observed exit code and distinguished from a permission denial and from an ordinary compile error. |
| R1138-6-4 | A missing program becomes a `PrerequisiteNeed` recorded in the need ledger, `Blocked` until a re-probe returns `Present`. |
| R1138-6-5 | A setup procedure comes from the trusted publisher declared for that program; ranking is not authority, and a lookalike host is refused and recorded. |
| R1138-6-6 | Installation is default-deny, granted per program, scoped to the workspace root, and never system-wide; a step writing outside the root is refused before execution. |
| R1138-6-7 | A procedure without a postcondition probe is refused; a successful command with a failing postcondition is `StillMissing`, never success. |
| R1138-6-8 | The ledger retains the recipe and provenance, never the installed payload; deleting both and re-running reproduces the same content id. |
| R1138-6-9 | Code execution may run in a `link-foundation/box` container or a per-conversation detached container with snapshot-by-default and command replay as a selectable fallback; the container has no network unless the task contract requires it. |
| R1138-6-10 | A deadline is a reported failure with the elapsed time, the deadline and the partial output; the descending-N ladder records every N it tried and its outcome. No budget silently truncates work. |
| R1138-6-11 | The browser states which runtime could be loaded and its size, loads it only on an explicit user action, and shows observed output only when a runtime ran the program. |
| R1138-6-12 | The absence of every execution environment is an honest refusal in the user's language, never a silent skip and never an unobserved output presented as observed. |

#### D208 — `docs/requirements-traceability.md`

**`docs/requirements-traceability.md`** — one row per R1138-6-x, following the honesty rules at `:9-18`; `not yet confirmed` for manual confirmation until a maintainer runs it.

#### D209 — `docs/case-studies/issue-710/plans/07-prerequisite-discovery-bridge.md:65-81`

**`docs/case-studies/issue-710/plans/07-prerequisite-discovery-bridge.md:65-81`** — tick only the boxes this plan's evidence actually closes, in the commit that closes each, and leave the rest with their reason, honouring `06-repository-task-generalization.md:74-75`: *"Use `[ ]` until evidence exists; record failing command/output before fixing."*

### Plan 07 — B7 learning loops that change behaviour

#### D210 — `docs/meta-algorithm.md:772-775`

**`docs/meta-algorithm.md:772-775`** (the #922 section, item 6) — replace:

> 6. **Load adopted link data** — only the checked-in
>    `data/seed/learned-methods.lino` reaches `MethodRegistry`. Learned records are
>    observable in the registry event but are separate from compiled handlers, so
>    adoption cannot silently introduce executable behavior or alter precedence.

with:

> 6. **Load and execute adopted link data** — only the checked-in
>    `data/seed/learned-methods.lino` reaches `MethodRegistry`. An adopted record whose
>    operations all bind to known recorders is compiled to a `RecipeProgram` and dispatched
>    **after** every compiled method, so a learned abstraction may add a capability and can
>    never pre-empt one. The answer's trace names it (`method:learned`), and adoption
>    required a qualifying `AdoptionEffect` — an `Improved` before/after observation on
>    held-out prompts in en, ru, hi, zh and es, with no regression anywhere.

#### D211 — `docs/meta-algorithm.md:761-764`

**`docs/meta-algorithm.md:761-764`** (item 3, "Infer, then withhold") — the title is now
wrong about the second half. Replace the heading phrase **"Infer, then withhold"** with

**"Infer, then validate on unseen experience"**, and append to the item:

> Withholding is about *inference*, not about *use*: a candidate is withheld from the
> traces that validate it, and is then adopted into live dispatch through the promotion
> gate and a reviewed pull request.

#### D212 — `docs/meta-algorithm.md:671-678`

**`docs/meta-algorithm.md:671-678`** (the promotion preamble) — replace:

> Every self-improvement loop above stops at *proposing* … and even then only as a `.lino`
> seed edit written onto a branch — never a direct push. Draft pull requests and human
> review stay the outer gate.

with:

> Every self-improvement loop above proposes; this protocol decides. A proposal that clears
> its benchmark ratchets is materialized as a `.lino` seed edit on a local branch and, with
> `--open-draft-pr`, published as a **draft** pull request — never a push to the default
> branch, never a merge, never marked ready. Human review of that pull request is the outer
> gate, which is where the gate belongs: a learned item is reviewable because it is visible
> in a diff, not because it is inert.

#### D213 — `docs/meta-algorithm.md:137-142`

**`docs/meta-algorithm.md:137-142`** — replace:

> Second, it is **self-improving in proposal-only form**: `src/meta_self_improvement.rs`
> reads this recipe against the live pipeline, detects drift between the algorithm-as-data
> and the algorithm-as-code, and proposes the additions and stale-citation removals that
> reconcile them — gated `off` by default, never writing the recipe back, so adoption stays
> a human review step (R340).

with:

> Second, it is **self-improving**: `src/meta_self_improvement.rs` reads this recipe against
> the live pipeline, detects drift between the algorithm-as-data and the algorithm-as-code,
> and proposes the additions and stale-citation removals that reconcile them. It proposes by
> default and still writes nothing itself; the seed edit it feeds passes the promotion gate
> and arrives as a reviewed draft pull request (R340).

#### D214 — `docs/requirements/issue-0922-method-learning-from-experience.md:13`

**`docs/requirements/issue-0922-method-learning-from-experience.md:13`** (R922-2) — replace
"Keep all learned candidates inert until benchmark-gated, human-confirmed promotion." with:

> Keep all learned candidates out of dispatch until benchmark-gated, human-confirmed
> promotion; after promotion they execute at last precedence with their effect proved and
> their use named in the trace.

#### D215 — `docs/requirements/issue-0922-method-learning-from-experience.md:16`

**`docs/requirements/issue-0922-method-learning-from-experience.md:16`** (R922-5) — replace
"Learned records are separate from compiled handlers, so dispatch order is unchanged" with:

> Learned records are appended after every compiled method, so no compiled precedence
> changes; a learned record with an unbound operation is reported and not dispatched.

#### D216 — `docs/requirements/issue-0559-general-meta-algorithm.md:39`

**`docs/requirements/issue-0559-general-meta-algorithm.md:39`** (R340) — replace "It must be
gated and proposal-only: the default `off` mode proposes nothing and it never writes the
recipe back" with:

> It must be proposal-only at the loop and gated at review: the loop proposes by default and
> never writes the recipe back itself; the seed edit it feeds clears the promotion ratchets
> and is published as a draft pull request for human review.

#### D217 — `docs/requirements/issue-0710-repository-and-retention-continuation.md:16`

**`docs/requirements/issue-0710-repository-and-retention-continuation.md:16`** (R710-R7) —
replace the tail "End-to-end automatic source-cache reconstruction remains open." with:

> `src/source_reconstruction.rs` executes the retained `rediscover:` edge on the next cache
> miss with no human command, emits an execution record for the recovery, and reports a
> hash divergence as `Diverged` — today's page never replaces historical evidence.

#### D218 — `docs/requirements/issue-0656-benchmark-gated-promotion-protocol.md:26`

**`docs/requirements/issue-0656-benchmark-gated-promotion-protocol.md:26`** (R472) — append:

> The protocol may now open a **draft** pull request (`--open-draft-pr`); it still never
> pushes to the default branch, never merges and never marks a pull request ready, so
> required GitHub checks on the actual head SHA and human review remain the final authority.

#### D219 — `ROADMAP.md:369` and `:428`

**`ROADMAP.md:369` and `:428`** both read "Anticipatory learning (predict next requests,
pre-learn) | Not done | #705". Replace both with:

> | Anticipatory learning (predict next requests, pre-learn) | Delivered by the #705 work carried forward from PR #887 into the #1138 pull request; PR #887 itself is closed as superseded | [#705](https://github.com/link-assistant/formal-ai/issues/705) |

(Plan 11 owns collapsing these duplicated status tables into one generated table; this plan
only makes both copies true.)

#### D220 — `ROADMAP.md:440`

**`ROADMAP.md:440`** — replace "Broader method construction and recipe mutation remain
incremental work" with:

> Adopted abstractions now execute at last precedence with a proved five-language effect;
> broader method *construction* and recipe mutation remain incremental work.

#### D221 — `ROADMAP.md:364` and `:423`

**`ROADMAP.md:364` and `:423`** — both describe self-improvement as "Partial". Replace the
parenthetical in each with:

> (learned items now change the next answer across the method registry, proved by
> before/after execution records on held-out prompts in en, ru, hi, zh and es; promotion
> publishes a reviewed draft pull request; broader classes and frontiers remain unproven)

#### D222 — `VISION.md:377-382`

**`VISION.md:377-382`** currently reads:

> The self-evolution frontier is explicit and benchmark-gated: proposals must pass tests and
> benchmark ratchets before a reviewed promotion materializes them as seed edits (issues
> #656, #701)

Append:

> A promoted item is not merely stored: it executes, at last precedence behind every
> compiled method, and the trace names it. The gate is the review of the pull request that
> carries the seed edit — learned items are reviewable because they are visible in a diff,
> never because they are inert.

#### D223 — New shard `docs/requirements/issue-0705-anticipatory-dreaming.md`

**New shard `docs/requirements/issue-0705-anticipatory-dreaming.md`** carrying PR #887's 22
`REQUIREMENTS.md` lines as R705-1..R705-6, plus a note that they arrived through the #1138
pull request rather than through PR #887, which is closed as superseded.

#### D224 — `docs/requirements/issue-1138-bottleneck-audit.md`

**`docs/requirements/issue-1138-bottleneck-audit.md`** (shared shard; this plan owns the B7
rows):

```
| R1138-B7-1 | An adopted learned item must execute in the live dispatch path and its use must be named in the trace. | … |
| R1138-B7-2 | Adoption requires a qualifying AdoptionEffect: an Improved before/after execution-record pair on held-out prompts in en, ru, hi, zh and es, with zero regressions. | … |
| R1138-B7-3 | The human gate is the review of a draft pull request carrying the seed edit, never the inertness of the learned item. | … |
| R1138-B7-4 | A forgotten cache payload is refetched automatically on the next miss, and a hash divergence is reported rather than substituted. | … |
```

#### D225 — `docs/requirements-traceability.md`

**`docs/requirements-traceability.md`** — R922-1..6 and R710-R1..R10 have **no rows at all**
today (`grep -c "R922\|R710-R"` → 0 for both). Add them, with R922-2/R922-5 pointing at
`tests/unit/specification/method_registry.rs` and R710-R7 at
`tests/unit/specification/source_reconstruction.rs`. Add R705-1..6 and R1138-B7-1..4.

### Plan 08 — B8 verifiable task routing

#### D226 — `VISION.md:343`

**`VISION.md:343`** currently reads, in part:

> "The synthesis step is now **general**: instead of resolving answers from seeded handlers, the universal 11-step loop **derives** them by composing decomposed sub-results over the links network. … the solver writes HumanEval/MBPP Python functions from parsed structure, source-grounded meanings, composed schemas, and bounded execution, and computes the GSM8K (`18`), MATH (`11`), and BIG-bench object-counting (`3`) answers."

The parenthetical values are curated-slice answers being cited beside upstream
capability, which is the confusion #1085 asked to end. Replace with:

> "The synthesis step is general across task *kinds*, not only across coding: any task carrying a verifiable expectation — a number, a count, an edited text, a value for a named unknown — enters the same recognise → discover → compose → verify → remember path, and the composed program is executed to produce the answer. Upstream scores are cited per suite and per slice from `data/benchmarks/external-results.lino`; the curated 13/13 slice is named as curated wherever it appears."

#### D227 — `ROADMAP.md:145`

**`ROADMAP.md:145`** (pillar 26) currently reads, in part:

> "other latest rows remain GSM8K 2/20, MATH 0/20, CoEdIT 0/20, and SWE-bench Lite 0/1."

Replace with the re-measured values and their date, and add:

> "Non-coding suites are answered by the same verifiable-task route as coding: `src/verifiable_task.rs` recognises the expectation, `src/coding/program_ir.rs` composes the derivation, and the program is executed in the bounded workspace to produce the value. The object-counting category table and the pattern-inference keyword array were deleted, not extended."

#### D228 — `ROADMAP.md:179`

**`ROADMAP.md:179`** currently reads:

> "| E29 | #314 | #320 | Compute math/word-problem and counting answers (GSM8K, MATH, BIG-bench) deterministically rather than seeding them. |"

The row is accurate about intent and misleading about outcome: the counting
answer *was* seeded, in `OBJECT_CATEGORIES`. Append: "Superseded for the upstream
suites by the verifiable-task route (#1138 B8), which deletes the seeded category
table."

#### D229 — `docs/benchmarks.md:284-295`

**`docs/benchmarks.md:284-295`** — the "Honest current numbers" table rows for
GSM8K, MATH, BIG-bench object counting and CoEdIT are updated with the
re-measured values and their date, so all rows share one measurement generation.
The preamble at `docs/benchmarks.md:279-282`:

> "The latest committed rows are dated `2026-09-15` for the coding suites, use solver version `0.349.2` … Other suite rows remain at their latest `2026-09-07` measurements:"

becomes a single-date statement once L22 lands, or keeps the split with the new
date if any suite could not be re-run.

#### D230 — `docs/benchmarks.md:307-312`

**`docs/benchmarks.md:307-312`** currently reads, in part:

> "The existing corpus rows remain recorded exactly as measured: `2 / 20` on GSM8K, `0 / 20` on the other scored core suites, and `0 / 1` on SWE-bench Lite."

Replace with the measured values, and add a sentence distinguishing derivation
gains from presentation gains:

> "Where a score moved because the answer's shape now matches the upstream grader's convention rather than because a new derivation succeeded, that is stated per suite; a presentation gain is not a reasoning gain."

#### D231 — `docs/benchmarks.md:16-33`

**`docs/benchmarks.md:16-33`** — add a row to "Suites at a glance":

> `| Verifiable-task paraphrases | #1138 B8 | `verifiable-task-paraphrases.lino` | `verifiable_task::routing` | 30 |`

#### D232 — `docs/benchmarks.md:31`

**`docs/benchmarks.md:31`** — the equation-corpus row's `minimum_pass_count` of
"72 (and ≥50 distinct verified types)" rises with each promoted limitation; L1
alone takes it to 73.

#### D233 — `docs/requirements/issue-0891-equation-corpus-ratchet.md`

**`docs/requirements/issue-0891-equation-corpus-ratchet.md`** — the shard must
record that the ten limitations are now candidates for the verifiable-task route
rather than permanent upstream constraints, and that each fix is promoted to a
`benchmark_case` in the same commit as the code change.

#### D234 — `docs/requirements/issue-0698-real-external-benchmark-harness.md`

**`docs/requirements/issue-0698-real-external-benchmark-harness.md`** — R529's
status cites `recorded_scores_are_honest_passed_over_total`; add that non-coding
suites are re-measured at every solver generation, so a row's date is part of its
honesty.

#### D235 — New shard `docs/requirements/issue-1138-verifiable-task-routing.md`

**New shard `docs/requirements/issue-1138-verifiable-task-routing.md`** with IDs
R1138-B8-1 … R1138-B8-9 covering: the shared task type, seed-driven recognition in
five languages, the projection onto the discovery path, execution-produces-the-answer,
the five self-checks, derivation memory that recomputes rather than replays,
deletion of the seeded category table, the extended no-memorization gate, and the
re-measurement. `REQUIREMENTS.md` is generated from the shards by
`scripts/assemble-requirements.rs`; no manual edit there.

#### D236 — `docs/requirements-traceability.md`

**`docs/requirements-traceability.md`** — add rows for every R1138-B8-*. Also
amend the R710-D12 row's neighbours: the file has no rows at all for R710-R1..R10,
R873, R919, R922, R924, R991 and R1085-2/3/11 (#1138 B11), and this plan's rows
must not repeat that pattern — each lands with its evidence column filled at
merge time, not "not yet confirmed".

#### D237 — `docs/meta-algorithm.md:144-167`

**`docs/meta-algorithm.md:144-167`** — the twelve-step recursive core is the
document that should already have described this route. Step 8 reads:

> "8. **Resolve each atomic leaf through registry-backed method dispatch** — the registry is the sole authority (R344)."

Add a sentence: "A leaf whose task carries a verifiable expectation resolves by
*executing* a derived program and observing its output, not by selecting a method
that formats a string; the observation is the evidence step 9 records." That
sentence is also B5's requirement, and this plan is the first place it becomes
true for a non-coding leaf.

#### D238 — `docs/meta-algorithm.md:214-218`

**`docs/meta-algorithm.md:214-218`** — the agentic recipe's step 2 pins
`SEARCH_QUERY`, `CANONICAL_SOURCE_URL` and `KB_PATH` as constants.

**reconciled: was restated here and in plan 02; now owned solely by plan 04 L12
and recorded once as row D181 below, because two plans rewriting one paragraph
is how a document acquires two versions of itself (plan 00 §9 X9).**

---

### Plan 09 — B9 handler migration ratchet

#### D239 — `VISION.md:320-326`

**`VISION.md:320-326`** — currently:

> "the solver reasons over Rust structures -- `MemoryStore` is a vector of
> events, the seed is parsed into Rust tables by `src/seed.rs`, and
> `src/solver.rs`, `src/engine.rs` and `src/main.rs` never read the doublets
> store."

Replace, at leaf 40, with:

> "the solver reads the doublets store: `src/seed_links.rs` projects every seed
> meaning, handler rule and handler promotion into content-addressed
> `LinkRecord`s at boot, and `src/rule_interpreter.rs` resolves every routing
> condition through `link_store::query`. The `.lino` files remain the
> human-reviewable source and export projection. `data/meta/debt-ratchet.lino`
> records `store_read_share`, the fraction of routing decisions resolved from
> the store; it is the one measure in that ledger whose strict direction is
> upward."

Until leaf 40 lands, the sentence stays true and is not touched — this is a
statement that must change *with* the code, not ahead of it.

#### D240 — `ROADMAP.md:362`

**`ROADMAP.md:362`** — currently:

> "Partial: registry precedence/route authority is data-driven; #699 batches 1-3
> migrated number constraints, `who_is`, `definition_merge` and the
> `program_synthesis` dead end (now a named skill gap), ratcheting the tree at
> 37 handler files / 48 `try_*` registry entries, with the remaining methods
> honestly pending in the ledger"

Replace with:

> "Partial, with a strict single-definition ratchet: 16 of 58 registry methods
> are migrated and 2 are justified-native; 40 remain pending, grouped into five
> meta-method families by `docs/case-studies/issue-1138/plans/09-handler-migration-ratchet.md`.
> The measured numbers live in `data/meta/debt-ratchet.lino` alone — 46 handler
> files, 39 `try_*` entries, 19 promotion predicates, 9 dispatch name special
> cases — and every one of them may only fall."

#### D241 — `ROADMAP.md:421`

**`ROADMAP.md:421`** — currently:

> "#918 recursively classifies all 46 mixed handler sources as migration debt
> and ratchets their 19,543 outside-core lines"

Replace with the measured values and the widened scan root:

> "#918 recursively classifies every handler source as migration debt and
> ratchets its lines; the scan root now includes `src/solver_handler_how.rs`,
> `solver_handler_how_synthesis.rs`, `solver_handler_units.rs` and
> `solver_handler_oracle.rs`, so a migration cannot lower the count by moving a
> file out of `src/solver_handlers/`. The current values are in
> `data/meta/core-boundary-ledger.lino`."

#### D242 — `ARCHITECTURE.md:181-185`

**`ARCHITECTURE.md:181-185`** — currently:

> "The browser worker mirrors the seed through `src/web/seed_loader.js`; because
> it names its handlers differently and runs its async fetch handlers in a later
> phase, full order-parity is impossible, so `tests/fixtures/routing-parity.lino`
> pins the *shared* precedence invariants both surfaces must honour"

Replace, at leaf 15, with:

> "The browser worker reads the same `data/seed/handler-precedence.lino` it
> fetches at startup: `src/web/seed_loader.js::parseHandlerPrecedence` supplies
> the order and `formal_ai_worker_20.js` holds a name-keyed handler registry
> that must be an exact permutation of it, asserted at load. Rows carry a
> `phase async` note where the worker runs a handler in its later fetch phase,
> and the parity test reorders a fixture row and asserts both surfaces change
> identically."

#### D243 — `ARCHITECTURE.md:162-165`

**`ARCHITECTURE.md:162-165`** — "Specialized handlers (`solver_handler_units`,
`solver_handler_how`, `solver_handlers`, `solver_handlers_policy`) are *plugged
into* the universal solver" — `solver_handlers_policy` no longer exists (the
policy handlers migrated to `data/seed/handler-rules.lino`); replace the list
with the five meta-method families and note that the named files are being
retired batch by batch.

#### D244 — `ARCHITECTURE.md:660-673`

**`ARCHITECTURE.md:660-673`** (Minimal Compiled Core) — add the sentence: "A
family interpreter (`retrieval_method`, `procedure_interpreter`,
`structural_operator`, `dialogue_state_query`) is admitted under **Generic
interpreters**; a handler is not. The boundary ledger records which category
each source claims, so a handler cannot be promoted by renaming it."

#### D245 — `docs/requirements/issue-0559-general-meta-algorithm.md:15-16`

**`docs/requirements/issue-0559-general-meta-algorithm.md:15-16`** — currently
"Issue #699 tracks that remaining migration honestly in
`data/meta/handler-migration-ledger.lino`." Add: "and the plan that retires it
is `docs/case-studies/issue-1138/plans/09-handler-migration-ratchet.md`, which
groups the 40 pending methods into five meta-method families and makes every
ceiling strict-downward." The R344 row's status text (`:43`) must lose the
phrase "the complete 55-method status" — the census is 58 precedence rows plus
5 prelude methods, i.e. 63.

#### D246 — `docs/requirements/issue-0918-*.md`

**`docs/requirements/issue-0918-*.md`** (the minimal-core shard) — record the
widened scan root and the single shared `source_files()` definition.

#### D247 — `docs/requirements-traceability.md`

**`docs/requirements-traceability.md`** — add rows for the #959 items, which
have none today: one row per "what to do" clause (ledger ratchet, contextual +
prelude migration, browser derivation, promotion seed, closure honesty,
`data/README.md`), each naming its automated test and its manual confirmation.
`REQUIREMENTS.md` is generated and must not be hand-edited; it regenerates from
the shards above.

#### D248 — `data/README.md`

**`data/README.md`** (24 lines, documents only `data/benchmarks/`) — describe
`data/seed/`, `data/cache/`, `data/overrides/`, `data/parity/`, `data/meta/`,
`data/view/` and `data/training/`. This is #959 item 6 and is a single leaf.

---

### Plan 10 — B10 intent routing

#### D249 — `ROADMAP.md`

**`ROADMAP.md`** — there is no row for capability routing today (`grep -n
"R745\|R758" ROADMAP.md REQUIREMENTS.md` returns nothing but an unrelated line
number match). Add one to the current table:

> "| Capability routing generalizes across phrasing and language (#745, #758) |
> Measured, not asserted: `data/meta/capability-routing-ratchet.lino` records
> intents measured, languages measured, paraphrases per intent, cases passing,
> cross-tool misroutes and silent unknowns; the 420-case suite lives in
> `data/benchmarks/capability-routing/`. #745 and #758 stay open until
> misroutes and silent unknowns read 0. |
> [#745](https://github.com/link-assistant/formal-ai/issues/745),
> [#758](https://github.com/link-assistant/formal-ai/issues/758) |"

#### D250 — `VISION.md:337`

**`VISION.md:337`** — currently ends "Every interface now reads its multilingual
responses, concept table, tool registry, language-detection rules, prompt
patterns, and intent-routing rule book from the shared `data/seed/` directory".
"Intent-routing rule book" names
`data/seed/intent-routing.lino`, a 477-line phrase list. Replace, as leaf 20
proceeds, with: "…, and its capability-routing table — the decision from object
type, act and locus to capability, with the phrase book it replaced deleted —
from the shared `data/seed/` directory".

#### D251 — `ARCHITECTURE.md:160-165`

**`ARCHITECTURE.md:160-165`** — "The pipeline runs the same way for every
prompt … because the universal solver is intentionally domain-agnostic" is true
of the solver and silently untrue of `src/agentic_coding/planner.rs`, which is
845 lines of ordered arms. Add after it: "The agentic planner runs the same
capability-routing table: `src/capability_routing.rs` derives the object type,
the act and the locus, and `data/seed/capability-routing.lino` maps the triple
to a capability, which `src/agentic_coding/capability_router.rs` turns into
whichever tool name the connected client advertises. Arm order is data
(`data/seed/planner-precedence.lino`) for the arms that remain."

#### D252 — `docs/requirements/issue-0745-intent-routing-generalization.md`

**`docs/requirements/issue-0745-intent-routing-generalization.md`** *(new shard;
no `issue-0745-*` or `issue-0758-*` file exists today — `ls docs/requirements`
confirms)* — one requirement row per #745 clause (intent not phrasing; no
cross-tool misroutes; never silently UNKNOWN; language parity; CI variation
matrix) and per #758 clause (route by capability; full shared set;
grep/glob/list_dir never UNKNOWN; per-CLI matrix), each with an honest status
and a named test. The 2026-07-25 maintainer re-measurement is quoted in the
shard header so the closure history is visible in the requirement itself.

#### D253 — `docs/requirements-traceability.md`

**`docs/requirements-traceability.md`** — #745 and #758 have **no rows at all**
today. Add one row per requirement created above, each naming the automated test
(`tests/unit/issue_1138_capability_routing.rs::…`) and the manual confirmation
(the ladder transcript, the #447 screenshots). `REQUIREMENTS.md` is generated
and regenerates from the shard.

#### D254 — `experiments/issue_840_task_ladder/README.txt`

**`experiments/issue_840_task_ladder/README.txt`** — the paragraph "The
historical v0.303.0 measurement was 8/24 … The current committed `results.json`
is the strict all-green baseline" stays (it is honest and load-bearing); add
that the dataset now covers five languages and the seven #1087 frontier prompts,
and that the route-only variant runs in the ordinary CI stage.

#### D255 — `tests/fixtures/routing-parity.lino`

**`tests/fixtures/routing-parity.lino`** — plan 09 rewrites its header; this
plan adds the capability-routing invariants that must hold on both surfaces, so
a browser fix and a Rust fix cannot diverge again (#720 and #721 were reported
against the wasm build, not the CLI).

#### D256 — `data/meta/learning-frontier-language-gap.lino` and
`data/meta/language-adoption-ledger.lino`

**`data/meta/learning-frontier-language-gap.lino` and
`data/meta/language-adoption-ledger.lino`** — both record Spanish at
`total_prompts "7"` / `classes "2"`. After leaf 7 they must be re-derived by
running the frontier, not hand-edited, so the Spanish numbers reflect the seven
routed classes rather than the two concept-lookup ones.

---

### Plan 12 — B12 selection heuristics

#### D257 — `docs/requirements/issue-0491-least-action-continuation.md:10`

**`docs/requirements/issue-0491-least-action-continuation.md:10`** (R491-C1) — replace:

> Partial: task-decomposition and failure-driven recursive-execution tests cover binary
> structure and atomic leaves. Complete decomposition of arbitrary natural-language
> obligations remains open.

with:

> `selection_heuristics::balanced_split` folds the n-ary output of `split_once_checkable`
> into exactly two children per non-leaf node, preserving every segment and its byte spans,
> and reports the achieved imbalance. `data/meta/selection-heuristic-ratchet.lino` counts
> the remaining non-binary work-unit nodes over the benchmark corpus and turns strictly
> downward each release. Complete decomposition of arbitrary natural-language obligations
> remains open, and a single-clause moonshot is reported as underivable pending plan 01.

#### D258 — `docs/requirements/issue-0491-least-action-continuation.md:12`

**`docs/requirements/issue-0491-least-action-continuation.md:12`** (R491-C3) — replace:

> Partial: existing candidate selection uses size/step costs. A shared measured-resource
> optimizer across all reasoning and execution paths remains open.

with:

> `selection_heuristics::ActionCost` is the shared cost type — steps, code size,
> deterministic resource units and leaf count — read by every ranker through the registry,
> including the draft portfolio and discovered-algorithm ordering. Elapsed wall-clock time
> is deliberately excluded from every ranking key and reported beside it instead, because a
> key that depends on machine speed is not reproducible.

#### D259 — `data/meta/draft-portfolio-recipe.lino:61`

**`data/meta/draft-portfolio-recipe.lino:61`** — the `meta_selector` record's
`least_action_order "cost_size, cost_steps, draft_index"` becomes a reference so there is
one place the order lives:

> `least_action_order "see data/meta/selection-heuristics.lino heuristic_least_action key_order"`

#### D260 — `data/meta/task-decomposition-invariant.lino`

**`data/meta/task-decomposition-invariant.lino`** — the `binary` field's reader. Add:

> `binary_reader "src/selection_heuristics.rs"`

and correct the existing `source_reader "src/intent_formalization/requirements.rs"` to name
the field it actually reads (`source_integrity_reader`).

#### D261 — `docs/meta-algorithm.md:150-151`

**`docs/meta-algorithm.md:150-151`** — step 3 of the recursive core currently reads:

> 3. **Decompose the frame as a recursive, bounded work-unit tree** (downward pass),
>    stopping at `max_decomposition_depth`.

Replace with:

> 3. **Decompose the frame as a recursive, bounded, binary work-unit tree** (downward
>    pass): every non-leaf unit has exactly two children, so a complete layer has 1, 2, 4,
>    8, … leaves (#491, #453), the split preserves every segment and its byte spans, the
>    achieved imbalance is reported, and the recursion stops at `max_decomposition_depth`.

#### D262 — `docs/meta-algorithm.md`, new section after the budget-search section (`:790-871`)

**`docs/meta-algorithm.md`, new section after the budget-search section (`:790-871`)** —
`## Selection heuristics (issues #491, #901, #802, #453)`, describing the registry, the
three roles, the four seeded heuristics and the seams the core calls them from, with the
same "Running it" block convention the other sections use:

```sh
# Verify the selection-heuristic catalog still matches the live source:
cargo test --test unit specification::selection_heuristics -- --nocapture
```

#### D263 — `VISION.md:189`

**`VISION.md:189`** (Universal Problem-Solving Algorithm, step 7) currently reads:

> 7. **Draft experiments and selection**: build testable drafts by (a) reusing known parts,
>    (b) reasoning from rules, and (c) random or evolutionary search where structure and
>    compute budget allow; select only a test-passing draft and record why it won.

Append:

> Selection is a registry heuristic, not a fixed rule: candidates are ranked by least action
> — fewest steps, smallest code, fewest resources — only among those that already pass every
> declared check; a tie in which two candidates win on different dimensions is a
> contradiction resolved by a value on the range between them; and where a discriminating
> probe exists, the search eliminates hypotheses by refutation instead of sampling for
> confirmation.

#### D264 — `VISION.md:99-100`

**`VISION.md:99-100`** currently reads:

> Split a task in two, split the halves, and continue until each leaf is directly solvable.
> Do not rate a task before splitting it.

Append:

> This is enforced, not aspirational: every non-leaf work unit has exactly two children, and
> `data/meta/selection-heuristic-ratchet.lino` counts the nodes that do not yet comply and
> turns strictly downward each release.

#### D265 — `VISION.md:192`

**`VISION.md:192`** (step 10, Simplification: "Pick the smallest sufficient form") — append:

> "Smallest" is `ActionCost`, and "sufficient" is checked first: an incomplete answer is
> never a cheaper one.

#### D266 — `ROADMAP.md`

**`ROADMAP.md`** — the words `least action`, `TRIZ`, `2-4-6` and `moonshot` appear nowhere
in the file today. Add one row to the status table at `ROADMAP.md:423`:

> | Selection and splitting heuristics (least action, TRIZ contradictions, refutation-first hypothesis search, balanced binary splitting) | Delivered as registry heuristics for #1138 B12; the non-binary work-unit ratchet is measured and turns strictly downward | [#491](https://github.com/link-assistant/formal-ai/issues/491), [#901](https://github.com/link-assistant/formal-ai/issues/901), [#802](https://github.com/link-assistant/formal-ai/issues/802), [#453](https://github.com/link-assistant/formal-ai/issues/453) |

#### D267 — New shard `docs/requirements/issue-0901-triz-contradictions.md`

**New shard `docs/requirements/issue-0901-triz-contradictions.md`** — R901-1..R901-4. The ID
`R901` is unused (`grep -n "R901" REQUIREMENTS.md docs/requirements/` → 0):

```
| R901-1 | Represent a trade-off between two criteria as a link with a selection value on the range between them, expressed in integer basis points so the record stays hashable. | … |
| R901-2 | Derive the selection value from the requirement's own clauses or from a seeded record; an underivable contradiction is named and left unresolved, never defaulted. | … |
| R901-3 | Hold the inventive and separation principles as seed data discoverable from a trusted source, so the catalog can be forgotten and rediscovered to the same content hash. | … |
| R901-4 | Apply contradiction resolution wherever the meta algorithm chooses among candidates, and record the resolving link in the trace. | … |
```

#### D268 — New shard `docs/requirements/issue-0802-hypothesis-search.md`

**New shard `docs/requirements/issue-0802-hypothesis-search.md`** — R802-1..R802-4:

```
| R802-1 | Maintain an explicit set of live hypotheses and eliminate them by observation, never by impression. | … |
| R802-2 | Choose the next experiment by minimizing worst-case survivors, so each probe ideally halves the space. | … |
| R802-3 | Prefer a probe that attempts to refute the leading hypothesis over one that would only confirm it. | … |
| R802-4 | When survivors remain and no discriminating probe exists, report not-confirmed-not-refuted with the survivors and the blocker named. | … |
```

#### D269 — New shard `docs/requirements/issue-0453-moonshot-splitting.md`

**New shard `docs/requirements/issue-0453-moonshot-splitting.md`** — the bare ID `R453` is

**taken** (`REQUIREMENTS.md:1349`, the Wikontic pipeline requirement from #686), so the rows
use the suffixed form `R453-M1..R453-M4`:

```
| R453-M1 | Split every non-leaf task into exactly two children, preserving every segment and its byte spans, so a complete layer has 1, 2, 4, 8, … leaves. | … |
| R453-M2 | Report the achieved imbalance of every split; never discard a segment to balance a tree. | … |
| R453-M3 | A task that cannot be split at the text level is reported as underivable with its blocker named, never certified atomic. | … |
| R453-M4 | Draw splitting approaches from discovered sources, deduplicated to the first historical source of each idea. | Open — depends on plan 01's live concept lookup. |
```

#### D270 — `docs/requirements-traceability.md`

**`docs/requirements-traceability.md`** — add rows for R491-C1..C4 (currently absent; the
only `R491` row at `:634` is the unrelated Unlicense requirement from #834), R901-1..4,
R802-1..4, R453-M1..M4 and R1138-B12-1..4, each pointing at the specification file that
pins it.

#### D271 — `docs/requirements/issue-1138-bottleneck-audit.md`

**`docs/requirements/issue-1138-bottleneck-audit.md`** (shared shard; this plan owns the B12
rows):

```
| R1138-B12-1 | The selection heuristics live in the one method registry as link data, are never route targets, and their precedence is a data edit. | … |
| R1138-B12-2 | No heuristic may reorder an unsatisfying candidate above a satisfying one, and no ranking key may depend on wall-clock time. | … |
| R1138-B12-3 | An empty heuristic table falls back to the deterministic identity ordering and says so in the trace. | … |
| R1138-B12-4 | The count of non-binary work-unit nodes is measured, recorded and strictly decreasing. | … |
```


## Vision ↔ requirements drift

Five requirement blocks now say things the vision either does not say or
overstates. The vision must stay the north star; the reconciliation is to keep
the target sentence and add one measured clause beside it, never to lower the
target.

**R1085 (the links network is not the system that reasons).** `VISION.md:5`
asserts "The associative network is the AI", and `VISION.md:318-325` already
concedes that "`src/solver.rs`, `src/engine.rs` and `src/main.rs` never read the
doublets store" and that "The link-cli store is a write-behind projection".
R1085-2 (`REQUIREMENTS.md:2458`) records the partial repair: `src/seed_links.rs`
projects seed and routing documents into one links network at startup and
precedence, cues and intent routes are link queries, with "Migration continues
smallest-first" and 40 handlers still pending. **Corrected 2026-09-16: the census
is 58 `handler-precedence.lino` rows plus 5 unledgered prelude methods, i.e. 63,
of which 16 are migrated, 2 are justified-native and 40 are pending; plan 09
leaf 5 adds the five prelude rows, which raises the recorded pending count from
40 to 45 as a corrected undercount. "40 of 56" appears in no ledger.** **Reconciled wording for
`VISION.md:5`:** keep the sentence, and append — "Today the network is the
system of record for seed, routing, precedence and cue lookup
(`src/seed_links.rs`, `src/rule_interpreter.rs`); the solver's own working state
is still Rust structures, and 40 of 63 registry methods await migration (plan 09)
(`data/meta/handler-migration-ledger.lino`). Closing that is the direction, not
a claim about today."

**R1021 (full-range coding and contribution artifacts).** `GOALS.md:96` promises
the complete self-coding chain; R1021-22 (`REQUIREMENTS.md:2405`) says the
definition of done — "a pull request opened by Formal AI from a real `solve`
run, green without a human editing the branch" — is "**Not achieved**", and
R1021-14 (`:2397`) says the release-cycle requirement is "**Not delivered by a
`solve` run**". `ROADMAP.md:437` contradicts both. **Reconciled wording for
`ROADMAP.md:437`:** "Partial. The census (#673), gated promotion (#656), release
metric (#657) and the per-release self-development loop (#924) are delivered and
enforced. The chain's own definition of done — a pull request opened by a real
`solve` run — is not achieved (R1021-22)." `GOALS.md:96` is unchanged: it is the
target.

**R924 (self-development loop).** `GOALS.md:109` says the share starts "honestly
at 0% and ratchets upward"; the ledger's last row is 171 bp for the release and
389 bp trailing, and `REQUIREMENTS.md:2397` still quotes "0.00%". **Reconciled
wording:** GOALS keeps "starting honestly at 0% and ratcheting upward"; every
document stops quoting a figure and renders it from
`data/meta/self-hosting-ledger.lino`, with `README.md:1024` carrying the one
rendered figure the vision promises to publish.

**R710 (dropped requirements, dynamic coding discovery, repository completion).**
`VISION.md:231` and `:171` say the internet is a public database and the store
its cache. R710-D2/D3 (`REQUIREMENTS.md:1575-1576`) prove that for the coding
path at 20/20 and 20/20; R710-R3, R710-R8 and R710-R9 (`:1602`, `:1607`,
`:1608`) record that prerequisite discovery, open-ended authoring and complete
obligation execution "remain open"; and `src/solver.rs:874-884` shows the
universal loop performs no retrieval at all. **Reconciled wording for
`VISION.md:231`:** keep the paragraph and append — "This holds today inside the
how-to (#991), coding-discovery (#710) and research-learning (#873) paths, which
fetch, content-address and replay from `data/seed/sources-registry.lino`. The
universal loop's own external-search step records `policy:no_fetch_capability`;
giving it a real lookup is bottleneck B1 of issue #1138." The same clause serves
`VISION.md:171` and `GOALS.md:29-30`.

**R914 (coding first, discover enough knowledge).** R914-8 and R914-9
(`REQUIREMENTS.md:1945-1946`) now read "Implemented"; the traceability rows for
the same IDs (`docs/requirements-traceability.md:741-742`) read "not delivered".
Whichever is right, the vision's claim is stronger than both: `VISION.md:134`
says "A prompt should trigger enough data collection and reasoning to justify
the response for that prompt." **Reconciled wording:** the two traceability rows
move to "delivered 2026-09-15; PR #888" with their suites named, and
`REQUIREMENTS.md:1945` gains the bound already true in the code — "Unknown
source knowledge still fails honestly rather than becoming a built-in answer"
is there; add "and a word outside the 82 seeded structural meanings reaches
`NoLookup` (`src/coding/concept_discovery.rs:87`)".

**The one thing the vision does not yet say.** The 2026-09-14 instruction — *"the
goal is not to know everything in advance, the goal to know how to get know
anything when it is needed"* — is the doctrine PR #888 was built against and it
appears nowhere in `VISION.md`, `GOALS.md` or `docs/architect-notes/`. Adding it
(D14, D22, D122) is the single highest-value vision edit in this plan, because
every bottleneck in #1138 is a restatement of it.

## Generation and validation

### How the two documents are produced today

`REQUIREMENTS.md` **is generated.** `scripts/assemble-requirements.rs`
concatenates the 111 shards in `docs/requirements/` in a total order read
entirely from file names (`preamble-` first, `issue-NNNN-` by zero-padded number
then slug, `doctrine-` last; `scripts/assemble-requirements.rs:14-21`,
`:43-65`), emits the "never edit this file" banner (`:41`), rewrites
shard-relative links to repository-root links (`:515-528`), and separates
sections by exactly one blank line (`:499-512`). `--write` rebuilds, `--split`
re-shards, and the bare invocation checks the file is current (`:23-26`).
`CONTRIBUTING.md:966` documents the workflow. A hand edit is detected by
`check_requirements_document` (named in `issue-710/plans/01-…md:90-93`).

`docs/requirements-traceability.md` **is hand-written.** No script generates it.
The only mention of the path outside tests is
`scripts/assemble-requirements.rs:519-522`, which merely knows how to rewrite a
link to it. Its own header says so: "CI enforcement of this table's freshness is
tracked by E105 (#957); this is the initial data population, not an automated
generator wired into CI yet" (`:15-17`). #957 is still open.

Validation today is **per-issue and partial**:

| Gate | What it enforces | Coverage |
| --- | --- | --- |
| `tests/unit/docs_requirements_issue_1021.rs:22-43` | R1021 IDs run contiguously from 1, derived from the shard | R1021-* only |
| `:58-67` | every R1021 ID has a row in both documents | R1021-* only |
| `:140-234` | specific rows keep specific verdicts ("not delivered", "not yet confirmed", "measured ") | 6 rows |
| `:298-360` | every test a traceability row cites by `::` name exists on disk | whole table |
| `docs_requirements_issue_909.rs:126`, `_891.rs:28-30`, `_893.rs:29-32`, `_894.rs:227-229` | per-issue row existence | 4 issues |
| `tests/unit/docs_requirements/benchmarks.rs:12-70` | `docs/benchmarks.md` table rows and `VISION.md` sentences equal the latest ledger row per suite, and both name the latest date | 2 documents, 9 suites |
| `tests/unit/docs_requirements_issue_710.rs:91-126` | 32 R710-NN rows in `REQUIREMENTS.md`, 31 `works-now`; `ROADMAP.md` contains "31 work now and 1 is superseded" | 1 issue |
| `tests/unit/architect_notes.rs:28-170` | VISION quotes nine architect clauses; notes carry ISO dates and index rows; five documents may not demand Rust shrink; GOALS may not rate a task | phrase-level |
| `scripts/check-debt-ratchet.rs` | four measured ceilings in `data/meta/debt-ratchet.lino` move down only | numbers, not prose |
| `scripts/check-minimal-core-boundary.rs` | `data/meta/core-boundary-ledger.lino` file/line ceilings | numbers, not prose |
| `scripts/check-worker-line-budget.rs` | per-module JS ceilings under `data/meta/worker-line-budget/`, 3,000-line end-state target (`:35`) | numbers, not prose |

Net effect: 49 `docs_*` files carry 360 prose assertions that pin *phrases*, and
almost nothing pins *numbers*. That is exactly backwards, and it is why every
finding in the "stale number" category exists while the phrase-level pins have
held.

## Solution options

Three ways to stop the findings above recurring. The first two differ in how much
is generated; the third differs in what is kept at all.

This is what #1089 (E111, open) and #958 (E106, open) ask for, and what
`REQUIREMENTS.md:2472` (R1085-16) records as an open sub-issue.

### Option A — render one status surface from the ledgers
Add `scripts/render-status.rs`, modelled exactly on
`scripts/assemble-requirements.rs`: it reads the ledgers, writes **one generated
file, `docs/status.md`**, and in check mode fails when it is out of date.

> **reconciled: this plan's draft wrote marked regions into five narrative
> documents. Plan 00 §4.5 asks for one file that VISION, ROADMAP and REQUIREMENTS
> link to instead of restating, and this plan's own risk 3 asks the same question
> — a generated region inside `VISION.md` violates `CONTRIBUTING.md:970-982`'s
> rule that a list belongs in a file of its own. Now: one `docs/status.md`, which
> every narrative document links; the only in-place regions retained are the two
> whose existing pin tests require the number to stand in the document
> (`docs/benchmarks.md` and `README.md`), fed by the same script (plan 00 §9 R18).**

The two retained regions are delimited the way generated blocks usually are, so
the surrounding prose stays hand-written:

```text
<!-- status :begin benchmarks (illustrative spacing; generated files omit it) -->
… generated table …
<!-- status :end benchmarks (illustrative spacing; generated files omit it) -->
```

Inputs, all already committed:

| Source | Fields consumed | Regions it feeds |
| --- | --- | --- |
| `data/benchmarks/external-results.lino` | `suite`, `date`, `slice`, `passed`, `total`, `solver_version` | `benchmarks` (generated into `docs/status.md`; the `docs/benchmarks.md` region is the one retained in place) |
| `data/meta/self-hosting-ledger.lino` | last `release` block: `tag`, `percentage_basis_points`, `trailing_percentage_basis_points`, `target_percentage_basis_points` | `self-hosting` (generated into `docs/status.md`; the `README.md` region is the one retained in place) |
| `data/meta/debt-ratchet.lino` | each `ceiling` `measure`/`value` | `debt` (generated into `docs/status.md`) |
| `data/meta/core-boundary-ledger.lino` | `source_file_count_max`, `outside_core_lines_max` | `debt` |
| `data/meta/handler-migration-ledger.lino` | counts of `status migrated` / `status pending` | `debt` |
| `data/meta/ladder-ratchet.lino` | `leaf_nodes_selected`, `leaf_nodes_passing`, `deepest_passing_level`, `recorded_on` | `ladder` (generated into `docs/status.md`) |
| `data/meta/worker-line-budget/*.lino` | per-module `ceiling` | `debt` |

Proposed new ledger for the one status dimension that has no ledger — per-requirement status:

```text
requirement_status_ledger
  generated_by "rust-script scripts/render-status.rs --write"
  requirement
    id "R710-D2"
    shard "docs/requirements/issue-0710-dynamic-coding-discovery.md"
    verdict "implemented"            # implemented | partial | not-delivered | superseded | withdrawn
    delivered "2026-09-15"
    pull_request "888"
    issue "710"
    tracker ""                        # the OPEN issue that owns the remainder, empty when none
    automated_test "tests/unit/coding_discovery/mod.rs::humaneval_first_twenty"
    manual "not yet confirmed"
```

`docs/requirements-traceability.md` then becomes a fully generated projection of
that ledger (ID, shard, verdict, delivered, automated test, manual), and the
`Line` column dies with D97. Each shard's status cell is still hand-written
prose — the ledger carries the machine-checkable verdict, the shard carries the
explanation — and a gate asserts the two agree on the verdict word.

Gates to add, all cheap `rust-script` checks registered in
`data/meta/ci-gates/`:

1. `check-requirement-status.rs` — every ID in `REQUIREMENTS.md` has a ledger
   entry and a generated traceability row; every ledger entry names an existing
   shard; the shard's verdict word matches the ledger's. Closes D95, D153.
2. `check-issue-citations.rs` — every `tracked in #N` / `tracked by #N` /
   `remains open … #N` in the twelve documents names an issue that is open in a
   committed `data/meta/issue-state.lino` snapshot refreshed by a scheduled job.
   Closes D154 and the 34 closed-tracker findings.
3. `render-status.rs --check` in the Lint job — closes every stale-number
   finding (D155) by construction.
4. Widen `tests/unit/docs_requirements/benchmarks.rs` from two documents to five,
   and assert that any line containing a curated pass ratio also contains an
   upstream ratio within the same paragraph. Closes D23, D29, D46, D55, D60, D85.

### Option B — keep hand-written tables, add only the parity gate
 Cheaper: only
gate 1 above, plus a `not yet confirmed` counter in the traceability header.
Rejected as the primary path because it fixes D95 and leaves every stale-number
finding to recur; it is, however, the right first commit if the render script
slips.

### Option C — delete the traceability table; move delivery, test and manual into the shards
 Rejected: `tests/unit/docs_requirements_issue_1021.rs:1-8` records
why the table exists ("a requirement without a row is a requirement nobody can
check later"), and #1090 is the open issue that owns finishing rather than
retiring it. Retiring is allowed only through #1090, explicitly, with the header
marked aspirational — which is option C's honest form and should be offered to
the maintainer as the alternative to filling 743 cells.

## Decision

**Option A is selected**, with Option B adopted as its first commit if the render
script slips and Option C offered to the maintainer as the honest alternative to
filling 743 manual-confirmation cells.

Reasons: only Option A closes the whole "stale number" category structurally
rather than one finding at a time — every one of the 24 stale-number findings is
the same defect, a number copied beside the thing it counts, and a generated
surface makes that defect unrepresentable. Only Option A gives the four
competing "single source of truth" claims (D26) a mechanical adjudicator: the
ledger owns numbers, and the documents link to them. And only Option A can be
run in `--check` mode in CI, which is what turns a finding list into a gate.

Rejections: **Option B** is rejected as the primary path because it fixes D95
and leaves every stale-number finding to recur — it is a parity gate over a
hand-written table, and a hand-written table is the thing that rotted.
**Option C** is rejected as a unilateral decision:
`tests/unit/docs_requirements_issue_1021.rs:1-8` records why the table exists
("a requirement without a row is a requirement nobody can check later"), and
#1090 is the open issue that owns finishing rather than retiring it. Retiring is
allowed only through #1090, explicitly, with the header marked aspirational —
which is Option C's honest form and is offered to the maintainer rather than
taken here. That is why #1090 is **not** in plan 13's `Closes` list.

## Architecture

The mechanism is three files and four gates, and every one of them is modelled on
something this repository already runs, so nothing here is a new kind of thing.

### `scripts/render-status.rs`

Modelled exactly on `scripts/assemble-requirements.rs` — same `--write` /
`--check` / bare-invocation shape, same banner, same total order read from file
names. It reads the seven ledgers tabulated below and writes **one** file,
`docs/status.md`, plus the two in-place regions whose pin tests require the
number to stand in the document. In `--check` mode it fails when any of the
three is out of date, which is what `render-status.rs --check` in the Lint job
means.

### `data/meta/requirement-status-ledger.lino`

The one status dimension with no ledger today: per-requirement status. Its schema
is below. `docs/requirements-traceability.md` becomes a generated projection of
it (id, shard, verdict, delivered, automated test, manual), and the `Line` column
dies with D97. Each shard's status cell stays hand-written prose — the ledger
carries the machine-checkable verdict, the shard carries the explanation — and
`check-requirement-status.rs` asserts the two agree on the verdict word.

### `data/meta/issue-state.lino`

A committed snapshot of every issue's open/closed state, refreshed by a scheduled
job, so `check-issue-citations.rs` can compare a "tracked in #N" citation against
something. Same discipline as the benchmark ledger: the snapshot goes stale
between refreshes, and the gate's own message says so rather than implying
freshness it does not have.

### Interaction with #1089's own target. #1089 asks for `ls tests/unit | grep -c
docs_` ≤ 5, down from 48; it is 49 today. Options A collapses most of them:
once numbers are generated and verdicts are ledger-backed, the per-issue prose
pins become redundant and can be retired in one commit per issue, keeping only
(a) the generated-region freshness check, (b) the architect-clause pins in
`tests/unit/architect_notes.rs`, (c) the benchmark ledger parity test, (d) the
requirement-status parity test, and (e) the issue-citation gate.

## Docs to update

This plan has no such section, and that is deliberate: it **is** the docs
authority. Rows D1-D155 are the statements that are wrong today; rows D156-D275
are the statements that become wrong when a plan lands, moved here verbatim from
plans 01-10 and 12 by the 2026-09-16 reconciliation (plan 00 §8). Every other
plan's "Docs to update" section is now a pointer to its rows here.

## Implementation leaves

Ordered so each leaf consumes the previous one. Every leaf is small enough to
commit alone.

### Mechanism first (unblocks the rest)

- [x] L1 Add `data/meta/requirement-status-ledger.lino` with the schema above,
      seeded from the 1,030 IDs in `REQUIREMENTS.md` and the 805 existing
      traceability rows (225 new entries).
- [x] L2 Add `scripts/render-status.rs` with `--write` / `--check`, the region
      markers, and the seven ledger inputs; register it in
      `data/meta/ci-gates/check-status-render.lino`. (already delivered before this audit: the script carries both modes, the `replace_region` markers, and the seven inputs named in its header, the `check_status_render` gate runs `--check`, and this session both modes ran green — "rendered 3 status surfaces" and "status surfaces are current")
- [x] L3 Add `scripts/check-requirement-status.rs` (gate 1) and register it.
- [x] L4 Commit `data/meta/issue-state.lino` and add
      `scripts/check-issue-citations.rs` (gate 2) plus the scheduled refresh job.
- [x] L5 Widen `tests/unit/docs_requirements/benchmarks.rs` to `ROADMAP.md`,
      `ARCHITECTURE.md` and `README.md`, and add the curated-beside-upstream
      assertion.
- [x] L6 Add the traceability-row rule to `CONTRIBUTING.md:966` and the
      "Documents pinned by tests" table (D68, D71).
- [x] L7 Add `docs/requirements-traceability.md` to the ROADMAP Verification
      Contract (D48).

### VISION.md

- [x] L8 D14 — add `docs/architect-notes/2026-09-14-know-how-to-get-to-know-anything.md`,
      index it, and quote it in "The Goal Is The Meta Algorithm".
- [x] L9 D10, D11, D12 — add the one-line current-state clause to the retrieval
      and formalization sentences. (done 2026-09-18: B1 landed, so all three clauses state the delivered live lookup through `src/concept_lookup.rs` instead of the planned `NoLookup`/`policy:no_fetch_capability` wording)
- [x] L10 D9 — add the PR #888 paragraph to "Current Direction". (done 2026-09-18: dated paragraph added after the B4 paragraph, with the B1 closure stated)
- [x] L11 D1, D2, D3, D4, D7, D8 — retire six closed-tracker citations. (done 2026-09-18: #280/#702/#656/#657/#704/#706/#707/#558 restated as deliveries with PRs; #705 named as the open frontier item)
- [x] L12 D5, D6 — co-cite upstream numbers on the curated sentences. (done 2026-09-18: benchmark paragraph rewritten with the 2026-09-18 full-slice rows beside every curated number)
- [x] L13 D13 — add `data/seed/sources-registry.lino` to the seed inventory. (done 2026-09-18: added to the inventory sentence; no count pinned, the registry holds 15 kinds and grows)

### GOALS.md / NON-GOALS.md

- [x] L14 D22 — add the live-concept-lookup Reasoning Goal. (done 2026-09-18: goal added to Reasoning Goals, stating the 2026-09-14 doctrine and the delivered `src/concept_lookup.rs` path)
- [x] L15 D15, D16 — remove the quoted share figure; point at the rendered one. (done 2026-09-18: the self-authoring-share goal now cites `data/meta/self-hosting-ledger.lino`/`docs/status.md` by reference; D16's ROADMAP half is leaf L23)
- [x] L16 D17 — qualify the `.lino` reconstruction claim with the `#[ignore]`d
      round-trip. (done 2026-09-18: goal qualified; no file count pinned, the census test derives it)
- [x] L17 D18, D19, D20, D21 — four current-state clauses; retire the graduated
      `#[ignore]` sentence. (done 2026-09-18: verified zero tracked-requirement ignores remain; the two remaining specification ignores are network-gated)
- [x] L18 D25 — record PR #644's stale state beside the #483 exception. (done 2026-09-18: NON-GOALS states the exception is declared, not delivered; also closed unassigned D24 with the post-B1 boundary wording)

### ROADMAP.md

- [x] L19 D47 — add the "Issue #710 Dynamic Coding Discovery (PR #888)" section
      with before/after numbers and plans 06/07's open items. (done 2026-09-18: section added before the Verification Contract; 1/20 before vs full-slice 14/164 and 49/500 after; plans 06/07 named as the open resume point)
- [x] L20 D27, D28, D29, D50 — replace four benchmark statements with generated
      regions. (done 2026-09-18: pillars 25/26, the intro narrative, and ARCHITECTURE §16 restated from the 2026-09-17/18 full-slice ledger rows with the first-20 rows kept as controls; D27/D28 were already fixed in HEAD)
- [x] L21 D37, D38, D45, D30, D41, D42 — replace six handler/debt/ladder numbers
      with generated regions. (done 2026-09-18: 49/19,741 core-boundary pair, 65/130 ladder at v0.320.0 with the ledger's own issue_847 pointer, worker 35 modules/30,175 ceiling sum/3,000 target, and the corrected debt-ratchet triple cited as current ceilings)
- [x] L22 D31, D32, D33, D34, D35, D40, D43, D44 — eight closed-tracker
      corrections, including the #658/#668 mix-up. (done 2026-09-18: #658 closed PR #691; the six delivery issues #665-#670 named individually and all stated open; #278/#283/#301/#279 as deliveries; #990/#991 closed with PRs; E69-E77 closed; #1085 heading cites PR #1086)
- [x] L23 D39 — rewrite the self-coding-chain row to agree with R1021-14/22. (done 2026-09-18: row reads Partial with the R1021-22 "not achieved" definition of done)
- [x] L24 D26, D49 — scope the authority sentence and the "no epic remains open"
      statement. (done 2026-09-18: header now scopes authorities per dimension; "specifically" added at both #244 statements)
- [x] L25 D36 — date the eighth-pass table in its heading. (already satisfied: the table sits under "## 2026-07-14 Requirement-Status Audit" and the ninth-pass section marks it as the historical record — no edit needed)
- [x] L26 D46 — downgrade the D5.4 row until L20 lands. (done 2026-09-18: row upgraded to Delivered with the five enforced surfaces named; docs_benchmarks ratio test is the enforcement evidence)

### ARCHITECTURE.md

- [x] L27 D51, D52, D53 — three generated numbers. (done 2026-09-19: the tree's numbers were already restated — WASM crate 2,156 lines re-derived and exact, 35 worker modules quoted from `data/meta/worker-line-budget/`; this leaf re-derived the 35 ceilings' sum and corrected the quoted 30,175 to the live 30,179, and the stale "~90,000-line core" figure is deleted rather than rendered, as D53 allowed)
- [x] L28 D56, D57, D54 — replace the stale open-batch and gap lists with the
      live open set. (done 2026-09-19: #658 is stated closed 2026-07-18 by PR #691 with the shrink-only ratchet carrying absorption; the false "two later batches are open" is replaced by closed E37-E55/E56-E68 plus the E69-E77 batch added with its delivered PR list and a pointer to ROADMAP's per-issue record of E78-E117; the six-closed-issues gap list is replaced by live trackers #959/#1087/#1088/#1089/#1090, #710 with PR #888, #705, and #665-#670)
- [x] L29 D55, D60 — co-cite upstream numbers. (done 2026-09-19: both were already restated — the 10-case industry slice carries the 13/13-floor pointer plus the upstream pairs HumanEval 14/164 full-slice `--online` and MBPP 49/500 cold-offline with the first-20 controls, pointing at `docs/status.md` and `docs/benchmarks.md`; the 1,440/1,440 ratchet carries "the upstream instructed-editing analogue, CoEdIT, scores 0/20")
- [x] L30 D59 — qualify step 4 and add the coding-discovery row. (done 2026-09-19: step 4 no longer claims plain "Implemented" — it states surface-form anchoring plus the landed issue #1138 B4 concept-graph formalization whose unresolved surfaces become explicit needs, matching the R1138-B4 ledger rows — and row 4b names the coding-discovery path `src/coding/concept_discovery.rs` over `src/concept_lookup.rs` and `data/seed/sources-registry.lino`)
- [x] L31 D58, D61, D62 — rename §16, refresh the requirement-range examples,
      scope the authority sentence. (done 2026-09-19: §16 is retitled "Audit History And Current Gaps"; the examples already reached R710-01…R710-32 and this leaf extended them with R710-D1…R710-D17 and R1138-B1…R1138-B12, both verified present in `REQUIREMENTS.md`; the authority sentence scopes this document to "the structure and wiring" with status, per-requirement status, and numbers pointed at ROADMAP, REQUIREMENTS, and the ledgers)

### README.md / CONTRIBUTING.md

- [x] L32 D63 — publish the rendered self-hosting figure and trend. (done 2026-09-18: the Self-Development Share section now points at the generated `docs/status.md` figure and the README status region, and states the current 171/389/267 values as read from that render)
- [x] L33 D64 — add the rendered "Measured today" benchmark block (closes #958's
      README half). (done 2026-09-18: "Measured Today" section added with the per-suite latest ledger rows; a future generated region can replace the hand-written table)
- [x] L34 D65, D66 — fix the benchmark-gap sentence; link the traceability table. (done 2026-09-18: the floored suites are named; the documentation map paragraph links `docs/requirements-traceability.md`)
- [x] L35 D67, D123 — state where the VISION-precedence rule is enforced, now
      that gate 2 exists.
- [x] L36 D69, D70 — fix the R536 reference and the repository map. (done 2026-09-18: D70 landed in the Project Structure tree; D69 resolved by L39 — with the shard's R536 row fixed, the `CONTRIBUTING.md` pointer to it stands)

### REQUIREMENTS.md — via shards only

- [x] L37 D72, D73, D74, D75 — rewrite R67 in
      `docs/requirements/issue-0012-holistic-vision-requirements.md`. (done 2026-09-18: R67 upgraded to Implemented for the B1 live lookup; stale line spans dropped; #843 closed as "removed by ... (PR #853)")
- [x] L38 D76, D77, D81 — R914-6/8/9 in
      `issue-0914-vision-implementation-planning-coding-first.md`. (done 2026-09-18: D81 renders the migration ledger — 16 migrated, 55 pending — with the shrink-only ceiling named; D76/D77 shard text verified correct, their traceability halves are L47)
- [x] L39 D82 — R536 in
      `doctrine-standing-doctrine-compiled-logic-interfacing-only-javascript-2026-08-04.md`. (done 2026-09-18: 35 modules / 30,179 lines dated 2026-09-18, ceilings sum re-stated, #658 closed with PR #691, 3,000-line target named)
- [x] L40 D83 — R1021-14 in `issue-1021-full-range-coding-and-contribution-artifacts.md`. (done 2026-09-18: already satisfied — the shard row already reads "not delivered by a `solve` run" without the stale 0.00% figure)
- [x] L41 D84, D85, D86 — R1085-9/10/14-17 in
      `issue-1085-the-links-network-is-not-the-system-that-reasons.md`. (done 2026-09-18: D84 names all three mechanisms with #1095/#1096 closed by PR #1086 and the predating ratchet record; D85 upgraded to Delivered after the ROADMAP #922 site gained its inline upstream pair; D86 names #1087–#1090 in the four rows)
- [x] L42 D89, D90, D91 — cross-reference the open R710-R rows in
      `issue-0710-repository-and-retention-continuation.md`. (done 2026-09-18: already satisfied — the ROADMAP #710 section names the open prerequisite-discovery items (D89, via D47), the self-coding row reads Partial (D90, via D39), and R710-R9 already states obligation-ledger delivery with `src/obligation_ledger.rs` and its tests present)
- [x] L43 D92 — widen R710-D15 in `issue-0710-dynamic-coding-discovery.md`. (done 2026-09-18: requirement widened to ROADMAP/ARCHITECTURE/README and the pin extended in `tests/unit/docs_benchmarks.rs` (before: 4/4 pass without it; after: 4/4 pass with it); stale `docs_requirements::benchmarks` path corrected)
- [x] L44 D94 — co-cite upstream in R922-3
      (`issue-0922-method-learning-from-experience.md`). (done 2026-09-18: the dated full-slice upstream pair now sits inside the row beside the curated 13/13)
- [x] L45 Run `rust-script scripts/assemble-requirements.rs --write` once, after
      L37-L44. (done 2026-09-18: rebuilt from 123 shards; R67 and the other shard fixes verified present in REQUIREMENTS.md)

### docs/requirements-traceability.md

- [x] L46 D95 — add the 225 missing rows, generated from the L1 ledger, grouped
      by the 23 shards listed in D95. (done 2026-09-18: the ledger known 298 missing rows were appended, grouped by shard in natural ID order, each `none recorded` / `not yet confirmed`)
- [x] L47 D76, D77, D78, D79, D80, D98 — correct the six wrong/untracked rows. (done 2026-09-18: all six checked against the rebuilt table — the stale ones re-cited to live evidence, R67 among them now citing the two `source_cache` tests; `check-requirement-status` parity holds)
- [x] L48 D97 — replace the `Line` column with the shard path. (done 2026-09-18: the shard path replaced the line number for all 848 existing rows, taken from the ledger)
- [x] L49 D99, D100, D103 — fix three rows that cite the wrong evidence. (done 2026-09-18: re-cited; R1085-10 now names the two `docs_benchmarks` tests, D100 was already satisfied)
- [x] L50 D101, D96, D102 — refresh the header with the live counts and add the
      R1137 / R1085-14-17 rows. (done 2026-09-18: header rebuilt around the Shard column with the 1,146-row and `not yet confirmed` counts dated to the rebuild; the R1137 and R1085-14…17 rows present)

### docs/

- [x] L51 D109, D110, D111, D112, D113, D114 — `docs/meta-algorithm.md`. (done 2026-09-18: counts and the four numeric-authority ledgers recorded, the "What none of these loops does yet" section added, the fallback-cost paragraph extended, and the discovery step rewritten to `concept_discovery::discover_with_lookup` with B1's live lookup)
- [x] L52 D105, D108 — `docs/benchmarks.md` cross-references. (done 2026-09-18: the pin sentence now names the VISION/ROADMAP/ARCHITECTURE/README coverage and the `curated_pass_ratios` test; D105/D108 were already satisfied)
- [x] L53 D115, D116 — `docs/philosophy.md`. (done 2026-09-18: the "Knowing how to get to know" section quotes the 2026-09-14 note and names the B1/B4 deliveries; VISION.md links the Present boundary section)
- [x] L54 D117, D118, D119, D120 — `docs/USER-JOURNEYS.md`. (done 2026-09-18: F2/F4 delivered-by citations fixed, F4 moved into the supported section with its PR evidence, new J12 and J13 journeys, matrix rows updated)
- [x] L55 D121, D122 — `docs/architect-notes/`. (done 2026-09-18: the frozen-at-date rule added to the folder README; the 2026-09-14 note's Source line records where the verbatim instruction is quoted — no 2026-09-15 note was fabricated, since no verbatim architect quote of it exists)

### Plans and case studies

- [x] L56 D124, D125, D126 — `issue-710/plans/README.md` resume section and
      status column, post-merge. (done 2026-09-18: "How to resume" rewritten for the merged tree and the plans-table status cells record the merge)
- [x] L57 D127, D128, D129, D130, D139, D141 — add dated closing-log lines rather
      than editing the append-only entries. (done 2026-09-19: the closing log covers D127–D130 by naming the owning ledgers; the plan 06 baseline sentence and the plan 07 110-shards line now carry their dates)
- [x] L58 D131, D132, D133, D134 — reconcile plan 01's B9/A16/B6/C6 rows. (done 2026-09-19: A16 marked done and enforced, B6 named partial with the #1138 B2 hand-off, B9 partial-by-design reconciled with plan 03 L7, C6 gained #447)
- [x] L59 D135, D137 — tick the four post-merge boxes in plans 03 and 04. (done 2026-09-19: all four ticked with the merge-SHA comment)
- [x] L60 D136, D138 — amend plan 04's tally and label the 0.15 % measurement. (done 2026-09-19: the tally is scoped to R710-01…32 with the continuation named, and the 0.15 % figure is labeled the pull-request-range measurement with the ledger's release figure beside it)
- [x] L61 D140 — name #710 and #1138 as the trackers in plans 06 and 07. (done 2026-09-19: both headers name issue #710 and their #1138 B3/B6 restatements)
- [x] L62 D143, D144, D145, D146, D147 — five `kernel-ratchet` dangling
      references, including the one in `data/meta/handler-migration-ledger.lino`. (done 2026-09-19: the four live references point at `data/meta/debt-ratchet.lino` naming the 2026-09-12 rename, the solution-plan D1.4 carries its withdrawal note, and the handler-migration-ledger line was already corrected by plan 09 leaf 5; D146's ceiling count was re-derived at twelve and dated rather than pinned)
- [x] L63 D148, D149, D150, D151 — `issue-1085/plans/README.md`. (done 2026-09-19: both stale counts re-rendered and dated, #1101 removed from the stay-open table with a note, the order of work marked complete, and a closing log added)
- [x] L64 D152 — disambiguate the eleven `#957`-namespace requirement IDs. (done 2026-09-19: the eleven IDs prefixed `#957-audit` with a legend sentence pointing at the audit NDJSON)

### Test updates the above forces

- [x] L65 `tests/unit/docs_requirements_issue_710.rs:122` — the pinned substring
      "31 work now and 1 is superseded" must survive L23's rewrite of
      `ROADMAP.md:434`, or the test changes in the same commit. (done 2026-09-19: the substring survives the rewrite and the suite passes — no test change needed)
- [x] L66 `tests/unit/docs_requirements_issue_1021.rs:147-160` — R1021-14/22 must
      keep reading "not delivered" / "not achieved" after L40. (done 2026-09-19: both rows keep their honest wording and the assertions pass)
- [x] L67 `tests/unit/architect_notes.rs:46-68` — the nine quoted clauses must
      survive L8-L13; adding a note also requires an index row (`:116-126`). (done 2026-09-19: no clause was removed and no note file was added, so the index stayed complete; `architect_notes` passes)
- [x] L68 `tests/unit/architect_notes.rs:137-155` and `:175-201` — the forbidden
      phrases and the R1085-1 "superseded"/"kernel" words must survive L41 and
      L62. (done 2026-09-19: the R1085-1 row still reads "Superseded" with "kernel" in both REQUIREMENTS.md and its shard; `architect_notes` passes)
- [x] L69 `tests/unit/docs_requirements/benchmarks.rs` — rewritten by L5; the
      `VISION.md` needles at `:57-60` and `:69` change shape when L10/L12 land. (done 2026-09-19: the ledger-derived checks — including the L43 widening into ROADMAP/ARCHITECTURE/README — pass in the docs_ run)
- [x] L70 `tests/unit/docs_requirements_issue_922.rs` — its "13/13 token
      retained" assertion is cited by the R1085-10 traceability row (D103) and
      must be replaced, not merely satisfied. (done 2026-09-19: the R1085-10 row cites the two `docs_benchmarks` tests as its automated evidence and the issue_922 suite passes unchanged in the docs_ run)
- [x] L71 `tests/unit/docs_requirements_issue_540.rs:64-85`,
      `_issue_451.rs:21-34`, `_issue_526.rs:28-52`, `_issue_890.rs:28-40`,
      `_issue_917.rs:28-40`, `_issue_922.rs:23-34`, `_issue_923.rs:26-38`,
      `_issue_563.rs:49-51`, `_issue_686.rs:48-50` — nine per-issue suites read
      `VISION.md` / `ARCHITECTURE.md` / `ROADMAP.md` / `docs/meta-algorithm.md`
      and will fail on a section rename; re-point them in the same commit. (done 2026-09-19: no pinned section name was renamed, so no re-pointing was needed; all nine pass in the docs_ run)
- [x] L72 `tests/unit/specification/{meta_algorithm,agentic_meta_algorithm,dreaming_meta_algorithm,document_verification_meta_algorithm,market_price_verification_meta_algorithm,links_network_terminology_meta_algorithm}.rs`
      — six grounding suites assert `docs/meta-algorithm.md` contains specific
      needles; L51 must keep them or update them. (done 2026-09-19: every docs-needle test passes with L51's edits in place. The dreaming recipe's `test_file` pointer, orphaned by the L76 gate collapse, was repointed to `tests/unit/docs_requirements/issue_540.rs` in the same batch; when the solver_search module split left `target_marker_positions` and `parse_ops` in the new `src/solver_search/problem.rs`, the budget-search recipe's two `source_file` pointers were repointed to that file and both grounding suites run green)
- [x] L73 `tests/issue_885_docs.rs:132,150-151` — README and VISION must keep
      linking `docs/philosophy.md` through L53. (done 2026-09-19: both links survived this plan's edits to README and VISION and every philosophy needle is intact; `cargo test --offline --test issue_885_docs` passes 11/11)
- [x] L74 `tests/issue_973_solve_flags.rs:32-37,224-234` — CONTRIBUTING, README,
      ARCHITECTURE, ROADMAP and GOALS are read for the solve-session policy; L6
      and L36 must not remove those needles. (done 2026-09-19: L6 and L36 touched none of the scanned roots' solve-session text; the CONTRIBUTING needles are intact and `cargo test --offline --test issue_973_solve_flags` passes 4/4)

### Leaves added by the 2026-09-16 reconciliation

Plan 13's coverage table lists #949 and #1089 as fully closed by this plan, and
no leaf delivered either. An issue in a `Closes` list with no leaf behind it is
the failure mode #710's audit named, so the leaves are added here rather than the
issues quietly downgraded (plan 00 §8).

- [x] L75 **Language-parity lint (#949 / E97; carry-over C49, C61).** Add
      `scripts/check-language-parity.rs` and
      `data/meta/ci-gates/check-language-parity.lino`: every meaning that
      declares a `lexeme` for one of en/ru/hi/zh/es must declare one for all
      five, or carry an explicit dated `uncovered_behavior` row naming the gap.
      Record the measured first value as the ceiling in
      `data/meta/debt-ratchet.lino` (strictly downward, added after plan 09
      leaves 1-5 so it enters through the strict checker). Then fix D49 and D61:
      Spanish is `status partial` in three places and absent from a fourth, and
      the record must say so once, in `docs/status.md`.
      Response templates are intentionally measured as a distinct data shape:
      `tests/unit/issue_1138_self_use_concept_lookup.rs` derives every missing
      `(intent, language)` pair from `multilingual-responses.lino` and requires
      exact dated coverage in `data/meta/response-language-parity-debt.lino`.
      Its ceiling starts at 93 incomplete intents (81 lack only Spanish and 12
      currently have only English), rejects growth, and falls only when actual
      localized response data is added; it does not invent translations to make
      the test green.
- [x] L76 **Collapse the gate ecosystem (#1089 / E111; D149).** With L1-L5
      landed, retire the per-issue prose pins one commit per issue, keeping only
      (a) `render-status.rs --check`, (b) the architect-clause pins in
      `tests/unit/architect_notes.rs`, (c) the benchmark ledger parity test,
      (d) the requirement-status parity test and (e) the issue-citation gate.
      Add `tests/unit/docs_requirements/count.rs` asserting the `docs_*` count is
      at or below the ceiling in `data/meta/debt-ratchet.lino`, strictly
      downward, target 5. The count is **49** today and rose since #1089 was
      filed; record that direction in the ledger's `note`.
      (done 2026-09-19: delivered in the tree — `tests/unit/docs_requirements/count.rs` reads the ceiling from the ledger, asserts it at-or-below and strictly downward, and pins the target 5 with the five survivors named; every per-issue suite is nested under `tests/unit/docs_requirements/`, the top-level `docs_*` count is 5, and the ledger's `docs_requirements_suites` note records the nesting)

### Leaf added by the 2026-09-17 full-suite measurement

- [x] L77 **Apply plan 02 rows D160-D168 and D276.** Plan 02 L24 records the
      four full-suite rows (HumanEval 164 and MBPP 500, cold-offline and
      `--online`) and the failure frontier; this leaf rewrites every document
      those rows supersede, in the same commit as the rows so the
      `latest_external_rows_are_published_from_the_ledger` pin never sits red:
      D160 `VISION.md` per-slice summary with the new run date; D161
      `ROADMAP.md` pillar 26; D162 both `ROADMAP.md` pointer swaps; D163 the
      honest-numbers table with its `Slice` column and the empty-source-cache
      control explicitly scoped to the first-20 slice; D164 the full-suite
      commands and the forget/rediscover round trip; D165 was already present
      in the suites-at-a-glance table and needs no edit; D166 R710-D2/D3 full
      suite plus R710-D17; D167 the new
      `docs/requirements/issue-1138-composition-from-sources.md` shard with
      `assemble-requirements` and `generate-requirement-status` re-run; D168
      the R1138-B2-* traceability rows and the R710-D2/D3 amendments; D276
      pillar 25. D169 stays owned by plan 04 L12. Each document keeps the
      needles its pin tests read (`docs_benchmarks`, `architect_notes`, the
      per-issue suites), and the SWE-bench clauses stay untouched for plan 03
      D173/D174.
      (done 2026-09-19: D160-D165 and D276 were already applied to the tree with the committed full-suite rows, verified row by row; D167's shard, its eight R1138-B2 rows in REQUIREMENTS.md and the traceability rows were delivered with the B2 work, and R710-D17 already existed — this leaf corrected what was still false: R710-D2/D3 now require the full suites with the first-20 as regression control and cite the committed rows, their traceability rows carry the right measurement dates, the "no `--online` 500-case row" sentence in `docs/benchmarks.md` was replaced by the committed 60/500 row, VISION's "cold-offline floor 9/164" was reworded to the session measurement it is (the ledger's only full-slice HumanEval row is 14/164 and the floor is 14), and the pillar-25 comparison gained the online MBPP row; REQUIREMENTS.md rebuilt from 123 shards and parity re-checked)

## Risks and open questions

1. **The 225 rows are a bulk edit with no reviewer.** Generating them from the
   L1 ledger means the ledger's first fill is the audit. Filling `verdict` and
   `automated_test` honestly for 225 IDs is real work, not a script; the
   temptation is to write "none recorded / not yet confirmed" 225 times, which
   satisfies the gate and teaches nothing. Mitigation: fill per shard, one
   commit per shard, and let `check-requirement-status.rs` require that a
   verdict of `implemented` names a test that exists.

2. **Making the manual-confirmation column honest may mean retiring it.** 743 of
   805 rows are unconfirmed and the number rose since #1085 D9 measured it. If
   the maintainer will not fund 743 manual confirmations, option C (mark the
   column aspirational in the header, per #1090) is the honest answer and should
   be asked explicitly rather than decided here.

3. **Generated regions inside hand-written prose are a merge-conflict surface.**
   `CONTRIBUTING.md:970-982` already states the two rules that make this safe (a
   list belongs in a file of its own; every union-merged file has a verifier).
   A generated region inside `VISION.md` violates the first. Open question:
   should the status regions live in `docs/status/*.md` includes instead, with
   the narrative documents linking them? That is cleaner but costs a reader one
   click on the number they came for.

4. **Issue-state snapshots go stale between refreshes.** Gate 2 compares against
   a committed snapshot, so a citation can be correct at commit time and wrong a
   day later. That is acceptable (it is the same discipline as the benchmark
   ledger) but it must be stated in the gate's own message, or the gate becomes
   a source of false confidence — exactly the failure `NON-GOALS.md:53` names.

5. **Which document owns pillar status after the render lands?** Today
   `ROADMAP.md:3`, `REQUIREMENTS.md:1524`, `ARCHITECTURE.md:11` and
   `docs/USER-JOURNEYS.md:10` all claim to be the single place. The plan above
   assigns dimensions, but the assignment is a maintainer decision, not an
   implementation detail.

6. **`ROADMAP.md`'s historical audit tables are load-bearing and misleading at
   once.** The eighth-pass table (`:359-378`) is explicitly kept "as the
   historical record" (`:401`), yet issue #1138 itself quotes its "37 handler
   files / 48 entries" as current. Dating the heading (L25) helps; deleting the
   table would lose the record. Open question: should historical audit tables
   move to `docs/case-studies/issue-651/` and leave only the current table in
   `ROADMAP.md`?

7. **Some findings are true of the code, not of the docs.** D10, D11, D12, D18,
   D113 describe a real capability gap (B1). Rewording the documents makes them
   honest but does not narrow the gap, and an implementation pass that only
   rewords risks reading as progress. Every one of those leaves should land with
   an explicit "this is a wording fix; the capability is tracked as #1138 B1"
   note in its commit message.

8. **Unverified:** whether `#957` (E105, traceability CI enforcement) is the
   right home for gates 1-3 or whether they belong to `#1089` (E111, collapse
   the gate ecosystem). The two overlap and neither names the other.
