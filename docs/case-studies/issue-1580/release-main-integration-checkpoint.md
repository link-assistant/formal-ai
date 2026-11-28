# Released main integration checkpoint

Main releases v0.356.0 at f574289f while the preceding merge is under local
validation. The publication preflight refuses the stale ancestry before any
remote write. Merge that release and preserve the branch's security-backport
description in its own new fragment after main consumes the original fragment.

Before the merge commit, the first complete run records 87 passes, 53 explicit
skips and one failed secret-scan inventory: its committed-head diff names a
file already deleted in the pending merge. Preserve that fail-closed ENOENT.
Commit the reviewed merge after scanning every existing staged path, then rerun
the full inventory against that committed source. No scanner exclusion changes.

The complete governed witness changes only four SHA references to the
version-updated Cargo manifest. The reader, source manifest, all 221 case
objects and all 968 unsupported objects remain equal after substituting those
four reviewed hashes. The official versioned writer performs the refresh.

Unicode controls and all 158 maintained issue observations pass. A fresh
complete 141-gate run against released main records 88 passes, 53 explicit
hosted-only skips and zero failures. This supersedes the earlier partial
integration run, whose failures and successful retries remain retained.

No local native build runs. The next published head still needs hosted native
and complete JavaScript execution. Remaining warnings, full template/practice
comparison and wider issue acceptance stay open. Supplied work earns zero
independent-authorship or reasoning credit.

Evidence: experiments/issue-1580/evidence/checks/release-main-integration/.
PR: https://github.com/link-assistant/formal-ai/pull/1581
