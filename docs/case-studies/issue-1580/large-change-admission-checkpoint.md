# Complete Git change admission

The current published Coverage job 136873976099 fails at full-job log line 1503
with spawnSync git ENOBUFS; Layered CI job 136873976315 fails at line 1495 with
the same error. Git's valid NUL path inventory exceeds the admission helper's
4 MiB child stdout bound. A failure cannot safely authorize an empty diff.

Spool the complete inventory to a private temporary file, decode 32 KiB chunks
with a stateful UTF-8 decoder, and retain every NUL-delimited path. The temporary
file is removed on success and failure. Existing Git process/stderr bounds and
failed-command propagation remain intact. Optional FORMAL_AI_CI_VERBOSE emits
only inventory byte/path totals, never untrusted path text.

The real-Git regression constructs 24,004 bare-tree paths without materializing
them in a worktree. The original command fails with ENOBUFS; the fixed command
preserves the exact complete set, including spaces, newline and Unicode, and
selects the final Rust path for native admission. Pull-request and dispatch
paths both pass; a missing comparison ref remains a terminal failure. This case
is imported by the existing default-branch policy suite. Hosted qualification
of the repaired inventory is pending the next published source.

Supplied implementation earns zero independent authorship/reasoning credit. The full issue remains unfinished.

PR: https://github.com/link-assistant/formal-ai/pull/1581
