## What happened

Scala run 2026-09-15 (backend 0.351.0, PR https://github.com/konard/test-hello-world-019fb330-00e1-73b9-955e-f357a1600d5b/pull/2, comment 5677731588):

```
Step `scalac Main.scala` for `Main.scala` failed. No exit code was reported…
/bin/sh: 1: scalac: not found
```

The session ended there; Hive Mind restarted it and got the identical result. The Hive Mind dind image has `kotlinc`, `javac`, `cargo` but no Scala toolchain. A regular LLM in the same container installs one (`curl -fL https://github.com/coursier/launchers/raw/master/cs-x86_64-pc-linux.gz | gzip -d > cs && ./cs setup -y`, or `apt-get install scala`, or falls back to `scala-cli`) and continues; the catalog already knows the install commands for each language (`rust/src/coding/catalog/languages.rs` records e.g. the Laravel/PHP setup notes), so the knowledge exists but is not used as an executable step.

## What already exists (verified on `main` d209aac64)

Plan 06 of #1138 is implemented as `rust/src/prerequisite/` (`probe.rs` recognises `command not found` via `NOT_FOUND_MARKER`, `publisher.rs` finds the official setup procedure, `install.rs` installs under a default-deny `InstallGrant` inside the workspace and re-probes, `ledger.rs` records it), and `data/seed/setup-publishers.lino` already pins official publishers for `rustc`, `python3`, `node`, `tsc`, `go`, `gcc`, `g++`, `java`, `javac`, `dotnet`, `ruby`, **`scalac`**, **`kotlinc`**, `php`, `docker`. The only caller of `install`/`publisher`/`ledger` outside the module is `rust/src/external_benchmarks/grade.rs:18-21` (Python for benchmarks). The agentic executor that Hive Mind drives (`agentic_coding/general_execution.rs`, `command_reroute.rs:67-84`, which finalizes on the first failed step) never calls it, which is why the Scala run stopped at `scalac: not found`.

## Requirements (what to do)

1. Wire prerequisite recovery into the agentic executor: when a step's output carries `NOT_FOUND_MARKER` (or the probe for the recipe's language returns absent before the first run), plan the pinned publisher's procedure (`publisher.rs`), run it through the client's shell tool under the grant, re-probe, and retry the step once. The recovery is planned as ordinary tool calls, so it works through Claude Code, Codex and the Agent CLI alike.
2. Grant model for agent mode: `formal-ai serve --agent-mode` gets an explicit, logged grant setting (`FORMAL_AI_INSTALL_GRANT=workspace` or a CLI flag), default-deny preserved; Hive Mind sets it for task containers (file the Hive Mind side in link-assistant/hive-mind#2324 R-list when wiring).
3. Prefer a self-contained runner the publisher documents when it avoids installing a compiler (`scala-cli`, `kotlinc -script`), recorded as the chosen procedure with its source.
4. The workflow the recipe writes sets up the same toolchain it verified with (one setup step per program, taken from the same publisher record, versions from #1168 (E133)).
5. The answer reports the recovery: which program was missing, which publisher procedure ran, the re-probe result.

## How to test

- Unit: a replayed Scala transcript whose first `scalac` step returns `/bin/sh: 1: scalac: not found` plans the pinned `scalac` publisher procedure next, then the re-probe, then the original step (`rust/tests/unit/issue_1159_toolchain_bootstrap.rs`).
- Unit: without a grant the plan stops with a clear "grant required" answer that names the missing program and the procedure it would run.
- Manual: the Hive Mind Scala row passes on a task image without Scala.

## Definition of done

- [ ] Requirement shard `docs/requirements/issue-1159-toolchain-bootstrap.md` with each requirement of this issue; `rust-script scripts/assemble-requirements.rs --write` regenerates `REQUIREMENTS.md`.
- [ ] Row(s) in `docs/requirements-traceability.md`: delivered (version/commit), automated test `rust/tests/unit/issue_1159_toolchain_bootstrap.rs` (registered in `rust/tests/unit/mod.rs`), manual confirmation (the Hive Mind run below).
- [ ] Case study `docs/case-studies/issue-1159/` with the 2026-09-27 log excerpt, the root cause and the fix.
- [ ] Changelog fragment `changelog.d/<YYYYMMDD_HHMMSS>_issue-1159-toolchain-bootstrap.md`.
- [ ] `RUSTUP_TOOLCHAIN=1.98.1 cargo test --test unit issue_1159_toolchain_bootstrap` and `RUSTUP_TOOLCHAIN=1.98.1 rust-script scripts/run-ci-gates.rs --stage rust` green; the fix is released (check `git merge-base --is-ancestor <commit> $(git describe --tags --abbrev=0)`).
- [ ] Manual confirmation: link-assistant/hive-mind's Hello World end-to-end matrix (link-assistant/hive-mind#2324 R5) row **formal-ai × agent (Scala)** passes on the released version, linked here.

## Depends on / blocks

- Depends on: #1163 (E128) for discovering the install command from the publisher's page instead of a stored one; plan 06 (`docs/case-studies/issue-1138/plans/06-prerequisite-discovery.md`).
- Blocks: the Scala row whenever `scalac` is absent from the task image.




