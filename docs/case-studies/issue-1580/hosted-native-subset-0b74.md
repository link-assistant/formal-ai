# Hosted native subset on 0b74

Pipeline [45206750486](https://github.com/link-assistant/formal-ai/actions/runs/45206750486)
executes the repaired release assertion and passes all 294 isolated admission
controls on source 0b74d3b102290b98d8ad3632e8b00e944b742d03. This observation
qualifies these previously pending controls:

- Each changelog collector executes six physical insertion cases. Both preserve
  existing bytes and ignore fenced pseudo-headings. Physical read-error,
  snapshot-cleanup and strict heading controls also pass.
- Both literal NUL-numstat parser controls pass. Actual missing-root and invalid
  UTF-8 read controls fail their observations as intended and pass their tests.
- The release contract checks the actual tri-state registry API successfully.

The independent registry job passes four registry unit tests and 44 CLI controls.
Security job 137804434394 executes the maintained Cargo audit wrapper against all
three committed locks: 490 primary, 339 historical and 91 harness dependencies.
The primary dependency closure still uses its documented Chrono exception. The
historical lock reports the allowed unmaintained unsafe-libyaml warning. Passing
these audits does not complete those migrations or publication authorization.

All sixteen bound production/configuration/lock sources are byte-identical
between this hosted head and the local reconciliation. The practice ledger now
records this execution for changelog insertion, committed-lock enforcement and
size-check controls. Original dated pending observations remain historical.

The [receipt](../../../experiments/issue-1580/evidence/checks/hosted-native-subset-0b74-20261123/manifest.json)
preserves complete downloaded job streams, selected success lines, source hashes,
both ledger versions and the initial log-download refusal. Review is targeted;
it does not claim every log line was read. Every archive member was reopened and
checked against its size and hash.

The full pipeline is still running at this checkpoint. Its original Node 22
scanner mismatch control fails from an empty cache; the corrected local repair
awaits a new hosted head. Complete pipeline qualification, source-size debt,
advisory migrations, all applicable template practices and every other unmet
criterion remain unfinished. [PR #1581](https://github.com/link-assistant/formal-ai/pull/1581)
stays draft and unmerged. Supplied work receives zero independent authorship credit.
