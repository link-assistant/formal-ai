# Issue #1548 solution plan

One section per requirement of [requirements.md](requirements.md). Each section
gives the options that were studied, the option selected with the reason, the
plan, and the existing components and libraries that the plan reuses. The
executable form of every plan is the issue named in the section: its
"Implementation steps" and "Acceptance" lists are the contract, and this file
only records why that issue has the shape it has.

Order of work is the order of the parent issue
[#1561](https://github.com/link-assistant/formal-ai/issues/1561):

| Stage | Purpose | Issues |
| --- | --- | --- |
| 0 | A session survives its first minute | #1558, #1560, #1557, #1559, #1562, #1189, #1205, #1206, #1208, #1209, #1211, #1484, #1485, #1486, #1191 |
| 1 | The meta algorithm executes | #1550, #1487, #1166, #1551, #1483 |
| 2 | Knowledge comes from discovery, not from stored answers | #1163, #1164, #1165, #1553, #1552, #1179, #491, #1167, #1185 |
| 3 | Delivery of real issues | #1162, #1488, #1489, #1170, #1490 |
| 4 | Remove what the meta algorithm replaced | #1554, #1556, #1491, #1500 |
| — | General question answering | #1555, #1192, #1193, #1194, #1222, #1223, #800, #869, #872 |

Stage 0 comes first because its new defects ended 8 of the 19 drafts on real
issues before any reasoning about the task started (5 on a missing file, 3 on
raw JSON) and put 5 more summaries in the wrong language, and a later stage
cannot be measured through Hive Mind while they are present.

## R1548-1 — parent issue with native blockers

| Option | Verdict |
| --- | --- |
| Move every blocker under one new parent | Rejected: GitHub allows one parent per issue, and 17 blockers already belong to #651, #1162, #1183 or #1207; moving them would destroy those plans |
| A list of links in the body of the parent | Rejected as the only form: a text list is not a dependency GitHub can query, and #1490 schedules drafts from native relations |
| Sub-issues for blockers without a parent, native `blocked by` for the rest, and the order written in the body | Selected |

Done on 2026-11-10 with the helpers in `experiments/issue-1548/`
(`create-planned-issue.sh`, `add-sub-issue.sh`, `add-blocked-by.sh`,
`fetch-issue-graph.sh`); every write is logged in `raw-data/github/writes/`.
The method is the [issue planning guideline](../../guidelines/issue-planning.md).
GitHub facts used: a parent holds up to 100 sub-issues in up to eight levels;
dependencies are created with `POST /repos/{owner}/{repo}/issues/{number}/dependencies/blocked_by`
(<https://docs.github.com/en/rest/issues/issue-dependencies>,
<https://docs.github.com/en/issues/tracking-your-work-with-issues/using-issues/adding-sub-issues>).

## R1548-2 — formalization and deformalization (naturalization) with the meta language

| Option | Verdict |
| --- | --- |
| Raise survival by adding sentence templates per language | Rejected: it is a stored answer per sentence shape and grows with every language |
| A controlled language that users must write in (Attempto Controlled English, FRETish of NASA FRET) | Rejected as the interface, kept as a reference: the vision accepts any natural language. Both show which constructs a logic-backed language needs |
| One abstract meaning with a formalizer rule and a deformalizer rule per construct, as Grammatical Framework does with one abstract syntax and many concrete syntaxes | Selected: parsing and linearization are inverse functions of the same grammar, which is what a round trip needs |

Plan (issue #1555): define the two terms once; classify the 320 measured
sentences by the construct that is lost; add constructs to the meta language in
order of loss, each with both rules; use the same representation for the text
of work items (#1487, #1166); measure a round trip through a second natural
language; report what cannot be represented as unresolved spans. Source code
keeps its own track: transformations through the meta language are #1167,
translation between the runtimes is #1500.

Components: [meta-language](https://github.com/link-foundation/meta-language)
(already an optional dependency, `meta-language = "1.0.0"`),
[links-notation](https://github.com/link-foundation/links-notation) 0.23.0,
`rust/src/formalization/`, `rust/src/translation/`,
`scripts/measure-round-trip-translation.mjs` and its ratchet
`data/meta/round-trip-translation-ratchet.lino`. References: Ranta et al. 2020
<https://research.chalmers.se/publication/517996/file/517996_Fulltext.pdf>,
Attempto Controlled English <https://arxiv.org/pdf/1406.2903>, FRET
<https://arxiv.org/pdf/2503.24040>.

## R1548-3 — the meta algorithm is truly recursive

| Option | Verdict |
| --- | --- |
| Keep the handler table and extend the recorded trace until it looks like the picture | Rejected: this is the present state; the trace and the answer have different causes |
| Write a second, new recursive solver beside the existing one | Rejected: `rust/src/recursive_execution.rs::solve_recursively` already implements attempt, split, bounded depth and recompose, and a second engine would be the fourth recursion in the source |
| Give `solve_recursively` an in-process executor and make it the single production entry; every early return becomes a method the procedure may choose | Selected |

Plan (issue #1550): eight steps, from the in-process `TaskExecutor` to the
removal of the trace-only stubs. The test of a task becomes an executed value
(#1551); candidates come from cache, derivation, discovery and composition;
selection orders the candidates that pass their test (#491, #1552).

Prior art checked: hierarchical task network planning refines compound tasks
into primitive ones until the plan is executable
(<https://arxiv.org/pdf/1403.7426>); Soar opens a subgoal for any impasse and
turns the result into a rule by chunking
(<https://doi.org/10.1007/978-1-4613-2277-1_2>); Parsel decomposes a task into
nested function descriptions and searches over combinations with tests, and
reports that decomposition without the test search does not give the gain
(<https://arxiv.org/abs/2212.10561>). The last result is the reason #1550 is
blocked by #1551: recursion without an executed test at each level is not
expected to help.

## R1548-4 and R1548-9 — discovery while reasoning, cached, trusted, just enough; internet knowledge and previous experience

| Option | Verdict |
| --- | --- |
| Ship more captured pages in the repository | Rejected: `data/cache` already holds 2,989 files; captures are stored answers one step removed |
| Turn live fetching on by default everywhere | Rejected alone: the browser runtime and offline users need the cache-only path; this is a choice between valid options and belongs to a switch (#1556) |
| Make the existing `cached_or_research` the miss path of every method of the recursive procedure, with a sufficiency rule in place of fixed ceilings | Selected |

Plan: #1165 (discovery on the production path; today the function is called
only from `rust/tests/unit/issue_1165_discovery_production.rs`), #1163 (the
internet as formal knowledge), #1164 (code examples as knowledge). The
sufficiency rule generalizes `rust/src/option_network.rs`, which already
continues research "while the difference is non-empty". Previous experience is
the procedure cache (`data/cache/coding-procedure-cache.lino`, shipped empty)
plus repair rules learned from failed drafts (#1491).

Components: [web-capture](https://github.com/link-assistant/web-capture) 0.3.37
and [web-search](https://github.com/link-assistant/web-search) 0.6.0 (both
dependencies), `data/seed/sources-registry.lino` (20 trusted sources with tier
and licence), `rust/src/source_fetch.rs` (cache on fetch, sha256, 60-day age).

## R1548-5 — each requirement becomes a test in the required programming language

| Option | Verdict |
| --- | --- |
| A test template per task and language | Rejected: stored answers again |
| Tests written after the solution from its own output | Rejected: such a test cannot reject the solution it came from |
| A requirement record with an observable; the test is derived in the meta language, rendered in the requested language, executed, and independent of any candidate | Selected |

Plan (issue #1551), with typed obligations from #1166 and the repair loop of
#1185. Evidence for the selection: CodeT executes generated tests against
candidates and scores by agreement (<https://arxiv.org/abs/2207.10397>); a 2025
study of failures in automated issue solving places a class of failures in the
iteration and validation phase (<https://arxiv.org/abs/2509.13941>). The three
Hello World runs of 2026-11-10 are the local evidence: no derived test said
"exactly one workflow runs the program" or "no compiled binary is committed".

Components: `rust/src/intent_formalization/obligations.rs`,
`rust/src/solver_handlers/test_generation.rs`, the bounded tool execution of
`rust/src/agent.rs`, the source-language projections of meta-language.

## R1548-6 and R1548-16 — reproduce or generate algorithms; replace hardcoded solutions

| Option | Verdict |
| --- | --- |
| Delete the stored programs now | Rejected: every answer they give would be lost before the procedure can derive it |
| Keep them and add derivation beside them | Rejected: the store answers first, so the derivation is never exercised |
| A census that runs each supported task with its stored answer withheld; stored rows become cache entries that carry the derivation that produced them; a row without a derivation is counted debt | Selected |

Plan (issue #1554): census as a ratchet; families in order of size (151 stored
programs, 138 compiled templates, canned handler rules, 13 pending and 26
justified-native handlers); knowledge expressed in the meta language so that
stored rows become outputs. Existing ledgers are reused, not replaced:
`data/meta/handler-migration-ledger.lino`, `data/meta/core-boundary-ledger.lino`.
Reuse of parts of algorithms is the chunk learning of the meta reasoner
(`data/seed/meta-reasoning.lino`, 32 primitives) and the procedure cache.

## R1548-7 — source code on GitHub by its licence

| Option | Verdict |
| --- | --- |
| Never read third-party source | Rejected: the issue asks to use it as reference |
| Trust the licence field of the GitHub API | Rejected alone: it describes the repository, not a file, and may be absent |
| Evidence from the file (SPDX identifier), the licence text, the manifest and the API; a licence table as data; one decision with four results — ideas only, copy (public domain), library, licence-respecting file copy; deny by default | Selected |

Plan (issue #1553). First change: the default of
`rust/src/coding/fragment_catalog.rs` from verbatim to shape only for an
unknown licence. Components and standards: SPDX identifiers
(<https://spdx.dev/ids>), the REUSE specification 3.3 for per-file licence
information (<https://reuse.software/spec-3.3/>), the existing
`valid_spdx_expression` and `license_forbids_commercial_reuse` gates of
`rust/src/coding_research_learning.rs`, which have no production caller today.

## R1548-8 and R1548-10 — requirements as axioms; study options, score, select, plan, execute

| Option | Verdict |
| --- | --- |
| Keep the in-repository module `rust/src/relative_meta_logic.rs` as the implementation | Rejected: it is arithmetic over supplied evidence, "modelled on" the upstream project, with no derivation of statements |
| Fork the upstream project into this repository | Rejected: the upstream is the place for missing features; a fork diverges |
| Depend on [relative-meta-logic](https://github.com/link-foundation/relative-meta-logic), keep the module as an adapter, report upstream what is missing | Selected |

Plan (issue #1552): obligations become axioms with probabilities and source
spans; contradictions are reported before work starts; open details are listed
as unknowns; candidate values come from the repository, trusted sources and
earlier similar tasks; at least two options are scored for every decision with
more than one valid option; the user is asked only under `guess_probability`.
Selection uses the least-action ordering of #491; sources of evidence are
checked by #1179. Execution of the selected plan in the target repository is
#1483: evidence, obligations, patch and independent checks connected into an
edit — the direct cause of the ten "planned, not executed" drafts.

State of the upstream on 2026-11-10: Unlicense, Rust, no release; open issues
#183, #185 (publish the crate), #186, #187, #188 in that repository. Until a
crate is published the dependency is a git dependency pinned to a commit, which
is why #1552 lists reporting upstream as part of its first step. Alternatives
compared: Markov logic and Bayesian networks need a fixed set of variables and
do not handle statements about statements; probabilistic logic networks and
NARS carry a strength and a confidence per statement
(<https://arxiv.org/abs/2412.19524>), which is the shape relative-meta-logic
already uses.

## R1548-11 and R1548-12 — contradictions updated; configuration; vision, docs and guidelines

Selected: correct the status sentences to what the ledgers measure, keep the
ledgers unchanged, and document each switch with what it changes today. Done in
this pull request (README, ROADMAP, VISION, GOALS, `docs/meta-algorithm.md`,
`docs/configuration/solver.md`, `docs/guidelines/issue-planning.md`). Switches
that change only the trace become switches that change behaviour, or are
removed, in #1556; its page is to be generated from data so that it cannot
disagree with the source again. A consistency check between status prose and
ledgers is part of #1179.

## R1548-13 — the three Hello World test repositories

No option to select: the results were collected after every session had ended
(README section 4). Each defect maps to an issue: duplicate workflow and the
restart that repeated the same steps (#1185, #1551); committed binary and
uncommitted workflow file (#1157, #1551); pull request title and description
not written (#1156, #1158); prompt of another session in the summary (#1559).
The three clients are the matrix of #1162.

## R1548-14 — failed attempts on real issues advance further

| Option | Verdict |
| --- | --- |
| Re-run every failed draft now | Rejected: the installed release has the same four defects, so the result is known; a re-run costs CI minutes and adds nothing |
| Fix each failed issue by hand | Rejected: the issue asks that Formal AI advances them |
| Group the failures by cause, make the cause an issue with the failed texts as regression fixtures, and re-run the drafts when the cause is closed | Selected |

The groups and their issues are in README section 5. Each of #1557, #1558 and
#1560 names the failed issue texts as fixtures in its acceptance list, and
#1170 owns the measure "a draft is retained through review".

## R1548-15 — orchestrator-side reports

Selected: report in link-assistant/hive-mind only what is reproducible with any
provider of an OpenAI-compatible API, in provider-neutral words, and comment on
an existing report where one covers the defect. Findings whose cause is the
model output (language of the summary, the context size the model reports, the
replayed prompt) stay in this repository. The list is in README section 6.

## R1548-17 — every open issue is typed

Done with `experiments/issue-1548/set-issue-type.sh` (GitHub issue type plus the
matching label). The guideline now requires the type at creation.

## R1548-18 and R1548-19 — the case study, and everything in one pull request

R1548-18 is this directory: the collected data under `raw-data/`, the analysis
in `README.md`, every requirement in `requirements.md` and one plan per
requirement in this file. The libraries and components that were checked are
named in the section of the requirement they serve and in
`raw-data/research/online-notes.md`.

R1548-19 has two halves. What the issue put in scope (documentation, issues
created and organized, evidence) is delivered in pull request #1549 and pinned
by `rust/tests/web/issue-1548-vision-blockers.test.mjs`. What the issue names
as a capability of the source is not delivered by a document: each such row is
"Partial" in `REQUIREMENTS.md`, names the issue that owns it, and that issue is
a sub-issue or a blocker of
[#1561](https://github.com/link-assistant/formal-ai/issues/1561). The test
fails when one of these rows is changed to "Implemented".

## Open issues outside this plan

These open issues have no parent and are not critical blockers of coding or of
general question answering; they stay where they are: #447, #453, #483, #557,
#825, #836, #861, #901, #1084, #1190, #1235, #1282, #1330, #1446.
