# Calendar seed integrity checkpoint

Draft work for [PR #1581](https://github.com/link-assistant/formal-ai/pull/1581).
The published input is `2ec6ef93a80c7081a46816d1242bc0f7b843d5ff`, including
main `015a683197c89e8f74375b097e8dd06a57dcea8f` (PR #1597).

## Reproduction and repair

[Main pipeline 44188508650](https://github.com/link-assistant/formal-ai/actions/runs/44188508650)
reports a duplicate `calendar-duration-sum` meaning at log lines 65552–65553
and a pipe-packed `year|years` table value at lines 65581–65583. Existing native
data guards reject both representations. The new JavaScript replays use the
existing fast calendar/currency test file, so they run before merging.
With unchanged seed data, 40 of 42 tests pass and these two replays fail.
A separate negative control detects duplicate meanings within and across files.

The addition cue becomes `calendar-duration-addition-cue`; the existing
`calendar-duration-sum` response intent remains unchanged. Twenty-one packed
rows become 55 separate language/surface pairs in their original order.
The complete migration receipt compares every expanded surface. JavaScript and
Rust consumers read the scalar rows directly; generated package seed mirrors
are refreshed through the maintained writer. All original calendar/currency
cases remain, with six additional English/Russian alias prompts in one case.

After the repair, all 43 calendar/currency tests and all four main-regression
ledger tests pass without skips. Rust formatting passes for both changed files;
the native implementations have not run locally. The ledger records both
main-only failures and selects the existing fast test file for each replay.

## Hosted failures and TypeScript mirrors

All 23 workflows on `2ec6ef93` finish: 21 succeed and two fail. The initial
failures in [Layered CI 44197052235](https://github.com/link-assistant/formal-ai/actions/runs/44197052235)
and [pipeline 44197053206](https://github.com/link-assistant/formal-ai/actions/runs/44197053206)
both occur during checkout: curl error 28, a bad pack header, and an unavailable
promisor blob. The blob is the six-byte `.bun-version`, rather than a large
evidence file. Full original logs and exact hashes are retained.

One failed-job retry leaves the pipeline checkout failure unchanged. The Layered
JavaScript group succeeds, exposing a stale TypeScript calendar/proofs mirror
at retry log lines 9493–9495. Both that mirror and the newly changed duration
mirror are regenerated with `scripts/translate-es.mjs`. The complete parity
check passes for all 717 sources, and all 12 translator tests pass.
Native type/lint CI passed on the published input; this is not proof of native
runtime behavior after the scalar-row change. The first complete local gate run
passes 86 checks with 52 explicit exclusions, zero failures. It predates the
health/cache fixture additions below. The final combined-source run also passes
86 checks with 52 explicit exclusions and zero failures in 660.018 seconds, using
a 256 MiB Node heap cap and no native execution. All 86 detailed log hashes and
all 14 checked source hashes are verified. Next-head hosted checks remain pending.

## Health and offline cache fixture replay

Recovered [main Layered run 44188508228](https://github.com/link-assistant/formal-ai/actions/runs/44188508228)
has two further stale fixtures. Group 5 job `134411832462`, lines 2253–2285,
expects a health object without the supported `mode` field. Group 7 job
`134411832372`, lines 2710–2728, expects a baseline after deleting the live-fetch
setting, although reads are now enabled by default. The analogous group 2
version-cache fixture is already repaired on this branch, but lacked fast-lane
selection.

Before the new fixture repair, the complete health file passes 8/9 cases and the
Hive file passes 8/9; the existing version-cache repair passes 6/6. The replay
places an offline `curl` stand-in first on PATH: it supplies only the two
checkout publisher shapes and records every call, with no actual network I/O.
The Hive failure deterministically obtains a live checkout pin from that
stand-in. Health production behavior and default-on source-read policy remain
unchanged.

The health expectation includes `read_write` and checks the `read_only` result
with separate configured/active paths, unchanged released-schema bytes, and no
new files. The entire Hive fixture file disables live source reads and restores
the caller setting. Cache/baseline replay also retains an explicit offline
setting inside its temporary environment.

After the repair, all 24 cases in the three complete files pass with zero mock
transport calls; all 15 fast-profile and four ledger checks pass. The three files
are selected in the fast PR inventory, now 243 files, and each main failure has a
ledger entry. Complete original/repaired fixture bytes, test logs, hashes and
mock-call observations are under
`experiments/issue-1580/evidence/checks/health-cache-fixtures/`. Final full gate
evidence is under its `all-gates-final/` directory; hosted checks for the next
head remain separate from this local result.

## Evidence and unfinished acceptance

Full before/after test logs, expanded surfaces, original seed bytes, generated
projection receipts, and TypeScript before-images are under
`experiments/issue-1580/evidence/checks/calendar-seed-integrity/`.
Hosted logs and metadata are under
`experiments/issue-1580/evidence/ci/{main-015a6831,published-2ec6ef93}/`.
The cancelled main Layered aggregate is partial. All five failed JavaScript job
logs are now recovered individually with their own hashes. Targeted failure
payload review is separate from full-log review. The native panic index contains
39 headers; long payload lines and truncated job tails remain explicitly unread
or incomplete rather than being counted as verified behavior.

The supplied-passage parent still loses context between child requests. The
finite eight-case context-binding probe uses mocked transport and records this
unresolved behavior separately from the already passing transport guards.
General context binding, native/main runtime failures, benchmark and warning
criteria, dependency migration, full template semantic review, historical
evidence and live identity remain unfinished. The PR stays draft and unmerged.
Independent Formal AI reasoning acceptance remains 0/20; supplied scripts,
patches and conclusions receive zero independent authorship credit.
