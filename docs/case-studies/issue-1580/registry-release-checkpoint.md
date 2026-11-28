# Registry and release verification checkpoint

This checkpoint belongs to [PR #1581](https://github.com/link-assistant/formal-ai/pull/1581).
The full issue remains unfinished and the PR remains draft and unmerged.

## Published CI and warnings

All 23 workflows on published commit 14ef1b5f8dc590733283b0e4cb0cd75e47ba89d2
completed successfully. Eight actual native jobs pass: workflow contracts 102,
coding 32, reasoning 141, interfaces 78, benchmarks 11 and Hive 20/10/7.
These are source-bound hosted results, separate from unchanged-original credit.
Full logs for all 23 workflows and selected native/JavaScript job logs are
preserved with raw/compressed hashes under evidence/ci/native-families-14ef1b5f/.
All 194 original collector archives and 46 additional full-run archives are
hash-verified. Only diagnostic/result selections were read; no literal review
of every successful log line is claimed.

The warning/error candidate scan finds 144 records: 20 echoed shell statements
and 124 emitted source-size headroom warnings. Pipeline raw lines 2106–2352
identify the warning files. Those warnings remain open, with thresholds
unchanged. Green workflows do not establish warning-free CI. The first full-log
collector hit Formal AI's 60-second shell limit after preserving 19 workflows;
a background worker resumes the remaining four and verifies all 23. Both
attempts are retained. Classification and raw locations are in diagnostic-review.json
and classified-diagnostics.jsonl beside the full-run records.

## Registry repair and fixture correction

Both release CLIs converted transport/status/JSON failures to missing-version
or missing-artifact decisions. Local commit 899de4d12 adopts the pinned Rust
registry helper with bounded retries and HTTP deadlines, validating returned
crate/tag identities. Only a genuine 404 establishes absence. Version-floor
and candidate checks complete before writes; existing source-qualified recovery
and publication guards remain intact.

A separate ten-minute hosted job compiles only the registry helper and both
actual release scripts. Loopback cases cover throttling, unavailable services,
dropped connections, permanent errors, malformed/wrong identities, actual 404,
retry recovery and published-version floors. This native job has not yet run
for the local changes. No local Rust build ran.

Local checks pass 36 admission, 96 maintained policy and 32 documentation policy
cases, Rust formatting and nine affected gates. Historical-source admission
controls fail as expected. These are not actual Rust behavior tests. The first
two-job policy mismatch is retained before correction in registry-uncertainty.

Fixture review then found that unpublished candidates returned 404 even in
wrong-identity mode. Commit 00f954c7b fixes the fixture and adds a real HTTP
regression for two unpublished versions. It fails before and passes after the
correction. The fixture-only entrypoint explicitly skips Cargo and claims no
release CLI execution. It, 36 admission checks and nine gates pass. Its receipt
in evidence/checks/registry-candidate-fixture supersedes the earlier Python
fixture digest without changing the original registry receipt.

## Review and remaining acceptance

Rust literal review reaches 139/373 files and 24,142 lines, including all 797
root guidance lines. The remaining 234 files, counterpart applicability and
runtime verification remain unfinished. The shared fragment overwrite/symlink
defect is reported at https://github.com/link-foundation/rust-ai-driven-development-pipeline-template/issues/254.
The template source is inspected; its CLI was not compiled or executed. The
report separately cites Formal AI's passing physical regression evidence.
No upstream repair is claimed.

One bounded independent CI freshness attempt retains its frozen request,
complete response and tool route in registry-uncertainty/independent-ci-freshness.
The response is unparseable and earns no accepted reasoning: 0/20. Supplied
commands, patches and source reviews earn zero authorship credit. Strict dated
original-source acceptance remains 31/53. Broad template, warning, budget,
identity/delivery, benchmark and full PR review requirements remain open.
