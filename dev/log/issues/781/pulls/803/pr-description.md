## Summary

Fixes #781 by making multi-source research observable, sequential, provenance-preserving, and portable across Agent, OpenCode, Claude Code, and Codex.

- emits concise localized narration before every search/fetch action and a cited synthesis after the research finishes;
- plans one research action per turn, retains each successful fetch under its own URL, and performs bounded follow-up research without loops;
- discovers Responses MCP namespace children and returns the exact namespace/name envelope Codex needs to dispatch them;
- normalizes known client wrappers only at the planner boundary while retaining raw tool output for auditing;
- adds opt-in exact per-dialog request/response JSONL through `FORMAL_AI_DIALOG_LOG_DIR` (off by default);
- connects normalized browser captures to shared-dialog conversion and adds a generic Links-based option/evidence network;
- runs the exact Russian issue prompt through all four real client CLIs in the release E2E workflow.

## Root causes

The reported stall was a chain of independent defects: batched research calls had no visible narration; open-world marketplace requests could be classified as local code search; Responses namespace children were hidden from the shared planner or re-emitted without their namespace; and Codex transport wrappers could be mistaken for source text. The evidence bundle contains a red/green reproduction for each boundary.

## Reproduction and tests

Before this fix, the minimum regression retained one source instead of the three returned by search, protocol responses could begin with a silent tool batch, and the exact issue prompt could select `grep`. The new regressions require one narrated search, three separately narrated fetch turns, and a final response containing all successful evidence and citations.

Verified locally with:

```text
cargo fmt --all -- --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test --all-features --verbose
cargo test --doc --verbose
cargo doc --all-features --no-deps (including docs.rs cfg/profile)
WASM build/size, web bundle/i18n/intent/language, changelog/version, file-size, and worker-budget gates
experiments/agent_cli_e2e/run_issue_781.sh (Agent + OpenCode + Claude + Codex)
```

GitHub Actions run [29715487352](https://github.com/link-assistant/formal-ai/actions/runs/29715487352)
passed the pushed code SHA, including tests, coverage, strict lint/docs, both
E2E jobs, release build, and crate package-size validation.

The deterministic four-client run records the following complete paths:

| Client | Search | Fetch | Server turns | Result |
|---|---:|---:|---:|---|
| Agent | 1 | 3 | 5 | cited synthesis |
| OpenCode | 1 | 3 | 6 | cited synthesis |
| Claude Code | 1 | 3 | 5 | cited synthesis |
| Codex | 1 | 3 | 5 | cited synthesis |

Each final client directory contains `client.log`, `formal-ai.log`, and the exact per-dialog JSONL. The CI fixture is deterministic; separate native-network captures preserve real provider, authentication, and anti-bot behavior without making the regression depend on mutable search results.

The exact post-merge release binary passed the four-client harness again.
OpenCode, Claude, and Codex completed cleanly; Agent exercised its documented
early-exit condition once and then passed via the bounded fresh-session retry.
Both attempts remain in the aggregate server/dialog evidence (2 searches,
6 fetches, 9 turns), so the client variability is visible rather than discarded.

## Evidence and safety

The complete investigation is under `dev/log/issues/781/pulls/803/`: raw issue/PR/CI data, validated screenshots, the reconstructed timeline, all 38 requirements, 15 root causes, online research, red/green logs, and four-client transcripts. The implementation does not claim current Amazon inventory or electrical compatibility where a seller/manufacturer page could not be fetched; the case-study recommendation explicitly distinguishes verified specifications, indexed leads, and conditional fallbacks.

Reported issue state:

![Reported empty/error state](https://github.com/link-assistant/formal-ai/blob/issue-781-cc22afbacef0/dev/log/issues/781/pulls/803/github/issue-comment-5016917806-error.png?raw=true)

![Reported stalled research state](https://github.com/link-assistant/formal-ai/blob/issue-781-cc22afbacef0/dev/log/issues/781/pulls/803/github/issue-comment-5016917806-stuck.png?raw=true)

Related client behavior is documented upstream with reproductions, workarounds,
and code-level suggestions: [Codex namespace routing](https://github.com/openai/codex/issues/14242#issuecomment-5018114084)
and [Agent's unknown-finish early exit](https://github.com/link-assistant/agent/issues/194#issuecomment-5018466226).
The evidence bundle also retains [OpenCode's resolved AI SDK finish-reason regression](https://github.com/anomalyco/opencode/issues/20465)
as a corroborating failure class without misattributing it as this PR's cause.
