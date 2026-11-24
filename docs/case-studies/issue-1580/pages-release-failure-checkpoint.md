# Pages after a publication failure

The pinned Rust template's complete issue-38 history exposes a current counterpart: the deployed release caller requires successful package publication before building Pages. A validated main build therefore loses its documentation update when the publisher fails. Removing that predicate alone is insufficient: the reusable Pages build checks out the current branch, so an advancing main branch can reject the fallback triggering commit at the existing identity guard.

The maintained staged generator now projects the two narrow condition changes into the deployed caller and candidate. Pages still requires a successful build, a non-cancelled run and an authorised main push or instant manual release. Existing release dependencies and successful release-child resolution remain intact. This repair waits for the release jobs to finish; it does not claim independent scheduling. All 52 historical stage operations and 104 bindings remain unchanged.

The reusable workflow selects between two trusted GitHub contexts. A successful publisher's child keeps the trusted branch checkout and existing exact identity check. The fallback checks out the immutable triggering commit. Neither checkout executes an input-selected ref, both disable persisted credentials, and the original equality guard rejects a different commit.

Reproduce the failure by evaluating the actual caller condition with a successful main build and a failed automatic or instant manual publisher. The new regression initially records five failures: four failed/skipped publisher contexts and the fallback checkout selector. The other eight controls pass before the fix. Afterward all 13 cases pass, including an actual temporary Git repository running the unchanged identity-check shell against triggering and child commits.

Validation is sequential and uses a 256 MiB JavaScript heap:

- The focused regression passes 13 cases; the existing Pages admission entrypoint passes all 64, including those 13. The trust suite separately passes five cases.
- Existing staged activation, maintained authority, native operation-view and production-profile files pass 3, 5, 15 and 18 cases respectively. The generated deployed projection is current.
- All seven touched JavaScript gates and whitespace checks pass. No Rust or application build runs locally.
- Preserve the initial admission failure: its old textual shell extractor included adjacent YAML comments as shell code. Parsing the named step with the existing YAML parser repairs the extractor without changing workflow assertions. Preserve the invalid gate-name invocation too; no gates ran in that attempt. The corrected inventory passes.

Hosted execution of this new fallback, actual Pages server acceptance and served source identity remain unverified. Earlier artifact upload or native assertions cannot establish those outcomes. Full eight-template comparison, remaining applicable practices, original native contracts, dependencies, live delivery and semantic macro coverage remain open in [PR #1581](https://github.com/link-assistant/formal-ai/pull/1581). Independently accepted Formal AI reasoning remains 0/20; supplied commands, scripts and conclusions receive zero credit.

Evidence: [complete original controls, failed attempts and passing outputs](../../../experiments/issue-1580/evidence/checks/pages-release-failure/supplied-investigation.json), [before/after source identities](../../../experiments/issue-1580/evidence/checks/pages-release-failure/source-identities.json), and [automated regression](../../../experiments/issue-1580/pages-release-failure.test.mjs).
