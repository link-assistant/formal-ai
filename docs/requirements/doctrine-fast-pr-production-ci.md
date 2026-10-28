## Standing Doctrine: Fast pull-request checks and production verification

The architect's 2026-10-28 instruction supersedes earlier requirements that
made every integration, platform build or live client test mandatory on each PR.

- **CI-FAST-1:** Use one bulk-drafted PR for related fixes, tests, documentation,
  vision and guidelines. Commit the complete batch before pushing; collect and
  fix all available CI failures before the next push. Do not wait idly on CI.
- **CI-FAST-2:** Default PR checks run on Linux and prioritize coding, Hello World,
  source edits, question answering, issue reporting, retry bounds and offline
  failure regressions. Keep lint, workflow/security checks, requirement consistency
  and JavaScript/TypeScript checks. Production native builds, packaging, Docker,
  broad benchmark ladders and heavy integration/E2E jobs are retained but deferred
  to main. Set FORMAL_AI_FULL_PR_CI=run to restore the retained PR jobs, including
  branch-dispatched checks. Existing narrower platform/coverage opt-ins still apply.
- **CI-FAST-3:** Main retains native tests, integrations, final production builds,
  signed/self-signed package verification where advertised, Docker publication and
  public-download verification. A green PR proves the selected fast profile only;
  delivery is complete only after the resulting main CI and configured release
  producers/consumers are green. Diagnose main failures and fix them promptly in
  a follow-up PR. Never label skipped suites as executed, passing or equivalent.
- **CI-FAST-4:** Real GitHub E2E effects are disabled by default. Enable a deliberate
  run with FORMAL_AI_REAL_GITHUB_E2E=true or the real-github-e2e dispatch input.
  Preserve immutable per-PR/per-release ownership, attempt bounds, first/repeat
  oracles, independent verification, cleanup and complete redacted evidence.
  Never rerun an old effect owner to make it green. Failure does not authorize
  blind mutation retries. Offline emulation can run frequently with no GitHub
  effects. Its coverage of live authentication, network and runtimes is limited.
- **CI-FAST-5:** Turn complete public CI logs and actual bot-created test records
  into small provenance-labelled unit fixtures. Cover the observed failure and
  held-out negative cases, not just a successful transcript. Retain every original
  integration/E2E test for opt-in execution. Do not claim that captured data
  establishes an unobserved live solve or full unit/E2E equivalence.
- **CI-FAST-6:** Close generated test issues after a verified test outcome through
  the trusted finalizer. Historical stale issues may be retired as not_planned
  with an explicit unverified comment only after exact bot/body/label/custody and
  inactivity checks. Never mark an unverified test completed or grant feature
  delivery credit. Revalidate immediately before mutation, stop on uncertain
  writes and preserve receipts; never touch human/modified/active issues.
- **CI-FAST-7:** Scheduled observations declared by authenticated workflow source
  do not invalidate an otherwise checked release. Required commit checks and
  publication obligations remain mandatory; unknown workflows, pending/failed
  push/manual checks, foreign sources and missing required checks still refuse.
- **CI-FAST-8:** On the workstation, run only a single necessary JavaScript test
  at a time. Never run Rust builds, tests, clippy, native products or local Docker
  builds. Keep finite time/transfer budgets, avoid broad filesystem scans and
  preserve disk space. Production verification belongs to CI.

Evidence for the initial regressions: generated-e2e fixtures capture 43 exact
public bot test issues and the v0.355.14 Agent claim failure caused by a late
scheduled task-ladder run. Retiring those issues is housekeeping with unverified
outcomes; it is not evidence that their coding tasks passed. Supplied literal
Formal AI edit recipes receive zero autonomous implementation credit.
