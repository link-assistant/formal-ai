# Validation summary

All commands below were run after merging `origin/main` at `ac9d9252`. The
complete raw output is retained beside this summary; empty formatter logs mean
the command exited successfully without diagnostics.

| Gate | Result | Primary evidence |
|---|---|---|
| Exact-source `cargo test --all-features --verbose` | Pass: 166 integration, 481 auxiliary, and 1,937 unit tests; 2 intentional network/exhaustive ignores; 0 failures | `final-cargo-test-all-features-exact-source.log` |
| `cargo clippy --all-targets --all-features -- -D warnings` | Pass | `final-post-merge-clippy-third-run.log` |
| `cargo fmt --all -- --check` | Pass | `final-clippy-fixes-fmt-rerun.log` |
| `cargo test --doc --verbose` | Pass | `final-post-merge-doc-tests.log` |
| Optimized release build | Pass | `final-post-merge-release-build.log` |
| Crate package-size gate | Pass: 4,403,529 compressed bytes (4.19 MiB), below 10 MiB | `final-post-merge-crate-package-size.log` |
| File-size, terminology, source-placement, worker-budget, version, changelog, and release-map gates | Pass | `post-merge-*.log`, `full-*.log`, focused source-placement logs |
| WASM build/size and web bundle/i18n/intent/language gates | Pass | `post-merge-wasm-*.log`, `post-merge-web-*.log` |
| Focused issue #781 regressions | Pass | `final-clippy-fixes-issue-781.log` |
| Opt-in per-dialog logger regressions | Pass | `focused-dialog-log-external-tests.log` |
| Four native client CLIs against the deterministic MCP fixture | Pass for Agent, OpenCode, Claude Code, and Codex | `four-client-harness-final-2.log` |
| Four native clients using the exact post-merge release binary | Pass; Agent required one bounded fresh-session retry, retained in full | `four-client-post-merge.log` and `../real-cli/four-client-post-merge/` |
| GitHub Actions on code SHA `049b392d` | Pass: tests, coverage, both E2E jobs, strict lint/docs, and package build | `../ci-logs/ci-cd-pipeline-29715487352.log` and its 1,500-line chunk audit |

The four-client harness asserts more than process exit: every successful client
must perform one search, fetch three independent URLs in separately narrated
turns, and return a final Russian synthesis containing all three citations. A
client early exit therefore cannot be mistaken for success.
