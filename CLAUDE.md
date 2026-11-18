# Claude Code instructions

Read [AGENTS.md](AGENTS.md) first. It holds the rules for every AI agent in
this repository, including the single push gate from issue #1191:

- drafting sub-agents never push;
- one CI/CD agent pushes once, after collecting every failure of the latest run;
- never push while a run is in progress on the branch;
- run the JavaScript checks locally before the push;
- no full local Rust build unless the change is Rust-only, and then only with a
  bounded shared `CARGO_TARGET_DIR`, `CARGO_INCREMENTAL=0`, a limited `-j` and
  `scripts/prune-build-cache.sh`.

[CONTRIBUTING.md](CONTRIBUTING.md) applies in full.
