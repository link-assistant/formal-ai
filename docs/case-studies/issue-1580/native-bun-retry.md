# Verified Bun setup for native transport replays

The focused native job at commit `a558f834` compiled successfully, then its
single Bun 1.4.2 release download returned HTTP 403. Run 44999045010, job
137136426355, raw log line 2427 records that failure. Consequently, Bun-dependent
transport replays did not execute and cannot be counted as verified.

The job now uses the existing `setup-bun-verified` composite already exercised by
the agent ladder. It retries an installer that supplies no executable exactly
once, then checks that Bun runs and matches the committed exact version. The
caller retains the same compile/lint admission and pin, and a final installation
failure still fails the job. Downstream replays still require successful Bun
setup. The repair adds no alternate download source or floating version.

A focused wiring regression fails before the change. The seven installer tests
pass, including missing, wrong and exact executable versions; all 162 native
admission tests pass in a sequential full-file check. The first full-file attempt
hit its existing 100ms expression deadline while local probe wrappers overlapped;
that failure is retained and the deadline is unchanged. The complete original
job log, failing control and both validation attempts are content-addressed in
`experiments/issue-1580/evidence/checks/native-bun-retry/manifest.json`.

Hosted recovery from this download failure remains unverified until the next
head runs. The wider issue acceptance remains unfinished in
[PR #1581](https://github.com/link-assistant/formal-ai/pull/1581).
