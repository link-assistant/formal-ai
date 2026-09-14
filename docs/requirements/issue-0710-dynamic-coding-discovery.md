## Issue #710 Dynamic Coding Discovery Continuation

The 2026-09-14 continuation audited every coding and benchmark requirement,
removed benchmark-specific synthesis bodies, and connected structural task
recognition to licensed external parts, composition, bounded verification, and
rediscoverable procedure memory. The implementation plan and before-state
evidence live in `docs/case-studies/issue-710/plans/`.

| ID | Requirement | Status / Evidence |
| --- | --- | --- |
| R710-D1 | Coding synthesis must derive answers from task structure and sourced operations, never from upstream case names or copied task sentences. | Implemented by `task_spec`, `concept_discovery`, and `composition`; `coding_discovery::no_memorization` scans the downloaded HumanEval/MBPP slice against `src/` and `data/seed/`. |
| R710-D2 | The first 20 HumanEval cases must be run honestly through the discovery path and report their upstream grading result. | Implemented by `benchmark run --suite humaneval --slice 20 --online`; local 2026-09-15 measurement is 3/20, up from the committed 0/20 row. |
| R710-D3 | The first 20 MBPP cases must be run honestly through the same discovery path and upstream assertions. | Implemented by `benchmark run --suite mbpp --slice 20 --online`; local 2026-09-15 measurement is 1/20, up from the committed 0/20 row. |
| R710-D4 | Structural coding prompts must enter program synthesis before arithmetic or concept lookup. | Implemented by `coding::task_spec::recognise` in intent formalization and handler precedence; pinned by `coding_discovery::routing`. |
| R710-D5 | Upstream benchmark runs must have an explicit online-discovery mode while PR ratchet checks remain offline. | Implemented by `benchmark run --online`, `run_suite_with_online`, and the scheduled workflow contract; pinned by `specification::external_benchmarks`. |
| R710-D6 | Wikifunctions, Python standard-library documentation, and Rosetta Code must be consumable as licensed, content-addressed function/example sources. | Implemented under `src/coding/function_catalog/` with captured fixtures and `coding_discovery::{wikifunctions,python_docs,rosetta}`. |
| R710-D7 | The coding research contract must name the real external source formats accepted by discovery, while the older procedure-rewrite loop remains explicit. | Implemented in `data/meta/coding-research-learning-contract.lino`; source-specific offline replay is pinned by the coding-discovery source tests. |
| R710-D8 | Requirement sentences must become a language-neutral concept map of structural meanings, candidate parts, evidence, and blocked needs. | Implemented by `concept_discovery::discover`; pinned by `coding_discovery::concepts`. |
| R710-D9 | Candidate parts must be composed into multiple drafts and selected only after CST validation and bounded executable tests. | Implemented by `composition::compose`, `python_render`, and `AgentWorkspace`; pinned by `coding_discovery::composition`. |
| R710-D10 | A verified coding procedure must be content-addressed, provenance-bearing, tamper-detecting, forgettable, and rediscoverable. | Implemented by `DiscoveredProcedureLedger`; pinned by `coding_discovery::ledger`. |
| R710-D11 | Conversational discovery must preserve the same task and concept identities across English, Russian, Hindi, Chinese, and Spanish. | Implemented with seed roles and structural meanings; pinned by the 25 held-out cases in `coding_discovery::multilingual`. |
| R710-D12 | “Count to N inclusive” must derive a complete parameterized range rather than resolve a fixed catalog task. | Implemented by the `range_inclusive` composition with a literal or parameter bound; pinned for N=100 in five languages. This is the coding-discovery portion of #1071, not the whole issue. |
| R710-D13 | Rosetta Code example requests must return attributed, non-generated code; execute-URL requests must run Rust only in the bounded workspace and must never become shell copy commands. | Implemented by `rosetta_code::fetch_example` and `try_rosetta_code_request`; pinned by the exact #862/#863 prompts and four-language paraphrases. |
| R710-D14 | A dated upstream result may regress only against the floor in force at that date; pull requests still may not lower any recorded result or floor. | Implemented by `external_benchmarks::ratchet`; pinned by the historical-floor regression tests. |
| R710-D15 | Published current benchmark numbers must be derived from the latest committed ledger rows, with newer local measurements clearly separated. | Implemented in `docs/benchmarks.md` and `VISION.md`; pinned by `docs_requirements::benchmarks::latest_external_rows_are_published_from_the_ledger`. |
| R710-D16 | This batch must carry a Formal-AI-authored leaf with model, session, evidence, and pull-request attribution. | Delivered by the self-hosting authoring run recorded in the commit trailers; automated by `specification::self_hosting_metric`. Manual confirmation is not yet recorded. |
