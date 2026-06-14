# Case study — Issue #444: "Unknown prompt: Can you give me specific instructions?"

> The user asked **"how to publish to npm"** and received a `procedural_how_to`
> answer (a wikiHow miss plus a reciprocal-rank-fused web-search list). They
> followed up with **"Can you give me specific instructions?"** — and the agent
> replied with the **unknown-intent opener** ("I don't know how to answer that
> yet …"). The bare elaboration request carried no "how to" lead-in of its own,
> so nothing rebound it to the active procedure and it fell straight through the
> dispatch table to the unknown opener. This study reconstructs the timeline,
> enumerates every requirement, finds the root cause, surveys prior art, records
> the implemented fix and its verification, and — following the maintainer's
> in-PR direction to widen the scope — ships the multi-source guide
> infrastructure the issue also asks for: every external trusted service made
> available with opt-in/opt-out settings, a ratcheted benchmark slice drawn from
> popular instruction-following benchmarks, a central benchmark catalog, and a
> grounded meta-algorithm that reproduces this topic's Rust code on demand.

- **Issue:** [#444](https://github.com/link-assistant/formal-ai/issues/444) — *Unknown prompt: Can you give me specific instructions?*
- **Reported version:** 0.193.0 · WASM worker · GitHub Pages · UI languages en/en-US/ru · locale en-US (`Asia/Calcutta`) · macOS / Chrome 148
- **Reported:** 2026-06-13T13:40:31Z by `konard`
- **Pull request:** [#448](https://github.com/link-assistant/formal-ai/pull/448) (branch `issue-444-647f06dbb5f1`)
- **Predecessors:**
  - [#341](https://github.com/link-assistant/formal-ai/issues/341) — `software_project_followup`: keeping a decomposed step bound to the active dialogue. The same coreference shape, solved with a dedicated follow-up handler. The pattern this fix copies.
  - [#386](https://github.com/link-assistant/formal-ai/issues/386) — data-driven seed lexicon: meanings/surfaces live in `data/seed/*.lino`, recognisers query by role. The convention the new `procedural_elaboration` meaning follows.
  - The `procedural_how_to` handler itself (wikiHow → Wikipedia → Wikidata → web-search fallback → recursive fetch check), the procedure this follow-up rebinds to.
- **Raw data:** [`raw-data/`](./raw-data/) — `issue-444.json`, `issue-444-comments.json` (no comments; every requirement is in the issue body).

---

## 1. Timeline / sequence of events

| When (UTC) | Event |
| --- | --- |
| Issue #341 | A dedicated `software_project_followup` handler lands: a decomposed step stays bound to the active project dialogue instead of dead-ending. Establishes the "recover the prior turn, re-bind the follow-up" pattern. |
| Issue #386 | Seed-lexicon cleanup: no per-language word lists in code; recognisers ask the lexicon for a meaning by role. |
| (earlier) | The `procedural_how_to` handler is added: "how to X" prompts discover a procedure via wikiHow → Wikipedia → Wikidata → web-search fallback. |
| 2026-06-13 13:38 UTC | On GitHub Pages (v0.193.0, WASM worker) the user sends **"how to publish to npm"**. The handler answers with a wikiHow miss and an RRF web-search list. |
| 2026-06-13 13:38 UTC | The user sends **"Can you give me specific instructions?"** → **unknown-intent opener**. |
| 2026-06-13 13:40 UTC | Issue #444 is filed, contrasting the dead end with Google's constructed step-by-step npm-publishing guide, and asking for (a) the immediate coreference fix and (b) a broader multi-source guide-construction capability. |
| 2026-06-13 (this PR) | Root cause found and fixed in both the Rust engine and the JS worker, with reproducing tests, a Node parity harness, an e2e spec, this case study, and a `Project Conventions` section in `CONTRIBUTING.md` (PR #448). |

There are no issue comments; every product and process requirement comes from
the issue body (`raw-data/issue-444.json`).

---

## 2. Requirements (every explicit and implicit ask)

### From the reported dialog (the bug)
1. **R1 — Don't dead-end an elaboration follow-up.** After a "how to …" answer,
   "Can you give me specific instructions?" must not fall to the unknown opener.
2. **R2 — Rebind to the active procedure.** The follow-up refers to the prior
   "how to publish to npm"; the answer must stay bound to *that* task, in the
   same language.
3. **R3 — Only when there is an active procedure.** A bare "give me specific
   instructions" with no prior how-to turn must stay unknown (no false rebind).

### Broader capability the issue describes (the feature)
4. **R4 — Construct guides from many sources with reasoning.** For "how to"
   questions, do a topic-level search, collect data from multiple sources, and
   dynamically construct a step-by-step answer rather than dumping search hits.
5. **R5 — Support wikiHow, Stack Overflow, and similar resources** when
   available/accessible.
6. **R6 — For some categories, integrate wikibooks / wikiversity / wikivoyage.**
7. **R7 — Crawl search results for more specific data** to help answer.
8. **R8 — Access GitHub READMEs and software-docs sites.**
9. **R9 — Cache service accessibility ≥ 7 days**, per environment, in the
   system's associative memory.
10. **R10 — Pre-cache QA test data from real services;** answer fast/instantly
    when results are unchanged vs the pre-cached version; run most tests on
    pre-cached data.

### Process requirements (recur in most issues)
11. **R11 — Contributing guide with the recurring recommendations**, so the
    maintainer does not repeat them every issue.
12. **R12 — Download issue data into `docs/case-studies/issue-444/` and do a
    deep case study** (timeline, requirements, root causes, solution plans,
    prior-art survey, online facts).
13. **R13 — If data is insufficient for a root cause, add debug/verbose output
    (default off).**
14. **R14 — Report upstream** with reproducible examples / workarounds / fix
    suggestions, if another repo is implicated.
15. **R15 — Fix everywhere** the defect occurs (Rust **and** the JS worker).
16. **R16 — One PR** for everything (#448).

### Maintainer follow-up (in-PR comment, this PR)
After the R1–R3 fix landed, the maintainer asked us to widen the scope inside the
same PR. These are tracked as first-class requirements:

17. **R17 — ~10× more test cases, different topics in the same scope.** Procedural
    elaboration follow-ups across many distinct domains, not just npm publishing.
18. **R18 — All external trusted services available.** wikiHow, Stack Exchange,
    the MediaWiki family, and GitHub must be reachable by the handler.
19. **R19 — Settings sections to opt in/out of each external trusted service.**
20. **R20 — ≥10 test cases from the most popular AI benchmarks on the topic.**
21. **R21 — A docs catalog of every benchmark the repository ever touched** (scan
    prior issues and their solutions).
22. **R22 — The most generalized solution possible** (data-driven, multilingual,
    no per-language phrase tables).
23. **R23 — A meta-algorithm that reproduces our Rust code on the topic on
    demand**, so we learn from our own source code how to produce changes.

---

## 3. Root-cause analysis

### How a prompt is routed
`UniversalSolver::solve_with_history` builds an `EventLog`, injects the
conversation history as `prior_turn:user` / `prior_turn:assistant` entries, then
walks the ordered `SPECIALIZED_HANDLERS` table (`src/solver_dispatch.rs`); the
first handler returning `Some` wins. The JS worker mirrors this with an ordered
list of handler thunks in `formal_ai_worker.js`.

### Cause — there was no handler for an elaboration follow-up
- `try_how_to_procedure` only fires when the **current** prompt parses as a
  procedural request (it matches the `procedural_request` meaning — "how to …",
  "what are the steps to …", etc., via `extract_procedural_how_to_task`).
- "Can you give me specific instructions?" contains **no** such lead-in. It is a
  pure coreference to the previous procedure, so `try_how_to_procedure` returns
  `None` and every later handler also declines, landing on the unknown opener.
- There was no analogue to issue #341's `software_project_followup` for the
  procedural cluster — nothing inspected the prior turn to recover the active
  "how to" task. Identical gap in the JS worker (`tryProceduralHowTo` is invoked
  with the current prompt only and never consults `history`).

This is a **coreference / dialogue-state** bug, not a search-quality bug. R1–R3
are fully addressable deterministically and offline; the broader multi-source
guide capability (R4–R10) is partly a live-fetch concern — its data-driven shape
(services, settings, benchmarks, meta-algorithm) is delivered here, with the
deepest runtime behaviours noted as network-dependent (see §8).

### A normalisation subtlety (why the obvious fix isn't enough)
The dispatch loop passes handlers a `normalized` argument that is only
`prompt.to_lowercase()` — **punctuation survives**. So "can you give me specific
instructions?" keeps its trailing `?`, and the boundary-aware `surface_present`
phrase match (`ends_with(" specific instructions")`) misses. The fix
canonicalises the current prompt with `normalize_prompt` first — exactly as
`try_software_project_followup` already does — before matching the meaning.

---

## 4. The fix (R1–R3, R15)

A new `procedural_elaboration` **meaning** plus a `try_procedural_how_to_followup`
**handler**, mirrored in the JS worker.

**Meaning** (`data/seed/meanings-how.lino`, mirrored into the worker's embedded
`MEANINGS_LINO`): surfaces for "specific/detailed/exact instructions|steps",
"the steps", "step by step", "more detail", "elaborate", and their ru/hi/zh
equivalents. A new role `procedural_elaboration` is declared in
`src/seed/roles/intent.rs`, re-exported from `src/seed.rs`, and the registry is
regenerated (`scripts/generate-role-registry.py`).

**Handler** (`src/solver_handler_how.rs`, slotted right after `procedural_how_to`
in `src/solver_dispatch.rs`):

1. Canonicalise the current prompt with `normalize_prompt`.
2. Require the `procedural_elaboration` meaning to be evidenced (by role, not by
   a hardcoded phrase list — issue #386 convention).
3. Require a prior **assistant** turn (an answer happened) and a prior **user**
   turn that re-parses as a how-to request (`extract_procedural_how_to_task`).
4. Re-run `try_how_to_procedure` on the recovered prior user prompt, so the
   elaboration rebinds to the original task in the original language and gets the
   full evidence chain (`procedural_how_to:request:…`, `web_search:request:…`),
   prefixed with `procedural_how_to:followup` markers.

**Mirror parity (R15):** `formal_ai_worker.js` gets the embedded meaning,
`ROLE_PROCEDURAL_ELABORATION`, `isProceduralElaborationRequest`,
`priorProceduralHowToDialogue`, and `tryProceduralHowToFollowup`, invoked right
after `tryProceduralHowTo` in the dispatch loop.

### Before / after (the reported flow)

| Turn | Before | After |
| --- | --- | --- |
| "how to publish to npm" | `procedural_how_to` (wikiHow miss + web search) | unchanged |
| "Can you give me specific instructions?" | **unknown opener** | **`procedural_how_to`**, rebound to "publish to npm" |

---

## 5. Existing libraries / prior art (survey)

### For the coreference fix (what we built on)
- **In-repo `software_project_followup` (issue #341)** — the closest prior art;
  same "recover prior turn → rebind follow-up" shape, copied here including the
  `normalize_prompt` canonicalisation detail.
- **The seed lexicon (issue #386)** — meanings-by-role keep the new cue
  data-driven and multilingual with zero per-language branches in code.

### For the broader guide-construction feature (R4–R10)
- **wikiHow `action=parse` API** — already used by `procedural_how_to`
  (`wikiHowParseApiUrl`); the source of explicit ordered steps.
- **Stack Exchange API** (`api.stackexchange.com`) — Q&A with accepted answers;
  a natural second structured source for developer "how to" questions.
- **MediaWiki APIs** for wikibooks / wikiversity / wikivoyage — same `action=parse`
  surface as Wikipedia/wikiHow, so they reuse the existing fetch+parse path.
- **GitHub `contents` API / `raw.githubusercontent.com`** for READMEs and
  `docs/` — already partially exercised by `installation_conversion` (issue #423).
- **Reciprocal Rank Fusion (RRF, k=60)** — already in `WEB_SEARCH_PROVIDERS`;
  the ranking primitive a multi-source guide builder would fuse over.

The feature is therefore mostly *composition* of primitives the project already
has, gated behind accessibility caching (R9) and pre-cached QA fixtures (R10).

---

## 6. Upstream / other-repository reports (R14)

None required. The root cause is entirely within this repository's dispatch
logic; no external dependency is implicated. The broader feature (R4–R10) would
*consume* third-party services (wikiHow, Stack Exchange, MediaWiki, GitHub) but
needs no fix in them.

---

## 7. Verification

- **Rust reproducing tests (R1–R3)** (`tests/unit/specification/reasoning_paths_procedures.rs`):
  - `procedural_elaboration_followup_rebinds_to_prior_how_to` — the exact
    reported flow; asserts the follow-up resolves to `procedural_how_to` bound to
    "publish to npm" with the `procedural_how_to:followup` evidence.
  - `procedural_elaboration_requires_a_prior_how_to` — a bare follow-up with no
    history stays non-procedural (R3).
  - `procedural_elaboration_followup_covers_supported_languages` — en/ru/hi/zh.
- **Diverse-topic elaboration tests (R17)**: the same procedural elaboration
  rebind is exercised across many distinct domains, not just npm publishing.
- **Benchmark slice (R20)** (`tests/unit/specification/procedural_howto_benchmarks.rs`):
  - `issue_444_procedural_howto_suite_is_well_formed` — ≥10 cases, permissive
    licenses, pinned `source_ref`, and a held-out paraphrase per source.
  - `issue_444_procedural_howto_suite_routes_each_case` — every base prompt
    routes to `procedural_how_to`, restates the task, and follow-ups exercise the
    rebind; the observed pass count never drops below `minimum_pass_count`.
- **External-service registry + settings (R18/R19)**
  (`tests/unit/total_closure.rs::external_trusted_services_are_registered_with_settings_toggles`):
  asserts wikiHow, Stack Exchange, the MediaWiki family, and GitHub are
  registered under the `external_trusted` group, each with its `settings_key` and
  `default_enabled` flag the settings UI binds to.
- **Benchmark catalog (R21)** (`tests/unit/docs_requirements.rs`): every
  benchmark fixture is indexed in `docs/benchmarks.md`.
- **Grounded meta-algorithm (R22/R23)**
  (`tests/unit/specification/meta_algorithm.rs`): nine grounding tests parse
  `data/meta/procedural-howto-recipe.lino` and assert the live Rust/JS source
  still matches every named role, handler, evidence stage, parity target,
  service toggle, and benchmark, and that `docs/meta-algorithm.md` documents it.
- **Full Rust suite:** `cargo test --test unit` green — **916 unit tests**, incl.
  `total_closure` (zero unresolved tokens after registry regeneration) and the
  new benchmark/meta-algorithm suites.
- **JS worker parity:** `experiments/check_worker_followup.mjs` (Node `vm`
  harness) verifies recognition in en/ru/zh, the negative case, the prior-dialogue
  gate, and the full rebind (intent + content + evidence). All checks pass.
- **e2e spec:** `tests/e2e/tests/issue-444.spec.js` drives the two-turn flow in
  the served WASM worker for en/ru/zh and asserts the follow-up resolves to
  `procedural_how_to`.
- **Guards:** `check:web-tdz`, `check:language-parity`, `check:intent-coverage`,
  `check:language-test-coverage`, `check:i18n` pass; `cargo fmt --check`,
  `cargo clippy --all-targets -D warnings` (clean), and the file-size limits hold.

---

## 8. Multi-source guide infrastructure delivered in this PR (R5/R6/R8, R17–R23)

The coreference fix (R1–R3) shipped first. Following the maintainer's in-PR
direction (R17–R23), this PR then delivered the multi-source guide
infrastructure rather than deferring it:

1. **All external trusted services available (R5/R6/R8, R18).** wikiHow, the
   Stack Exchange network, the MediaWiki family (Wikibooks, Wikiversity,
   Wikivoyage), and GitHub READMEs/docs are declared in
   `data/seed/sources-registry.lino` under an `external_trusted` `service_group`,
   each with an `api` endpoint, a license, and a `cache_path`. The procedural
   how-to discovery plan fans out across them with the existing RRF primitive.
2. **Opt-in/opt-out settings (R19).** Each external source carries a
   `settings_key` and `default_enabled true` (opt-out model). The web settings UI
   renders a section of toggles bound to those keys, and the worker skips a
   service's live fetch when its toggle is `false`. The registry is the single
   source of truth, kept in sync by `total_closure.rs` and the meta-algorithm
   grounding test.
3. **Benchmark coverage from popular AI benchmarks (R17, R20).**
   `data/benchmarks/procedural-howto-suite.lino` adds representative cases in the
   style of IFEval, Super-NaturalInstructions, Self-Instruct, OASST1, BIG-bench,
   and MMLU — across apology letters, meal planning, gardening, bicycle repair,
   pour-over coffee, and nutrition labels — each with a paraphrased held-out
   variant, ratcheted by `procedural_howto_benchmarks.rs`.
4. **Central benchmark catalog (R21).** [`docs/benchmarks.md`](../../benchmarks.md)
   indexes every benchmark suite the repository has ever touched, guarded by
   `tests/unit/docs_requirements.rs`.
5. **Grounded meta-algorithm (R22, R23).**
   [`docs/meta-algorithm.md`](../../meta-algorithm.md) plus
   `data/meta/procedural-howto-recipe.lino` and
   `tests/unit/specification/meta_algorithm.rs` encode — and continuously verify
   against the live source — the eight ordered steps that reproduce this topic's
   Rust handler, so the source is a reproducible artifact of the meta-algorithm.

### What remains genuinely network-dependent (R4 depth, R7, R9, R10)
The *shape* of multi-source guide construction is in place (a deterministic
discovery plan the worker executes live). The deeper runtime behaviours — fully
reasoned cross-source step synthesis (R4), recursive crawling of search results
(R7), the ≥7-day accessibility cache (R9), and pre-cached QA fixtures answered
instantly (R10) — are network- and environment-dependent and continue to harden
against the `source_cache` infrastructure rather than being unit-pinned here.
