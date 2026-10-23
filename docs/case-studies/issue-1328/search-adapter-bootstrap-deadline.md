# Release blockers: setup deadlines, security audit, and owned cleanup

After PR #1401 merged, default-branch Coverage run
[39971329880](https://github.com/link-assistant/formal-ai/actions/runs/39971329880)
cancelled browser coverage shard 1 at its 15-minute job limit. The search-adapter
setup step ran from 15:02:39 to 15:16:34 UTC on 2026-10-23 without output.
Its full log has SHA-256
`1988ec9520dc888237103a9bde5905a10bca48aaf4d6a601042bd8f618788f12`.
Coverage never started; the missing coverage artifact was a consequence.

The installer used an unbounded, quiet APT index update when ripgrep was absent.
The log does not identify the stalled mirror or distinguish the two quiet APT
commands. Both commands now use the existing shared APT wrapper: two attempts,
60 seconds per attempt, a five-second delay, and the existing official Ubuntu
HTTPS alternate after a deadline. Signed package verification remains enabled.
The installed command is probed under a separate ten-second deadline. Both
coverage callers have a three-minute step backstop within the original job cap.
No coverage test, inventory, threshold, or required result is skipped or waived.

The regression executes the actual installer with fake process boundaries and no
network or package installation. It covers an existing adapter, successful
installation, transient index failure, a stalled update and alternate recovery,
permanent failure exhausting exactly two attempts, and a failed version probe.
The transient-failure test fails against the original installer. The stalled
fixture exercises the shared process-group deadline rather than simulating its
return status.

## Security audit discovered during follow-up validation

The follow-up's Cargo audit job
[120497996615](https://github.com/link-assistant/formal-ai/actions/runs/39978123387/job/120497996615)
reported Tokio 1.53.2 under
[RUSTSEC-2026-0386](https://github.com/RustSec/advisory-db/blob/main/crates/tokio/RUSTSEC-2026-0386.md).
Update both direct requirements and both committed Cargo lockfiles to the patched
1.53.3 release, including the older experimental reproduction's Tokio 1.53.1.
Its checksum comes from the public crates.io index; the old and new releases
have identical dependency, feature, and Rust-version metadata. No other locked
package changes, and the advisory is not ignored. Native builds and the real
Cargo audit validate the update in CI; no crate is downloaded or compiled locally.

The dependency update also requires refreshing the governed source witness.
[Browser shard 4](https://github.com/link-assistant/formal-ai/actions/runs/39990184256/job/120545719484)
correctly rejected its stale Cargo fixture hashes. Regenerate through the
maintained explicit review command with the exact reader and previous-witness
hashes. Full object comparison permits only the reviewed Cargo content-hash
references to change; all cases, assertions, profiles, scope, counts, other
source identities, and the reader remain identical. The guarded test fails before
this refresh and all its controls pass afterward. No guard or test is skipped.

## Split release history before it exceeds the maintained file limit

[Lint lane 1](https://github.com/link-assistant/formal-ai/actions/runs/39990184167/job/120555670495)
correctly rejected the generated ledger at 1,506 lines after the version commit
appended another release. Its full log has SHA-256
`ffcb2e8f00265f91f22ad632beb110e9d99dd5204047f6824c7acb16827e8776`.
Generated committed data remains subject to the existing 1,500-line limit.

Split the historical release records into explicitly referenced parts without
changing a record or its order. The metric reader loads the parts before current
records. The appender rotates complete records whenever another append would
cross the limit; the newest record stays in the root for status projection.
Existing parts are immutable, archive references are canonical and bounded,
nested references and symlinks are rejected, and an interrupted publication
cannot silently overwrite history. The version producer stages the parts; its
freshness and published-source checks admit only the exact numbered `.lino`
metadata paths. Source files and unrelated paths retain their original guards.

The JavaScript storage twin performs the initial lossless split and exercises
hundreds of future appends, invalid references, size bounds, and interrupted
writes without compiling Rust. Native controls run against the Rust storage
implementation in CI. The initial split preserves every historical record line
byte for byte, and generated status surfaces are refreshed through their
maintained renderer.

## Bound checkouts and include the required release fragment

The follow-up's changelog gate correctly found no new fragment after the base
release consumed earlier fragments. Add a patch fragment describing this PR's
actual fixes. JavaScript tier 8 and native coverage shard 1 independently reported
the same stale witness and oversized ledger already addressed above; their full
logs introduce no additional engine failure.

The Intel Mac producer was cancelled after remaining in the protocol checkout
step, with no compilation step reached. GitHub returns HTTP 404 for its log, so
the underlying infrastructure cause is unconfirmed. Its downstream
[CLI job](https://github.com/link-assistant/formal-ai/actions/runs/39990184348/job/120570130735)
then correctly refused the missing same-run native artifact. Its full log has
SHA-256 `d04041d9e9815c3d6f410410c09dd5654cebac9e44bb4b98b46e7d2c276ba8f5`.
Apply explicit two-minute deadlines to desktop workflow checkouts, preserving
all source refs, credential settings, fetch depths, job caps, and artifact guards.
An automated parsed-object comparison permits only the deadline changes. The
deployed-workflow regression covers every producer and consumer checkout.

## Recheck queued cleanup plans before publishing a claim

Main's finalizer run
[39982797815](https://github.com/link-assistant/formal-ai/actions/runs/39982797815/job/120519109289)
failed before issue writes because two immutable claims existed for the same
owner. Its full 29,383-byte log has SHA-256
`5d282cf8942e7a6afa5dfc469f578debd1d8f7d79735eeacb4aaf433c57a4d93`.
The claims were published at 16:08:41 and 16:19:43 UTC on 2026-10-23.
The second run finished preparation at 16:08:10, before the first claim existed,
and its sweep did not start until 16:19:11.

Preparation ran before the sweep acquired per-owner job concurrency. A queued
sweep then materialized its old plan without checking whether another run had
already claimed the owner. The apply-time uniqueness assertion correctly refused
the duplicate; weakening that assertion would permit uncertain repeated writes.

Revalidate the exact queued claim, producer, policy, immutable owner digest, and
current claim census inside the serialized sweep before publishing anything.
An already claimed owner produces an explicit pre-claim decision and
no new claim or cleanup request. Claim upload, apply, and terminal-outcome upload
all require fresh eligibility. The original apply-time uniqueness and uncertain
write quarantine remain enforced. An incomplete cleanup receipt is retained
instead of being labelled completed. Existing ambiguous claims remain quarantined;
this fix does not delete history or grant another mutation attempt.

Regressions interleave two producers that prepared before either published,
verify the second cannot publish a duplicate, reject changed queued identities
even after quarantine, and evaluate the actual workflow's effect conditions for
both eligibility outcomes. Existing ownership, interrupted-write, and complete
closure tests remain required. All validation uses fake transports and local
temporary files, without writes to real GitHub issues.
