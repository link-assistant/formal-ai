# Issue 531 Requirements

The table below converts issue #531 and the initial PR instructions into
checkable requirements. Runtime implementation requirements are intentionally
marked as planned because this PR is the requested research and proposal pass.

| ID | Requirement | Proposed solution or status |
| --- | --- | --- |
| R531-01 | Create a detailed work plan and follow it step by step. | Tracked in the issue-solver plan and reflected in this case-study structure: raw evidence, requirements, architecture inventory, solution plan, tests, and PR update. |
| R531-02 | Read the issue, PR, comments, and review comments thoroughly. | Raw GitHub snapshots are preserved in `raw-data/issue-531.json`, `raw-data/issue-531-comments.json`, `raw-data/pr-642.json`, `raw-data/pr-642-conversation-comments.json`, `raw-data/pr-642-review-comments.json`, and `raw-data/pr-642-reviews.json`. |
| R531-03 | Collect all relevant data under `docs/case-studies/issue-531`. | Implemented by this directory and its `raw-data/` evidence files. |
| R531-04 | Research `linksplatform/Data.Doublets.Sequences` deeply, especially C# converters. | Implemented for the first pass by cloning the upstream repository, saving repository metadata, saving the checked commit, listing converter files, and preserving selected converter/source excerpts. |
| R531-05 | Reimplement Rust sequence support only after dependency analysis, including unique symbols and sequence initialization. | Planned as Phase 1 in `solution-plan.md`: unique symbol allocation, sequence markers, sequence index, sequence walker, Links Notation persistence, and backend tests. |
| R531-06 | Start pattern inference from associative deduplication/compression. | Planned as Phase 3: repeated pair/sub-sequence replacement with an auditable compression trace and exact expansion back to the original sequence. |
| R531-07 | Support 1D sequence and text pattern inference. | Planned as Phase 4. The first 1D matcher should operate on link-native sequence IDs, with text handled through string/unicode-to-symbol conversion instead of direct ad hoc strings. |
| R531-08 | Support 2D image/grid pattern inference with symmetry, rotation, translation, and analogy-like transformations. | Planned as Phase 4. Grids should be projected into row, column, diagonal, boundary, and transformed sequences so the same sequence machinery can compare rotations, reflections, shifts, and relative positions. |
| R531-09 | Consider ontology and seed meanings so the system can explain pattern concepts. | Planned as Phase 5: seed meanings for sequence, pattern, repetition, compression, deduplication, symmetry, rotation, translation, analogy, invariant, and transformation. |
| R531-10 | Check related theory repositories and existing Formal AI work. | Implemented by preserving metadata for `link-foundation/meta-theory` and `link-foundation/relative-meta-logic`, and by inventorying current `link_store`, `substitution`, `solver`, `meta_core`, and text-deduplication code. |
| R531-11 | Search online for known components, libraries, and algorithms that may apply. | Implemented by `raw-data/online-research.md`, covering Data.Doublets.Sequences, SEQUITUR, Re-Pair, ARC-AGI, meta-theory, and relative-meta-logic. |
| R531-12 | List all requirements and propose multiple solution plans where the issue is broad. | Implemented by this file and `solution-plan.md`, which splits the work into staged acceptance gates. |
| R531-13 | Apply deduplication/pattern matching to relevant AI tasks or benchmarks when implementation begins. | Planned as Phase 7: text fixtures, sequence compression fixtures, ARC-style grid fixtures, and solver/meta fact-checking examples. |
| R531-14 | Keep diagnostics and tracing available but default-off for future pattern inference. | Planned for implementation phases: compression traces and matcher candidates should be inspectable through data artifacts without noisy default runtime logs. |
| R531-15 | Preserve work in the prepared branch and PR. | This research pass is tracked by PR #642 on branch `issue-531-ebee6863aca0`. |
| R531-16 | Treat the first session as research and proposal work; maintainers decide the next coding direction. | Implemented by stopping this PR at docs, evidence, requirements, and test coverage instead of shipping an unreviewed broad runtime subsystem. |
