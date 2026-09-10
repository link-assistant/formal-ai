# Plan 01 -- the ladder pays for the agent, not for copying the repository

Closes #1072; makes #1107 item 3's one remaining ladder cheap; fixes #1109.

## Root cause (measured, see plans/README.md)

110-minute run, 25 minutes of agent work. Per leaf: a 966 MB `git archive`
extract + `git init` + `git add .` + commit, then `cargo check --lib` and
`cargo test --test unit` in a path cargo has never seen, so a full crate
recompile every leaf. Plus unbounded agent turns.

## Steps

- [x] 1. **One workspace, reused.** `run.sh` creates a single
      `git worktree add --detach "$STAGE/work" HEAD` once. Between nodes:
      `git -C "$work" checkout -q -- . && git -C "$work" clean -qfdx
      -e .agent-ladder` then re-apply the node's fixture commit. Same path
      every node, so cargo's fingerprints hold and the second leaf's
      `cargo check` is incremental (seconds).
      - Why not a worktree per node (as #1072 first suggested): a new path per
        node is exactly what defeats the cargo cache.
      - The agent's project identity is the root commit. A worktree shares the
        repository's real root commit, so `@link-assistant/agent` reuses one
        snapshot store instead of minting one per node (#1072 §2, 31 GB).
- [x] 2. **Sparse checkout.** `git -C "$work" sparse-checkout set --no-cone
      '/*' '!/dev/' '!/docs/case-studies/' '!/docs/assets/'` before the first
      checkout. Leaves edit `src/`; verify-node compiles `src/` and runs
      `tests/unit`. Confirm `cargo check --lib` in the sparse tree passes before
      relying on it (build.rs / include_str! may read `data/`; `data/` stays).
- [x] 3. **Per-node budget.** Wrap the agent call in `timeout --kill-after=10
      "${LADDER_NODE_BUDGET:-240}"`; record `FAIL agent_timeout` in run.log.
      Worst case 32 x 4 min = 128 min, still inside the job's 180. Default 240 s
      is 10x the median leaf (22 s) and above every passing leaf measured.
- [x] 4. **Stream progress to the job log.** `run_one` prints one line per node
      (`node id start`, `node id PASS|FAIL reason elapsed`) to stdout, not only
      to run.log, so the Actions log is not blank for 90 minutes. This is what
      the maintainer's screenshot showed: a long step with nothing to read.
- [x] 5. **#1109 orphan sweep.** In `LinkCliLinkStore::open_at`, before
      acquiring the lock, remove sibling
      `.<name>.database.<pid>.<n>.tmp*` files whose `<pid>` is not alive
      (`kill(pid, 0)` via `libc`, or `/proc/<pid>` on Linux and
      `kill -0` semantics through `std::process::Command` where libc is not
      already a dependency -- check `Cargo.toml` first). Unit test: write a
      fake orphan with pid 4194304+1, open, assert gone; write one with our
      own pid, assert kept.
- [ ] 6. **Verify locally** with `TREE_DEPTH=5 NODE_FILTER=2.2.2.2.2` (one
      leaf) and then two leaves, timing `cargo check` on the second.
- [ ] 7. **Doc + changelog.** `experiments/issue_1028_agent_cli_ladder/README`
      (if present) and `changelog.d/` fragment naming the before/after minutes.

## Verification (CI)

- Ladder wall time on a 32-leaf run under 45 minutes (from 110).
- `leaf_nodes_passing` not below the record (15); plan 04 raises it.
- No new files under `~/.local/share/link-assistant-agent/snapshot/` per node
  beyond the first (assert count in run.sh; print it in the summary).

## If this goes wrong

Each step is independent. Step 1 is the one with correctness risk (a stale
file surviving between nodes): the `clean -fdx` plus a `git status --porcelain`
assertion before each node makes a dirty tree a hard failure, never a silent
one.

## Log

- 2026-09-09: steps 1-5 landed. Local two-leaf run on macOS exercised the
  worktree, sparse checkout, reset, budget and progress path end to end; the
  leaves themselves failed because the rename recipe emits GNU-only `sed -i`
  with `\b` (filed as #1110), so the incremental-`cargo` timing comes from the
  CI job log (verify-node now prints it). Two macOS portability fixes went into
  the harness on the way (`setsid`, BSD `sed -i`); `declare -A` still needs
  bash 4+, so on a Mac run it with Homebrew bash.
- Step 4 in `04-ladder-leaf-fixes.md` moved the cue check into a seed rule
  (`agentic_continuation`, new interpreter mode `whole`) after the kernel
  ratchet refused the Rust version: 112,824 lines against a 112,805 ceiling.

