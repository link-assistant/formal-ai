# Committed primary lockfile checkpoint

The Rust template's complete `scripts/check-cargo-lock.rs` requires a binary
lockfile to exist in the HEAD commit. Formal AI's fixed binary crate lives in
`rust/`, and its audit wrapper checked only the Git index. A newly staged
`rust/Cargo.lock` therefore passed even when HEAD did not contain it.

The regression creates a finite temporary Git repository, commits an independent
lockfile, stages the primary lock without committing it, and runs the actual
production shell wrapper with a recording Cargo executable. Before the repair,
the command exits zero and invokes Cargo; the regression fails. The wrapper now
checks `git cat-file -e HEAD:rust/Cargo.lock` before invoking any Cargo command.
The unchanged regression passes with exit one and no Cargo invocations.

All 11 inventory controls pass, including unusual path bytes, failed/malformed
inventories, untracked and absent primary locks, and collection of independent
audit failures. The adjacent blocked-advisory fixture previously mocked only
`git ls-files`; its first rerun fails at the new HEAD guard. It now makes a real
temporary commit and delegates other Git operations to the actual executable.
All eight existing blocked-proof controls pass with their assertions intact.
The three additional advisory-proof and eight repository-audit contract controls
pass, and the shell lint gate passes. No local Rust build, dependency resolution
or online advisory audit is performed.

The complete local gate inventory subsequently passes: 89 applicable gates,
53 explicit hosted/Rust exclusions, and no failed gate in 595.86 seconds.
Its complete summary and the latest detailed logs for all 89 gates are preserved.
Six detailed logs were overwritten by the subsequent passing packaging checks;
the manifest distinguishes those later observations from the other 83 original
outputs. The first archive attempt used an incorrect log directory and failed;
its diagnostic is retained. Skipped gates remain unqualified.

The inventory regression is registered in the optional local feedback inventory.
Hosted pull requests and main already discover it through their full default
JavaScript test glob.
The template applicability decision records focused local verification and
pending hosted audit qualification separately. This repairs the observed
committed-lock gap; complete template and issue acceptance remain unfinished.
All supplied scripts and conclusions receive zero independent-authorship credit.

Evidence: [manifest](../../../experiments/issue-1580/evidence/checks/committed-primary-lock/manifest.json).
Pull request: https://github.com/link-assistant/formal-ai/pull/1581.
