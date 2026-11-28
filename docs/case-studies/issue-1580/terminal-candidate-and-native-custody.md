# Terminal failures, request custody and exact JavaScript candidates

PR: https://github.com/link-assistant/formal-ai/pull/1581

Published `db19866f4383a62afd333f16077ca3522639f936` has 33 terminal workflows:
30 succeeded, two failed and one intentionally skipped. All 163 checks, 32
annotations and 33 complete workflow log streams are retained. The complete
streams were scanned and 88 selected diagnostic candidates plus the actual
failure contexts were reviewed; this is not a literal reading of all 85,164
Pipeline lines. The five failed check records have three underlying causes:

| Cause | Complete workflow log evidence | Local repair and verification |
| --- | --- | --- |
| Conservative eight-shard projection exceeds 360 seconds | Layered run 45032341706, lines 9286–9302 and 20686–20704 | Nine complete-suite shards retain every test, timing floor and execution boundary; all 27 scheduling controls pass. Hosted wall times remain pending. |
| Strict native target has no caller of `first_heading_offset` | Pipeline 45032342321, lines 65850–65861 | Actual UTF-8 byte-offset, CRLF and fence regression calls the helper. Rust formatting passes; hosted compilation remains pending. |
| Request phase tracing appends raw bodies without the payload-custody opt-in | Pipeline lines 77179–77182 and 77237–77241; isolated job 137240607723 lines 830–833 | The native formatter requires `FORMAL_AI_LOG_PAYLOAD_CUSTODY` before appending bodies. Two real wrapped-TUI cases require metadata in both modes, raw diagnostic bodies only after opt-in, private dialog capture in both modes and no body in the terminal. Native execution remains pending. |

The existing request flags enable phase metadata; they do not independently
authorize raw body custody. This preserves the explicit legacy payload mode and
complete private dialog files. The JavaScript logger already requires an
explicit payload option. No local native compiler or binary ran.

The pinned Rust template also gates native work on an exact checked JavaScript
candidate. Formal AI's fast source job now appends `tested-sha` only after its
checks, a 40-hex event identity, actual `HEAD`, a clean index/worktree and a
writable output are verified. The native caller requires that job to succeed
with the same SHA. Actual temporary Git repositories cover wrong/missing event
identity, staged/tracked/untracked changes, failed output writes and retained
prior output. Workflow predicates refuse missing or failed prerequisites and
wrong receipts. The regression fails before workflow integration and passes
afterward. This scoped gate does not qualify the wider JavaScript/native parity
practice. All 52 canonical operations, 104 bindings and staged-release bytes are
preserved by checked regeneration.

Six focused JavaScript files, canonical projection, logging policy and Rust
formatting pass. The first gate wrapper used two invalid names, and the actual
abbreviation gate found 18 names in newly retained evidence envelopes. Full-word
renames preserve every compressed payload and manifest object; corrected checks
pass. A source-only scan passes for 214 changed files, including 179 compressed
records, after moving the scan outside Formal AI's 60-second command boundary.
The original interrupted wrapper is retained. A work-plan warning is repaired
by preserving 940 historical lines byte exactly in a linked file; the active
plan becomes 496 lines and both size checks pass. No threshold is raised.

Remaining observed limits are explicit: old Layered shards two and seven cross
270 seconds; native cache writes report 11 failures without a backend diagnosis;
publish authority is unknown; the isolated issue-1017 dependency remains subject
to its existing allowed `unsafe-libyaml` maintenance warning. The temporary CI
diagnostic variable was restored to its originally absent state after all jobs
finished. Next-head diagnostic execution is still needed.

Evidence: [terminal manifest](../../../experiments/issue-1580/evidence/ci/db19866f-terminal/manifest.json) and
[local source, failures and verification manifest](../../../experiments/issue-1580/evidence/checks/exact-candidate-and-native-custody/manifest.json).
All wider applicable-template practices, production/runtime delivery, complete
aggregate review, trusted epochs, serving identity and genuine authorship remain
unfinished. Supplied code and controllers receive zero independent credit. This
PR remains draft and unmerged.


## Complete local inventory replay

The complete maintained local gate inventory at clean source `dd99013af518469481fb8072c4d964b1f59b3cf1`
finishes with **89 passes, 54 explicit CI-only exclusions and zero failures**.
The generated-file and status-renderer checks pass in this replay. The command
leaves the worktree clean and performs no native build. Earlier failed wrappers
and their corrections remain preserved; these results do not qualify excluded
native checks or any production release. See the [exact-source gate evidence](../../../experiments/issue-1580/evidence/checks/terminal-candidate-full-gates/manifest.json).
