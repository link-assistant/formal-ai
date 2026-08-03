# Issue #914 Requirements

Issue [#914](https://github.com/link-assistant/formal-ai/issues/914) is a
meta-planning issue in the lineage of
[#244](https://github.com/link-assistant/formal-ai/issues/244) and
[#651](https://github.com/link-assistant/formal-ai/issues/651), with a new
emphasis: **coding via formal logical reasoning** as the first learned skill,
so Formal AI can accelerate its own development. Each row below is one
requirement extracted from the issue body, with the verification that shows
where this branch satisfies it or which created issue owns it.

| ID | Requirement | Verification |
| --- | --- | --- |
| R914-1 | Use all previous issues, pull requests, and their comments, plus the requirements and vision files, as the input evidence for the plan | `raw-data/` snapshots (issues updated since the 2026-07-14 audit, epic-status sweep), plus the audits cited in [`README.md`](README.md) §1 |
| R914-2 | First update the documentation so it fully tracks the implementation progress of all requirements | `ROADMAP.md` ninth-pass audit section (2026-08-03); guarded by `issue_914_case_study_and_planning_docs_are_traceable` |
| R914-3 | Everything in docs must be in sync with the actual state of the code | Ninth-pass status refresh: every closed epic of E37-E68 moved out of "open" wording; per-area statuses re-verified against `src/` and `tests/` (see [`README.md`](README.md) §3) |
| R914-4 | After docs are in sync, create all the issues needed to fully implement the vision | [`proposed-issues.md`](proposed-issues.md) drafts E69-E77; opened issue URLs are recorded at the top of that file |
| R914-5 | The system must be able to learn the universal problem-solving algorithm, making it possible to truly solve translation between languages (natural and formal) | Owned by E70 (general natural↔formal translation) and E75 (method learning); current state and gap recorded in [`README.md`](README.md) §3 |
| R914-6 | Keep a minimum core of algorithms plus a data seed whose metadata is rich enough to problem-solve any problem the way people do | Owned by E71 (minimal-core boundary and seed-metadata audit); builds on the closed #559/#699 handler-migration mandate |
| R914-7 | Reasoning itself must not use neural networks; the formal reasoning implementation must cover all existing test cases and much more | Standing invariant (NON-GOALS.md); regression floor restated as a binding design rule in [`proposed-issues.md`](proposed-issues.md); coverage growth owned by E76 |
| R914-8 | Learn to discover enough knowledge from the internet and external sources to solve all tasks, with coding first | Owned by E72 (research-driven coding knowledge loop); builds on #873 and #896 |
| R914-9 | Coding first: once Formal AI can code, use that skill to speed up its own development | Owned by E69 (coding-ladder ratchet) and E77 (self-development loop); baseline evidence is the #848 ladder (2/13, zero write effects at baseline) |
| R914-10 | Work with unknowns: gather missing information autonomously, ask the user as few questions as possible, and only the requirement/real-world questions no one else can answer | Owned by E73 (question-necessity protocol); builds on the E21 unknown-reasoning loop and the `guess_probability`/`questioning_rigor` knobs |
| R914-11 | Formal AI must be well integrated with link-assistant/hive-mind via agentic harness CLIs/TUIs | Owned by E74 (hive-mind end-to-end integration gate); builds on closed #655/#703 and hive-mind#2059 |
| R914-12 | The result must be issues created in this repository representing the full plan | Issue URLs recorded in [`proposed-issues.md`](proposed-issues.md) after opening |
| R914-13 | Build on the best previous experience; make the algorithm more general and smarter while still supporting everything already supported | "Keep the regression floor" binding rule in [`proposed-issues.md`](proposed-issues.md); every epic lists the existing components it generalizes instead of replacing |
| R914-14 | Any critical code problems that block the vision are planned to be fixed first, so the plan builds on a solid foundation | E69 is the explicit blocker epic: it consolidates the open agent-harness defects #902-#909 behind the observable-effect coding ladder before capability epics land |
| R914-15 | Collect the issue data into `docs/case-studies/issue-914`, do a deep case-study analysis with online research, list each and all requirements, and propose solutions and solution plans per requirement, checking known existing components/libraries | This folder: [`README.md`](README.md), this file, [`solution-plan.md`](solution-plan.md), [`proposed-issues.md`](proposed-issues.md), `raw-data/github/`, `raw-data/online-research.md` |
