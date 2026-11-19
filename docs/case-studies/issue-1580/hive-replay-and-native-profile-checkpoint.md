# Hive delivery replays and the native test profile

Published `cf22c611` completes all 23 workflows: 22 succeed and the [CI/CD pipeline](https://github.com/link-assistant/formal-ai/actions/runs/44606365461) is cancelled after the native job reaches its existing 20-minute deadline. All 104 collection payload hashes verify. The separate Layered CI shard totals are 2,901 JavaScript passes and 17 skips; focused nested runs are not counted again.

The native log has 178 passes and one failure across completed groups, plus three observed scheduling passes before cancellation. This is 181 observed passes, not a completed native suite. The benchmark process has no completion receipt. Library, unsupported-language, HTTP and fresh catalog-artifact stages are not reached. The preserved offline calibration assertion, both new offline instruction controls and all five actual Google Trends catalog/session cases pass. The original-source replay union remains 23/53; thirty original panic criteria and three original budget criteria remain unfinished.

## Failure reproduction and repairs

The failed assertion is `procedures::elided_how_connect_requests_cover_supported_languages` at `reasoning_paths_procedures.rs:525` (native log lines 1873–1878). Hindi `कैसे कनेक्ट करें mysql को node js से` produces lowercase Latin words in its candidate instead of the existing `कनेक्ट-करें-Mysql-को-Node-Js-से` expectation. Chinese has the same affected title construction. The earlier localized recovery applied Russian-style lowercase slugs to every non-English language. Native and browser binders now retain Russian/Spanish lowercase slugs while restoring the established Hindi/Chinese title form and preserving Devanagari marks. Existing browser title and slug binders are reused.

The native procedure test is unchanged from published `cf22c611`. It is not byte-identical to original main `015a6831`: earlier changes localized the API host and Russian slug. The complete original-to-current diff is retained, and this file earns no untouched-original-source replay credit.

The workflow also overrides `[profile.test]` with `CARGO_PROFILE_TEST_OPT_LEVEL=0`. The repository's existing measured profile at `rust/Cargo.toml:207` records 52.86 seconds unoptimized versus 9.34 seconds at level 2 for an in-process test, at about 40 seconds additional compilation. The override is removed from both test stages, inheriting level 2 with debug assertions and overflow checks still enabled. The 20-minute job deadline is unchanged. This explains a source of excess runtime; the repaired head still requires hosted timing evidence.

Both new local regressions fail before repair. Final focused checks pass 49 CI policy cases, seven source-walk cases, eight existing procedure-synthesis cases and eight mapping cases: 72 JavaScript passes. Rust formatting and all fourteen affected gates pass across the initial twelve successes and two repaired budget rechecks. The first added browser helper exceeded the unchanged 36-line allowance; both failure logs are retained, and reuse of existing binders restores 36 lines without changing the allowance. No local Rust build runs.

## Full Hive replay inputs

Commits `ea373c69` and `37bcbb2a` admit the complete existing Hive workflow module, its extended contracts, real workspace tools and consumed fixtures. The first admission adds 39 native cases, including physical source-read/write observations; compiler/network fixtures retain their explicit simulated status. The second adds five existing real Git delivery/restart/idempotence cases plus one prerequisite assertion. Missing compilers or command tools must fail rather than silently skip. Required tools include Git, Node, jq, Python, Rust, timeout, Kotlin and Java.

These 45 pending native cases do not establish live hosted delivery. The delivery harness uses actual temporary Git repositories and compilers, but GitHub/CI responses remain explicit fixtures. The focused target is expected to contain 288 Rust cases plus four HTTP cases after this admission; none of that expected total is claimed executed on the pending head. Separate local admission checks pass 55 and 56 JavaScript cases, seven gates and Rust formatting at their respective source commits.

## Evidence and unfinished acceptance

Evidence is retained under `experiments/issue-1580/evidence/ci/offline-catalog-cf22c611/` and `evidence/checks/{hive-native-admission,hive-native-validation,hive-full-delivery-admission,hive-full-delivery-validation,native-deadline-and-title}/`. Previous publication receipts are archived in `evidence/checks/offline-catalog-publication/`. Fresh issue, PR conversation, inline and review bodies remain unchanged, at counts 1/11/0/0.

Complete reviewed production diffs now cover 101 of 106 selected paths. Five controlled-language data diffs and the rest of the roughly seventeen-thousand-file PR remain unfinished; this selected review is not a whole-PR preservation proof. All wider frozen requirements, eight-template applicability, dependency migrations and warnings, historical cache/Docker and protected-writer evidence, real live delivery/model identity, benchmark floors and general source/parent/passage binding remain active. Independent accepted Formal AI reasoning remains 0/20. [PR #1581](https://github.com/link-assistant/formal-ai/pull/1581) stays draft and unmerged.
