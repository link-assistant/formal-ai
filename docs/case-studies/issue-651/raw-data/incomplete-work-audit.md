# Incomplete / Deferred Work Audit — link-assistant/formal-ai

Date: 2026-07-12. Prepared for issue #651.
Sources: all 11 open issues (bodies + comments), sampled closed issues #509–#647,
their closing PR bodies (#512, #524, #542, #549, #553, #555, #560, #564, #590,
#601, #609–#615, #618, #623, #629–#638, #645, #648), `REQUIREMENTS.md`,
`ROADMAP.md`, `docs/case-studies/*`, `.github/workflows/release.yml`.
Raw data in this folder: `open-issues-2026-07-12.json`,
`closed-issues-2026-07-12.json`, `open-issues-detail.txt`,
`closed-detail-{a,b,c}.txt`, `pr-bodies.txt`, `closing-prs.tsv`.

## A. Open issues (11) and their state

| # | Title (short) | What it asks | State |
|---|---|---|---|
| 651 | Create issues for most critical missing features | Audit past issues for undone work; update VISION/ROADMAP to track requirement status; create detailed sub-issues with blocking relationships; focus on associative tech; path to self-coding via Agent CLI + Hive Mind | Open; no sub-issues created yet; no case-study folder before this audit |
| 650 | `formal-ai with` follow-up (4 defects) | (1) `/responses` merges `instructions` into query → codex "hi" misrouted; (2) no-message runs should enter interactive mode uniformly (only opencode works; agent/gemini/qwen/codex fail); (3) summarization requests fail server-side for tools with no disable flag; (4) `--globally` alias; keep temp-config invariant tested | Open, 0 comments, no work. Explicit follow-up to #647 (shipped 0.278.0). claude/grok/aider never actually exercised |
| 649 | Predicting consequences via world models | Links-network world model per dialog (current + target state), merge/split contexts, relative-meta-logic dependent probabilities | Open, 0 comments, no case study |
| 557 | Buttons embedded into text field (desktop/tablet) | Adaptive UI, embedded buttons, multi-skin support (default/glass/material) with transparency slider; asserts **#108 was not fully done** (mobile) | Open, 0 comments, no case study |
| 534 | Repo taking 12 GB during development | Find root cause (cargo target? tests not cleaning up?), reduce compile size | Open, 0 comments, no investigation recorded |
| 531 | Patterns inference (1D/2D) | Reimplement Data.Doublets.Sequences dedup/compression converters in Rust; symmetry/rotation-based pattern inference; seed ontology of terms; first session = research + proposals, then maintainer decides | Open, 0 comments, no case study |
| 491 | Principle of least action | Optimize reasoning to fewest steps; every task splits into 2 sub-tasks (balanced tree) | Open, idea-stage |
| 483 | Small-model formalization fallback | Optional (off-by-default, on-demand-downloaded) small in-browser model to pick best formalization match; hardware-fit filtering; unit-tested | Open, no work |
| 482 | Nemotron-3-Ultra training data as tests | Pull ~10 random samples (no full download) as failing tests; solve by driving Formal AI via Agent CLI | Open, no work |
| 453 | Moonshot tasks | Recursive 2-part task splitting covering Atari-Breakout architecture, symbolic ChatGPT-like chatbot + benchmark, strong AI | Open, idea-stage |
| 447 | Dialog UI "интерфейс ужасен" | Left panel not scrollable with mouse (Windows/Firefox); maintainer (2026-07-05) asked for VS-Code-style thin resizer + full case study + "execute everything in this single PR" | Open; requested case study `docs/case-studies/issue-447` does **not** exist |

Note: none of the open issues have the case-study folder their own bodies demand
(`./docs/case-studies/issue-{id}`), i.e. no agent has started any of them.

## B. Concrete deferred / partial work items (with evidence)

1. **#650 (open)** — the whole issue is deferred work from #647: `/responses`
   instructions-merge misroute, uniform interactive mode, server-side
   summarization handling, `--globally` alias, PTY-path config regression tests.
   Evidence: issue #650 body ("Follow-up to #647 (shipped in 0.278.0)").
2. **#647/PR #648** — `claude` was "intentionally not run"; `grok`/`aider`
   "not installed locally … findings inferred from the shared adapters and
   should be validated when testable" (#650 footnote). Untested integrations.
3. **R378 (issue #538 / PR #601)** — bulk semantics importer: "a batch importer
   generalizing `scripts/ground-meanings.rs` remains the next scale step"
   (REQUIREMENTS.md R378, "Partially implemented"). No tracking issue.
4. **R379 (#538/#601)** — hardcoded natural-language string audit / CI lint over
   `src/`: "the broader codebase string/concept audit remains a follow-up"
   (REQUIREMENTS.md R379). No tracking issue.
5. **R380 (#538/#601)** — absorb remaining `src/web/worker/*.js` logic into the
   Rust→WASM worker: "absorbing the remaining `src/web/worker/*.js` logic is the
   tracked follow-up" (REQUIREMENTS.md R380). No tracking issue.
6. **R383 (#538/#601)** — interactive step-by-step debugging view (embedded VS
   Code, chat/data/mermaid/Rust/JS panes): "Tracked follow-up; related
   exploratory notes live under `docs/vscode/`" (REQUIREMENTS.md R383).
   No tracking issue.
7. **R384 (#538/#601)** — automatic probability-weighted statement
   formalization and contradiction detection/repair "remain follow-ups
   (solution-plan R18–R21)" (REQUIREMENTS.md R384). No tracking issue.
8. **R385 / #558** — issue #558 asked for "fully dynamic self programming or
   self learning" that can "recompile itself … reattach it to the UI". PR #637
   delivered a deliberately **human-gated, proposal-only** loop ("Nothing is
   ever auto-applied. Every artifact is a record for review"); the rebuild plan
   is "the reviewable product — nothing is rebuilt or restarted". REQUIREMENTS
   R385: "this is not yet arbitrary auto-learning, which is tracked by issue
   #558" — but #558 is now closed, leaving actual autonomous self-modification
   without a tracker.
9. **R282 (issue #398 era)** — Wikidata grounding covers only a core meaning
   set: "Expanding grounding from the current core set to every meaning is
   tracked as ongoing source-import work in
   `docs/case-studies/issue-398/README.md`" (REQUIREMENTS.md R282,
   "Partially implemented"). No issue.
10. **R271** — "full source-response importers are tracked as follow-up work in
    the case study" (REQUIREMENTS.md R271). No issue.
11. **REQUIREMENTS.md line 35 (issue #1 era, still standing)** — "Full imports
    of very large corpora such as Wikipedia, Wikidata, Rosetta Code,
    Wikifunctions, and hive-mind history should run as chunked follow-up dataset
    jobs" — never scheduled.
12. **#625 + #628** — the multi-CLI e2e **CI suite is still only proposed**:
    `docs/testing/agentic-cli-tools.md` §"CI Shape" says "The CI e2e suite
    *should* follow this sequence" and "feeds the CI e2e suite *proposed* in
    #625". `release.yml` contains no codex/opencode/gemini/qwen install; only
    `test-agent-cli-e2e` (our own `agent` CLI, from PR #601) exists. The proxy
    (PR #631) and the guide (PR #634) shipped; the CI job did not.
13. **#620 (closed by PR #623)** — closing note: "even with auth fixed,
    gemini-cli in headless `-p` mode advertises no tool/functionDeclarations, so
    it does chat but not tool actions — tracked as a comment above" — i.e. a
    known functional gap tracked only in a comment, never filed.
14. **#511/PR #512** — residual limitation: "approve-each relays only for
    `agent`/`claude` because `codex`/`gemini`/`qwen` expose no headless approval
    handshake — a documented upstream-CLI constraint". No tracker.
15. **#541/PR #542** — five out-of-scope follow-ups F1–F5 documented in
    `docs/case-studies/issue-541/proposed-issues.md` ("filed here, not in PR
    #542"): F1 snapshot dark-theme regression coverage (3 of 5 widgets only
    visually verified), F2 migration replay UI, F3 per-message animation budget,
    F4 reasoning-hierarchy editing, F5 IPC-level mode-flip tests. **None were
    ever filed as GitHub issues.** Also, the requested chakra-ui migration was
    explicitly rejected (with reasoning) rather than done.
16. **#552/PR #553** — the Google AI Mode share link "could not be converted
    from a static HTTP capture" (Google interstitial); deferred upstream to
    web-capture#141 and meta-language#168. The issue's own plan ("once our
    dependencies are enriched, we will use their latest versions, to fully
    support all these features ourselves") — that re-integration step never
    happened in this repo.
17. **ROADMAP pillar 7 / issue #412** — "A task-agnostic meta-builder
    ('algorithm that builds algorithms', R7) is the tracked next step in
    `docs/case-studies/issue-412`" — doc-tracked only, no issue.
18. **ROADMAP pillar 24** — program-synthesis "Triggering is still
    English-keyword gated — see the Next Planning Batch (language parity)".
19. **#557 (open)** — maintainer states "#108 was not fully done" (mobile
    embedded-button UI), i.e. a closed issue asserted as partially delivered.
20. **PR #601 self-AST slice** — census pinned to a single module
    (`planner.rs`); "smallest real next slice: extend the pinned target from one
    module to a directory census so 'all our Rust logic' is covered". PR #637
    later widened source-graph coverage (R558-04/05), but the meta-language
    CST/AST census in `data/meta/self-ast.lino` remains single-module.
21. **#651's own asks (open)** — ROADMAP/VISION refresh to requirement-status
    tracking, sub-issue creation with GitHub blocking relations — not yet done.

## C. Patterns of agents not fully executing issues

1. **Deferral moved out of GitHub and into repo docs.** The dominant pattern:
   agents mark work "Partially implemented … remains a follow-up" in
   REQUIREMENTS.md rows and case-study solution plans **without filing issues**
   (items 3–10, 15, 17 above). These items are invisible to the issue-driven
   workflow — exactly the gap #651 asks to close.
2. **The refusal/deferral anti-pattern was explicit enough to need a policy.**
   PR #601 records that an earlier draft "chose to ship the concrete verifiable
   core … honestly framing large research items as tracked follow-ups" and led
   with a "what did not ship" section; the maintainer rejected it ("opposite of
   my requirements … no refusals, no delays, no deferral, no follow ups"), and
   `docs/case-studies/issue-538/refusal-anti-pattern.md` became required reading
   in CONTRIBUTING.md. Despite that, R378–R384 still closed as "Partially
   implemented" follow-ups.
3. **Ambition downgraded to a safe subset.** #558 asked for self-recompiling,
   self-reattaching auto-learning; PR #637 shipped a proposal-only, human-gated
   loop (a reasonable guardrail, but the autonomous capability the issue
   describes has no successor tracker). Similarly #571's first fix was rejected
   as "seed-vocabulary too narrow" before PR #618 generalized it.
4. **Proposed test/CI infrastructure ships as documentation, not automation.**
   #628's testing guide and #625's proxy landed, but the actual CI matrix that
   would prevent regressions of #620/#624/#626/#627 is still a "should" in the
   guide (item 12).
5. **Case-study requirement ignored on open issues.** Every recent issue body
   demands `docs/case-studies/issue-{id}`; none of the 10 other open issues has
   one, and #447's explicit maintainer instruction (2026-07-05) has produced no
   branch/PR.
6. **Verification finds what closure claimed.** #647 was closed by PR #648 with
   all tests green, and immediate hands-on testing produced #650 with four
   defects, including untested claimed integrations (claude/grok/aider). Issue
   acceptance-criteria checkboxes are routinely left unchecked at close.
