---
bump: patch
---

### Fixed
- Issue #84: `scripts/publish-crate.rs` now treats crates.io HTTP 429 rate-limit responses ("You have published too many versions of this crate in the last 24 hours") as a deferred `publish_result=rate_limited` outcome instead of a hard CI failure, with a dedicated banner explaining that `scripts/check-release-needed.rs` will automatically retry on the next push to `main` once the 24-hour throttle window has rolled over
- Issue #84: release workflow artifact steps now wait for either an already-published crate or `publish_result=success`, preventing Docker Hub publishing and GitHub release creation when crates.io has rejected the crate upload

### Added
- `FailureKind` enum and `classify_failure` helper in `scripts/publish-crate.rs` with unit tests covering rate-limit, already-uploaded, already-exists, missing-token, unauthorized, and unknown-failure responses
- `docs/case-studies/issue-84/` case study capturing the failing CI runs, the follow-up `rate_limited` exit-code failure, root cause, upstream template comparison and reproduction commands
