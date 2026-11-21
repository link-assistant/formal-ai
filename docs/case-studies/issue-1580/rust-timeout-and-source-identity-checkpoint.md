# Rust timeout documentation and source identity

The current Rust template README says every test-matrix job is capped at ten
minutes. Its parsed workflow instead gives the test job a thirty-minute
backstop and the test command a 900-second (fifteen-minute) internal budget.
Two maintained checks verify that mismatch against exact Git blobs and check
a disposable wording correction. The portable per-test Cargo limitation stays
explicit. No Cargo command or upstream workflow was executed for this check.
The reproduced drift is reported in
[Rust template issue #256](https://github.com/link-foundation/rust-ai-driven-development-pipeline-template/issues/256).
An upstream report and wording candidate do not establish an upstream repair.

The investigation also found a false source identity in our review ledger:
its recorded Rust commit had 41 hexadecimal characters. Fresh Git commit and
recursive-tree observations verify the actual 40-character commit
`5f1e7a96acbf7a93b567195443cf6e5158fbbf1f` and all twelve previously recorded
changed-file blobs. Their content identities match. The ledger now uses that
verified commit and preserves the old string as correction evidence. Historical
API responses are retained unchanged, including responses requested with the
invalid ref; those responses alone did not verify a valid commit identity.

A maintained regression fails on the original ledger and passes after the
correction. It checks complete commit syntax across fourteen review ledgers
and the timeout fixture. This does not assert that every ledger's contents or
every current and historical template source has received complete review.

All 133 cases in the maintained template-observation entry pass without skips.
The naming, file-size, tests-as-documentation and whitespace checks pass. The
preceding worker extraction separately passes all 141 local gate entries
(88 passes and 53 explicit skips) and removes four source-size warnings; 87
warnings remain under unchanged thresholds. Final worker commit and previous
publication receipts are also retained with this checkpoint.

Raw before/fixed observations, API identities, prior issue discussion, submitted
report, validation streams and helper sources are compressed with verified
original and archive SHA256 identities in
`experiments/issue-1580/evidence/checks/rust-timeout-and-source-identities/`.
Supplied implementation and commands receive zero independent authorship or
reasoning credit. Full template/practice, original budget/live, complete source
warning, security child semantics and complete issue acceptance remain open.
PR #1581 remains draft and unmerged.
