# Main 444575f2 regression investigation

This checkpoint accompanies [PR #1581](https://github.com/link-assistant/formal-ai/pull/1581)
for [issue #1580](https://github.com/link-assistant/formal-ai/issues/1580).
The merged default head is `444575f2ac28f609c94497865f125f99415ff6d5`.
All 16 push workflows on that head completed: thirteen succeeded and three
failed. The push-only paginated inventory avoids confusing skipped
`workflow_run` records with validation of the default head.

## Preserved failures and candidate repairs

The complete raw logs are compressed under
`experiments/issue-1580/evidence/ci/main-444575f2/`; indexes retain original
line numbers. Targeted excerpts are reviewed, not every line of every log.

| Run | Original log evidence | Candidate repair and verification |
| --- | --- | --- |
| [Agentic CLI Matrix 44163377753](https://github.com/link-assistant/formal-ai/actions/runs/44163377753) | Global-config/undo checks observe newly created empty `.formal-ai/logs/serve-*.lino` files (Gemini lines 3330–3335, OpenCode 5228–5233, Qwen 6785–6790). | Native startup probes directory writability with a private, exclusively created temporary file and removes it; the unchanged event writer opens the session log on its first event. A native regression preserves existing logs, checks the file-as-directory rejection, and verifies the first actual write. **Native execution remains pending.** |
| [Layered CI 44163377677](https://github.com/link-assistant/formal-ai/actions/runs/44163377677) | Offline workflow/cache fixture assertions fail at lines 19311/20005 and 22416/23174 after production read-only retrieval becomes default-on. | Offline fixture suites explicitly set and restore their network mode. Both files pass (2 and 6 tests). Default-on behavior, real HTTP reads, explicit denies, and permission guidance remain covered by 12 defaults and 36 permission cases. |
| [CI/CD Pipeline 44163378148](https://github.com/link-assistant/formal-ai/actions/runs/44163378148) | The canonical data parser rejects the backslash-escaped JSON scalar in `data/seed/client-integrations.lino:413` (log lines 76984–76987). | A single-quoted outer LiNo scalar preserves the decoded JSON value without treating inner JSON double quotes as LiNo delimiters. The maintained package mirror passes its 482-demand byte check; the native data-parser test still needs execution. |

Other current fixes include strict path-bound OpenCode Read envelope decoding,
request-local brief/line/language constraints, and the packaged native fixture
path. The six real client-output replays and 38 shared request-context fixtures
remain covered. The fixture uses its mirrored path inside the Rust package;
the canonical source and mirror remain byte equal.

The first complete local gate invocation reports four failures: six new
abbreviated feedback filenames, the repository-external fixture include, and
the same worker line overrun in both budget checks. Renaming immutable feedback
archives preserves their bytes. The worker drops one generic overview comment,
remaining within the original 1,375-line ceiling. Actual source fingerprints
and both generated projections are refreshed without weakening identity checks.
No executable worker code or budget ceiling changes in this step.

## Verification and limits

All twelve focused JavaScript files pass: 134 tests, zero failures or skips.
Full logs and hashes are in `final-focused-tests.json` and its referenced
archives under the checkpoint directory. Source formatting uses Rust edition
2024; package mirroring is current. The complete governed object comparison
visits 7,727 pairs, retains 221 supported and 964 unsupported cases, and finds
zero changes from the immediately preceding reviewed baseline.

The first all-gate run is retained as a failure. The final-source rerun has its
own receipt under `final/`; do not substitute the earlier invocation for it.
No local Rust build, native runtime execution, or independent Formal AI
authorship credit is claimed.

The native failure-header inventory identifies thirty distinct failed-test
headers plus five 1,080-second terminations (full shards 2, 3, 7 and 8, and
specification shard 4). Timeouts can conceal additional failures, so thirty
is not a complete failure count. These include older synthesis/benchmark
failures and new default-network/argument expectations. The earlier count of
38 refers to a different default head and must not be presented as this run's
complete count. All remain active investigation work; the PR stays draft.

The final source gate invocation completes with 86 passes, 52 explicit native/context exclusions and zero failures in 624.294 seconds, with a 256 MiB Node heap cap and cumulative child peak of 1,056,532 KiB. All 86 per-gate archives and the complete aggregate match their recorded hashes. Main advances to 058e9bef during verification (PRs #1600/#1602); preserve this atomic draft locally, merge the new default head and revalidate the combined tree before publishing. This is not native execution or final-head hosted success.
