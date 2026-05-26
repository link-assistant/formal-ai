# Issue #244 Case Study: Plan Issues To Implement Our Vision Fully

## Scope

Issue: <https://github.com/link-assistant/formal-ai/issues/244>

Pull request: <https://github.com/link-assistant/formal-ai/pull/245>

Branch: `issue-244-75334b422fcf`

Issue #244 is a **meta-planning** issue, not a bug report. It asks us to:

1. Update the documentation so it **fully tracks the implementation progress of
   all requirements** and is **in sync with the actual state of the code**.
2. After the docs are in sync, **create the GitHub issues** that represent the
   full plan to implement the vision: a system that learns a *universal problem
   solving algorithm*, truly translates between natural and formal languages,
   keeps a *minimum core of algorithms and a data seed* with enough metadata to
   problem-solve like people do — **without using neural networks for the
   reasoning itself** — and that covers all existing test cases and much more.
3. Collect issue-related data into `docs/case-studies/issue-244`, do a deep case
   study (including online research), list **each and all** requirements from
   the issue, and propose **solution plans for each requirement**, checking
   known existing components/libraries that solve similar problems.

This case study is the analysis behind the planning. The companion deliverables
are:

- [`ROADMAP.md`](../../../ROADMAP.md) — the new implementation-progress tracker
  that maps every vision pillar to its real code state and to the planning issue
  that closes the gap (the "documentation in sync with code" deliverable).
- [`proposed-issues.md`](proposed-issues.md) — the full text of every planning
  issue created for #244 (the "create all the issues" deliverable).
- The `Issue #244` row block added to [`REQUIREMENTS.md`](../../../REQUIREMENTS.md).

> 2026-05-26 update: the first planning batch (E1-E14, issues #246-#259) has
> been implemented and merged to `main`. This case study now preserves the
> original audit as historical context and records the post-implementation audit
> that created E15-E20 (#278-#283) for the remaining partial requirements.

## Captured Artifacts

Downloaded and generated artifacts live under `raw-data/`:

- `issue-244.json`, `issue-244-comments.json` — the issue body and comments
  (no comments at collection time).
- `issue-survey.md` — the conclusions of the initial all-issues survey
  (127 issues, #244 the only open one at collection time, so no duplicate
  planning issue existed). The full machine dump is intentionally not vendored;
  the per-issue history lives in
  `REQUIREMENTS.md`, and reproducing every historical title verbatim trips the
  repository-hygiene guard.
- `pr-245.json`, `pr-245-comments.json`, `pr-245-review-comments.json` — the
  prepared PR metadata and its (empty) comment snapshots at collection time.
- `ci-runs-branch.json` — branch CI state before changes (green on the initial
  commit `67a9fc5`).
- `online-research.md` — summarized external prior art (neuro-symbolic KG
  reasoning, Abstract Wikipedia/Wikifunctions, OpenCog AtomSpace/Hyperon,
  Lean/Z3/program synthesis) with citations.
- `code-audit.md` — the structured audit of the actual implemented state of
  `src/`, the seed, and the test suite that this plan is built on.
- `closed-issues-2026-05-26.json`, `merged-prs-2026-05-26.json`,
  `deferred-marker-search-2026-05-26.txt`,
  `ignored-tracked-tests-2026-05-26.txt`, and
  `next-batch-issues-2026-05-26.txt` — the post-implementation audit showing
  which follow-up issues closed, which deferred markers remain, and which
  next-batch issues were opened.

## Timeline

| Time (UTC) | Event |
| --- | --- |
| 2026-05-25 18:36 | Issue #244 opened by `konard` with labels `bug`, `documentation`, `enhancement`, asking to update docs and plan the full set of vision issues. |
| 2026-05-25 18:37 | Draft PR #245 prepared for branch `issue-244-75334b422fcf`; branch CI green on the initial commit. |
| 2026-05-25 | Codebase audit completed: 11-step solver loop exists but routing is still keyword/intent based; 69 `#[ignore]`-tagged "tracked requirement" tests enumerate the vision gaps; `ARCHITECTURE.md` §16 lists four architecture open questions. |
| 2026-05-25 | Online research collected: Abstract Wikipedia/Wikifunctions, OpenCog AtomSpace/Hyperon, Lean/Z3 confirmed as the closest prior art for the meaning-anchored translation, associative store, and deterministic verification pillars. |
| 2026-05-25 | `ROADMAP.md` written, planning issues drafted in `proposed-issues.md`, and the issues created in the repository. |
| 2026-05-26 | `origin/main` merged into the issue branch; the E1-E14 implementation PRs (#260, #261, #263-#275) and the courtesy PR #276 were incorporated. |
| 2026-05-26 | Closed issues, merged PRs, deferred markers, and tracked specification tests were audited; no `#[ignore = "tracked requirement: ..."]` tests remained. |
| 2026-05-26 | Six remaining partial requirements were opened as E15-E20: #278, #279, #280, #281, #282, and #283. |

## Requirements Extracted From Issue #244

Each clause of the issue is turned into an explicit requirement with a status.
"Status" here is about *this PR's deliverables for #244*, not about the vision
features themselves (those are tracked in `ROADMAP.md`).

| ID | Requirement (from the issue text) | Status in this PR |
| --- | --- | --- |
| Q1 | Use all previous issues, PRs, comments, and the vision files to ground the plan. | Done — all 127 issues surveyed (`raw-data/issue-survey.md`), `VISION.md`/`GOALS.md`/`NON-GOALS.md`/`REQUIREMENTS.md`/`ARCHITECTURE.md` read, code audited. |
| Q2 | First update documentation to **fully track the progress** of implementation of all requirements. | Done — `ROADMAP.md` added as the progress tracker; `REQUIREMENTS.md` and `ARCHITECTURE.md` references reconciled. |
| Q3 | Ensure **everything in docs is in sync with the actual state of the code**. | Done — `ROADMAP.md` is grounded in `raw-data/code-audit.md`; overstated/stale references corrected. |
| Q4 | Create **all the issues** needed to fully implement the vision. | Done — see `proposed-issues.md` and the created issues listed below. |
| Q5 | Enable the system to learn the **universal problem solving algorithm**. | Implemented for the first batch by E2/#247 and PR #261; further learning/package work is tracked by #279, #281, and #283. |
| Q6 | Truly solve **translation between languages (natural and formal)**. | Implemented for the first batch by E3/#248, E6/#251, and E7/#252; broader skill/code lowering continues in #283. |
| Q7 | Keep a **minimum core of algorithms and a data seed** with enough metadata to problem-solve like people do. | Partially implemented through seed files, cache/formalization, link-store projection, and skill compilation; remaining native store and package work is tracked by #278 and #281. |
| Q8 | Problem-solve **like people do**, in the way expected from AI, but **without neural networks for the reasoning itself**. | Implemented by the symbolic universal loop and formal decision procedure; symbolic probabilistic ranking is tracked by #279. |
| Q9 | Provide **formal reasoning** that covers all current test cases **and much more**. | Implemented by E8/#253 and PR #268 with decision-procedure modules under `src/proof_engine/decision.rs`. |
| Q10 | Learn to **work with unknowns** and gather missing information ourselves. | Implemented through the universal loop and source cache; probabilistic evidence for ranking unknowns is tracked by #279. |
| Q11 | Ask the user **as few questions as possible**; only ask what cannot be answered by the system itself. | Implemented by E4/#249 and PR #264 through temperature-based clarify-vs-guess selection. |
| Q12 | **Build on previous experience**; make the algorithm more general and smart while still supporting everything already supported. | Implemented by merging E1-E14 with active regression coverage; the narrowed follow-up batch records remaining work without reopening closed requirements. |
| Q13 | If there are **critical problems blocking the vision**, plan to fix them **first** (solid foundation). | Done — E1/E2 were planned first, then implemented; the refreshed roadmap now scopes the remaining partial requirements as E15-E20. |
| Q14 | Collect issue data into `docs/case-studies/issue-244` and **search online** for additional facts. | Done — `raw-data/` + `online-research.md`. |
| Q15 | Do a **deep case study analysis**; list each and all requirements; propose **solution plans per requirement**. | Done — this document + `proposed-issues.md`. |
| Q16 | Check **known existing components/libraries** that solve a similar problem or can help. | Done — see "Existing components and libraries" below and `online-research.md`. |

## Post-Implementation Audit (2026-05-26)

The 2026-05-26 audit checked all closed issues, merged PRs, stale deferred
markers, and tracked specification tests after merging `origin/main`.

Findings:

- E1-E14 (#246-#259) are closed and backed by merged PRs #260, #261, and
  #263-#275.
- Issue #262 is also closed by PR #276.
- `tests/unit/specification/` no longer contains `#[ignore = "tracked requirement: ..."]`
  tests.
- Six remaining requirements were still partial or open and were created as the
  next batch: #278 default native doublets store, #279 symbolic probabilistic
  ranking, #280 desktop wrapper, #281 associative packages/permissions, #282
  Rust/WASM browser parity, and #283 generalized skill compiler.

## Initial State Of The Code (2026-05-25 Audit Summary)

This section is preserved as the initial planning snapshot. It no longer
describes the current `main` state after E1-E14 were merged.

The full audit is in `raw-data/code-audit.md`. Headlines:

- **The 11-step universal solver loop exists** as the outer skeleton in
  `src/solver.rs`, but the **inner routing is still keyword/intent driven**:
  `handle_specialized_pattern()` dispatches to ~35 hand-written handlers chosen
  by seed keyword/phrase/token/combo rules. Every prompt does *not yet* walk a
  single formalize → search → decompose → candidates → validate → select loop.
- **`SolverConfig` already carries the knobs** (`guess_probability`,
  `questioning_rigor`, `max_decomposition_depth`, `agent_mode`,
  `diagnostic_mode`, `offline`, `cache_ttl_seconds`, `temperature`, …) with
  `FORMAL_AI_*` env overrides. `temperature` exists but has **no softmax helper**.
- **An append-only event log exists** (`src/event_log.rs`) with content-addressed
  ids and all the documented event kinds, but it is **in-process** and the
  durable store is a custom `MemoryStore` (`.lino`), **not doublets-rs/doublets-web**.
- **Formalization is alias based** (`src/concepts.rs`); the **full Wikidata
  P-id/Q-id extraction** over arbitrary prompts is not implemented.
- **Translation pipeline is real** (`src/translation/`: Wiktionary parsing,
  Wikidata SPARQL, `formalize → meaning → deformalize → match_source_formatting`)
  but the link-native **meaning-id invariants** (synonyms share a meaning id,
  traces include the intermediate meaning record, untranslatable flagged) are
  still tracked, not enforced.
- **Agent mode is guarded but not executed**: chat never runs user code; there
  is no sandbox, action log, confirmation flow, time budget, or secret guard.
- **69 `#[ignore]`-tagged tests under `tests/unit/specification/`** are the
  precise, machine-checkable backlog. They are the acceptance criteria for the
  planning issues: each epic below names the tests it must graduate out of
  `#[ignore]`.

`#[ignore]` "tracked requirement" tests by file:

| Spec file | Ignored tests | Theme |
| --- | --- | --- |
| `reasoning_loop.rs` | 11 | Universal loop steps 1–9, termination, confidence |
| `links_network.rs` | 10 | Doublet reduction, type chains, append-only, source/trace links, schema version, addressability, validation |
| `agent_isolation.rs` | 9 | Agent opt-in, sandbox, action log, failure traces, confirmation, time budget, secret guard, revocation |
| `transparent_state.rs` | 8 | Network query, "what do you know about X", no leak, diagnostic opt-in, "why", retraction, export, "list my facts" |
| `source_cache.rs` | 8 | Source URL, `fetched_at`, TTL refresh, cache hit, content hash, conflict surfacing, explicit flush, offline |
| `translation_via_links.rs` | 7 | Meaning-id preservation, target surface, synonyms share meaning, language tags, intermediate meaning, code translation, untranslatable flag |
| `code_generation.rs` | 6 | Top-10 languages, execution links, isolation level, algorithm+tests, program translation, failure traces |
| `chat_surface.rs` | 6 | Refuse unbounded, declare execution status, diagnostics off, impulse recorded, trace link, extend-network path |
| `openai_compatibility.rs` | 2 | Bearer auth, refuse tool call without agent mode |
| `telegram_surface.rs` | 1 | Tap-to-inspect trace link |
| `network_visualization.rs` | 1 | Graph beside chat never blocks replies |

`ARCHITECTURE.md` §16 open questions: (1) full P/Q formalization, (2) softmax
temperature helper, (3) doublets-rs backend, (4) natural-language-skill compiler.

## Vision → Gap → Plan

The gap analysis maps each VISION.md pillar to its real status and the epic that
closes it. The full matrix lives in `ROADMAP.md`; the planning issues are:

| Epic | Title | Closes (tracked tests / open questions) | Foundation? |
| --- | --- | --- | --- |
| E1 | Unified doublet-links store (doublets-rs + doublets-web) | `links_network` storage invariants; ARCH §16.3 | **Yes (blocker)** |
| E2 | Make the universal reasoning loop the only entry path | `reasoning_loop` (11); `chat_surface` impulse+trace | **Yes (blocker)** |
| E3 | Full Wikidata P/Q-id formalization engine | ARCH §16.1; formalization for E6/E10 | Foundation |
| E4 | Temperature-based interpretation selection + clarify-vs-guess | ARCH §16.2 | — |
| E5 | Public-knowledge source cache with provenance | `source_cache` (8) | — |
| E6 | Translation via link-native meanings | `translation_via_links` (7) | — |
| E7 | Code generation & cross-language translation | `code_generation` (6) | — |
| E8 | Formal reasoning engine (relative-meta-logic / SMT) | proof beyond the fixed theorem table; Q9 | — |
| E9 | Chat-over-experience queries | `transparent_state` (8) | — |
| E10 | Links-network invariants & dynamic type system | remaining `links_network` | — |
| E11 | Agent mode with isolated execution | `agent_isolation` (9); `chat_surface` refuse-unbounded | — |
| E12 | Authenticated API + tool-call gating | `openai_compatibility` (2) | — |
| E13 | Network visualization + trace links on every surface | `network_visualization`; `telegram_surface`; `chat_surface` execution-status + diagnostics-off | — |
| E14 | Natural-language skill compilation | ARCH §16.4; VISION computation model | — |

This table is the initial E1-E14 plan. Those issues are now closed; the current
status and the E15-E20 follow-up batch live in `ROADMAP.md`.

## Solution Plans Per Requirement

Each epic's full problem statement, proposed approach, existing components to
reuse, and acceptance criteria are in [`proposed-issues.md`](proposed-issues.md).
For E1-E14, the acceptance criteria were the original tracked tests to graduate.
For E15-E20, the criteria are the remaining partial requirements discovered by
the 2026-05-26 audit. The design principles that bind them:

- **Foundation first (Q13).** E1 (one doublet store as the source of truth) and
  E2 (one reasoning loop as the only entry path) are blockers; the other epics
  build on them. This is the "solid foundation" the issue asks for.
- **Keep the regression floor (Q12).** No epic may remove an already-supported
  behavior. The existing green tests are the floor; the first batch graduated
  tracked tests, and follow-up work must add or narrow tests instead of deleting
  passing ones.
- **Determinism and traceability (Q8).** Every epic preserves "same prompt +
  same config ⇒ same answer", seeded randomness from the impulse hash, and a
  `trace:` pointer the user can inspect.
- **Web as cache, not teacher (Q10).** External knowledge is cached with
  provenance (E5); offline mode refuses lookups; nothing is learned into weights.

## Existing Components And Libraries

Reused or referenced (details and citations in `online-research.md`):

- **`linksplatform/doublets-rs` / `doublets-web`** — the long-term doublet
  store family; E1 added the boundary/projection, and #278 tracks making
  `doublets-rs` the default native physical store.
- **`link-assistant/relative-meta-logic`** — optional future backend candidate
  for formal reasoning; E8/#253 added the current decision-procedure layer.
- **`link-assistant/calculator` (`link-calculator`)** — already integrated;
  the model for delegating a hard sub-problem to a verified engine.
- **Wikidata / Wikipedia / Wiktionary** — meaning anchors and per-language
  surfaces for E3/E6; cached via E5.
- **Abstract Wikipedia / Wikifunctions** — prior art for rendering a
  language-independent meaning into any language (E6); watch their renderers as
  a source of deterministic per-language generation rules.
- **OpenCog AtomSpace / Hyperon (MeTTa)** — prior art for "graph rewriting +
  rule-as-data + self-modifying rules" (E10, E14); we use doublets + Links
  Notation as the reviewable, restricted cousin.
- **Lean / Z3 / first-order saturation synthesis** — prior art for deterministic
  verification and program synthesis (E7, E8).
- **`lino-i18n`, `lino-objects-codec`, `lino-arguments`** — Links Notation
  tooling already in the repo.

## Created Planning Issues

The first 14 epics below were opened against this repository on 2026-05-25. See
`proposed-issues.md` for the full body of each and `ROADMAP.md` for the current
status.

| Epic | Issue |
| --- | --- |
| E1 | [#246](https://github.com/link-assistant/formal-ai/issues/246) |
| E2 | [#247](https://github.com/link-assistant/formal-ai/issues/247) |
| E3 | [#248](https://github.com/link-assistant/formal-ai/issues/248) |
| E4 | [#249](https://github.com/link-assistant/formal-ai/issues/249) |
| E5 | [#250](https://github.com/link-assistant/formal-ai/issues/250) |
| E6 | [#251](https://github.com/link-assistant/formal-ai/issues/251) |
| E7 | [#252](https://github.com/link-assistant/formal-ai/issues/252) |
| E8 | [#253](https://github.com/link-assistant/formal-ai/issues/253) |
| E9 | [#254](https://github.com/link-assistant/formal-ai/issues/254) |
| E10 | [#255](https://github.com/link-assistant/formal-ai/issues/255) |
| E11 | [#256](https://github.com/link-assistant/formal-ai/issues/256) |
| E12 | [#257](https://github.com/link-assistant/formal-ai/issues/257) |
| E13 | [#258](https://github.com/link-assistant/formal-ai/issues/258) |
| E14 | [#259](https://github.com/link-assistant/formal-ai/issues/259) |

The post-implementation audit opened the next batch on 2026-05-26:

| Epic | Issue |
| --- | --- |
| E15 | [#278](https://github.com/link-assistant/formal-ai/issues/278) |
| E16 | [#279](https://github.com/link-assistant/formal-ai/issues/279) |
| E17 | [#280](https://github.com/link-assistant/formal-ai/issues/280) |
| E18 | [#281](https://github.com/link-assistant/formal-ai/issues/281) |
| E19 | [#282](https://github.com/link-assistant/formal-ai/issues/282) |
| E20 | [#283](https://github.com/link-assistant/formal-ai/issues/283) |

## Verification

This is a documentation + planning PR. Verification:

- Repository quality checks run clean: `cargo fmt --check`,
  `cargo clippy --all-targets --all-features`,
  `rust-script scripts/check-file-size.rs`, `cargo test`, `cargo test --doc`.
- Release guard checks pass with PR-like env:
  `rust-script scripts/check-changelog-fragment.rs`,
  `rust-script scripts/check-version-modification.rs`.
- The initial plan was internally consistent: every original `#[ignore]`
  "tracked requirement" test and every `ARCHITECTURE.md` §16 open question was
  assigned to exactly one E1-E14 epic. The post-implementation audit confirmed
  zero tracked ignored tests remain and assigned the remaining partial
  requirements to E15-E20.
