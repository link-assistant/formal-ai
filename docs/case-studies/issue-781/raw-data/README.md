# Raw-data inventory

All captures were made on 2026-07-19 UTC unless their embedded metadata says otherwise.

- `issue*.json`, `issue-timeline.json`: complete issue API records and comments.
- `pr*.json`: PR, conversation, review-comment, and review API records.
- `chatgpt-share.html`: original supplied share HTML.
- `chatgpt-web-capture.json`: normalized `web-capture shared-dialog` result (35 turns).
- `chatgpt-direct.demo-memory.lino` and `chatgpt-adapter.demo-memory.lino`: byte-identical Formal AI exports.
- `google-ai-mode-static.html`: static Google challenge response.
- `google-ai-mode-browser.json`: browser fallback diagnostic with no transcript.
- `amazon-*-browser.html`: the two exact issue URLs plus the newly found Tonton candidate captured through Chromium; all contain Amazon's automated-access notice.
- `ubuy-tonton-browser.*`: independent live capture attempt for an exact-ASIN mirror of the Tonton listing.
- `shopee-browser.html`: browser attempt for model-specific connector corroboration; the product application remained a loading shell.
- `*-browser.log`, `playwright-install.log`: capture execution evidence.
- `web-search-*.json`: current project-native search attempts when provider results were available.
- `reproduction-before-fix.log`: failing one-source regression.
- `npm-ci-baseline.log`: unrelated baseline lockfile/runtime setup failure retained for reproducibility.
- `../agent-cli-evidence/parallel-*`: Agent CLI 0.25.0 executing the three-fetch
  plan, followed by the reproducible upstream `unknown` finish-reason exit.
- `../agent-cli-evidence/recursive-*`: the one-fetch-per-round control
  experiment, which exits at the same boundary and rules out parallel fan-out
  as the cause.

The successful CI harness overwrites `../agent-cli-evidence/formal-ai.log` and
`../agent-cli-evidence/agent-cli.log` with the final release-binary replay. It
asserts the live search, all three server-side fetch plans, and all three Agent
CLI fetch executions. The deterministic Rust whole-task regression covers the
subsequent synthesis and exact URL citations because of the documented Agent
CLI transport limitation.

The reopened issue's complete PR #803 evidence is intentionally stored under
`dev/log/issues/781/pulls/803/`. Its `real-cli/four-client-final-2/` directory
contains native Agent, OpenCode, Claude, and Codex client logs, Formal AI server
logs, and exact per-dialog JSONL. The four-client release harness uses a local
three-source MCP fixture so protocol behavior is deterministic; the native
network experiments in adjacent directories preserve external variability.

HTML files are source evidence, not trusted product specifications. A successful HTTP response or a descriptive URL is not treated as verification when the rendered product data is absent.
