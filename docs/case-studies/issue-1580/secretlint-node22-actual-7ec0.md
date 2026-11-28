# Actual Node 22 scanner qualification

[Security run 45218148807](https://github.com/link-assistant/formal-ai/actions/runs/45218148807)
succeeds on source 7ec084a7fc1b6a909b18a0c6ff9acf5b1f44449f.
Its actual locked-scanner job uses Node 22.23.5 and npm 10.9.9.
All seven maintained engine cases pass. Plain and compressed clean files are
accepted; plain and compressed findings are masked and refused; corrupt gzip
is refused. All five scanner cases execute the actual scanner and leave no
installation directory.

The repaired empty-cache mismatch control reaches npm's EUSAGE lockfile
validation and refuses before installation. This supersedes the previous
ENOTCACHED failure, preserved in the cold-cache repair receipt. The actual
locked audit reports zero vulnerabilities. This audit does not clear the
separate low OpenSSF dependency scores or the Cargo migration warnings.

Artifact 14838503659 contains ten payload members. Every member was verified,
and all seven recorded source digests match the committed source. The receipt
retains the original ZIP, complete job log, paginated jobs, artifact metadata,
runtime, all seven result records and every case diagnostic. Review covers
all result records and case logs, with targeted job-log lines 178-216; it does
not claim literal reading of the whole job log.

[The complete receipt](../../../experiments/issue-1580/evidence/checks/secretlint-node22-actual-7ec0-20261123/manifest.json)
qualifies these seven engine controls. The complete current cohort, remaining
template practices and original issue criteria are separate unfinished work.
[PR #1581](https://github.com/link-assistant/formal-ai/pull/1581) stays draft
and unmerged. Supplied work receives zero independent authorship credit.
