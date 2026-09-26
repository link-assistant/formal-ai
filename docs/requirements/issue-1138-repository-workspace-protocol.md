## Issue #1138 Repository Workspace Protocol

Repository authoring uses one default-deny protocol, covered by
`rust/tests/unit/issue_1138_repository_workspace.rs` and
`rust/tests/unit/issue_1138_locate_targets.rs`.

| ID | Requirement | Status / evidence |
| --- | --- | --- |
| R1138-3-1 | A repository task carries an origin and an exact base commit; a branch name is refused as a base. | Implemented by the repository-workspace request parser; covered by `rust/tests/unit/issue_1138_repository_workspace.rs`. |
| R1138-3-2 | Named targets are located by repository census or literal/path occurrence; ambiguity resolves to no target. | Implemented by `rust/src/repository_workspace/locate.rs`; covered by `rust/tests/unit/issue_1138_locate_targets.rs`. |
| R1138-3-3 | Named tests run and record command, exit code, and output before an obligation can be satisfied. | Implemented by the verification stage; covered by `rust/tests/unit/issue_1138_named_tests.rs` and `rust/tests/unit/issue_1138_repository_workspace.rs`. |
| R1138-3-4 | The offered patch is a unified diff computed from the tree and verified against the exact base. | Implemented by the workspace outcome; covered by `rust/tests/unit/issue_1138_repository_workspace.rs`. |
| R1138-3-5 | SWE-bench, the coding ladder, and self-authoring consume the same protocol document. | Partial: SWE-bench and the coding ladder enter `WorkspaceProtocol`; `scripts/author-change-with-formal-ai.sh` still owns a separate Agent-CLI authoring loop, so the third caller is not unified yet. Covered by `rust/tests/unit/issue_1138_repository_workspace.rs` and `rust/tests/unit/issue_848_coding_ladder.rs`. |
| R1138-3-6 | Commands are default-deny and allowed by program, subcommand, and argument shape from seed data. | Implemented by the repository command allowlist; covered by `rust/tests/unit/issue_1138_command_allowlist.rs`. |
| R1138-3-7 | Authoring refuses commits by default; an allowed commit carries all self-hosting trailers and exact model evidence. | Partial: `run_solve` is default-deny and its produced commit payload passes the canonical attribution parser, but the legacy authoring shell has not yet been reduced to that entry point. Covered by `rust/tests/unit/issue_1138_solve_cli.rs` and `rust/tests/unit/specification/self_hosting_metric/solve_attribution.rs`. |
| R1138-3-8 | A missing prerequisite remains an evidenced unsatisfied need, never a pass or skip. | Implemented by the protocol outcome ledger; covered by `rust/tests/unit/issue_1138_repository_workspace.rs`. |
| R1138-3-9 | The protocol document is forgettable and regenerates to the same content id. | Covered by `rust/tests/unit/specification/repository_workspace_protocol.rs`. |
