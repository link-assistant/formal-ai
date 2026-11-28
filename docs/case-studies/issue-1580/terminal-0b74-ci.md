# Terminal 0b74 CI observation

All 33 workflows on source 0b74d3b102290b98d8ad3632e8b00e944b742d03
finish: 31 succeed, Security fails and one explicitly skips. The
[complete receipt](../../../experiments/issue-1580/evidence/checks/hosted-terminal-0b74-20261123/manifest.json)
retains every workflow log stream, paginated job inventories and terminal checks.
All archive members were reopened and verified by size and SHA-256.

[Pipeline 45206750486](https://github.com/link-assistant/formal-ai/actions/runs/45206750486)
passes its producer and all ten native contract groups: 441 selected tests.
Its isolated admission harness passes 294 tests, including the corrected registry
tri-state assertion. The separate registry job passes four unit and 44 CLI tests.
[Named changelog, size-parser and three-lock audit execution](hosted-native-subset-0b74.md)
is qualified against unchanged source identities.

[Security 45206750111](https://github.com/link-assistant/formal-ai/actions/runs/45206750111)
contains the only failed job, 137804434929. The actual Node 22 scanner passes clean,
compressed-clean, finding, compressed-finding and corrupt-compression cases.
Its empty-cache mismatched-lock control fails before semantic lock validation.
Complete original npm output and the reproduced
[cold-cache repair](secretlint-cold-cache-control.md) are retained.
The repaired actual Node 26 probe passes all seven local cases; repaired hosted
Node 22 execution remains pending. The separate
[binary inventory repair](dependency-inventory-bytes.md) also awaits its new
hosted source.

Targeted diagnostic review finds no other failed job. Two fatal Git diagnostics
in successful Layered CI are expected missing-tag and detached-checkout refusal
controls. This review retains all successful log bytes without claiming every
line was read or every warning was repaired.

Dependency Review reports OpenSSF scores of 2.8 for npm/debug and 2.3 for
npm/require-from-string, below its existing threshold of 3. The historical Cargo
lock still reports unmaintained unsafe-libyaml. Release preflight still reports
unknown crates.io publication authority. These observations stay visible and
unfinished; thresholds, advisory policy and publication proof are unchanged.

[PR #1581](https://github.com/link-assistant/formal-ai/pull/1581) stays draft and
unmerged. Broader source review, template adoption, production delivery, original
artifact retention, source-size debt and all other unmet original criteria remain
open. Supplied work receives zero independent authorship credit.
