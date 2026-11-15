---
bump: patch
---

### Fixed
- Hive original delivery no longer aborts unstarted clients when another CI run is still pending (#1485). An unclaimed client leg now rechecks a bounded number of times. If a prerequisite is still pending, the leg writes a `deferred-before-effects` receipt (`formal-ai-hive-client-outcome/v1`) with its identity, the blocking run IDs and statuses, a timestamp, proof that nothing was claimed, and zero execution and authorship credit, then exits without claiming anything.
- Verification, ledger and acceptance jobs read that typed outcome instead of looking for claim or result artifacts that were never produced. A deferred-only run is reported as deferred and incomplete, never as accepted. A failed, cancelled, skipped or timed-out prerequisite, or a changed source, is a refusal. A claim held by this run is an interrupted leg, handled by the existing cleanup and never retried.
- A published candidate resumes on the blocking run's completion event through a `hive-wakeup-<run>-<role>` artifact. The resumed run re-admits only the same immutable candidate and allows at most eight continuations. The existing claim ledger still decides which clients run, so a claimed leg is never executed again.
