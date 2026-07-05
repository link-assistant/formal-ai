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
| R558-03 | The result should be a self-healing algorithm that reasons about itself, tests a new version, and promotes improvements when tests and the user accept them. | A candidate improvement is generated in an isolated workspace, rebuilt, tested, reviewed, and only then written to mainline history as an approved learning record. | Not yet built. Existing CI and PR workflows supply pieces but no single promotion protocol. |
| R558-04 | The entire source code of Formal AI must be translatable to Links/meta language and present in seed data. | Every source file has a source-to-links projection with provenance, checksum, parser version, and link references to owning requirements/tests. | Built slice exists: PR #601 stores one `self-AST` census for `src/agentic_coding/planner.rs`; the full repository source-to-links graph is not built. |
| R558-05 | The Links/meta representation must round-trip back to source code. | A links-to-source run regenerates at least one module byte-for-byte or semantically equivalent after formatting, then scales to all owned code. | Implemented slice in PR #637: `SourceRoundTrip::for_pinned_target()` parses a real module through the meta-language links network and confirms `source → links → source` reproduces it byte-for-byte (`faithful = true`), verified by `tests/unit/issue_558_self_healing.rs`. Scaling the round-trip to all owned code (and driving edits through it) remains. |
| R558-06 | Formal AI must be able to recompile and reattach the improved code to the UI. | A generated source change can rebuild `formal-ai`, restart or hot-swap the local server/worker, and make the UI use the accepted version. | Not yet built. Current release/desktop/server pieces can be reused but are not wired into a self-update loop. |
| R558-07 | Users must be able to ask for changes in the AI system through this mechanism. | Natural-language change requests create requirements, tests, patches, and a reviewable PR through the same repair loop. | Built slice exists through Agent CLI and issue-solver flows, but Formal AI itself cannot yet drive arbitrary self-change end to end. |
| R558-08 | Users must be able to ask how Formal AI itself works and receive answers grounded in its source and data. | The answer cites linked source/data/test artifacts from the source-to-links graph rather than relying on prose docs only. | Built slice exists through docs, graph endpoints, and file summarization; no source-linked whole-system explanation graph exists yet. |
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
#637 additionally lands the first implemented slice of the first two groups: the
closed, human-gated `RepairCase` loop (R558-01) with its verified source-to-links
round-trip (R558-05), reachable through the agentic interface as the fifth recipe
and covered by `tests/unit/issue_558_self_healing.rs` and
`tests/integration/issue_558_self_healing.rs`. The remaining group-one/-two work
(general repair, whole-repo projection, Links-to-source regeneration, and
rebuild/reattach) stays for later slices, all human-gated by design.
