# Template and Hive Mind comparison

The snapshot directories record exact upstream revisions and complete CI trees,
so this audit is reproducible rather than a comparison against moving `main`.

## Practices already present and retained

Least-privilege token defaults, per-job timeouts, concurrency cancellation,
locked/retried Rust tooling installation, change detection, formatting,
Clippy/tests/docs with warnings denied, package-size checks, release
attestations, checksum validation, release idempotency, and `!cancelled()` on
fan-in paths. Formal AI is ahead of the templates on rustdoc enforcement,
artifact provenance, and explicit top-level permissions.

## Practices applied for issue 798

- Warning-as-error coverage now includes direct rustc and every desktop target.
- Warning allowances are narrow and documented instead of output-filtered.
- Packaging warnings are fixed at their source by bundling.
- `Cargo.lock` synchronization is checked with `cargo metadata --locked` before
  the test matrix, and both automatic and manual release paths install and run
  the exact artifact that crates.io serves after publication.
- Opt-in diagnostics remain available through
  `FORMAL_AI_MACOS_SIGN_DEBUG=1` and
  `INSTALL_NODE_DEPENDENCIES_VERBOSE=1`; defaults remain quiet.
- Regression tests exercise policy text and executable signer behavior.

## Reviewed differences not copied blindly

- A fresh-merge helper is unnecessary here: GitHub's `pull_request` checkout
  already tests `refs/pull/<n>/merge`; adding a second synthetic merge would be
  redundant and can diverge when the event merge SHA changes.
- The release workflow's `always()` sites intentionally upload failure evidence
  or heal a release created by an earlier job. Replacing them mechanically would
  lose diagnostics; cancellation is already guarded on job fan-in paths.
- A repository-wide default Bash shell would change Windows matrix semantics.
  Desktop Release already scopes Bash where its scripts require it.
- The monolithic release workflow exceeds the advisory 1,500-line guideline,
  but splitting security-sensitive publish jobs during an incident fix would
  expand risk without addressing either cited failure. Its jobs all have
  explicit timeouts and are covered by workflow source tests.
- Action SHA pinning is desirable, but none of the requested templates supplies
  a maintained update mechanism and all use moving major tags. Converting every
  action without automated update ownership would quickly create stale security
  pins; this is recorded rather than partially applied.
- The JavaScript template's secret scan is an unpinned `npx --yes` install. The
  maintained gitleaks action now requires a separate license for organization
  repositories, which this workflow does not have. Copying either would add a
  mutable execution path or a guaranteed CI failure, so neither is presented as
  a working security control. The gitleaks tag/API and licensing evidence is
  preserved under `github/` for a future licensed rollout.
