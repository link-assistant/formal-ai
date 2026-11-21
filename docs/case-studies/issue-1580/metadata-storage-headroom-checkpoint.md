# Maintained metadata storage headroom

The rename map and release-ledger archive exceeded the unchanged 1,400-line
warning threshold. The existing include loader now reads the complete rename
body from a named shard. All rename fields, rule order, resort entries and
existing exclusions are unchanged; the sole added exclusion names the shard.
The original map is exactly reconstructible from its container and shard.

The release archive is split only between complete records. Concatenating its
record bodies reconstructs the original archive exactly, and the expanded
ledger retains all records, order and inherited defaults. Both writers rotate
new data at 1,400 lines while retaining compatibility with legacy 1,500-line
archives. Read bounds, canonical names, symlink rejection and exclusive writes
remain enforced. New boundary regressions fail before and pass after repair;
the existing storage suite and complete maintained issue suite pass.

The full 141-entry gate replay found a stale generated status line census.
The maintained status generator updated just the root-ledger line count; the
failed replay and successful focused replay are both retained. Source scanning
reports 44 warnings, zero violations, with unchanged thresholds. Native Rust
format checks pass; native execution remains for hosted CI.

Evidence: `experiments/issue-1580/evidence/checks/metadata-storage-headroom/`.
Supplied procedures receive zero independent authorship credit. Wider issue
work remains unfinished, and PR #1581 stays draft and unmerged:
https://github.com/link-assistant/formal-ai/pull/1581.
