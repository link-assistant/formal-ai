# Cold-cache lock-mismatch control

The actual Node 22 scanner job failed on source
0b74d3b102290b98d8ad3632e8b00e944b742d03. Its five scanner cases and locked
audit passed. Only the deliberately inconsistent lock control failed: offline
npm returned ENOTCACHED before it could diagnose the wrong package version.
A developer's populated npm cache had hidden this prerequisite.

The mismatch phase now uses a private cache and allows the required metadata
lookup, with no retries and a 30-second fetch limit. It still requires npm's
semantic EUSAGE lock-consistency error and verifies that node_modules was not
created. The production scanner installation, exact lock, disabled lifecycle
scripts, engine validation, masking and cleanup are unchanged.

A small regression replays the actual maintained mismatch phase through its
Python AST. It runs real npm against a loopback metadata server and an empty
cache. The server records every request; only secretlint metadata is permitted,
and a tarball request fails the assertion. The original phase fails this control
with ENOTCACHED; the corrected phase returns the required EUSAGE before installing
anything. Validating a different error as success would not pass this test.

Reproduce with `node --test --test-concurrency=1
experiments/issue-1580/secretlint-cold-cache-control.test.mjs`.
All ten maintained scanner tests and all 49 maintained boundary tests pass.
Security and fast-profile neighbors pass 22 and 29 tests. The repaired actual
seven-case engine probe passes locally under Node 26.13.0/npm 11.20.0, including
the private-cache lock refusal and zero-vulnerability audit. Counts overlap.

The [complete receipt](../../../experiments/issue-1580/evidence/checks/secretlint-cold-cache-control-20261123/manifest.json)
retains the original hosted job log, validated artifact and source identities,
failed harness attempts separately from the genuine before-fix failure,
passing observations and complete repaired source. It does not claim the
corrected seven-case probe has passed under hosted Node 22; that remains pending.
This repair earns zero independent Formal AI authorship credit and does not
complete issue #1580.

[Pull request #1581](https://github.com/link-assistant/formal-ai/pull/1581)
