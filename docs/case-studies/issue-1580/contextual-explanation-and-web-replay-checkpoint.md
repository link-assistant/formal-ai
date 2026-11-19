# Contextual explanation and existing native web contracts

The [published validation-origin batch](https://github.com/link-assistant/formal-ai/pull/1581) is `5f64ad16450a84ae1384d6fba753ab0efbd47866`. All 23 workflows finish: 21 succeed; Layered CI and the native pipeline fail. The complete failed workflow/job payloads and all 96 collection hashes are retained in `experiments/issue-1580/evidence/ci/event-origin-5f64ad16/`.

Actual hosted execution passes 147 Rust cases and four HTTP cases. The reasoning target passes 47 cases and fails one. All four new validation-origin controls, nine unchanged debug-session cases and six of seven pronoun cases pass. The strict original-source replay union stays 17/53, with 36 original panics and three original budget failures still without qualifying replay. This head is not green.

## Existing failures and repairs

The [native run](https://github.com/link-assistant/formal-ai/actions/runs/44560230055/job/135687590817) fails its unchanged `seeded_coreference_contexts_normalize_terminal_multilingual_surfaces` assertion at job-log lines 1772–1775. Reproduce by asking for a Rust program that lists the current directory, retaining the user/assistant turns, then asking `Explain the program?`. Expected intent is `coreference_program_artifact`; this published head instead returns `explanation_research`. The native fact row runs researched explanations before the coreference row and can answer from a general cached page instead of the actual antecedent.

The repair reuses the existing seeded reference and real user-antecedent resolver at the entry to `try_fact_lookup_with_client`. A resolvable contextual reference yields to the coreference row. Equivalent lazy fallbacks retain the seeded/live/explanation order within the handler's original 326 lines. No prompt-specific answer, altered test expectation or status bypass is added. Without an actual matching antecedent, ordinary seeded/live facts and researched explanations remain eligible. The complete original pronoun test file remains byte-identical. Hosted repair execution is pending.

The [Layered CI run](https://github.com/link-assistant/formal-ai/actions/runs/44560229853/job/135687319050) fails only `generated native registry is current`; workflow-log lines 9549–9578 show the stale-registry error. The actual new optional origin field changed the complete `thinking.rs` source witness, while the constant's value and translation were unchanged. The official generator refreshes that one hash in the 158-object registry. All complete objects are equal after normalizing exactly that hash. Its unchanged five-case regression fails 4/1 before regeneration and passes 5/0 afterward.

The explicit governed writer produced serialization-only changes: complete deep equality found no changed inventory paths. Its output was discarded after that equality proof, preserving the entire governed baseline byte-for-byte at SHA-256 `e08d3960fe47d0365a81a299b3bf3a59cc6afb851993fda312a4179f82cce2c8`. Counts remain 1,189 total / 221 supported / 968 unsupported; no unsupported case receives native credit.

## Existing contracts admitted for native execution

The focused reasoning target now includes all 30 existing web-request/fetched-page cases, including the existing extended child module, and all seven existing fixture-transport fact/explanation cases. The existing modules and assertions stay intact. Plain `mod web_requests;` preserves Rust's actual child-module directory. Admission includes every new source and the consumed web-formalization fixture, with unrelated-source rejection retained. This adds 37 cases: 85 reasoning / 185 total Rust cases plus four HTTP cases are expected, with execution still pending.

The native fixture cases retain ordinary cited explanations, seeded records ahead of live lookup, offline refusal and actual source-derived facts. Browser parity checks retain all seven seeded multilingual program references and negative controls, plus five cited-explanation controls. These JavaScript checks do not certify native execution.

Eight finite offline worker observations preserve query evidence but omit queries in answer prose. They are observations rather than passing native search-content assertions. The actual six existing page-formalization tests pass. The initial wrong module-path formatting attempt and the real failing admission test remain retained with zero passing credit.

## Validation and limits

The final focused run passes 101 JavaScript cases across eight files and Rust 2024 formatting. The complete 139-entry lightweight gate inventory finishes with 87 passes, zero failures and 52 explicit native/context exclusions. All 98 final focused, aggregate and detailed gate payloads and all nine source hashes are verified in `experiments/issue-1580/evidence/checks/contextual-web-verified/review.json`. No Rust build or native test runs locally.

An earlier full gate run records 86 passes, one abbreviation failure and 52 exclusions. The failure is an archived filename, not a product binding. The two new archived `tests-as-docs` log names are spelled out without changing their compressed bytes; final checks retain the original failure. No ceiling is raised. A subsequent inventory passes 85 gates and fails two boundary checks because the first guard draft grows that handler by six lines. The equivalent lazy fallback chain preserves the original 326 lines; this failure is retained separately in `contextual-web-final/` and a complete fresh run verifies the repair.

The publication documentation replay initially passes four gates and fails two file-size checks because two newly staged Java page manifests exceed 1,500 lines. Reformatting preserves their complete decoded objects and every page entry in compact per-page records. The failure logs and before/after identities are retained in `contextual-web-publication-boundary/`; no limit is raised. The affected checks are replayed before publication.

Java historical review reaches 182/254 continuous pages at pinned head `dd803a2d9fa5b3a992b435acce73162274fb9e8b`: 10,874 complete lines, 135 complete files, one partial transitive inventory. The complete historical package registry and dependency inventory are retained as historical evidence, without asserting current versions or applicability. Pages 183–254 and wider template applicability remain unfinished.

Every broader acceptance criterion stays tracked in the work plan. Accepted independent Formal AI reasoning remains 0/20. Supplied code, scripts, tests and reading earn zero authorship credit. Keep PR #1581 draft and unmerged.
