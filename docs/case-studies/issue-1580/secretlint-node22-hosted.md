# Locked secret scanner on the hosted Node 22 runtime

The existing scanner closure passed actual engine controls locally under Node
26, while the ordinary hosted gate checked a clean changed-file scan. That
clean result did not verify the negative controls under Node 22.

The Security workflow now runs the existing seven-case probe on Node 22 for
ordinary pull requests, main, scheduled checks and explicit dispatches. Its
read-only sparse checkout selects the pull request head when present and retains
the actual configuration, locked dependencies, scanner and archive decoder.
The probe records the Node/npm versions, checkout commit and SHA-256 identities
of all seven source/configuration files before installing anything. A requested
Node major mismatch rejects the run before creating fixtures or installing the
scanner.

The seven controls use actual npm and the actual maintained scanner:

- Clean plain text and clean gzip each succeed.
- A synthetic credential in plain text or gzip fails, with the finding masked.
- A corrupt gzip fails.
- An inconsistent package lock is rejected before installation.
- The committed scanner dependency closure passes npm audit.

Temporary scanner installations are checked after each scan. The probe returns a
failure unless all seven controls complete and pass. The job has a ten-minute
budget and its probe step has five minutes. It preserves partial observations
after a failed probe, using an artifact name that includes the run and attempt.

Four new regressions fail before the change and pass afterward. They check the
hosted policy, execute the maintained shell command with a recording failing
probe, perform a real sparse Git checkout of the required closure, and execute
the real probe with a wrong Node major. These local tests make no package
installations and do not claim an actual Node 22 engine run.

Reproduce the focused check with
`node --test --test-concurrency=1 experiments/issue-1580/secretlint-engine-workflow.test.mjs`.
The existing `bounded-subprocess-boundaries` entry point includes all four
controls and passes 48 tests; security and fast-profile neighbors pass 22 and
29 tests. The runner-label check and seven affected JavaScript gates pass.

Original sources, failing and passing logs, commands and source identities are
retained in the [evidence receipt](../../../experiments/issue-1580/evidence/checks/secretlint-node22-hosted-20261123/manifest.json).
The exact-source hosted seven-case execution remains pending at this checkpoint.
This substep does not complete the other requirements of issue #1580 and earns
no independent Formal AI authorship credit.

[Pull request #1581](https://github.com/link-assistant/formal-ai/pull/1581)
