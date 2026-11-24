# Search-adapter APT recovery

[Layered CI 45151782730](https://github.com/link-assistant/formal-ai/actions/runs/45151782730)
at f7a71cb86a16e78da0e9ca055c13710d62277739 failed before the JavaScript tests in groups 7 and 8.
The complete log records a regional EC2 Ubuntu mirror. Group 7 reaches ripgrep
unpacking before its first deadline at line 23445; the next attempt fails at
line 23461 because dpkg was interrupted. Group 8 exhausts both original
deadlines, at lines 22530 and 22553, while still using the EC2 mirror.

The fallback previously recognized only the Azure URI. The shared helper now
also recognizes an exact official regional EC2 Ubuntu URI token and selects
the existing official HTTPS alternate. Signed source stanzas, priorities,
comments, file mode, ownership/path checks and unrelated hosts retain their
existing controls. This also fixes the shared browser preparation helper.

The search installer opts into pending-package configuration after exit 124.
Recovery runs inside the next existing privileged and deadline-bound attempt,
before its index/install commands. Recovery errors propagate; a stalled
configuration still ends at the last attempt's deadline. Generic callers
default to recovery disabled. The installer keeps two 60-second attempts,
the five-second delay, the 150-second enclosing budget and the separate
ten-second installed-adapter probe.

Repair commit 0096a8168cb9f611ac55a611cce602884c888607 has two executable regression files:

- rust/tests/web/agentic-search-installer.test.mjs: the same twelve current
  cases have five behavioral failures against the previous scripts and twelve
  passes after repair. Mocks exercise real process deadlines, interrupted unpack,
  successful and failed recovery, stalled recovery, opt-out, invalid input and
  the observed EC2 mirror.
- rust/tests/web/official-ubuntu-mirror-fallback.test.mjs: the same sixteen
  current cases have one failure before repair and sixteen passes afterward.
  They also preserve signed sources, idempotence, mode, comments, lookalikes,
  signature failures and the original finite attempt count.

Three adjacent integration files also pass: 54 tests across five files,
eight affected local gates, three separate shell syntax checks and whitespace
validation. All JavaScript files ran sequentially. The exact current regression
bytes were copied into an isolated baseline fixture; no live repository source
was reverted for that comparison. No system APT/dpkg effect or Rust build ran.

The [complete receipt](../../../experiments/issue-1580/evidence/checks/search-adapter-apt-recovery-20261123/manifest.json)
contains the whole failing hosted log, original excerpts, exact baseline/source
bindings, complete before/after outputs, failed draft attempts and orchestration.
Every archive member was read back and checked for byte length and SHA-256.
Supplied edits and scripts receive zero independent authorship credit.

Hosted repair qualification and the other original acceptance criteria remain
unfinished in [PR #1581](https://github.com/link-assistant/formal-ai/pull/1581).
