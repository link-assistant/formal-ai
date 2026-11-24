# Restore the original eight-step verifier-failure regression

The original Scala Hive test allowed eight planner turns. Separate physical
source checks had consumed a ninth turn; raising the fixture ceiling hid that
regression. The preserved before replay reaches the verifier on turn eight but
never returns its final answer. This change restores the original ceiling.

One bounded read-only command now observes the declared supporting owners. The
primary owner keeps its original probe. Every member retains its own identity,
digest, preimage and mutation epoch: writing one owner cannot authorize a later
edit to another. The shared kernel permits at most 16 distinct owned paths and
16 MiB of combined source. Symlinks, escaping paths, missing members, foreign or
reordered members, incomplete nested receipts and duplicate callbacks fail.
Existing single-file callbacks remain supported. JavaScript, TypeScript and
Rust consume the same maintained protocol; no language-specific shortcut was
added.

The actual replay now performs issue read, primary probe/write, supporting
probe, both supporting writes, failed verifier and final answer within eight
turns. Its answer reports the exact compiler failure without claiming verified
execution. Source probes and writes are physical; compiler and issue responses
remain the original fixture mocks, so this is not live compiler delivery.

Seven source-receipt tests pass, including independent mutation epochs,
contradictory callbacks and actual UTF-8 filesystem observations. The complete
Hive consumer and resume-protocol files also passed. All 142 local gate entries
finished: 89 pass and 53 explicitly require hosted native/context checks. An
initial TypeScript currency failure remains archived beside official generation
and its final passing check. Selected rustfmt and native projection checks pass;
native execution remains pending on the next hosted head. No local Rust build
or native execution was performed.

The [original requests, failures, source preimages and verification streams](../../../experiments/issue-1580/evidence/checks/owned-source-batch/manifest.json)
retain exact byte hashes. The first duplicate-callback control exposed a real
draft defect; invalidating all owners from the contradictory receipt repaired
it without weakening the test. Supplied implementation and controller work earn
zero independent authorship credit. Wider issue acceptance remains unfinished.

Pull request: https://github.com/link-assistant/formal-ai/pull/1581.
