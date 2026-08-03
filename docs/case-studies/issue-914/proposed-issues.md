# Issue #914 Proposed Issues: Epic Batch E69-E77

Nine epics continue the numbering from E68 (the #651 batches). They come
from the gap analysis in [`README.md`](README.md) section 3 and the
per-requirement plans in [`solution-plan.md`](solution-plan.md). E69 is
the foundation blocker (R914-14: critical vision-blocking code problems
are fixed first); every capability epic states which E69 outcomes it
depends on. Each epic below is one GitHub issue, labeled `enhancement`,
linking parent issue #914.

## Opened issues

Opened on 2026-08-03 from this document, each with the `enhancement`
label, linking parent issue #914:

- E69: <https://github.com/link-assistant/formal-ai/issues/916>
- E70: <https://github.com/link-assistant/formal-ai/issues/917>
- E71: <https://github.com/link-assistant/formal-ai/issues/918>
- E72: <https://github.com/link-assistant/formal-ai/issues/919>
- E73: <https://github.com/link-assistant/formal-ai/issues/920>
- E74: <https://github.com/link-assistant/formal-ai/issues/921>
- E75: <https://github.com/link-assistant/formal-ai/issues/922>
- E76: <https://github.com/link-assistant/formal-ai/issues/923>
- E77: <https://github.com/link-assistant/formal-ai/issues/924>

## Design rules that bind every epic

1. **Foundation first.** No capability epic ships behavior that depends on
   an agent-harness path still failing an E69 ladder rung. Fix the
   blocker, then build on it (R914-14).
2. **Keep the regression floor.** Every existing test keeps passing;
   generalization replaces special cases only when the general path proves
   it covers them (R914-7, R914-13).
3. **No neural networks in reasoning.** The reasoning path stays fully
   symbolic; the only sanctioned exception remains the strictly opt-in
   #483 formalization fallback, which never steers (NON-GOALS.md).
4. **Verification by observed effect.** Success claims come from executed
   checks and observed workspace or network effects, never from narrated
   output (the #848 rule).
5. **Web as cache, not teacher.** External knowledge enters only through
   the provenance-tracked source cache with recorded license and fetch
   metadata.
6. **Determinism and traceability.** Same prompt plus same config yields
   the same answer; every learned artifact lands in the append-only log
   with a replayable trail, and promotion stays human-gated behind the
   #656 benchmark ratchet.

## E69 — Coding-ladder ratchet over agent-harness fixes — FOUNDATION, BLOCKER

**Problem.** Issue #914 makes coding the first skill to complete, and
requires critical vision-blocking code problems fixed first. The #848
executable coding-task ladder measures the skill honestly: at baseline 2
of 13 rungs pass and zero write-effect tasks succeed. The open defect
cluster #902-#909 explains why: success is reported despite exit code 1
(#905, #908), agent mode reduces a coding task to writing a plan file
(#904), the native CLI argument vector is built wrong (#903), the codex
path loses its provider block (#902), caller framing hijacks intent
routing (#907), the language router misfires (#906), and `--global`
headless configuration is incomplete (#909). Until these are fixed, every
coding-first epic builds on sand.

**Approach.** Consolidate #902-#909 behind the ladder as a ratchet: each
fix must move at least one rung from red to green, with the observed
workspace effect (file created, test passing, commit made) as the only
accepted evidence. Wire the ladder score into CI as a monotonic gate in
the style of the issue #408 ratchet (1,440 local checks) so the score can
never silently regress. Close the epic when every write-effect rung
passes.

**Existing components.** `experiments/issue_847_coding_ladder/` dataset
and runner; `src/agentic_coding/` planner, driver, and capability router;
`src/orchestration/` replayable sessions; the #656 promotion gate; open
issues #902-#909 (this epic coordinates them, it does not duplicate
them).

**Acceptance criteria.**
- Every open defect in #902-#909 is fixed or explicitly closed with a
  recorded reason, each fix tied to a named ladder rung.
- The coding ladder passes all write-effect rungs; the score is enforced
  as a monotonic ratchet in CI.
- No success path reports completion without an observed workspace
  effect; exit codes propagate to the reported outcome.
- The regression floor holds: all existing tests pass.

## E70 — General natural-formal translation through the meta language

**Problem.** The vision requires truly solving translation between
languages — natural and formal (issue #914). Today `src/translation/`
round-trips four natural languages through the semantic meta language,
and #890 projects solved proofs into Rust and Python, but formal
languages are not yet first-class translation targets: there is no
general path from a natural statement to a logic formula, proof
obligation, or program specification and back.

**Approach.** Make formal languages concrete syntaxes of the existing
meta language: one abstract meaning layer, many projections, following
the abstract/concrete split proven by Grammatical Framework (whose
Informath line translates mathematical text to Lean and back) and the
unambiguous-entry design of Attempto Controlled English — as design
references, implemented natively on the link substrate. Grammar and
lexicon metadata live in the seed so adding a language stays a data
change (rule 5 of VISION.md's rule shapes). Round-trip survival (#526)
extends to natural-to-formal-to-natural pairs.

**Existing components.** `src/translation/` and its meta language;
`src/proof_program.rs` plus `data/seed/proof-program-templates.lino`
(#890); `src/intent_formalization.rs` P/Q anchoring; NSM primes in
`src/summarization/`; Grammatical Framework, ACE/APE, Universal
Dependencies, Open English WordNet, and FrameNet as external references
and license-safe metadata sources (see the issue #914 online research).

**Acceptance criteria.**
- A natural-language statement in any seed language translates to at
  least one formal target (logic statement, proof obligation, or program
  specification) and back, surviving the round trip.
- The formal targets are seed-defined projections of the meta language,
  not per-pair translators.
- The #526 round-trip suite extends to the new pairs and passes.
- Depends on E69 only where translation output is executed as code.

## E71 — Minimal-core boundary and seed-metadata audit

**Problem.** The vision requires a minimum core of algorithms plus a data
seed with metadata rich enough to problem-solve the way people do (issue
#914). The meta algorithm is already executed as data, but the #559
mandate is unmet: about 19,600 lines across 40 files remain in
`src/solver_handlers/` after the #699 migration, and no audit defines
which metadata each seed record must carry.

**Approach.** Define the core boundary explicitly (meta algorithm, link
store, interpreters, surfaces) and put every handler on a burn-down
ledger: migrate to seed rules, promote into the documented core with a
stated reason, or delete. Gate the boundary with a ratchet script in the
style of `scripts/check-hardcoded-language.rs`. In parallel, audit seed
records against a declared metadata schema for problem solving — roles,
preconditions, effects, units, examples — taking FrameNet's
frame-and-role shape and Wikidata's typed properties as vocabulary
sources, and record per-record gaps as data.

**Existing components.** `src/recipe_interpreter.rs` executing
`data/meta/recursive-core-recipe.lino`; the #699 migration machinery and
its handler registry; `data/seed/` (117 lino files); the burn-down-gate
script pattern; FrameNet and Wikidata vocabularies.

**Acceptance criteria.**
- A documented core boundary exists and a ratchet script enforces that
  handler code outside it only shrinks.
- Every remaining handler has a ledger entry: migrated, promoted with
  reason, or deleted.
- The seed metadata schema is documented and at least the concept records
  used by the coding path satisfy it, with gaps recorded as data.
- The regression floor holds.

## E72 — Research-driven coding knowledge loop

**Problem.** The system must learn to discover enough knowledge from the
internet and other sources to solve all tasks, coding first (issue
#914). Retrieval today is answer-oriented: search results are fused and
presented, but retrieved material does not become reusable, verified
coding capability, and open issues #873 and #896 track exactly this
demand.

**Approach.** Add the loop: when a coding task hits a recorded skill gap,
plan a research query, fetch through the provenance-tracked source cache,
formalize the retrieved material into the meta language, compile a
candidate procedure (#897 machinery), verify it by execution in the
bounded workspace, and keep it only when execution proves it — with
source, license, and fetch metadata attached. Failed research rounds
update the gap record so "not knowing is not the end" (#873): the gap
itself schedules the next round.

**Existing components.** `src/web_search_core.rs` and
`src/search_fusion.rs`; `src/source_fetch.rs` provenance cache;
`src/knowledge.rs` oracles (Rosetta Code, Wikifunctions, Stack Overflow
snapshots); `src/skill_procedure.rs` and verified procedures (#897);
`src/program_skill_gap.rs`; open issues #873 and #896.

**Acceptance criteria.**
- At least one coding task that fails as a skill gap is solved end to end
  by the research loop, with the learned procedure kept with full
  provenance and replayed deterministically from cache in CI.
- Procedures learned from research are marked as such and pass the same
  execution verification as hand-seeded ones.
- Live fetching stays opt-in; offline mode replays the cache.
- Depends on E69 for the execution-verification path.

## E73 — Question-necessity protocol

**Problem.** The system must work with unknowns, gathering missing
information itself and asking the user as few questions as possible —
only the requirement-level and real-world questions nobody else can
answer (issue #914). The clarify-vs-guess selector, the
at-most-one-question unknown path, and the #527 question catalog exist,
but nothing proves a given question was necessary before it was asked.

**Approach.** Require a necessity trace for every user-facing question:
a recorded three-step search showing the answer was not in memory, not
derivable from the workspace, and not discoverable from sources within
budget. Classify surviving questions — requirement-level unknowns
(intent, preferences, real-world facts only the user holds) may be
asked; factual unknowns are researched instead and only logged. Add a
questions-per-solved-task benchmark that can only ratchet down.

**Existing components.** `src/translation/selection.rs`
(`guess_probability`, `questioning_rigor`, smallest-question selection);
`src/solver_unknown_reasoning.rs`; the #527 question catalog and
classifiers; the append-only event log for the trace; E72's research
loop as the autonomous alternative to asking.

**Acceptance criteria.**
- Every asked question carries a replayable necessity trace in the event
  log; questions without one are refused by the solver itself.
- The requirement-vs-fact classification is seed data, not code.
- The questions-per-task benchmark exists and is wired as a ratchet.
- The regression floor holds, including existing clarify-vs-guess tests.

## E74 — Hive-mind end-to-end integration gate

**Problem.** Formal AI must be well integrated with link-assistant/
hive-mind through agentic harness CLIs and TUIs (issue #914). The pieces
exist — the orchestration module (#703) dispatches external CLIs, the
server speaks the OpenAI and Anthropic protocols, and hive-mind#2059
specifies `solve ISSUE_URL --tool agent --model formal-ai` — but no gate
proves a full circle in either direction.

**Approach.** Build one replayable end-to-end scenario per direction and
keep both as permanent gates. Direction one: hive-mind drives the Agent
CLI with Formal AI as the model behind the OpenAI-compatible server, and
the gate asserts an observed workspace effect landing as a commit,
following the byte-exact replay pattern of
`experiments/issue_890_agent_cli.sh` and its CI job. Direction two:
Formal AI's orchestrator dispatches an external agent CLI on a
hive-mind-shaped issue task, with the hash-chained session replayed in
CI. Evidence lands as case-study folders.

**Existing components.** `src/orchestration/` (#703);
`src/server.rs` protocol namespaces; the `formal-ai with` wrapper and
`data/seed/client-integrations.lino`;
`scripts/mine-hive-mind-dataset.rs`; the #890 Agent-CLI evidence and
replay-gate pattern; closed groundwork in #655; hive-mind#2059.

**Acceptance criteria.**
- Both directions have committed evidence folders and deterministic
  replay scripts gated in CI.
- Direction one succeeds through the exact hive-mind invocation shape
  from hive-mind#2059, with an observed workspace effect.
- Failures anywhere in the chain propagate honestly (no narrated
  success), reusing E69's exit-code guarantees.
- Depends on E69.

## E75 — Method learning for the universal problem-solving algorithm

**Problem.** The system must be able to learn the universal
problem-solving algorithm (issue #914) — not only execute it. Today the
11-step loop is data (`data/meta/recursive-core-recipe.lino`) run by
`src/recipe_interpreter.rs`, and `src/algorithm_discovery.rs` can
discover parameterized algorithms from traces, but nothing feeds solved
and failed problem experience back into the recipe or the method
registry.

**Approach.** Mine the append-only event log for recurring step
sequences across solved problems, compress them into candidate method
abstractions (corpus-guided abstraction in the DreamCoder line; the Rust
`stitch_core` crate is the external reference for making the compression
fast), and register candidates in the method registry strictly as
proposals. Adoption goes only through the #656 benchmark-gated promotion
protocol with human confirmation, so self-modification stays gated while
the core loop becomes improvable from experience.

**Existing components.** `src/recipe_interpreter.rs`;
`src/method_registry.rs` and `src/meta_method_dispatch.rs`;
`src/algorithm_discovery.rs` (inert, green-gate approved);
`src/learning_cycle.rs` and adoption ledgers; `src/promotion.rs` (#656);
Stitch as external reference.

**Acceptance criteria.**
- At least one method abstraction is proposed from real event-log
  traces, survives the benchmark gate, and is adopted into the registry
  through the human-confirmed promotion path.
- Proposals are inert until promoted; rejected proposals remain recorded
  with reasons.
- The recipe-equals-source test
  (`tests/unit/specification/recursive_core_recipe.rs`) still passes
  after adoption.
- The regression floor holds.

## E76 — Formal-reasoning coverage growth

**Problem.** Reasoning must stay free of neural networks while the
formal-reasoning implementation grows to cover all existing test cases
and much more (issue #914). The proof engine currently covers
propositional SAT and linear arithmetic; equality reasoning, rule-based
inference at scale, and richer decision procedures are missing, and no
external reasoning benchmark corpus is exercised.

**Approach.** Widen the symbolic kernel in verified steps: equality and
rewriting through e-graph saturation (egg and egglog are MIT-licensed
Rust libraries), scalable rule inference through embedded Datalog
(Ascent) or pure-Rust Prolog (Scryer) where a dependency is justified
against reimplementation, and SMT-style procedures either native or via
the `z3` bindings behind an optional feature. Every addition lands with
benchmark cases drawn from the corpora surveyed in the issue #914 online
research, scored honestly through the #698 external-benchmark harness,
and the existing suite is the floor.

**Existing components.** `src/proof_engine/` (SAT, linear, library,
presenters); `src/external_benchmarks/` (#698); `src/probability.rs` and
`src/world_model.rs`; egg, egglog, Ascent, Scryer Prolog, z3 and cvc5
Rust bindings (licenses recorded in the online research).

**Acceptance criteria.**
- At least two new reasoning capabilities (for example equality
  saturation and rule-based inference) land with external benchmark
  scores recorded in `data/benchmarks/`.
- No neural inference enters the dependency tree; any new dependency is
  license-checked and feature-gated.
- All pre-existing reasoning tests still pass unchanged.

## E77 — Self-development loop, coding first

**Problem.** Once Formal AI can code, that skill must speed up Formal
AI's own development (issue #914) — the reason coding comes first. The
self-hosting metric (#657) measures the share honestly from its
baseline, and the promotion protocol (#656) gates changes, but no
recurring loop routes real repository work through Formal AI itself.

**Approach.** After E69's write-effect rungs pass, establish the loop:
each release cycle, at least one real, reviewable repository change
(documentation sync, seed data update, test addition, or a small fix) is
produced by Formal AI — directed through the hive-mind path from E74 or
the Agent CLI directly — landing as a normal reviewed pull request. The
self-hosting ledger in `data/meta/` records each contribution, the share
is reported per release, and the target only ratchets up. Every change
passes the same review, CI, and promotion gates as human work.

**Existing components.** The #657 self-hosting ledger and metric;
`src/promotion.rs` (#656); `src/self_source_links.rs` and
`src/self_ast_census.rs` (#673); the agentic-coding recipe
(`data/meta/agentic-coding-recipe.lino`, #468); E69's ladder and E74's
integration gate.

**Acceptance criteria.**
- At least one merged pull request per release cycle is authored by
  Formal AI end to end, with replayable session evidence.
- The self-hosting share is reported per release from the ledger and is
  wired as a ratchet (may not silently decrease).
- Every self-authored change passes unmodified review, CI, and promotion
  gates.
- Depends on E69 and E74.
