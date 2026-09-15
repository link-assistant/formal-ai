## Issue #1137 Pre-Merge Four-Client Routing Replay

Issue [#1137](https://github.com/link-assistant/formal-ai/issues/1137) records
that Agent, OpenCode, Claude, and Codex expose different tool vocabularies, so a
single-client or held-out-only pull-request gate cannot prove a routing change
before it reaches `main`.

| ID | Requirement | Status / Evidence |
| --- | --- | --- |
| R1137-1 | A pull request that changes agentic routing must run the full real-client replay before merge. | Implemented: `scripts/detect-code-changes.rs` derives `agentic-routing-changed` from the complete PR diff whenever a tracked path under `src/agentic_coding/` changes; `.github/workflows/release.yml` passes `full-replay: true` to the reusable Agent CLI workflow for that PR. The path classifier and caller expression are pinned by `detect_code_changes::tests::agentic_source_changes_request_the_four_client_replay` and `ci_cd::issue_1137_agentic_routing_replay`. |
| R1137-2 | The pre-merge replay must retain Agent, OpenCode, Claude, and Codex and prove search, fetch, and cited synthesis. | Implemented: the full-replay-only `run_issue_781.sh` step retains the four-client default and its per-client search/fetch/final assertions; `the_full_replay_still_exercises_each_supported_client` pins the caller, harness, and client inventory. |
| R1137-3 | Pull requests outside the routing boundary should retain the cheaper held-out gate. | Implemented: the new detector output is false outside `src/agentic_coding/`; the existing `main`, schedule, and manual full-replay conditions remain unchanged. The classifier regression exercises unrelated coding and documentation paths. |
