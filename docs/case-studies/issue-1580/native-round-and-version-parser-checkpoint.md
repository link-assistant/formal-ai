# Terminal native round and version parser, 2026-11-21

The published head `27e895c1103926b4f73862c0eb15fa77fb505089` has 27 terminal
workflows: 25 success, one skip, and one failure. All eight native contract
families and the standalone release registry CLI job pass. The only failure is
[Layered CI 44860043349](https://github.com/link-assistant/formal-ai/actions/runs/44860043349):
full-log lines 7010–7030 and 8070–8105 identify the stale generated native source
witness. Commit `858943eb` repairs the two hashes using the maintained generator;
the five existing source-witness regressions pass. The archive retains 27 full
run logs and verifies all 354 command streams by raw/compressed hashes, bytes
and lines. This verifies custody, not literal or semantic review of every log.
The initial assessment's overly narrow native-family name selector matched no
jobs; the retained job metadata supports the corrected count of eight.

The read receipt extraction moves existing classifiers out of `progress.rs`,
reducing it from 1,000 to 893 lines. Exact helper-body comparisons, Rust
formatting, source projection and generated witness checks pass. The existing
18 JavaScript planner/fetch cases pass. The maintained source scanner now
reports 91 warnings and zero ordinary or embedded violations under unchanged
limits. Native execution of this extraction still needs the next hosted head.

The standalone Rust PR version check previously selected the first document
`version` assignment. A dependency table before `[package]` could hide a package
version change or reject a dependency-only change. It now reuses the actual
Cargo package parser. Four new CLI regression groups cover both directions,
root/rust layout movement, quoting, inherited or missing versions, duplicate
keys and invalid types. The hosted isolated registry harness builds the exact
standalone script. Its new cases have not yet executed; local pre-fix evidence
for this script evaluates the old expression without running native code.

The actual JavaScript PR admission check also used a partial TOML expression.
Three new regression groups fail against that old implementation and pass
unchanged after the repair. A small isolated Python 3.11+ standard-library TOML
reader validates the complete manifest, including valid quoted/inline/escaped
spellings and invalid duplicates or multiline-string decoys. Unknown identity
still fails closed. The immutable GitHub base observation and no-Git-fallback
controls remain enforced. Sparse checkout includes the exact parser file.

All 115 default-branch policy cases, 18 production-profile cases and three
checkout cases pass sequentially. Five touched JavaScript gates pass. Rust
formatting, source projection, generated witness and deployed staged-release
currency checks pass without local native compilation or execution. The
canonical workflow refresh changes only the version-check checkout; all 52
staged operations and their bindings remain identical. Python availability and
the new native regression groups need next-head hosted qualification. The
earlier full 88-gate result predates these edits and is retained separately.

The evidence includes failed controller attempts, the initial failed projection
check, an incorrect script-path probe, and the partial parser integration round
whose old test canary blocked the intended parser child process. They do not
receive passing verification credit. Supplied controllers and conclusions
receive zero independent reasoning or authorship credit.

Complete template comparison, outstanding source and security semantics,
maintenance migrations, live publication/runtime checks and final diff review
remain unfinished. [PR #1581](https://github.com/link-assistant/formal-ai/pull/1581)
stays draft and unmerged. Evidence is under
`experiments/issue-1580/evidence/checks/terminal-native-and-version-parser/`
and `experiments/issue-1580/evidence/ci/native-families-27e895c1/`.
