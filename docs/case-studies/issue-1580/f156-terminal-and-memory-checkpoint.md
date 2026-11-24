# Terminal native qualification and included-string memory candidate

Published `f156be6e821addc2c3c405f482d830493183e3c0` has 27 terminal workflows:
24 success, one skip and two failures. The Pipeline, all eight native contract
families, and the actual release registry-script job pass. This qualifies the
previous process-group acknowledgement, pipeline input and failed-fetch native
repairs on that source. All 302 retained command streams and all 27 full run logs
verify by raw/compressed hashes, byte counts and line counts. Retention does not
mean every line of every log has been reviewed.

[Layered CI 44872982415](https://github.com/link-assistant/formal-ai/actions/runs/44872982415)
fails at full-log lines 13160-13162 because the generated repository-definition
TypeScript twin is stale after the merged main change. The official generator
updates that one file; the complete 729-file currency check passes. Its twelve
focused translation tests and the touched JavaScript gates pass.

[Security 44872977660](https://github.com/link-assistant/formal-ai/actions/runs/44872977660)
fails at full-log line 4820: the optional included-string observer still exceeds
the unchanged four-minute deadline with four threads. Both standard SARIF gates
pass. Its complete 21,050-event evaluator trace has no completed query and shows
large global path-resolution/CFG dependencies; overlapping predicate timings
must not be added. Increasing threads alone did not resolve the failure.

The included-string candidate now uses the producer's positive CODEQL_RAM up to
8192 MiB, retaining 4096 MiB when absent. Invalid or insufficient memory settings
fail before any CLI execution. The resolver and query use the same selected
budget; resolved heap must remain at least 2048 MiB and no larger than that budget.
The separate macro profile stays at 4096 MiB. Query bytes, all nine columns,
unexpanded/other rows, payloads, strict SARIF checks, four-thread cap, disk budget
and four-minute clocks remain unchanged. The adapter regression fails on the old
4096-only adapter and passes after repair. Three adapter cases, 22 strict-security
cases and touched gates pass. These mocks establish argument/manifest behavior;
hosted performance of the memory candidate remains unverified.

Current Go source comparison accounts for all 39 blobs. Ten malformed PR needs
inputs reproduce false success; its schema candidate passes eighteen controls,
and the maintained template test entry point passes 113 cases. The report is
[Go issue 23](https://github.com/link-foundation/go-ai-driven-development-pipeline-template/issues/23).
[C++ issue 39](https://github.com/link-foundation/cpp-ai-driven-development-pipeline-template/issues/39)
retains eighteen original/candidate controls. Neither report proves an applied
upstream repair or complete template practice adoption.

The six current C++ changed files, eight C# changed files and twelve Rust changed
files have separate pinned ledgers. Rust history distinguishes literal review
from consecutive byte-identical block comparison. The exact current C++ transfer
tests pass twice, C# merger/pipeline tests pass eighteen cases, and Rust workflow
tests pass five cases. Template tests use narrow fixtures: no native build,
complete upstream suite, real Cargo audit, live Pages deployment or publication
is inferred. Current Rust PRs 255, 253 and 248 and all three comment types were
read. Remaining current and historical template trees and applicable practices
are still incomplete.

Evidence: `experiments/issue-1580/evidence/ci/native-families-f156be6e/` and
`experiments/issue-1580/evidence/checks/f156-failure-repairs/index.json` retain
925 investigation/source records with round-trip hashes. Failed harness attempts,
truncated previews and negative controls retain their original status.
Supplied commands, patches and reviews earn zero independent authorship credit.

The unchanged source warning inventory, per-child security semantics, dependency
migrations, live publication/runtime/model identity, complete original acceptance
and whole-PR review remain unfinished. [PR #1581](https://github.com/link-assistant/formal-ai/pull/1581)
stays draft and unmerged.
