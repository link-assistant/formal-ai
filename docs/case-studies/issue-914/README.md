# Issue #914 Case Study: Vision Implementation Planning, Coding First

Issue [#914](https://github.com/link-assistant/formal-ai/issues/914) asks
for a fresh full-vision planning pass in the lineage of
[#244](https://github.com/link-assistant/formal-ai/issues/244) (the first
epic batch, E1-E34) and
[#651](https://github.com/link-assistant/formal-ai/issues/651) (the gap
audit that produced E37-E68), with one new emphasis: **coding via formal
logical reasoning is the first skill to complete**, so that Formal AI can
speed up its own development. The issue asks, in order: (1) bring the
documentation fully in sync with the actual state of the code, (2) create
all the issues needed to fully implement the vision, fixing critical
vision-blocking code problems first, and (3) keep the evidence for the plan
in this folder.

The extracted requirement list lives in [`requirements.md`](requirements.md)
(R914-1 to R914-15). Per-requirement solutions are in
[`solution-plan.md`](solution-plan.md). The full epic bodies and the record
of opened issues are in [`proposed-issues.md`](proposed-issues.md). This
plan is verified by the docs-traceability test
`issue_914_case_study_and_planning_docs_are_traceable` in
`tests/unit/docs_requirements_issue_914.rs`.

## 1. Collected Data

- `raw-data/github/issue.json` — issue #914 body and metadata, plus
  `issue-comments.json` (empty at collection time), and the matching
  pull-request #915 snapshots (`pull-request.json`,
  `pull-conversation-comments.json`, `pull-review-comments.json`,
  `pull-reviews.json`).
- `raw-data/issues-since-2026-07-14.tsv` — all 152 issues updated since the
  eighth roadmap audit (2026-07-14), with state and title, used to
  re-verify every roadmap row against reality.
- `raw-data/online-research.md` — the external landscape: symbolic
  reasoning engines, non-neural NL-to-formal bridges, program synthesis
  without large language models, knowledge seeds, and the 2024-2026
  "verification convergence" trend, with licenses and Rust availability.
- Prior audits reused as input: `ROADMAP.md` (eight audit passes, vision
  pillars table, 2026-07-14 requirement-status table), `REQUIREMENTS.md`
  (73 per-issue tables, 687 requirement rows), `VISION.md`, `GOALS.md`,
  `NON-GOALS.md`, `docs/philosophy.md`, and the #244/#651 case studies.
- Epic-status sweep of the open planning batches (2026-08-03): of the
  E37-E68 issues, only #665, #666, #667, #668, #669, #670 (delivery
  breadth), #700 (si-units), #705 (anticipatory dreaming), and #710
  (dropped-requirements backlog) remain open, plus the #651 parent. All
  other epic issues (#656-#664, #671-#674, #681, #682, #686, #687,
  #698-#699, #701-#704, #706-#709) are closed with merged pull requests.

## 2. Requirements

See [`requirements.md`](requirements.md) for the full R914-1 to R914-15
table. The short form: use all prior evidence (R914-1); sync docs with code
(R914-2, R914-3); create the full issue plan (R914-4, R914-12); learn the
universal algorithm and general natural-formal translation (R914-5); keep a
minimal core plus a metadata-rich seed (R914-6); no neural networks in
reasoning, and formal reasoning must keep covering every existing test case
(R914-7); learn to discover knowledge from the internet, coding first
(R914-8, R914-9); minimize user questions to requirement-level unknowns
(R914-10); integrate with hive-mind through agentic harness CLIs (R914-11);
generalize without regressions (R914-13); fix vision-blocking code problems
first (R914-14); and keep the analysis in this folder (R914-15).

## 3. Current State And Gap Per Theme

Each subsection states what the code actually does today (verified against
`src/` and `tests/` on this branch, v0.324.1, ~150k lines of Rust in
`src/`, 2,427 unit tests plus 300 integration tests and 93 browser specs)
and what gap the new epics own.

### 3.1 Coding via formal reasoning, coding first

Current state: three coding layers exist — the data-driven template
catalog (`src/coding/`), the cached knowledge oracle for uncatalogued
languages (`src/knowledge.rs` over Rosetta Code, Wikifunctions, Stack
Overflow snapshots), and program synthesis with executed verification
(`src/solver_handlers/program_synthesis.rs` plus the bounded `src/agent.rs`
sandbox). The agentic loop is 48 files in `src/agentic_coding/` and
`src/orchestration/` dispatches external agent CLIs from a data-defined
registry. Issue #848 built the executable coding-task ladder
(`experiments/issue_847_coding_ladder/`): at baseline **2 of 13 rungs pass
and zero write-effect tasks succeed**, with verification by observed
workspace effect rather than narration.

Gap: the defect cluster #902-#909 (#902 fixed on main during this
planning pass, #903-#909 still open) shows the agent harness losing
provider blocks, reporting success on exit code 1, reducing coding tasks
to plan files, and misrouting caller framing. These are the critical
vision-blocking problems R914-14 requires fixing first. Owned by **E69**
(ratchet the ladder over the consolidated harness fixes) and **E77** (turn
working coding into a measured self-development loop).

### 3.2 Natural-formal translation and learning the universal algorithm

Current state: `src/translation/` (about 4,500 lines) implements the real
`source -> formalize -> semantic meta language -> deformalize -> target`
pipeline over Wikidata, Wikipedia, and Wiktionary with no built-in
translation table; round-trip survival is enforced by
`every_supported_language_pair_round_trips_via_meta_language`. The
11-step universal loop (`src/solver.rs`) is executed as data from
`data/meta/recursive-core-recipe.lino` by `src/recipe_interpreter.rs`.

Gap: only four seed languages (en, ru, hi, zh) are fully covered, formal
languages (logic, proofs, programming languages) are translation targets
only in narrow paths (#890 proofs), and the universal algorithm is
executed as data but not yet *learned or improved* by the system itself.
Owned by **E70** (general natural-formal translation) and **E75** (method
learning: the algorithm improves through recorded problem-solving
experience).

### 3.3 Minimal core of algorithms plus a metadata-rich data seed

Current state: the seed is 117 `.lino` files (about 48,500 lines) driving
responses, language detection, intent routing, handler precedence, the
coding catalog, and the client registry; the meta algorithm itself is seed
data proven equal to the live source by
`tests/unit/specification/recursive_core_recipe.rs`.

Gap: the #559 mandate ("memory plus meta algorithm only") is not met —
`src/solver_handlers/` still holds about 19,600 lines across 40 handler
files even after the #699 migration, and no audit states which metadata
each seed record must carry for human-like problem solving. Owned by
**E71** (minimal-core boundary and seed-metadata audit).

### 3.4 Discovering knowledge from the internet

Current state: capture-then-fuse retrieval — a multi-engine search planner
(`src/web_search_core.rs`), reciprocal-rank and statement-level fusion with
provenance (`src/search_fusion.rs`), content-addressed cached fetching via
a `curl` subprocess (`src/source_fetch.rs`), live access opt-in through
`FORMAL_AI_LIVE_API`, providers DuckDuckGo, Internet Archive, Wikipedia,
Wikidata, Wiktionary.

Gap: discovery is answer-oriented, not learning-oriented — retrieved
knowledge does not yet become reusable coding procedures, and the open
issues #873 ("not knowing is not the end") and #896 remain. Owned by
**E72** (research-driven coding knowledge loop: fetch, formalize, compile
to a procedure, verify by execution, keep with provenance).

### 3.5 Working with unknowns and asking fewer questions

Current state: three mechanisms — temperature-gated clarify-vs-guess with
the smallest disambiguating question (`src/translation/selection.rs`), an
at-most-one-question unknown-reasoning path
(`src/solver_unknown_reasoning.rs`), and the #527 generated question
catalog (`src/question_generation.rs`).

Gap: no protocol proves a question was *necessary* — that the answer could
not have been obtained from memory, the workspace, or the internet first,
and nothing separates requirement-level questions (only the user can
answer) from fact questions (the system should research). Owned by **E73**
(question-necessity protocol).

### 3.6 Hive-mind integration through agentic harnesses

Current state: three touchpoints — `data/seed/projects.lino` answers "What
is Hive Mind?" in every seed language; `scripts/mine-hive-mind-dataset.rs`
mines hive-mind issues, pull requests, and CI runs as a dataset; and
`src/orchestration/` (issue #703, closed) dispatches external agent CLIs
with deny-by-default permissions and replayable hash-chained sessions.

Gap: no end-to-end gate proves the reverse direction — hive-mind driving
Formal AI as the model behind an agentic CLI
(`solve ISSUE_URL --tool agent --model formal-ai`, hive-mind#2059) — and
no benchmark measures a full issue-to-pull-request round trip. Owned by
**E74** (hive-mind end-to-end integration gate).

### 3.7 Formal reasoning without neural networks, covering all tests

Current state: reasoning is fully symbolic — `src/proof_engine/` (SAT and
linear decision procedures, proof library, presenters), symbolic
probability (`src/probability.rs`), world models with justification-based
truth maintenance (`src/world_model.rs`), and zero neural inference in the
dependency tree (NON-GOALS.md; the only sanctioned exception is the
strictly opt-in #483 formalization fallback). All 2,427 unit tests pass on
this foundation.

Gap: coverage must grow toward "all existing test cases and much more" —
the proof engine's decision procedures are narrow (propositional SAT,
linear arithmetic), and external benchmark corpora (from the components
surveyed in `raw-data/online-research.md`) are not yet exercised. Owned by
**E76** (formal-reasoning coverage growth).

### 3.8 Documentation sync

Current state: ROADMAP.md's 2026-07-14 audit table is stale in at least
four rows — external benchmarks (#698), multi-source search fusion (#709),
parallel candidate portfolios and budget search (#662, #704), and
world-model dialogue behaviors (#702) all shipped and closed after the
audit, yet were still listed "Not done" or "Partial".

Fix: the ninth-pass audit section in `ROADMAP.md` (2026-08-03, this
branch) re-verifies every row of the 2026-07-14 table against the epic
sweep in section 1 and against `src/`, and records the E69-E77 batch as
the open planning batch. `REQUIREMENTS.md` gains the Issue #914 table.

## 4. Planned Epics

Nine epics, numbered continuing from E68. E69 is the foundation blocker;
the capability epics build on it. Full bodies, binding design rules, and
opened-issue URLs are in [`proposed-issues.md`](proposed-issues.md).

| Epic | Title | Owns |
| --- | --- | --- |
| E69 | Coding-ladder ratchet over agent-harness fixes | R914-9, R914-14 |
| E70 | General natural-formal translation | R914-5 |
| E71 | Minimal-core boundary and seed-metadata audit | R914-6 |
| E72 | Research-driven coding knowledge loop | R914-8 |
| E73 | Question-necessity protocol | R914-10 |
| E74 | Hive-mind end-to-end integration gate | R914-11 |
| E75 | Method learning for the universal algorithm | R914-5 |
| E76 | Formal-reasoning coverage growth | R914-7 |
| E77 | Self-development loop, coding first | R914-9 |

## 5. Verification

- `tests/unit/docs_requirements_issue_914.rs` —
  `issue_914_case_study_and_planning_docs_are_traceable` asserts that
  `REQUIREMENTS.md` carries the R914 rows, that `ROADMAP.md` carries the
  ninth-pass audit and the E69-E77 batch, and that every file in this
  folder exists with its stated sections.
- The epic sweep in section 1 is reproducible from
  `raw-data/issues-since-2026-07-14.tsv` plus the GitHub issue states
  recorded in `proposed-issues.md`.
- No behavior code changed on this branch, so the full existing test suite
  is the regression floor.
