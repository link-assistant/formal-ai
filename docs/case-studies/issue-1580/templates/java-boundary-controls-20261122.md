# Java manual quality checks and fragment preservation

Two earlier source observations now have bounded executable controls at unchanged
template source `dd803a2d9fa5b3a992b435acce73162274fb9e8b`. This adds evidence to
the active review; complete template semantic and runtime qualification remain open.

The copied 610-line workflow has Git blob
`15fbed88c68753227e4d12abbbc3eb3ad23b7bff` and SHA256
`0736b24479c96110729f681b95545ea15c8b565905ca3169c5b9a34a86b6672b`.
The test evaluates its actual guards with GitHub's documented default `success()`
rule. Manual detection is skipped, lint consequently skips, while the explicit
build guard accepts skipped lint. Both manual release guards can then accept a
successful build. This is an expression-model result, not an observed hosted
dispatch. Candidate lint/build guards require both quality jobs on dispatch;
five push/PR change flags and failure, skip and cancellation controls remain.

The unchanged complete collector CLI has Git blob
`659ecc112fe75039abd66af0ac61bdb9849c84ad` and SHA256
`afefed46692d3f56e122c13328810d7a26e7448b38656ba0c6fc0920ae41017a`.
Actual isolated Node processes show exit zero and fragment deletion for valid
CRLF, malformed and empty descriptions whose text is omitted. A mixed valid and
malformed batch loses the malformed input too. Disposable candidate copies
normalize CRLF and reject invalid descriptions before writes: valid LF/CRLF
notes survive in the changelog, while rejected batches preserve the exact input
bytes and original changelog. README preservation is checked in every case.

Run `node --test --test-concurrency=1 experiments/issue-1580/template-java-boundary-controls.test.mjs`.
All five cases pass. Restoring both original candidates with
`ISSUE_1580_JAVA_KEEP_ORIGINAL_DISPATCH_GUARD=1` and
`ISSUE_1580_JAVA_KEEP_ORIGINAL_FRAGMENT_READER=1` yields the expected two failures
and three passes. Eleven adjacent release-integrity, project-version and
inventory/source-identity cases also pass. Tests run sequentially; no Maven,
hosted dispatch or publication is executed. Full streams, source identities and
the two original script failures are retained in the accompanying evidence archive.

Formal AI already uses explicit status functions and successful native, build,
preflight and secret prerequisites in its relevant release guards. Its aggregate
correctly fails the current native compile error, which does not qualify a main
manual release. Its native collector reads a complete snapshot, validates that
every source is unchanged before removal and accepts CRLF/plain descriptions.
Java requires frontmatter and Formal AI permits plain fragments; preserve those
different formats. The Java reproduction does not establish Formal AI's behavior
for every mixed empty-fragment batch. Native execution remains hosted-only.

The report drafts are under `experiments/issue-1580/upstream/`. Publish immutable
executable evidence before filing. Supplied code and analysis earn zero independent
Formal AI authorship or accepted-reasoning credit.
