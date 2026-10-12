## Issue #961 macOS CI Parity

Issue [#961](https://github.com/link-assistant/formal-ai/issues/961) groups four
portability failures discovered by a complete macOS test run. Linux behavior
was green, but the macOS package wrapper, a canonical-path expectation, two PTY
test launchers, and the seed synchronization check relied on GNU/Linux behavior.
The audit, requirements, root causes, alternatives, tests-first reproduction,
and live self-hosting evidence are in `docs/case-studies/issue-961/`.

| ID | Requirement | Status / Evidence |
| --- | --- | --- |
| R961-1 | The macOS packaging retry wrapper creates a unique BSD-compatible log path. | Delivered: `formal-ai-macos-package.log.XXXXXX` ends in the placeholder BSD `mktemp` replaces; the source contract and existing parallel packaging tests pin it: `rust/tests/issue_961_macos_portability.rs`::`package_log_uses_a_bsd_portable_mktemp_template`. |
| R961-2 | Session diagnostics compare canonical proxy-log paths. | Delivered: `issue_757_session_files` writes through a symlink alias, canonicalizes the created file, and asserts the same value as the product; `rust/tests/issue_961_macos_portability.rs`::`proxy_log_expectation_matches_the_canonicalized_product_path` keeps that assertion in place. |
| R961-3 | PTY integration tests run with BSD and util-linux `script(1)`. | Delivered: Shared `rust/tests/integration/pty.rs` selects exact platform argv, safely shell-quotes the util-linux command, waits for readiness, and keeps stdin open until `script` exits; both affected tests call it; `rust/tests/issue_961_macos_portability.rs` (`pty_tests_do_not_embed_util_linux_only_script_flags`, `pty_input_waits_for_tui_readiness_before_writing`) pins the dialect choice and readiness wait. |
| R961-4 | Seed sync handles an empty destination under Bash 3.2 nounset semantics. | Delivered: `sync-seed.sh` guards the array expansion by length; the issue test checks source order and runs an empty-destination sandbox through `/bin/bash`: `rust/tests/issue_961_macos_portability.rs` (`seed_sync_guards_an_empty_destination_array`, `seed_sync_reaches_the_orphan_pass_with_an_empty_destination`). |
| R961-5 | The full test suite runs on a supported macOS CI runner. | Delivered: `.github/workflows/release.yml` runs the same test job on `ubuntu-latest` and `macos-15-intel`, with platform-specific measured budgets; `free-runner-disk.sh` uses POSIX `df -Pk` so the macOS job reaches Cargo. Pinned by `rust/tests/issue_961_macos_portability.rs` (`full_test_matrix_runs_on_a_supported_macos_image`, `macos_test_bootstrap_uses_portable_df_syntax`, `runner_disk_cleanup_accepts_a_bsd_shaped_df`). |
| R961-6 | Every requirement and their composition remain falsifiable. | Delivered: `rust/tests/issue_961_macos_portability.rs` contains per-requirement and whole-task contracts; issue and PR case studies preserve the original audit and final evidence. |
