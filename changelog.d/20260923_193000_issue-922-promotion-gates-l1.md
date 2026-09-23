---
bump: patch
---

### Changed
- The promotion-gate runners execute `cargo test` with
  `--manifest-path rust/Cargo.toml`, and `improve --promote` anchors gate
  replay to the compiled checkout root instead of the invocation
  directory. Plan 16 L1 moved the crate one level below the repository
  root, so every canonical gate command was failing to find a manifest
  and reporting `blocked:0/1`, which kept the issue #922 agent-CLI E2E
  lane red; the suite manifests, the fixed unit-specification command,
  and their pins now agree with the form `docs/benchmarks.md` already
  documented.
- The committed `data/seed/learned-methods.lino` is now the byte-exact
  product of the promotion protocol. The 2026-09-20 re-derive wrote the
  file with a trailing newline, but seed-edit generators trim their
  payloads (`adopted_seed_lino` ends in `trim_end`) and the agent that
  authors the materialized file echoes the task text without its final
  newline, so the protocol can only ever produce a file without one;
  the stray byte made the issue #922 provenance `cmp` fail by a single
  EOF.
- `rust/examples/issue-922-method-learning/run.sh` resolves
  repository-root-relative fixtures through a `REPO_ROOT` indirection
  (comparison target, stderr classifier), matching the post-L1 layout
  where the script's `ROOT` is the crate root.
- The external replay in that harness now passes
  `--no-summarize-session --compaction-models "(same)"`. The Agent CLI
  summarizes sessions by default through a hosted provider that rejects
  calls from outside its own client, so the run failed at teardown with
  an `UnhandledRejection` even though the file write through the local
  formal-ai server had succeeded; the flags keep every model call on the
  provider under test, matching the canonical
  `experiments/agent_cli_e2e/run_agent_cli.sh` invocation.
