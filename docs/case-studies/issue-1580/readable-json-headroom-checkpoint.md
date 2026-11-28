# Readable JSON headroom and complete local validation

Three maintained JSON documents approached the unchanged 1,500-line data limit:
the issue-1584 feasibility ledger and the meta-executor/source-fetch fixtures.
Compact only flat primitive leaves that fit within 100 columns. Nested parents
stay expanded; numbers, duplicate keys, strings and ordering retain their exact
non-whitespace tokens. No fixture value, scanner exclusion or limit changes.
The feasibility producer uses the same formatter so its next write retains this
layout. Its formatting round trip is tested; the expensive feasibility run was
not repeated and receives no new execution claim.

The regression fails before formatting on actual maintained-document headroom.
All five cases pass afterward, including large integer spelling, exponent
notation, duplicate keys, escaped strings, malformed JSON, invalid widths,
line-width bounds and idempotence. Independent character-walk and parsed-value
comparisons verify all three complete preimages and final documents. Their line
counts fall from 1,419/1,424/1,401 to 834/1,084/996. Both existing fixture consumer
suites pass. The maintained issue owner imports these cases and passes all 191
tests. The complete 141-entry local JavaScript inventory passes 88 checks with
53 explicit skips and zero failures. No local native build was performed.
The unchanged-limit source scan retains 60 warnings; these are still unfinished.

Published fbac28e5 now has 26 terminal workflows: 22 successful, one skipped and
three failed. All eight native contract families and the registry checks pass.
Coverage and Layered CI exceed the old Git child stdout bound; Pipeline Detect
Changes has the same error at job-log line 1,511. Its native workflow-contract
observer rejects a helper already compiled into two test owners at lines
2,037-2,040, with 194 passes and one failure at line 2,123. Pipeline Status
correctly reports those two failed dependencies at line 204. Local commits
repair both causes, but the next published source still needs hosted validation.

Evidence: experiments/issue-1580/evidence/checks/readable-json-headroom/ preserves
the initial failure, final passing tests, exact fixture preimages, complete gate
streams and source warnings with byte counts, line counts and SHA256 hashes.
The 26 complete hosted workflow observations and failed job streams are retained
under experiments/issue-1580/evidence/ci/integrated-repairs-fbac28e5/.
The API job stream and gh run view stream are different textual renderings of
the same job; their separate hashes are retained without equating their bytes.

All eight template/practice comparisons, original live/budget acceptance,
CodeQL per-child semantics, complete PR review and the independent Formal AI
reasoning floor remain unfinished. Supplied code and conclusions earn zero
independent credit. The PR stays draft and unmerged.

PR: https://github.com/link-assistant/formal-ai/pull/1581
