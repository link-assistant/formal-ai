# Current main integration and warning repair checkpoint

This checkpoint merges main `97fe40bc1027f8b37238416bfb099e064935e218`
without rewriting either parent's history. Issue #1580 remains unfinished and
[PR #1581](https://github.com/link-assistant/formal-ai/pull/1581) stays draft.

## Reproduction and changes

The complete initial merge gate run exposed four failures: translation ledger
totals, an unresolved Git-index duplication in notation input, stale rendered
status, and the genuinely stale compiled worker. The index was resolved; the
status writer and the translation ledger's official rendering functions restore
currency. Every existing translation source hash, item and workaround identity
is checked; no new translation is claimed. A bounded upstream translation retry
exhausted its 256 MiB heap. That failed experiment is retained and was not repeated.

Main's client replay exposed the interaction between its concise OpenCode policy
and the retained output constraints. All report choices and descriptions now fit
two lines. Same-language validation uses the actual seeded language registry,
preserving explicit unsupported-language refusal and script-leak detection.
The JavaScript, TypeScript and native implementations retain equivalent behavior.

The VSIX release protocol must capture both `rust/tests/e2e` and
`packages/minimatch-compat` before importing selected source. Conflict resolution
initially expanded a different checkout. The actual VSIX checkout now retains
both main's smoke dependencies and the strict artifact adapter. The regression
executes a real sparse checkout and the real capture script: before it fails at
`install: cannot stat .../rust/tests/e2e/.npmrc`; afterward all captured bytes
match. Removing either smoke directory still fails.

## Verification and limits

- Request-context checks pass. Client report replay passes 13 JavaScript checks;
  the native replay is explicitly skipped locally and retained in hosted CI.
- Desktop artifact layout passes all five tests, including the before/after
  sparse capture regression and missing-directory controls.
- Four touched Rust files pass `rustfmt --check`. No local Rust build ran.
- All 141 local gate entries finish: 87 pass, 53 are hosted-only exclusions,
  and the compiled-worker currency gate remains a genuine failure. Full logs
  retain each individual verdict. The next hosted worker dispatch must compile
  the exact pushed source before its receipt can be accepted.
- Official baseline regeneration preserves all 221 governed case identities
  and 968 unsupported cases; a complete structural comparison finds no change.
- Whitespace review against main passes outside the exact preserved session log.
  Parent-relative checks flag inherited issue-1609 and issue-1580 session logs;
  equality to main or the published PR parent is verified for each excluded
  historical file. Those observations are preserved rather than rewritten.

The [evidence manifest](../../../experiments/issue-1580/evidence/checks/main-warning-merge/manifest.json)
binds the initial failures, final checks and actual sparse-checkout regression.
These are local results, with zero independent authorship credit. Final-source
hosted behavior, remaining template practice mapping and the contribution
authorship floor remain open.

## Later main integration

Main advanced to `9669945ad320dc8a96ebb5a88cfe05dbafb2f030` before the push gate. Its embeddings API, seed translations, release version and native replay are retained. The independent-replay regression initially failed (`9 !== 8`); the expected inventory now includes the new replay, and its admission condition allows it to run after sibling failures while still requiring successful native lint and a noncancelled run. The exact before/after logs are retained.

The focused policy, embeddings and server JavaScript checks pass, touched native files pass rustfmt, and the complete gate run records 87 passes, 53 hosted exclusions and the single stale-worker failure. Complete structural review of the governed baseline finds only four Cargo fixture hash changes, matching the retained release version change from 0.356.1 to 0.356.2; all other leaves are equal. Source whitespace checks preserve exact, byte-identical historical session logs. Hosted compilation is still required for the new source.
