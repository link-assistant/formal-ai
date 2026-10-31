## Standing Doctrine: Fast pull-request checks and production verification

The architect's 2026-10-28 instruction supersedes earlier requirements that
made every integration, platform build or live client test mandatory on each PR.

The 2026-10-29 documentation-only exception is defined in
[issue #1455 requirements](issue-1455-documentation-and-evidence.md): automatic
docs-only pushes/PRs run lightweight documentation checks and produce no release
by default. Mixed code changes, schedules and explicit manual runs retain their
existing verification obligations.

- **CI-FAST-1:** Use one bulk-drafted PR for related fixes, tests, documentation,
  vision and guidelines. Commit the complete batch before pushing; collect and
  fix all available CI failures before the next push. Do not wait idly on CI.
- **CI-FAST-2:** Default PR checks run on Linux and prioritize coding, Hello World,
  source edits, question answering, issue reporting, retry bounds and offline
  failure regressions. Keep lint, workflow/security checks, requirement consistency
  and JavaScript/TypeScript checks. The fast lint job uses maintained JavaScript
  checks; the original native/WASM lint gates stay on main or explicit main-only qualification.
  Production native builds, packaging, Docker,
  broad benchmark ladders and heavy integration/E2E jobs are retained but deferred
  to main. FORMAL_AI_FULL_PR_CI cannot reenable PR builds or branch-dispatched
  builds. Existing narrower platform/coverage opt-ins apply only on main.
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

- **CI-FAST-9:** Archive discovery may consider a unique platform-neutral ZIP
  from an authenticated canonical publisher release catalog. It remains only a
  candidate: reject ambiguity, incompatible explicit platform/architecture,
  missing digest, ungranted effects and failed actual probes. A generic archive
  mentioned nowhere in unauthenticated context remains refused. Select runner
  command grammar from observed help, never a hardcoded release version. Retain
  full bounded selection/probe diagnostics; no compiler execution occurs locally.

- **CI-FAST-10:** Canonical package data and every demanded rust/embedded byte
  mirror must agree before merge. Regenerate through the existing JavaScript
  mirror-package-data generator and commit its outputs; never manually edit
  generated mirrors or defer this cheap parity check to production compilation.
- **CI-FAST-11:** Visual replay/font/GIF export integration tests are retained
  but default-off with FORMAL_AI_FULL_TUI_REPLAY=run or full agent replay opt-in.
  Their explicit finite test deadlines include export overhead. Keep ordinary
  readiness, literal typing and PTY checks enabled; use real captured startup
  frames in cheap unit tests. A skipped export test grants no visual acceptance.
- **CI-FAST-12:** Keep the implementation, captured-log regressions, requirement
  register, vision, contributor guidelines and handoff in the same working PR.
  Before merge, verify all applicable checks against its exact current head.
  After merge, follow that source through main, version preparation, configured
  release consumers and public artifact verification. Record the source commit,
  actual version, workflow outcomes and immutable artifact identities. A previous
  release, a skipped job or an unrelated green run cannot satisfy this delivery.
- **CI-FAST-13:** Reuse the digest-verified same-run complete source-history
  artifact for native, desktop and extension consumers. Keep their independently
  pinned workflow protocol separate. Advertise detached selected history during
  release-tag fetch negotiation so existing objects are not downloaded again.
  Preserve complete unfiltered source bytes, published tag/default-branch
  ancestry checks, original finite budgets and failure policy. Capture actual
  failed transfers as small unit fixtures, including detached-head and advancing
  default-branch controls. Stage packaging console evidence only after packaging
  starts; missing evidence after an attempted package operation remains a failure.
  An unacquired hosted runner has no execution credit: retain its structured
  annotations and explicit unavailable-console outcome instead of inventing logs.
- **CI-FAST-14:** Desktop runtime packages contain the prepared renderer, engine,
  browser and actual production dependencies. Exclude the private repository
  package and its source/build outputs from dependency collection. Audit ASAR
  metadata before installer creation, including unpacked entries, so recursive
  source inclusion fails early. Replay captured packaging paths in fast unit
  tests and check exclusions with the installed packager in hosted packaging.
  Never mask oversized-payload failures with generic retries or higher limits.
  Refresh reviewed complete source witnesses after referenced configuration
  changes; fast source-binding checks must detect stale embedded bytes before
  main, while preserving full reader and complete-object parity checks.

## Default-branch build amendment (2026-10-31)

- **CI-FAST-13:** No PR or feature-branch check dispatch may compile Cargo,
  Docker, WASM or application products, or deploy Pages, even with the old full-PR
  override. Production consumers must independently select authorized main source.
- **CI-FAST-14:** Remove redundant compilation from package verification after
  the actual compiler succeeds. Retain the cold independent Docker build as a
  main-only opt-in; production image publication depends on its actual native
  producer. Keep immutable source, byte, producer and attempt checks.
- **CI-FAST-15:** PR change/version admission uses JavaScript without installing
  Rust tooling. Combine selected JavaScript unit suites in one runner. Preserve
  measured main sharding and optimization where needed; run no Rust locally.
- **CI-FAST-16:** Browser E2E and box-language integrations are retained but
  default-off; restore explicitly on main. Preserve critical coding/QA/reporting
  coverage and log-derived unit regressions without claiming E2E equivalence.

See [the operational policy](../ci-cd/default-branch-builds.md), including the
existing non-atomic cross-service publication boundary. A green PR is fast-source
qualification; complete production delivery still requires all release consumers.

- **CI-FAST-17:** Source-fixture changes must refresh complete governed objects,
  including cached source text, through the reviewed reader. Preserve case IDs,
  assertions and unchanged native-source hashes. Fast PR checks must compare the
  complete objects as well as fixture digests; matching hashes alone do not
  establish consistency of the program representation.

- **CI-FAST-18:** Derive the conservative PR timeout maximum and selected workflow
  from current definitions, retaining the existing ceiling and fail-closed checks.
  Do not maintain a synchronized measurement counter. Resolve npm aliases to
  their actual publisher for currency checks and preserve alias syntax during
  updates; local packages have no registry currency, while their declared
  registry dependencies remain checked. Capture observed failures in shared
  fast JavaScript and native gate fixtures.
