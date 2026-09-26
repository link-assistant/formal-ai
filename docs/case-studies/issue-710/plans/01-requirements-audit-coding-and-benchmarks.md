# Plan 01 — Requirements audit: coding and benchmarks

The instruction: "collect all issues requirements from all previous issues,
check requirments document, and double check what is fully done, and what is
done partially focus with coding related issues and benchamarks".

## Inputs

| Input | What it contributes |
| --- | --- |
| The 2026-07-14 full-history audit (issue #710, `../raw-data/report-*.md`) | verdicts for all 329 closed issues and 317 merged PRs up to v0.285.0; the 32-row checklist is re-verified at the current PR head in `../README.md` (31 `works-now`, 0 `still-broken`, 1 `superseded`) |
| The 2026-08-04 full-history audit (issue #957, `docs/case-studies/issue-957/raw-data/req-chunk-*.ndjson`) | 944 individually verified requirement records from #1–#929; the 296 that name code, programming, benchmarks, algorithms, synthesis, execution, Rosetta Code or Wikifunctions are extracted to [`../raw-data/coding-and-benchmark-requirements-2026-09-14.md`](../raw-data/coding-and-benchmark-requirements-2026-09-14.md) |
| Issues #930–#1137 (`gh issue list --state all`) | 119 issues opened after that audit; the coding/benchmark ones are rows below |
| `REQUIREMENTS.md` (assembled from 106 shards in `docs/requirements/`) | the R-rows for #96, #283, #304/#317, #349, #362, #395, #408, #412, #468, #559, #656, #674, #698, #848, #873, #914, #917, #918, #919, #922, #923, #932, #936, #991, #1085 |
| `ROADMAP.md` requirement-level status table | the rows still marked Partial / Not done for coding and benchmarks (lines 362–438) |
| `docs/requirements-traceability.md` | delivered / automated test / manual confirmation per R-row |
| `data/benchmarks/external-results.lino` and the 2026-09-14 scheduled run | the honest upstream numbers |

The baseline columns below were checked against the branch at `cde14085d`
(= `main` v0.350.0 plus this PR), not against an issue being closed. The final
column in section B was re-verified against the completed plan-03 tree on
2026-09-15. "Done" means the production path does it and a test pins it;
"partial" names what is missing; "not done" means no production path exists.
Requirement IDs written `#957-audit R<issue>-<index>` cite the issue #957
audit namespace (`docs/case-studies/issue-957/raw-data/req-chunk-*.ndjson`),
not `REQUIREMENTS.md` ids; a reader looks them up in that NDJSON, since no
requirement shard carries them.

## A. What "done" already means for the coding path — the regression floor

These are done and must not regress. They are the parts the discovery design
composes; none of them is replaced.

| Row | Requirement | Evidence | Status |
| --- | --- | --- | --- |
| A1 | Generated code is verified by execution in a bounded workspace before it is shown, and the execution status is reported honestly (#957-audit R324-3, #957-audit R331-6, #440) | `src/agent.rs::AgentWorkspace` (5 s budget, cleared env); `program_synthesis.rs::verify_python_candidate`; `code_task.rs` `rustc` for new Rust targets | done |
| A2 | Every generated program is validated as a CST through the meta-language links network, not by string checks (#386-9, #395) | `src/coding/cst.rs` → `meta_language::LinkNetwork::parse`; `data/seed/program-cst-grammars.lino` | done |
| A3 | Coding vocabulary lives in seed meanings in en/ru/hi/zh(/es), not in Rust literals (#395-1, R341, R379) | `data/seed/meanings-program-synthesis.lino`, `meanings-coding-tasks.lino`, `coding-idioms.lino`; gate `scripts/check-hardcoded-language.rs` | done |
| A4 | Multiple candidate drafts, selection by generated tests, least-action ranking (#704) | `src/draft_portfolio.rs` (`PortfolioLeaf`, k drafts, ranked survivors) | done as an engine; the synthesis leaf feeds it only the seeded strategies |
| A5 | Budget-driven random/evolutionary search when no reusable part exists (#662) | `src/solver_search.rs` (arithmetic reachability only) | done for one domain |
| A6 | A miss is a named skill gap, never a catalogue recitation (#699 batch 3) | `src/program_skill_gap.rs` (`write_program_skill_gap`) | done |
| A7 | Research over cached, content-addressed sources with provenance; live access opt-in; offline replay (#873, #896, #991) | `src/source_fetch.rs::CachedSourceClient`, `src/source_research.rs`, `src/how_to_guide.rs`, `tests/fixtures/issue-991/` | done |
| A8 | Trusted source registry with tiers, licenses and opt-outs as data (#709, #991) | `data/seed/sources-registry.lino` (wikidata, wiktionary, wordnet, wikipedia, wikihow, stackexchange, wikifunctions, rosetta_code, wikibooks, …) | done as data; `wikifunctions` and `rosetta_code` have only an `opensearch` URL and no consumer that reads a function or an implementation |
| A9 | Repeated verified execution traces become algorithm proposals; adoption is gated (#531, #656, #848-9, #922) | `src/algorithm_discovery.rs`, `src/workspace_change_learning.rs`, `src/promotion.rs`, `improve --promote` | done as protocol |
| A10 | The real upstream harness: fetch at run time, upstream grading, `passed/total`, ratchet, weekly job, `benchmark_unavailable` (#698 R528–R535) | `src/external_benchmarks/`, `.github/workflows/external-benchmarks.yml`, `data/benchmarks/external-results.lino` | done |
| A11 | Curated industry slice 13/13 with held-out variants and a `minimum_pass_count` ratchet (#304/#317) | `data/benchmarks/industry-suite.lino`, `tests/unit/specification/benchmarks.rs` | done — but see B1: the two coding cases pass through per-task bodies |
| A12 | Coding-modification suite 4/4 in four languages (#362); text/code-edit profile 1440/1440 (#408) | `coding-modification-suite.lino`, `text-manipulation-suite.lino` | done |
| A13 | Coding ladder through the real Agent CLI, 65/130 at v0.320.0, ratcheted (#848, E69 #916) | `data/meta/ladder-ratchet.lino`, `experiments/issue_1028_agent_cli_ladder` | done as measurement; capability partial by its own number |
| A14 | Upstream prompt shapes (HumanEval imports, MBPP asserts) transfer to the synthesis path (#1085 D5.3) | `src/coding/python_signature.rs`, `tests/unit/issue_1085_upstream_prompt_transfer.rs` | done for the signature; the body is memorized (B1) |
| A15 | Every failing upstream case becomes a learning-frontier input; the workflow is red on regression and warns on stagnation (#1085 D5.1–D5.2) | `data/meta/learning-frontier-upstream-benchmarks.lino` (80 prompts), `benchmark run --frontier-record` | done |
| A16 | The upstream row is cited beside every curated 13/13 citation (#958, #1085 D5.4) | `VISION.md` "Current Direction"; `ROADMAP.md` | done and enforced: `VISION.md`, `ROADMAP.md`, `ARCHITECTURE.md` and `README.md` cite the upstream rows beside the curated numbers, and `tests/unit/docs_benchmarks.rs` pins the published rows to the latest committed ledger rows — the 2026-09-07 staleness the audit warned about is closed |

## B. Partial or not done — the rows this PR works

| Row | Requirement (source) | Verified state at `cde14085d` | Verdict | Evidence after plan 03 landed |
| --- | --- | --- | --- | --- |
| B1 | "We should not hard code solutions for any of tests" (2026-09-14); "remove per-case handlers", "internet-research-driven synthesis" (#957-audit R395-1); "the coding algorithm itself is auto-discovered during execution — truly general, truly dynamic" (#957-audit R395-6); D5's test "above 0/20 through generalised rules … not per-case branches" (#1085) | `src/solver_handlers/program_synthesis.rs::synthesize_python_candidate` returns one of three static bodies (`has_close_elements`, `similar_elements`, `count_vowels`) and `None` otherwise; the browser worker mirrors the same three (`formal_ai_worker_06.js:216`, `formal_ai_worker_13.js`); `data/parity/cross-runtime-synthesis.json` pins the third | **not done** | L2–L8 replaced those bodies with `task_spec` → `concept_discovery` → `composition` → bounded verification; `coding_discovery::no_memorization` rejects upstream entry points and task prose in production sources. |
| B2 | HumanEval 20-slice upstream score (#698, #1085 D5, DoD 5 "a coding suite above zero") | 2026-09-14 run: `passed=1 failed=19` — the one pass is B1's `has_close_elements`; 12 cases answer "I could not determine …" (the unknown opener written into the `.py` file), 7 answer with no code at all, 2 produce wrong code | **done** | The generalized path passes **20/20** on the upstream first-20 slice, including an empty-source-cache control, and the dated ledger floor is 20. |
| B3 | MBPP 20-slice upstream score (#698) | `passed=0` on every recorded run; `similar_elements` is task 2 and is memorized, yet the recorded row is 0 (the MBPP prompt shape reaches the handler only when the lexicon roles match) | **done** | The generalized path passes **20/20** with live official-source discovery and **18/20** from an empty source cache; the two cold gaps are externally defined sequences rather than embedded benchmark answers. |
| B4 | A coding prompt must be recognised as a coding prompt before any lexical route (R559 problem frame; #906 language router; #745 out-of-box generalisation) | `formal-ai chat --prompt "Complete this Python function … def sum_product(numbers: List[int]) …"` answers with the Wikipedia definition of *Color* (formalized as an arithmetic task, then concept lookup on a fuzzy token) | **not done** | `route_for_prompt` now assigns structural coding shapes before arithmetic/concept lookup; `coding_discovery::routing` pins the former HumanEval/8 misroute and the arithmetic counterexample. |
| B5 | The benchmark solver may research: "discover enough knowledge in the internet" (2026-09-14); "treat the internet as a public database and the local store as a cache" (`VISION.md`) | `external_benchmarks::benchmark_solver()` forces `offline: true`; nothing in library mode can reach a source | **not done** | `benchmark run --online` and the scheduled workflow enable live discovery; the PR-time ratchet remains deterministic and offline over the ledger. |
| B6 | Rosetta Code and Wikifunctions as sources of ready algorithm parts (#957-audit R1-5 `unclear`; R289 #412) | `src/knowledge.rs` embeds 25 reviewed snippets (Rosetta/Wikifunctions/Hello World/SO) as a static oracle; no live path reads a Wikifunctions function, its implementations, testers or labels; the registry's `wikifunctions`/`rosetta_code` entries are search URLs only | **partial** (static cache, no discovery) | `function_catalog::{wikifunctions,rosetta_code}` now use `CachedSourceClient`, licensed captures and content hashes; offline fixture tests pin function parts, implementations, testers and attributed task-page examples. The 25 embedded `ORACLE_SNAPSHOTS` snippets **remain** in `src/knowledge.rs` as the offline cache of record; replacing that unfilled cache with retrieval from the live sources is issue #1138 B2, not this PR. |
| B7 | Research-driven coding procedures (#919 R919-1…6) | delivered for one shape only: a source page must already be in the `formal_ai_coding_procedure_v1` format and the executor is a workspace *rewrite*; a real page (Wikifunctions, docs.python.org, Rosetta) never satisfies `captured_procedure` | **partial** (the loop exists; it cannot learn from real sources) | The learning contract now admits Python documentation, Wikifunctions and Rosetta formats; L4–L7 feed their source-backed parts through the same discovery/composition cycle. |
| B8 | "understand each word/concept", "formalization itself may need recursive knowledge collection from trusted sources" (2026-09-14); formalization chain Wikidata → Wikipedia → Wiktionary (`VISION.md`) | word lookups exist for definitions (`solver_unknown_reasoning.rs`, `definition_merge`); nothing turns a docstring phrase such as "closer to each other than given threshold" into concepts and then into parts | **not done** | `concept_discovery` resolves seeded structure first, then local procedures, Python docs and Wikifunctions, with bounded unknown-word lookup; multilingual fixture tests pin the resulting `ConceptMap`. |
| B9 | "reconstruct from the formalized knowledge … the step by step guide/algorithm" (2026-09-14); "an ALGORITHM BUILDER over templates" (#957-audit R412-2, deferred); "the meta-algorithm CONSTRUCTS the … algorithm from this specific task" (#957-audit R423-2, deferred); #957-audit R1-4 universal problem solving without prior knowledge | `meta_algorithm_builder.rs` records the evidence shape; `coding-idioms.lino` composes list operations from seed idioms (issue #395) — the only place composition exists today | **partial by design** | `composition` builds and ranks candidate trees from seed-declared structural idioms/runtime templates; exact tests cover direct, tuple, pairwise and reduce/filter compositions plus failure trails. Plan 03 L7 records the same work as implemented — the three previously memorized tasks pass by derivation — and there is no contradiction: composition is implemented over *seeded* idioms, while composing from *retrieved* sources is issue #1138 B2. |
| B10 | "so it is always easy to forget all discoverable experience, and rediscover it" (2026-09-14); "record enough similar sequences … arrive to algorithm by deduplicating" (architect note 2026-09-11) | learned procedures exist for rewrites (#919) and workspace changes (#848-9); no ledger for a discovered program construction; no test forgets and rediscovers | **not done** | `discovered_procedures` stores content-addressed provenance; tests pin cache hits, deletion and identical rediscovery, plus rejection and re-derivation of a tampered entry. |
| B11 | Every coding surface in five languages: a Russian/Hindi/Chinese/Spanish "write a Python function that …" reaches the same path (R848-7, #706) | synthesis roles are seeded in en/ru/hi/zh/es for the two curated tasks; the upstream shapes are English code, the conversational shape is not exercised through discovery | **partial** | `coding-discovery-paraphrases.lino` supplies 25 held-out prompts across en/ru/hi/zh/es; all 25 reach equivalent specs and verified compositions. |
| B12 | "Support counting to 100 or any number selected by user" (#1071, open) | catalog task `count_to_three` is a fixed program; "count to 100" has no route | **not done** | Parametric `count_to_n` binds the requested number to seeded `range_inclusive`; five-language tests execute and verify output through 100 without a per-task handler. |
| B13 | "Execute https://rosettacode.org/wiki/Copy_stdin_to_stdout in Rust" (#862, open); "example of how to do copy stdin to stdout in Rust" (#863, open) | both open; the URL is mangled into a shell command (#862), the example request runs `cp` (#863) | **not done** | Exact prompt tests fetch and attribute the Rust section, then compile/run only the fetched example in the bounded workspace; the URL is never treated as a shell command. |
| B14 | The ratchet must never read history as a regression: `humaneval 2026-07-20 … 2026-09-07: passed=0 is below the recorded minimum_pass_count=1` failed the 2026-09-14 scheduled run after that same run raised the floor | `src/external_benchmarks/ratchet.rs::violations` compares every dated row of a suite against the suite's current `minimum_pass_count` | **defect** | `ratchet::violations` now evaluates each row against the floor then in force; tests pin historical rows, a later regression, and rejection of a lowered floor. |
| B15 | External-benchmark numbers quoted in docs must match the ledger (#958 automated test) | `docs/benchmarks.md` "Honest current numbers" is the 2026-08-10 table; `VISION.md` quotes 2026-09-07 | **partial** | `docs/benchmarks.md` and `VISION.md` now match the latest committed ledger rows; the docs test derives its expected values from that ledger. |
| B16 | Each pull request carries Formal AI-authored work, "as big as it can be, but as small as it actually can" (architect note 2026-09-11; gate `scripts/check-formal-ai-contribution.rs`, relaxed after 24 h) | the PR's two authored leaves date from 2026-08-01 and are the two commits plan 00 §3 repairs; nothing in the 2026-09 continuation is Formal AI-authored yet | **partial** | The branch binary and external Agent CLI authored `coding-discovery-recipe.lino` in session `ses_f5ec49c02ffe6yDSkWK0x1t0p1`; the committed evidence pointer links its secret-scanned, unlisted raw trace. |

## C. In scope of the audit but out of scope of this PR — named, not dropped

These are coding or benchmark requirements that are not done. They stay open
under their owners; this PR neither closes nor narrows them. They are listed
so the audit is complete and so nobody reads their absence as "done".

| Row | Requirement | Owner | Why not here |
| --- | --- | --- | --- |
| C1 | Telegram: compile and run every code example in a `link-foundation/start` docker container; halve iterations on a 1-minute timeout; 10-minute hard fail with a verbose log (#957-audit R8-1/2/4, #930 E78) | #930 (open) | a Telegram execution backend, independent of discovery; the discovery path produces verified code the same pipeline would run |
| C2 | Per-conversation detached execution containers with snapshot/replay (#957-audit R331-5, #937 E85) | #937 (open) | container lifecycle work |
| C3 | Installation-guide ↔ script corpus of 50+ top GitHub projects (#957-audit R423-1, #939 E87) | #939 (open) | corpus growth for a delivered converter |
| C4 | SWE-bench Lite 0/1 (#698) | #698 ledger | repository-level patching through the official harness; the discovery path is a prerequisite, not the whole |
| C5 | GSM8K 2/20, MATH 0/20, BIG-bench object counting 0/20, CoEdIT 0/20 | #698 ledger, #1085 D5 | math, counting and text editing; the same discovery loop applies later, the coding focus was asked for first |
| C6 | The frontier user prompts #720, #721, #722, #724, #869, #1063, #447 (#1087 E109) | #1087 (open) | not coding; each needs its own four-language paraphrase set |
| C7 | The links network as the executable system of record; edits as link substitutions over the CST network (#1085 D1, D2) | #1085 follow-ups | the discovery path records links and reads seed links; making the store the interpreter is D1's own work |
| C8 | Browser execution of Python (WebVM/Pyodide, #670 E51) | #670 (open) | the browser worker cannot verify Python today; plan 02 §10 states the boundary honestly |
| C9 | Handler-migration ledger ratchet and seeded promotion predicates (#959 E107) | #959 (open) | this PR removes three per-task branches, which moves the ledger the right way; the epic is wider |

## D. What the `REQUIREMENTS.md` check found

- The document is generated from the shards; a hand edit fails
  `check_requirements_document`. New rows for this work go into a new shard,
  `docs/requirements/issue-0710-dynamic-coding-discovery.md`, and
  `REQUIREMENTS.md` is regenerated (leaf L14).
- Rows R919-2 and R919-3 read as if research already learned from real sources;
  the status text will say that the accepted source format was the
  repository's own procedure format until this PR (B7). No row is deleted, no
  status is quietly downgraded: the wording gains the fact and the date.
- Row R289 (#412) claims the oracle "generalises the coding catalogue beyond
  its built-in languages"; it does so from 25 embedded snippets. The row gains
  the live Wikifunctions path as its evidence once L4 lands.
- The traceability table (`docs/requirements-traceability.md`) gets a row per
  new requirement with the automated test and "not yet confirmed" for manual
  confirmation until someone confirms it by hand (konard's 2026-08-04 rule).

- [x] inputs collected; raw table generated (`../raw-data/coding-and-benchmark-requirements-2026-09-14.md`, 296 rows)
- [x] A/B/C/D tables written against `cde14085d`
- [x] B-rows re-verified after plan 03 lands (each row's final column records its evidence)
