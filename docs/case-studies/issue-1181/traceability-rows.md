# Issue #1181 Traceability Rows

Rows for the shard
`docs/requirements/issue-1181-cli-binaries-and-env-switches.md`, kept here for
the maintainer to merge into `docs/requirements-traceability.md` (the shared
table is not edited by this change). Honesty rules follow the table's own
header: `not yet confirmed` means no manual confirmation has been recorded for
this shard.

| ID | Shard | Delivered | Automated test | Manual confirmation |
| --- | --- | --- | --- | --- |
| R1181-1 | docs/requirements/issue-1181-cli-binaries-and-env-switches.md | 2026-09-30 (this pull request); observable on the next tagged release | workflow: the `cli` job's smoke test runs `formal-ai --version` from each leg's own archive; no automated assertion exists until a release carries the assets | not yet confirmed |
| R1181-2 | docs/requirements/issue-1181-cli-binaries-and-env-switches.md | 2026-09-30 (this pull request); observable on the next tagged release | workflow: "Package the CLI archive" + "Smoke test CLI archive" steps (exact `--locked` build command, contents asserted on the runner) | not yet confirmed |
| R1181-3 | docs/requirements/issue-1181-cli-binaries-and-env-switches.md | 2026-09-30 (this pull request); observable on the next tagged release | workflow: `finalize` fragment merge + `builder_of`/label-loop coverage with the existing INCOMPLETE reporting | not yet confirmed |
| R1181-4 | docs/requirements/issue-1181-cli-binaries-and-env-switches.md | 2026-09-30 (this pull request) | reviewed: `[package.metadata.binstall]` in rust/Cargo.toml; `cargo binstall` only resolvable once R1181-1 assets exist | not yet confirmed |
| R1181-5 | docs/requirements/issue-1181-cli-binaries-and-env-switches.md | 2026-09-30 (this pull request) | reviewed: `install_cli_prebuilt`/`Install-CliPrebuilt` reuse the existing checksum-verified download helpers; install.ps1 not machine-parsed (no PowerShell on the implementing workstation) | not yet confirmed |
| R1181-6 | docs/requirements/issue-1181-cli-binaries-and-env-switches.md | 2026-09-30 (this pull request) | rust/tests/unit/issue_1181_boolean_env_parser.rs | not yet confirmed |
| R1181-7 | docs/requirements/issue-1181-cli-binaries-and-env-switches.md | 2026-09-30 (this pull request) | rust/tests/unit/issue_1181_boolean_env_parser.rs (`silent_switch_now_accepts_one_as_the_issue_reported`, `bool_env_value_parser_maps_spellings_to_clap_results`) | not yet confirmed |
