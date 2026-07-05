# Issue 558 Requirements: Auto Learning

This file extracts the requirements from
<https://github.com/link-assistant/formal-ai/issues/558>. It also separates
what issue #538 / PR #601 already proved from what remains for a closed
self-learning loop.

Status legend:

- **Documented in this PR** - this PR preserves the analysis and delivery plan.
- **Implemented slice in PR #637** - this PR lands a working, tested, human-gated
  slice of the requirement (short of the full loop).
- **Built slice exists** - a useful implementation slice already exists, but it
  is not the full issue #558 loop.
- **Not yet built** - no implementation exists beyond adjacent pieces.

| ID | Requirement | Acceptance criterion | Current status |
| --- | --- | --- | --- |
| R558-01 | When Formal AI cannot answer a question or input, it must not simply fail. | A failed prompt emits a structured failure trace with known facts, unknown facts, attempted methods, and the next repair hypothesis. | Implemented slice in PR #637: `src/self_healing.rs` composes a failure trace into a unified, auditable `RepairCase` (emitted as `data/meta/self-healing-case.lino`). A single canonical failure class is covered end to end; generalizing to every failure class remains. |
| R558-02 | The system must rethink and improve its own meta algorithm using the meta algorithm itself. | A failure trace can trigger an Agent-CLI-driven repair run that changes a solver method, data record, or test, then proves the change with automated checks. | Implemented in PR #637: `src/repair_strategy.rs` is the *general* front of the repair loop. The self-healing slice runs the solver-method path on one canonical failure; this generalises it. `RepairStrategy::classify` reads an arbitrary `UnknownTrace` — the same trace the self-healing loop reasons about — and, purely deterministically from the trace's own prompt and event signals, maps it onto exactly one of the three targets issue #558 names (`RepairTarget::SolverMethod` / `DataRecord` / `Test`), so the loop is *total* — every failure is classified. For each it composes the grounded repair plan: a rationale (the signal it keyed on), a proposed change scoped to the target class, and the automated verification that must be green before any human promotion. It stays proposal-only and human-gated, and neural inference stays a NON-GOAL: the classification and plan are deterministic functions of the trace, and the "change" is a plan a human or Agent CLI executes, never generated code applied automatically. Reachable through the agentic interface as the tenth recipe (`src/agentic_coding/repair_strategy.rs`), which emits the three canonical strategies as `data/meta/repair-strategies.lino` (pinned byte-for-byte because it depends only on self-contained canonical traces, not the whole source tree). Covered by `tests/unit/issue_558_repair_strategy.rs` and `tests/integration/issue_558_repair_strategy.rs`. |
| R558-03 | The result should be a self-healing algorithm that reasons about itself, tests a new version, and promotes improvements when tests and the user accept them. | A candidate improvement is generated in an isolated workspace, rebuilt, tested, reviewed, and only then written to mainline history as an approved learning record. | Implemented in PR #637: `src/learning_ledger.rs` is the single promotion protocol. `LearningLedger::promote` records a `RepairCase` as a durable approved learning record only when *both* the benchmark gate is green *and* a human approves, and refuses every other case with a specific reason (`TestsNotGreen` / `NoReviewableProposal` / `SourceNotFaithful` / `HumanDeclined` / `AlreadyPromoted`). A repeated failure is answered from the ledger (`lesson_for` / `knows`) instead of re-derived. The ledger serialises to Links Notation pinned byte-for-byte to `data/meta/learning-ledger.lino`, and is reachable through the agentic interface as the seventh recipe (`src/agentic_coding/ledger.rs`), covered by `tests/unit/issue_558_learning_ledger.rs` and `tests/integration/issue_558_learning_ledger.rs`. |
| R558-04 | The entire source code of Formal AI must be translatable to Links/meta language and present in seed data. | Every source file has a source-to-links projection with provenance, checksum, parser version, and link references to owning requirements/tests. | Implemented in PR #637: `build.rs` embeds every owned `src/*.rs` file (the `OWNED_SOURCE_FILES` manifest) so the entire tree is present in our data, and `src/self_source_graph.rs` content-addresses all of it (`owned_manifest`) and projects modules through the sole CST/AST engine. `SourceGraph::owned` is the exhaustive whole-repository projection; every owned file round-trips byte-for-byte (locked by the ignored-by-default `exhaustive_whole_repo_round_trip_is_lossless` test and the `project_source_graph` example). Reachable through the agentic interface as the sixth recipe (`src/agentic_coding/source_graph.rs`). |
| R558-05 | The Links/meta representation must round-trip back to source code. | A links-to-source run regenerates at least one module byte-for-byte or semantically equivalent after formatting, then scales to all owned code. | Implemented in PR #637: `SourceRoundTrip::for_pinned_target()` parses a real module through the meta-language links network and confirms `source → links → source` reproduces it byte-for-byte (`faithful = true`), and `src/self_source_graph.rs` now scales that round-trip to **all** owned code — every owned module reconstructs byte-for-byte via `SourceGraph::owned` (`is_fully_faithful`, `coverage_permille == 1000`). Verified by `tests/unit/issue_558_self_healing.rs` and `tests/unit/issue_558_source_graph.rs`. Driving accepted *edits* back through the round-trip remains. |
| R558-06 | Formal AI must be able to recompile and reattach the improved code to the UI. | A generated source change can rebuild `formal-ai`, restart or hot-swap the local server/worker, and make the UI use the accepted version. | Implemented in PR #637: `src/rebuild_plan.rs` composes the final step of the self-change loop as a deterministic, human-gated `RebuildPlan`. It is *derived from an already-accepted change* (`RebuildPlan::for_accepted_change` takes an `AcceptedChange`, which only exists after a green benchmark gate *and* an explicit human approval — the same gate the ledger and change request enforce), so a rebuild can never precede acceptance. It grounds every reattached UI artifact against the real repository bytes and the owned manifest — the crate manifest, the server/CLI entry (`src/main.rs`, resolved through `owned_manifest` and panicking if absent), the worker glue (`src/web/formal_ai_worker.js`), and the UI entry (`src/web/index.html`) — so the plan cannot reference a fabricated artifact; the regenerated `formal_ai_worker.wasm` is deliberately the rebuild's *output*, referenced in the pipeline steps rather than pinned as an input. The plan is an ordered, five-step pipeline (recompile the crate → regenerate the WebAssembly worker → reattach it to the UI → hot-swap the local server → verify the UI uses the accepted version), and *every* step is observable (what proves it happened) and reversible (how to roll it back), so keeping the swap stays a reviewable human decision. Nothing is rebuilt or restarted: the plan is the reviewable product, so the "recompile and reattach" guardrail (observable, testable, reversible, human-approved) is preserved, and neural inference stays a NON-GOAL — the plan is a deterministic function of the accepted change and the embedded artifacts. Reachable through the agentic interface as the eleventh recipe (`src/agentic_coding/rebuild_plan.rs`), which emits the plan *live* as `rebuild-and-reattach.lino` (never pinned, because the grounded artifacts' content ids change with every source edit, like the source-graph, explain, and change-request recipes). Covered by `tests/unit/issue_558_rebuild_plan.rs` and `tests/integration/issue_558_rebuild_plan.rs`. |
| R558-07 | Users must be able to ask for changes in the AI system through this mechanism. | Natural-language change requests create requirements, tests, patches, and a reviewable PR through the same repair loop. | Implemented in PR #637: `src/change_request.rs` turns a natural-language "change Formal AI itself" request into a structured, reviewable proposal — a normalised requirement, a proposed test name, and an ordered patch plan against a *grounded* target module (`ChangeRequest::for_module` resolves the target's `content_id` from the owned manifest and *panics* on any path the repository does not ship, so a request can never target fabricated source). It serialises to Links Notation as the reviewable pull request a human reads. `ChangeRequest::review` reuses the *same* two acceptance conditions as the learning ledger — a green `BenchmarkGateReport` *and* an explicit `HumanApproval` — refusing every case that is not both (`TestsNotGreen` / `HumanDeclined`), so no user request is applied automatically. Neural inference stays a NON-GOAL: the requirement, test, and patch plan are deterministic functions of the request and its grounded target, and the *patch* is a plan a human or Agent CLI executes, not generated code. Reachable through the agentic interface as the ninth recipe (`src/agentic_coding/change_request.rs`), covered by `tests/unit/issue_558_change_request.rs` and `tests/integration/issue_558_change_request.rs`. |
| R558-08 | Users must be able to ask how Formal AI itself works and receive answers grounded in its source and data. | The answer cites linked source/data/test artifacts from the source-to-links graph rather than relying on prose docs only. | Implemented in PR #637: `src/self_explanation.rs` composes a grounded answer to "how does Formal AI work?" — an ordered set of topics, each citing the *real* artifacts it rests on. Every `CitationKind::Source` citation resolves its `content_id` from the owned manifest and *panics* if the path is not an owned source file, so a fabricated citation cannot even be constructed; data/test citations point at generated `data/meta/*.lino` and `tests/**` files whose on-disk existence is checked. The explanation serialises to Links Notation anchored to the whole-source manifest content id, and is reachable through the agentic interface as the eighth recipe (`src/agentic_coding/explain.rs`), covered by `tests/unit/issue_558_self_explanation.rs` and `tests/integration/issue_558_self_explanation.rs`. |
| R558-09 | Deeply analyze what went wrong in PR #601 and why issue #538 was not fully delivered, focused on auto-learning. | A case-study document identifies concrete gaps, stale status drift, and delivered versus missing auto-learning capabilities. | Documented in this PR by `pr-601-gap-analysis.md`. |
| R558-10 | Collect data related to the issue under `docs/case-studies/issue-558`. | Issue, PR, comments, reviews, search captures, PR diff, related issues/PRs, and online research are checked into `raw-data/`. | Documented in this PR. |
| R558-11 | Search online for additional facts and existing components/libraries that solve similar problems. | Research notes compare SWE-agent, OpenHands, Reflexion, DSPy, Tree-sitter, rustdoc JSON, syn, and rowan against Formal AI's needs. | Documented in this PR by `raw-data/online-research.md`. |
| R558-12 | List all requirements and propose possible solutions and solution plans for each. | This requirements matrix plus `solution-plan.md` map every requirement to a phase, reusable component, and acceptance gate. | Documented in this PR. |

## Requirement Grouping

The requirements form four implementation groups:

1. **Failure-to-repair loop:** R558-01 through R558-03.
2. **Source as link-native data:** R558-04 through R558-06.
3. **User-facing self-change and self-explanation:** R558-07 and R558-08.
4. **Case-study process:** R558-09 through R558-12.

The fourth group is protected by `tests/unit/docs_requirements_issue_558.rs`. PR
#637 additionally lands implemented slices of the first two groups. The closed,
human-gated `RepairCase` loop (R558-01) with its verified source-to-links
round-trip is reachable through the agentic interface as the fifth recipe and
covered by `tests/unit/issue_558_self_healing.rs` and
`tests/integration/issue_558_self_healing.rs`. The whole-repository
source-to-links projection (R558-04/R558-05) embeds every owned source file and
proves all of them round-trip byte-for-byte (`SourceGraph::owned`), reachable as
the sixth recipe and covered by `tests/unit/issue_558_source_graph.rs` and
`tests/integration/issue_558_source_graph.rs`. The promotion protocol that closes
the loop (R558-03) is `src/learning_ledger.rs`: a green, faithful, human-approved
`RepairCase` becomes a durable approved learning record and is recalled on a
repeated failure, reachable as the seventh recipe and covered by
`tests/unit/issue_558_learning_ledger.rs` and
`tests/integration/issue_558_learning_ledger.rs`. The grounded self-explanation that
opens group three (R558-08) is `src/self_explanation.rs`: a "how does Formal AI work?"
answer whose every source citation is content-addressed against the owned manifest
(fabricated citations fail to construct), reachable as the eighth recipe and covered by
`tests/unit/issue_558_self_explanation.rs` and
`tests/integration/issue_558_self_explanation.rs`. The user-driven self-change that
completes group three (R558-07) is `src/change_request.rs`: a natural-language "change
Formal AI itself" request becomes a requirement, a proposed test, and a patch plan
against a target module grounded in the owned manifest (a fabricated target fails to
construct), merging only through the same green-gate-plus-human-approval loop as the
ledger, reachable as the ninth recipe and covered by
`tests/unit/issue_558_change_request.rs` and
`tests/integration/issue_558_change_request.rs`. The general front of the failure-to-repair
loop that completes group one (R558-02) is `src/repair_strategy.rs`: the self-healing slice
repairs a single canonical failure by synthesising a solver method, and this generalises it —
`RepairStrategy::classify` reads an arbitrary failure trace and deterministically maps it onto
exactly one of the three targets the issue names (a solver method, a data record, or a test),
composing the grounded, human-gated repair plan for each, so the loop is *total* rather than
bound to one case. It is reachable as the tenth recipe and covered by
`tests/unit/issue_558_repair_strategy.rs` and
`tests/integration/issue_558_repair_strategy.rs`. The final step of group two (R558-06) is
`src/rebuild_plan.rs`: once a change is accepted through the same green-gate-plus-human-approval
loop, `RebuildPlan::for_accepted_change` composes the ordered, observable, reversible pipeline to
recompile Formal AI, regenerate the WebAssembly worker, reattach it to the grounded UI artifacts,
hot-swap the local server, and verify the UI uses the accepted version — a proposal-only plan a
human or Agent CLI runs, never an automatic rebuild, reachable as the eleventh recipe and covered
by `tests/unit/issue_558_rebuild_plan.rs` and `tests/integration/issue_558_rebuild_plan.rs`. The
one remaining group-two thread (driving accepted Links-to-source *edits* back through the
byte-for-byte round-trip before that rebuild) stays for a later slice, all human-gated by design.
