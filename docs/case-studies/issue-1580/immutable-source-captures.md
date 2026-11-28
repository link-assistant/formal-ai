# Complete historical capture conservation

Eight historical source captures exceeded the existing maintained-source warning
thresholds. They are now gzip archives beside their former paths. The consumed
fixture `experiments/issue-1580/fixtures/immutable-source-captures.json` records
every original logical path, complete byte count, line count and SHA-256 digest.
`gzip -dc <original-path>.gz` recovers the exact original bytes.

The focused conservation regression fails before archival and passes afterward.
It verifies all eight complete contents against the pre-change digests. The
archive and original bytes were also compared directly during the change.
No source-size limit, warning threshold or historical transcript was weakened.

Exact-name and parent-directory searches found historical transcript/index
references for these captures. One historical verifier refers to a former
`/private/tmp/` workspace, rather than consuming the retained repository file;
its original script is retained unchanged. The canonical dormant staged workflow
has live consumers, so it remains a separate, unfinished size warning.

These transport changes earn zero independent Formal AI authorship credit. The
wider issue is unfinished. Pull request:
https://github.com/link-assistant/formal-ai/pull/1581.
