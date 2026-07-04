# Issue 558 Solution Plan

The issue #558 target is a safe auto-learning loop, not autonomous mutation of a
running binary. Formal AI should learn by creating reviewable artifacts: failure
traces, source/data links, candidate patches, tests, and approved lessons. The
installed system changes only after rebuild, verification, and human approval.

## Phase 0: Evidence And Status Correction

- Preserve issue #558, issue #538, PR #601, related GitHub searches, PR diff,
  and online research under `docs/case-studies/issue-558/raw-data/`.
- Correct the root `REQUIREMENTS.md` rows for issue #538 so delivered Agent CLI,
  diagram, and self-AST slices are not described as missing.
- Add the issue #558 requirements matrix, gap analysis, and solution plan.
- Protect the documentation contract with
  `tests/unit/docs_requirements_issue_558.rs`.

## Phase 1: Failure-To-Repair Ledger

- Introduce a `repair_case` data model with prompt, normalized task, failed
  answer, known facts, unknown facts, attempted methods, selected repair
  hypothesis, patch links, validation result, reviewer decision, and promoted
  lesson.
- Reuse the issue #364 self-improvement framing and the existing unknown-answer
  traces as the first failure sources.
- Use the Reflexion pattern from the research notes: write explicit lessons from
  feedback into memory instead of pretending the model weights changed.
- Make every repeated failure query prior approved lessons before generating a
  new answer or repair attempt.

## Phase 2: Source-To-Links Repository Compiler

- Build a repository compiler that projects owned source files into Links data
  with parser version, source checksum, file path, symbol ownership, and
  requirement/test references.
- Use Tree-sitter through the existing meta-language path for lossless CST
  coverage across Rust, JavaScript, TypeScript, HTML, CSS, JSON, and docs-like
  structured files.
- Use rustdoc JSON as the Rust semantic API inventory: crate items, visibility,
  documentation, paths, and source spans.
- Keep `syn` as a Rust-only transformation aid when it simplifies experiments,
  but require it to emit the same link-native model rather than a parallel
  ontology.
- Evaluate rowan for the later editable syntax-tree layer once the first
  full-file round trip exists.

## Phase 3: Links-To-Source Round Trip

- Start with one Rust module that has stable formatting and existing tests.
- Regenerate that module from link-native data, run `cargo fmt`, and assert
  either byte-for-byte equality or a deliberate semantic equivalence check.
- Expand to source packages by ownership boundary, not by file count.
- Reject promotion when checksum, parser version, generated source, formatter,
  or tests disagree.

## Phase 4: Self-Repair Agent Loop

- Generalize the PR #601 Agent CLI recipe driver into a repair executor that
  accepts a `repair_case` instead of a known recipe axis.
- Reuse SWE-agent and OpenHands design patterns: repository navigation, file
  edits, shell execution, tests, sandboxing, and GitHub PR handoff are explicit
  tools whose outputs feed the next step.
- Keep execution in a throwaway workspace. The loop may propose patches, but it
  must not silently write accepted changes into the user's current checkout.
- Use DSPy-style structured signatures and metrics for solver methods so a
  "learned" change is a typed method/data rewrite with a validation score, not
  an opaque prompt tweak.

## Phase 5: Learning Promotion And UI Reattach

- After approval, attach the accepted patch to a durable learning record and
  merge it through the normal PR path.
- Rebuild the server, CLI, WASM worker, and UI surface that the patch affects.
- Restart or hot-swap only after acceptance gates pass and the new version can
  explain which requirement, source links, tests, and repair case caused the
  change.
- Show users the active version, accepted learning records, and rollback path.

## Acceptance gates

| Gate | Required proof |
| --- | --- |
| Failure trace | A failed input creates a readable `repair_case` with unknowns, attempted methods, and a proposed repair target. |
| Source-to-links | Each projected source file has a checksum, parser version, link graph, and requirement/test references. |
| Links-to-source | At least one module regenerates from links and passes formatting plus focused tests. |
| Repair executor | A sandboxed Agent CLI run can turn a `repair_case` into a patch and test transcript. |
| Learning record | Reviewer-approved repairs are stored as reusable lessons and queried on repeated failures. |
| Rebuild/reattach | The accepted version rebuilds and the affected UI or service surface uses it after explicit approval. |
| Self-explanation | "How does Formal AI work here?" answers cite source, data, tests, and learning records from the graph. |

## Requirement Mapping

| Requirement | Primary phase |
| --- | --- |
| R558-01 | Phase 1 |
| R558-02 | Phase 4 |
| R558-03 | Phase 5 |
| R558-04 | Phase 2 |
| R558-05 | Phase 3 |
| R558-06 | Phase 5 |
| R558-07 | Phase 4 |
| R558-08 | Phase 2 and Phase 5 |
| R558-09 | Phase 0 |
| R558-10 | Phase 0 |
| R558-11 | Phase 0 |
| R558-12 | Phase 0 |
