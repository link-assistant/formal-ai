# Issue 1328: bounded CI failure collection

`node scripts/collect-ci-failures.mjs --conclusion failure --runs 4 --pages 1`
collects the latest four failed Actions runs from each of
`link-assistant/formal-ai` and `link-assistant/hive-mind`. Omit `--conclusion`
to inspect recent runs of every conclusion; failures, cancellations and skips
are separate. Repeated `--repo owner/repository` selects repositories.

The collector only invokes `gh api` GET requests. Authentication uses the
caller's existing GitHub CLI session. It performs no retries for authentication,
permission or missing-log responses. Transient failures have a finite retry
budget. Every numeric option must be a finite, bounded safe integer before IO.
Options are `runs`, `pages`, `jobs`, `logBytes` (retained bytes),
`logDownloadBytes` (transfer budget), `timeoutMs` and `retries`.
Defaults retain 64 KiB per log and transfer at most 2 MiB per log; hard maxima
are 100 runs, five pages, 30 jobs/run, 1 MiB retained, 8 MiB transferred,
60 seconds/request and two retries. Execution is sequential. Job lists are
explicitly marked truncated when their first bounded page excludes jobs.

Streaming retention preserves the actual beginning and end of a complete
response. `logsTruncated` marks omitted middle bytes; `logsTailComplete` is
true only when the complete response finished successfully.
`logsDownloadTruncated`, `logsTimedOut`, `logsUnavailable` and
`logBytesReceived` distinguish transport limits from retention limits.
An incomplete response does not prove an absent diagnostic. GitHub's redirect
ignored a suffix Range header in a separate probe; no suffix-range coverage
is claimed. API responses are also bounded and never silently paginated.

Diagnostics are candidate observations for human review, not automatic
root-cause declarations. `observedOutcome` records explicit no-change and
recovered-publication receipts separately; it never overrides the job conclusion. Fingerprints normalize UUIDs, timestamps, temporary
paths and commit identities, while retaining meaningful error codes and
argument names. Groups retain repository, run, head SHA, job and public URLs.
The known categories cover invalid action arguments, schema binding,
repetition/no progress, refspecs, absent delivery, unwritable logs and GitHub
integration permissions; other failures remain reviewable. Quoted agent
payloads and echoed script/configuration lines are excluded to avoid treating
an issue description or an instruction as an observed failure.

Output contains redacted diagnostics rather than raw logs. Known credentials,
email addresses, user home paths, IP addresses and account numbers are
redacted. Review output before sharing: arbitrary application-specific secret
formats cannot be inferred reliably. Do not commit raw logs, agent payloads,
account metadata or signed download URLs.

## Source-bound observations, 2026-10-20

The first bounded sample examined 12 recent runs/repository; it contained no
failed runs (many Formal AI runs were queued). A second sample selected the
latest four failed runs/repository. These are distinct sampling claims.

* Hive Mind at `8f22b87c515adece0427cb59e429531f35d7760a`:
  [draft run 39429701466/job 118651222463](https://github.com/link-assistant/hive-mind/actions/runs/39429701466/job/118651222463)
  and [draft run 39429262030/job 118649696385](https://github.com/link-assistant/hive-mind/actions/runs/39429262030/job/118649696385)
  both end with the same sequence: authenticated-user API returns HTTP 403,
  gist upload returns HTTP 403, repository fallback repeats the unsupported
  user lookup, and the runner repeats that first-error annotation. Both selected logs fit a
  1 MiB transfer budget. These are two directly observed occurrences of the
  same permission/capability diagnostic. Later receipts show that both runs successfully published their logs to the PR branch, then exited 5 with `Finished without changes`. The upload denial recovered; the absent source delivery is the terminal observed outcome. A first-error annotation alone must not be presented as its root cause.
  [Health job 118659320255](https://github.com/link-assistant/hive-mind/actions/runs/39432056518/job/118659320255)
  reports 13 preceding draft failures sharing that first error; that count is
  the health job's assertion, not 13 independently inspected logs.
* Formal AI at `48b21b39c4276a5d424fd768cd90b0a739e45d9d`:
  [self-hosting evidence job 118664411473](https://github.com/link-assistant/formal-ai/actions/runs/39433510662/job/118664411473)
  fails with `fatal: Not a valid object name origin/`, exit 128. The complete
  157,136-byte response was consumed with start/tail retention. This is a
  refspec/base-reference construction failure, not an unavailable artifact.
* [Reusable fixture consumer job 118671634700](https://github.com/link-assistant/formal-ai/actions/runs/39433546134/job/118671634700)
  uploads 552 bytes to blob storage, then artifact finalization returns
  non-retryable HTTP 404, `artifact not found`. The verifier's later missing
  `.release-fixture/collected.json` is a downstream symptom; the observation
  does not establish that the original producer failed to create content.
* [Native projection job 118664524916](https://github.com/link-assistant/formal-ai/actions/runs/39433543109/job/118664524916)
  fails in `validateProjectionSelector` with `ERR_ASSERTION`: requested
  pull-request projection source is unsupported without authenticated merge
  selection (`1329` versus an expected empty selector). Its later missing
  evidence upload is downstream of this explicit source-selection refusal.
* The initial bounded jobs request for Coverage run `39433507663` was
  unavailable. It is not counted as inspected or fixed.

These observations do not claim that every infrastructure failure is fixed.
The local collector makes repeated failure evidence reviewable without
changing either upstream repository, workflow permissions or tokens.

## Formal AI execution and authorship limits

The collection, repository bootstrap, checks and source-maintenance exercise
run through maintained JavaScript Formal AI `planChatStep`/`drive` tool calls.
The collector and its tests are supplied integration/bootstrap source, with
zero autonomous source-authorship credit. Bash execution receipts demonstrate
execution, not independent invention of that source.

Small fresh-module semantic requests (positive safe integer, identity,
constant and equality) initially stopped at `IndependentVerificationRequired`.
Supplying an independent identity test exposed a separate driver refusal:
`unowned-create-request`. The general driver repair permits only exact
maintained coding-research prerequisite reads of its declared module/test;
it grants no source-write authority and checks every subsequent plan against
both maintained producers. Missing research/search capabilities remain honest
gaps. Source maintenance can genuinely derive trailing-whitespace edits from
observed source; its successful execution must not be presented as autonomous
authorship of the collector algorithm.

The source-maintenance leaf completed with actual Read, Edit, Read receipts
and `verified: true` for trailing whitespace after an import statement.
Whitespace inside a comment was first refused as `SourceMaintenanceProtectedLexeme`;
that protection remains intact. Independent identity source creation now reads
the declared module and acceptance test, then reports `SearchToolMissing`
without creating a module or claiming completion.
