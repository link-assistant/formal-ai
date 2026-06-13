# Case study — Issue #444: "Unknown prompt: Can you give me specific instructions?"

> The user asked **"how to publish to npm"** and received a `procedural_how_to`
> answer (a wikiHow miss plus a reciprocal-rank-fused web-search list). They
> followed up with **"Can you give me specific instructions?"** — and the agent
> replied with the **unknown-intent opener** ("I don't know how to answer that
> yet …"). The bare elaboration request carried no "how to" lead-in of its own,
> so nothing rebound it to the active procedure and it fell straight through the
> dispatch table to the unknown opener. This study reconstructs the timeline,
> enumerates every requirement, finds the root cause, surveys prior art, records
> the implemented fix and its verification, and scopes the larger
> multi-source guide-construction feature the issue also asks for.

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
are fully addressable deterministically and offline; R4–R10 are a separate,
live-fetch capability (see §8).

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

- **Rust reproducing tests** (`tests/unit/specification/reasoning_paths_procedures.rs`):
  - `procedural_elaboration_followup_rebinds_to_prior_how_to` — the exact
    reported flow; asserts the follow-up resolves to `procedural_how_to` bound to
    "publish to npm" with the `procedural_how_to:followup` evidence.
  - `procedural_elaboration_requires_a_prior_how_to` — a bare follow-up with no
    history stays non-procedural (R3).
  - `procedural_elaboration_followup_covers_supported_languages` — en/ru/hi/zh.
- **Full Rust suite:** `cargo test` (lib + unit + integration + source) green
  (852 unit tests, incl. `reference_closure` after registry regeneration).
- **JS worker parity:** `experiments/check_worker_followup.mjs` (Node `vm`
  harness) verifies recognition in en/ru/zh, the negative case, the prior-dialogue
  gate, and the full rebind (intent + content + evidence). All checks pass.
- **e2e spec:** `tests/e2e/tests/issue-444.spec.js` drives the two-turn flow in
  the served WASM worker for en/ru/zh and asserts the follow-up resolves to
  `procedural_how_to`.
- **Guards:** `check:web-tdz`, `check:language-parity`, `check:intent-coverage`,
  `check:language-test-coverage` pass; `cargo fmt --check`, `cargo clippy`
  (clean in changed files), and the 1000-line file-size limit hold.

---

## 8. Follow-up opportunities (the multi-source guide builder, R4–R10)

R1–R3 are shipped in this PR. R4–R10 describe a substantial, network-dependent
capability that is hard to unit-test deterministically and is best tracked as its
own issue so it can be designed against the caching (R9) and pre-cached-fixture
(R10) requirements rather than rushed alongside the coreference fix. The shape:

1. **Topic search → multi-source collection.** Fan out across wikiHow, Stack
   Exchange, MediaWiki family (wikibooks/wikiversity/wikivoyage by category),
   GitHub READMEs, and docs sites; fuse with the existing RRF primitive.
2. **Guide construction by reasoning.** Extract ordered steps from each source,
   align/merge them into a single deduplicated procedure with citations —
   projecting the constructed guide from the event log, never a hardcoded table.
3. **Accessibility cache (R9).** Record each service's reachability per
   environment in associative memory with a ≥ 7-day TTL, so unreachable services
   are skipped quickly.
4. **Pre-cached QA fixtures (R10).** Snapshot real responses for the test corpus;
   answer instantly when live results match the snapshot, and run most tests
   against snapshots for fast iteration.

This PR deliberately scopes to the deterministic, fully testable coreference fix
(R1–R3) plus the process requirements (R11–R16), and documents R4–R10 here as the
design for a dedicated follow-up.
