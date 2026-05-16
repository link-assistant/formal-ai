---
bump: patch
---

### Fixed
- Issue #84: `scripts/publish-crate.rs` now classifies crates.io HTTP 429 rate-limit responses ("You have published too many versions of this crate in the last 24 hours") as `publish_result=rate_limited` with a dedicated banner explaining that `scripts/check-release-needed.rs` will automatically retry on the next push to `main` once the 24-hour throttle window has rolled over, instead of surfacing the failure as the misleading "Failed to publish for unknown reason"

### Added
- `FailureKind` enum and `classify_failure` helper in `scripts/publish-crate.rs` with unit tests covering rate-limit, already-uploaded, already-exists, missing-token, unauthorized, and unknown-failure responses
- `docs/case-studies/issue-84/` case study capturing the failing CI runs, root cause, upstream template comparison and reproduction commands
