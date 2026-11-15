# Instructions for AI agents working on formal-ai

This file is for any automated contributor, such as Claude Code, Codex, the
hive-mind `solve` command or Formal AI itself. `CLAUDE.md` points here.
[CONTRIBUTING.md](CONTRIBUTING.md) still applies in full. This file restates
only the rules an agent breaks most often, because each one has cost real CI
time ([issue #1191](https://github.com/link-assistant/formal-ai/issues/1191)).

## The single push gate

A push starts several workflows at once. Each push that only fixes part of the
previous run's failures pays for the whole matrix again. Pull request #1188
needed 100 commits and 11 `CI/CD Pipeline` runs, and none of them was green.
The rules below make every push count:

1. **Drafting sub-agents never push.** A sub-agent that drafts code, tests or
   documentation commits locally (or hands its diff back) and stops there.
   Only the coordinating agent touches the remote.
2. **One CI/CD agent pushes, once per round.** Exactly one agent owns pushes to
   the branch. It pushes only after it has collected every actionable failure
   of the latest run on the current head, and fixed all of them in the same
   batch.
3. **Never push while a run is in progress on the branch.** Check first:

   ```bash
   gh run list --branch "$(git branch --show-current)" --status in_progress --json databaseId,workflowName
   gh run list --branch "$(git branch --show-current)" --status queued --json databaseId,workflowName
   ```

   Both lists must be empty, or must hold only runs for an older head that you
   are deliberately superseding. Otherwise, keep drafting while the run
   finishes, then collect its failures (rule 2).
4. **Run the JavaScript checks locally before the push.** Two kinds of check
   apply:
   - the test files next to the change, run one at a time:
     `timeout 600 node --test --test-concurrency=1 <file>`;
   - the gates the change touches. These run in JavaScript and never compile
     Rust: `node experiments/formal_ai_subagent/local-gates.mjs --only <gate>`.

   `--list` shows every gate. Run the whole set
   (`node experiments/formal_ai_subagent/local-gates.mjs`) before a push that
   touches shared data such as `data/meta`. The fast pull-request lane runs the
   same twins, so a failure here would have failed there too.
5. **No full local Rust build unless the change is Rust-only.** CI compiles
   Rust: the focused `native-type-check` job on pull requests, and the full
   matrix on main. Locally, run `rustfmt --check <file>` on the files you
   changed. That command needs no build.
6. **If a change is Rust-only and you must build locally, keep the build
   bounded.** Use one shared target directory for every worktree and agent,
   with no incremental cache, few jobs, and a prune afterwards:

   ```bash
   export CARGO_TARGET_DIR="$HOME/.cache/formal-ai-target"  # one directory, shared by all worktrees
   export CARGO_INCREMENTAL=0                               # no incremental cache to grow without bound
   export CARGO_BUILD_JOBS=2                                # a limited -j for plain cargo commands
   CARGO_TEST_JOBS=2 scripts/cargo-test.sh --test unit <filter>  # one module, -j 2, then prunes
   scripts/prune-build-cache.sh                             # prunes old artifacts after any other cargo command
   ```

   `scripts/prune-build-cache.sh` follows `CARGO_TARGET_DIR`. It keeps only
   the latest build's artifacts, and by default it caps the tree at 4 GB
   locally (`CARGO_TARGET_MAX_SIZE_MB`).

## Where things live

- Production-only failures are recorded in
  [`data/meta/main-only-regressions.lino`](data/meta/main-only-regressions.lino).
  These are failures that main recorded after a green pull request. When main
  fails after a green pull request, add the failure there together with the
  narrowest pre-merge check that replays it.
- The fast pull-request JavaScript inventory is
  [`data/benchmarks/fast-pull-request-javascript-tests.json`](data/benchmarks/fast-pull-request-javascript-tests.json).
- Runner labels are pinned, and `node scripts/check-runner-labels.mjs` rejects
  `-latest` labels.
- Experiments go in `./experiments`. There is no top-level `examples/`
  directory; Rust examples live in `rust/examples/`.
- Never commit agent scratch state such as `.claude/`.
