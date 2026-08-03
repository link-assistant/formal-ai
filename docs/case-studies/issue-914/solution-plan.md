# Issue #914 Solution Plan

One plan per requirement group, each naming the existing components (in
this repository and outside it, from
[`raw-data/online-research.md`](raw-data/online-research.md)) that the
solution builds on, and the epic that owns the remaining work. Epic bodies
live in [`proposed-issues.md`](proposed-issues.md).

## Plan 1 — Documentation In Sync With Code (R914-1, R914-2, R914-3)

Approach: treat the 2026-07-14 audit table in `ROADMAP.md` as a set of
claims and re-verify every claim against the epic-status sweep
(`raw-data/issues-since-2026-07-14.tsv`) and against `src/`. Record the
result as a ninth-pass audit section dated 2026-08-03, correcting the
stale rows found (external benchmarks #698, search fusion #709, candidate
portfolios #662/#704, world-model dialogue #702 — all shipped after the
eighth pass but still listed as missing). Add the Issue #914 requirement
table to `REQUIREMENTS.md`, and guard the whole structure with the
docs-traceability test `issue_914_case_study_and_planning_docs_are_
traceable`, following the pattern of
`tests/unit/docs_requirements_issue_890.rs`.

Existing components: ROADMAP.md audit-pass convention (eight prior
passes), REQUIREMENTS.md per-issue tables, the Verification Contract at
the end of `ROADMAP.md`, the docs-traceability test family in
`tests/unit/`.

Done on this branch; no epic needed.

## Plan 2 — Coding First On A Solid Foundation (R914-9, R914-13, R914-14)

Approach: the #848 coding-task ladder already measures coding honestly
(2 of 13 rungs at baseline, zero successful write effects, verification by
observed workspace effect). The blocker is not missing capability design
but the agent-harness defect cluster #902-#909 (#902 was fixed on main
during this planning pass; #903-#909 remain open): success reported on
exit code 1 (#905, #908), coding tasks reduced to plan files (#904),
native CLI argument vectors built wrong (#903), lost provider blocks
(#902), caller framing hijacking intent routing (#907), language-router
misfires (#906), incomplete headless configuration (#909). E69 fixes these
behind the ladder as a ratchet: each fix must move a rung from red to
green, and the ladder score becomes a monotonic gate in CI, mirroring the
1,440-check ratchet used for issue #408. E77 then closes the loop: once
write-effect rungs pass, route one real repository change per release
through Formal AI itself and extend the `data/meta/` self-hosting ledger
so the self-development share is measured from its honest baseline, as
#657 established.

Existing components: `experiments/issue_847_coding_ladder/` dataset and
runner, `src/agentic_coding/` (planner, driver, capability router),
`src/orchestration/` session replay, the #656 benchmark-gated promotion
protocol, the #657 self-hosting ledger, `src/self_ast_census.rs`.

Owned by E69 (foundation, blocker) and E77.

## Plan 3 — General Natural-Formal Translation (R914-5, part one)

Approach: extend the proven meta-language pipeline
(`src/translation/`, round-trip contract from #526) from the four seed
natural languages toward *formal* targets as first-class languages: logic
statements, proofs, and programs become concrete syntaxes projected from
the same `MeaningId` layer that natural languages already use, the way
issue #890 did for proofs to Rust and Python. Adopt the abstract-syntax /
concrete-syntax split proven by Grammatical Framework (one abstract tree,
many concrete languages, near 40 in its resource library; Ranta's
Informath demonstrates mathematical text to Lean and back) as the design
reference, and an Attempto-style controlled natural language as the
guaranteed-unambiguous entry path, while keeping the implementation
native to this repository's link substrate rather than importing GF
itself. Grammar and lexicon metadata come from the seed (Plan 4), so
adding a language stays a data change.

Existing components: `src/translation/` and its semantic meta language,
`src/proof_program.rs` plus `data/seed/proof-program-templates.lino`
(#890), `src/summarization/` NSM primes, Grammatical Framework and
ACE/APE as design references, Universal Dependencies treebanks and Open
English WordNet and FrameNet as license-safe grammar and frame metadata
sources.

Owned by E70.

## Plan 4 — Minimal Core Plus Metadata-Rich Seed (R914-6)

Approach: define the core boundary as "the meta algorithm, the link
store, and the interpreters" and audit everything else for migration to
seed data. Continue the #699 handler burn-down (about 19,600 lines across
40 handler files remain in `src/solver_handlers/`) with a per-handler
ledger: each handler is either migrated to seed rules, promoted into the
documented core with a stated reason, or deleted. In parallel, audit the
seed for problem-solving metadata: every concept record should carry the
frame-style metadata people use to solve problems (roles, preconditions,
effects, units, examples), taking FrameNet's frame-and-role shape and
Wikidata's typed properties as the vocabulary sources. Gate the boundary
with a ratchet script in the style of
`scripts/check-hardcoded-language.rs`.

Existing components: `src/recipe_interpreter.rs` executing
`data/meta/recursive-core-recipe.lino`, the #699 migration machinery, the
five rule shapes (VISION.md), `data/seed/` (117 files), FrameNet and
Wikidata as metadata vocabularies, the existing burn-down-gate script
pattern.

Owned by E71.

## Plan 5 — No Neural Reasoning, Growing Formal Coverage (R914-7)

Approach: the invariant is already enforced (NON-GOALS.md; no ML crates in
the dependency tree; the only sanctioned exception is the opt-in #483
formalization fallback) and every existing test is the regression floor.
Growth comes from widening the proof engine beyond propositional SAT and
linear arithmetic: equality and rewriting via e-graph saturation (the egg
and egglog libraries are MIT-licensed Rust), relational and rule-based
inference via an embedded Datalog (Ascent) or pure-Rust Prolog (Scryer)
where a dependency is justified, and SMT-style decision procedures either
native or through the `z3` bindings as an optional feature. Each addition
lands only with benchmark cases drawn from the external corpora surveyed
in the online research, scored honestly through the #698
external-benchmark harness.

Existing components: `src/proof_engine/` decision procedures and library,
`src/external_benchmarks/`, `src/probability.rs`, egg and egglog, Ascent,
Scryer Prolog, z3 and cvc5 Rust bindings, Popper-style learning from
failures as a later reference.

Owned by E76.

## Plan 6 — Internet Knowledge Discovery, Coding First (R914-8)

Approach: turn retrieval into learning. Today the pipeline captures and
fuses search results to answer questions; E72 adds the loop that converts
retrieved material into *verified coding procedures*: fetch documentation
or examples for an unknown coding task (providers already include Rosetta
Code, Wikifunctions, and Stack Overflow snapshots), formalize into the
meta language, compile into a candidate procedure (the #897 verified
procedure machinery), verify by executing in the bounded workspace, and
keep the procedure with full provenance only when execution proves it.
Failures become recorded skill gaps (#873: not knowing is not the end),
which seed the next research round.

Existing components: `src/web_search_core.rs`, `src/search_fusion.rs`,
`src/source_fetch.rs` provenance cache, `src/knowledge.rs` oracles,
`src/skill_procedure.rs` and the #897 verified procedures,
`src/program_skill_gap.rs`, open issues #873 and #896 as the tracked
demand.

Owned by E72.

## Plan 7 — Working With Unknowns, Minimal Questions (R914-10)

Approach: add a necessity proof to every question. Before the solver may
ask the user anything, it must record a three-step search trace showing
the answer was not in memory, not derivable from the workspace, and not
discoverable from cached or live sources within budget — otherwise the
question is answered autonomously and only logged. Questions that survive
are classified: requirement-level unknowns (preferences, intent, real-world
facts only the user holds) may be asked; everything else may not. The
existing smallest-question selection and the at-most-one-question rule
stay; the protocol makes them auditable, and a benchmark counts questions
per solved task so the number can only ratchet down.

Existing components: `src/translation/selection.rs` clarify-vs-guess with
`guess_probability` and `questioning_rigor`, `src/solver_unknown_
reasoning.rs`, the #527 question catalog and its
grammaticality/meaningfulness classifiers, the append-only event log for
the necessity trace.

Owned by E73.

## Plan 8 — Hive-Mind Integration Gate (R914-11)

Approach: prove the full circle in both directions with one replayable
end-to-end scenario per direction. Direction one (Formal AI as the model):
hive-mind's `solve ISSUE_URL --tool agent --model formal-ai`
(hive-mind#2059) drives the Agent CLI against the OpenAI-compatible
server, and the gate asserts an observed workspace effect — a real file
change landing as a commit — not narrated success, reusing the #848
verification style and the issue #890 Agent-CLI evidence pattern
(byte-exact replay script plus CI job). Direction two (Formal AI as the
orchestrator): `src/orchestration/` dispatches an external CLI on a
hive-mind-shaped task with the hash-chained session record replayed in
CI. Both scenarios become case-study evidence folders and a permanent
integration test.

Existing components: `src/orchestration/` (#703), `src/server.rs`
protocol namespaces, the `formal-ai with` wrapper and
`data/seed/client-integrations.lino`, `scripts/mine-hive-mind-dataset.rs`,
`experiments/issue_890_agent_cli.sh` as the replay-gate template, closed
issue #655 groundwork.

Owned by E74.

## Plan 9 — Learning The Universal Algorithm (R914-5, part two)

Approach: the algorithm is already data (`recursive-core-recipe.lino`)
executed by an interpreter, so learning it means proposing recipe
improvements from recorded experience. E75 adds a proposal loop: mine the
event log of solved and failed problems for recurring step sequences,
compress them into candidate method abstractions (the corpus-guided
abstraction approach that DreamCoder pioneered and the Rust `stitch_core`
crate makes fast is the reference), and register candidates in the method
registry as *proposals* that only promotion (#656 benchmark gate plus
human confirmation) can adopt. This keeps self-modification human-gated
while making the core loop itself improvable, which is the literal reading
of "the system learns the universal problem-solving algorithm".

Existing components: `src/recipe_interpreter.rs`, `src/method_registry.rs`
and `src/meta_method_dispatch.rs`, `src/algorithm_discovery.rs` (inert
discovery with green-gate approval), `src/learning_cycle.rs` and the
adoption ledgers, `src/promotion.rs`, Stitch as the external reference for
trace compression.

Owned by E75.

## Plan 10 — Evidence And Case Study (R914-15)

Approach: this folder. Raw GitHub snapshots, the post-audit issue sweep,
the online component research, the requirement table, this plan, and the
epic bodies with recorded issue URLs, all guarded by the traceability
test.

Done on this branch; no epic needed.
