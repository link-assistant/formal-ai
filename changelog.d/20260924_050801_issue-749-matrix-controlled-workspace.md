---
bump: patch
---

### Fixed
- `issue_749_shell_routing::whole_shell_task_matrix_routes_without_web_search`
  failed only in CI (run 35919283963, job on tip 02a810b80): its "run the
  tests" row resolves through `formal-ai:workspace-test`, which picks the
  command from the first marker file in the process's working directory.
  The prebuilt binaries issue #1055 ships run from the repository root,
  where plan 16 L1's bun umbrella (`bun.lock`, `package.json`) wins and the
  answer is legitimately `bun test`; a local `cargo test` starts the same
  binary in `rust/`, where `Cargo.toml` wins and the answer is `cargo
  test`. The engine's cwd anchoring is the design; the matrix was the part
  making an implicit assumption. It now holds in a cargo workspace the test
  controls (a temp dir with a `Cargo.toml` marker, restored on drop), the
  same pattern its unit twin established when L1 landed. Verified by
  running the compiled integration binary from the repository root, the
  exact condition that failed.
