## Issue #914 Vision Implementation Planning, Coding First

Issue [#914](https://github.com/link-assistant/formal-ai/issues/914) is a
meta-planning issue in the lineage of
[#244](https://github.com/link-assistant/formal-ai/issues/244) and
[#651](https://github.com/link-assistant/formal-ai/issues/651): sync the
documentation with the actual state of the code, then create all the issues
needed to fully implement the vision, with coding via formal logical
reasoning as the first skill to complete and critical vision-blocking code
problems fixed first. The case study, gap analysis, epic bodies (E69-E77),
and opened-issue record live in `docs/case-studies/issue-914/`.

| ID | Requirement | Status |
| --- | --- | --- |
| R914-1 | Use all previous issues, pull requests, comments, and the requirements and vision files as input evidence for the plan. | Collected: `docs/case-studies/issue-914/raw-data/` holds the GitHub snapshots and the 152-issue post-audit sweep; prior audits are cited in the case-study README. |
| R914-2 | First update documentation to fully track implementation progress of all requirements. | Implemented by the ninth-pass audit in `ROADMAP.md` (2026-08-03) and this table; guarded by `issue_914_case_study_and_planning_docs_are_traceable`. |
| R914-3 | Documentation must be in sync with the actual state of the code. | Implemented: four stale eighth-pass rows corrected (#698, #709, #662/#704, #686/#702 shipped after the eighth pass) and every area re-verified against `src/` and the epic sweep. |
| R914-4 | After the docs are in sync, create all the issues needed to fully implement the vision. | Implemented: epics E69-E77 drafted in `docs/case-studies/issue-914/proposed-issues.md` and opened on GitHub with URLs recorded there. |
| R914-5 | The system learns the universal problem-solving algorithm, making it possible to truly solve translation between natural and formal languages. | Tracked: E70 owns general natural-formal translation; E75 owns method learning over the recipe interpreter and method registry. |
| R914-6 | Keep a minimum core of algorithms plus a data seed whose metadata is rich enough to problem-solve the way people do. | Partial with enforcement from #918: the accepted four-part boundary, recursive handler ledger, metadata schema, complete coding-path floor, per-record gap data, and shrink-only CI ratchets are documented in `docs/case-studies/issue-918/`; handler migration is tracked row by row in `data/meta/handler-migration-ledger.lino` (16 rows `status migrated`, 55 rows `status pending` as of 2026-09-18), and the shrink-only `handler_migration_pending` ceiling in `data/meta/debt-ratchet.lino` can only move down. |
| R914-7 | No neural networks in reasoning; formal reasoning covers all existing test cases and much more. | Standing invariant (NON-GOALS.md) restated as a binding design rule for every epic; coverage growth with external benchmark scoring is E76. |
| R914-8 | Learn to discover enough knowledge from the internet and other sources to solve all tasks, coding first. | Implemented as a growing general mechanism: E72/#919 supplies research-to-verified-procedure learning over the provenance-tracked source cache; PR #888 adds bounded Python documentation, Wikifunctions, Rosetta Code, and official OEIS adapters, strict source formalization, structural composition, executable selection, forgettable procedure memory, and cold-cache rediscovery tests. Unknown source knowledge still fails honestly rather than becoming a built-in answer. |
| R914-9 | Coding first: once Formal AI can code, that skill speeds up its own development. | Implemented with measured boundaries: E69/#916 closes the write-effect dependency, the issue-1021 action ladder passes 16/16, and E77/#924 requires real Agent-CLI-authored repository work per release cycle. PR #888 adds multiple session-backed Formal-AI-authored leaves and raises the first-20 HumanEval/MBPP upstream rows to 20/20 through generalized discovery. |
| R914-10 | Work with unknowns, asking the user as few questions as possible and only requirement-level ones. | Tracked: E73 adds the question-necessity protocol over the existing clarify-vs-guess, unknown-reasoning, and #527 question-catalog mechanisms. |
| R914-11 | Integrate well with link-assistant/hive-mind through agentic harness CLIs and TUIs. | Tracked: E74 owns the replayable end-to-end gate in both directions, including the hive-mind#2059 invocation shape. |
| R914-12 | The result is issues created in this repository representing the full plan. | Implemented: opened-issue URLs recorded in `docs/case-studies/issue-914/proposed-issues.md`. |
| R914-13 | Build on the best previous experience; generalize without dropping anything already supported. | Binding design rule ("keep the regression floor") in the epic batch; every epic lists the existing components it generalizes. |
| R914-14 | Fix critical vision-blocking code problems first, so the plan builds on a solid foundation. | Implemented in the plan: E69 is the foundation blocker consolidating #902-#909 behind the coding-ladder ratchet; every dependent epic declares its E69 dependency. |
| R914-15 | Collect the data into `docs/case-studies/issue-914` with deep analysis, online research, the full requirement list, and per-requirement solution plans checking existing components. | Implemented: README, requirements, solution plan, proposed issues, raw GitHub data, and the component/license research live in that folder. |
