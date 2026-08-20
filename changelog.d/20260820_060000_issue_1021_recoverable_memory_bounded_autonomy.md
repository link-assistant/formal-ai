---
bump: minor
---

### Added

- Added versioned recoverable memory (#946): a candidate version is written
  against a byte-for-byte snapshot and a digest-pinned baseline, and a version
  that fails to compile, fails a baseline specification, or edits the baseline
  it is judged against is rolled back to the last one that passed.
- Added bounded autonomy with a stuck-recovery limit (#947): the recovery loop
  reads an injected clock, stops after its limit -- one hour by default -- and
  asks with the plan it accumulated, and keeps per-command permission and full
  trust as separate opt-ins so delegating commands is not delegating choices.
