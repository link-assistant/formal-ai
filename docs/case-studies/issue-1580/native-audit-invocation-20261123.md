# Native audit invocation contract

The [17a native admission job](https://github.com/link-assistant/formal-ai/actions/runs/45103300864/job/137459248016)
compiled successfully, then failed one of 251 workflow controls. Original job
lines 2090–2093 and 2162–2169 identify the audit source selector: it still
searched for a bare `cargo audit` line after the real call moved behind
`run_audit`. The wrapper was introduced to reject zero-exit incomplete registry
observations; both actual current-head Cargo audits complete successfully.

The native test now selects the actual `run_audit cargo audit` invocation. Its
existing `--deny warnings` and no-`--no-yanked` assertions remain unchanged.
Primary and independent audit policies, production code and all suite inventories
remain unchanged. Sixteen actual-wrapper inventory controls, eight blocked-proof
controls and three advisory-proof controls pass sequentially. Formatting and
source selection are checked without local Rust execution. Next-head native
execution remains required.

The [manifest](../../../experiments/issue-1580/evidence/checks/native-audit-invocation-20261123/manifest.json)
retains the complete original failed job, all local streams, before/after native
source and actual audit wrapper. Every archive member round trips exactly.
Supplied work receives zero independent credit and full issue acceptance remains
unfinished.
