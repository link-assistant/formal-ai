# File-size checks preserve incomplete observations

An actual one-byte invalid UTF-8 `src/broken.rs` made the JavaScript production
gate print its clean result and exit 0 after a read warning. Its directory walker
also discarded an unavailable root. The matching Rust checker discarded both
errors through the same success condition. These are failed observations rather
than evidence that the file is within its limit.

The JavaScript and Rust checks now retain read and walk errors, report the path
and reason on stderr, continue collecting findings, and return a failed result.
Excluded directory components and non-source formats retain their existing
boundaries. Rust prunes the same directories outside the Git inventory as its
JavaScript twin before observing walk errors.

A separate physical checkout under `target/checkout` exposed an absolute-path
exclusion: both oversized `src/retarget.rs` and `.github/scripts/tool.rs` were
hidden. The JavaScript regression failed before the repair with exit 0.
Both checkers now apply exclusions to paths relative to the scanned checkout.
The template already incorporates this practice after
[issue 217](https://github.com/link-foundation/rust-ai-driven-development-pipeline-template/issues/217).

## Reproduction and automated checks

Create `src/broken.rs` containing the single byte `ff`, then run the repository's
`scripts/check-file-size.mjs` by absolute path from the fixture directory.
The fixed checker exits 1 with the invalid UTF-8 error and no clean result.
The maintained regression also checks an unavailable root, exact preserved
bytes, excluded/unlisted/binary controls, and a checkout beneath `target/`.

Run `node --test --test-concurrency=1 rust/tests/web/scripts-js-twins-checks.test.mjs`.
The native counterparts live in `scripts/check-file-size/tests.rs` and are
imported by the existing focused `workflow_contracts` target as well as the full
unit target. Native execution of these new cases remains pending hosted CI;
no local native build was performed.

## Upstream observation and limits

The complete current template helper at `260ae6f826d6bff0cbfe2280098c287a4c09b3f4` still drops
read and walk errors. It is reported in
[issue 262](https://github.com/link-foundation/rust-ai-driven-development-pipeline-template/issues/262).
That report is derived from upstream source and the downstream JavaScript
reproduction; it does not claim an executed upstream native reproduction.

The retained check records include the first invalid validation command names
and a later shell-wrapper interruption, followed by corrected background checks.
Supplied patches and controllers receive zero independent authorship credit.
Issue 1580 remains unfinished.
