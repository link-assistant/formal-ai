# Current CodeQL producer and source-review checkpoint

[Security run 44982847617](https://github.com/link-assistant/formal-ai/actions/runs/44982847617)
completed successfully on `e93881cb64a78a19253088c19f09a1f1b816229c`.
The explicit branch producer has read-only permissions and disables SARIF upload.
It executes both Actions and Rust analysis and the two diagnostic queries.

All 33 original downloaded files (91,420,395 bytes) were compared byte for byte
with their gzip archives. Raw and archive hashes, line counts, artifact run/head
identities and both actual query sources were verified. Both SARIF producers
report successful invocations and zero results, with no warning/error execution
notifications. Rust additionally retains 56 note notifications: 55 consistency
labels and one extractor telemetry record. Labels alone do not establish a
count of affected nodes or complete semantic extraction.

The macro query returns 123,054 rows across 2,296 files, including 641 unexpanded
outside-context calls. The included-string query executes and returns 2,230 rows
across 260 files, including 257 unexpanded calls without CFG expansions.
Another 791 literal expansions expose the token `""`; this observation is not
independent verification of the source payload or its security semantics.
The unexpanded rows and original query outputs remain archived for follow-up.
These successful producers do not prove complete macro or included-string
semantics and do not resolve existing alerts on main.

Evidence: `experiments/issue-1580/evidence/ci/codeql-producer-e93881cb/`.
The producer index is split into four bounded parts. The additional source-review
inventory under `evidence/checks/aggregate-review-follow-up/` records only complete
diffs actually read and binds them to their original base/head and hashes.
The later workspace and file-size repairs have separate current-source and test
evidence; their new native cases still require the next hosted run.

Issue 1580 remains unfinished. Whole-PR feature preservation, every applicable
template practice, independent authorship and external delivery criteria remain
open. Supplied commands and reviews receive zero independent authorship credit.
