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
| R558-02 | The system must rethink and improve its own meta algorithm using the meta algorithm itself. | A failure trace can trigger an Agent-CLI-driven repair run that changes a solver method, data record, or test, then proves the change with automated checks. | Not yet built as a general loop. PR #601 proves bounded Agent CLI recipes, not arbitrary meta-algorithm repair. |
| R558-03 | The result should be a self-healing algorithm that reasons about itself, tests a new version, and promotes improvements when tests and the user accept them. | A candidate improvement is generated in an isolated workspace, rebuilt, tested, reviewed, and only then written to mainline history as an approved learning record. | Implemented in PR #637: `src/learning_ledger.rs` is the single promotion protocol. `LearningLedger::promote` records a `RepairCase` as a durable approved learning record only when *both* the benchmark gate is green *and* a human approves, and refuses every other case with a specific reason (`TestsNotGreen` / `NoReviewableProposal` / `SourceNotFaithful` / `HumanDeclined` / `AlreadyPromoted`). A repeated failure is answered from the ledger (`lesson_for` / `knows`) instead of re-derived. The ledger serialises to Links Notation pinned byte-for-byte to `data/meta/learning-ledger.lino`, and is reachable through the agentic interface as the seventh recipe (`src/agentic_coding/ledger.rs`), covered by `tests/unit/issue_558_learning_ledger.rs` and `tests/integration/issue_558_learning_ledger.rs`. |
| R558-04 | The entire source code of Formal AI must be translatable to Links/meta language and present in seed data. | Every source file has a source-to-links projection with provenance, checksum, parser version, and link references to owning requirements/tests. | Implemented in PR #637: `build.rs` embeds every owned `src/*.rs` file (the `OWNED_SOURCE_FILES` manifest) so the entire tree is present in our data, and `src/self_source_graph.rs` content-addresses all of it (`owned_manifest`) and projects modules through the sole CST/AST engine. `SourceGraph::owned` is the exhaustive whole-repository projection; every owned file round-trips byte-for-byte (locked by the ignored-by-default `exhaustive_whole_repo_round_trip_is_lossless` test and the `project_source_graph` example). Reachable through the agentic interface as the sixth recipe (`src/agentic_coding/source_graph.rs`). |
| R558-05 | The Links/meta representation must round-trip back to source code. | A links-to-source run regenerates at least one module byte-for-byte or semantically equivalent after formatting, then scales to all owned code. | Implemented in PR #637: `SourceRoundTrip::for_pinned_target()` parses a real module through the meta-language links network and confirms `source → links → source` reproduces it byte-for-byte (`faithful = true`), and `src/self_source_graph.rs` now scales that round-trip to **all** owned code — every owned module reconstructs byte-for-byte via `SourceGraph::owned` (`is_fully_faithful`, `coverage_permille == 1000`). Verified by `tests/unit/issue_558_self_healing.rs` and `tests/unit/issue_558_source_graph.rs`. Driving accepted *edits* back through the round-trip remains. |
| R558-06 | Formal AI must be able to recompile and reattach the improved code to the UI. | A generated source change can rebuild `formal-ai`, restart or hot-swap the local server/worker, and make the UI use the accepted version. | Not yet built. Current release/desktop/server pieces can be reused but are not wired into a self-update loop. |
| R558-07 | Users must be able to ask for changes in the AI system through this mechanism. | Natural-language change requests create requirements, tests, patches, and a reviewable PR through the same repair loop. | Built slice exists through Agent CLI and issue-solver flows, but Formal AI itself cannot yet drive arbitrary self-change end to end. |
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
`tests/integration/issue_558_self_explanation.rs`. The remaining group-one/-two/-three
work (general repair for every failure class, driving accepted edits back through the
round-trip, rebuild/reattach, and user-driven self-change) stays for later slices, all
human-gated by design.
