TASK (tag REPO-PROTO): deliver R1138-3-5 (docs/requirements/issue-1138-repository-workspace-protocol.md): "SWE-bench, the coding ladder, and self-authoring consume the same protocol document."

State:
- SWE-bench and the coding ladder enter `WorkspaceProtocol` (`rust/src/repository_workspace/`).
- Self-authoring runs `formal-ai solve --produces …`, which branches into `crate::authoring_loop::run_authoring` (`rust/src/cli_solve.rs`). That is a separate loop that never reads the protocol document.
- The wrapper `scripts/author-change-with-formal-ai.sh` is already a thin `exec` of `solve`, and it refuses to commit without `--commit` (R1138-3-7, round 14).

Do:
1. Make `run_authoring` load `WorkspaceProtocol` and drive its stages: open, locate, edit by the Agent CLI session, verify the artifact contract, outcome, commit gate. Where the authoring loop has a stage the protocol lacks (spawning `formal-ai serve`, harvesting the Agent CLI session id), add it to the protocol document as data, not as a branch in code.
2. Make `repository-protocol.lino` evidence the same for all three callers, and pin that with a test that runs each caller against fake executables (see `rust/tests/unit/ci-cd/authoring_effects.rs`) and compares the stage names they record.
3. JS first: if a JS twin of the protocol exists (`js/server`, `js/agentic`), keep it in step. If none exists, record that in the row as the remaining parity gap instead of claiming parity.
4. Update R1138-3-5 honestly. Ledger rows: T170–T179.

No cargo locally. `rust/src` files stay ≤1000 lines. `rustfmt --edition 2024 --check` must pass on every changed file.
