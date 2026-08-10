# Issue #961 requirement traceability

| Requirement | Source | Implementation | Proof |
| --- | --- | --- | --- |
| R961-1 BSD-safe package log | Issue “What to do” 1 | `desktop/scripts/package-macos-with-retry.sh` | Source contract and `tests/unit/ci-cd/macos_package_retry.rs` |
| R961-2 canonical log expectation | Issue “What to do” 2 | `tests/issue_757_session_files.rs` | Symlink-alias integration fixture |
| R961-3 portable PTY tests | Issue “What to do” 3 | `tests/integration/pty.rs` and two migrated callers | Exact dialect argv tests, readiness-gated input, and both real integrations |
| R961-4 Bash 3.2 empty array | Issue “What to do” 4 | `scripts/sync-seed.sh` | Guard-order contract and sandboxed `--check` behavior |
| R961-5 macOS full-suite CI | Issue “How to test” | `.github/workflows/release.yml` | Matrix source contract and macOS Actions result |
| R961-6 standing evidence | Issue “Standing clauses” | `docs/case-studies/issue-961/` and `docs/case-studies/pull-request-987/` | Whole-task test and committed GitHub/session evidence |
