# Bound search-adapter setup before coverage begins

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
