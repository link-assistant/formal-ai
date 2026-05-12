# Issue 8 Case Study: Telegram Bot Interface

## Summary

Issue [#8](https://github.com/link-assistant/formal-ai/issues/8) asks for a simple Telegram bot interface on top of the existing symbolic assistant. It also asks that generated code examples be compiled or run where practical, that outputs be shown with clear execution status, and that the library, CLI, web, and Telegram surfaces stay aware of their own execution limits.

The implementation adds a `POST /telegram/webhook` route to the existing Rust server. Telegram updates are converted into prompts for the same `FormalAiEngine` used by the library, CLI, HTTP API, and web demo. Replies are returned as direct Telegram `sendMessage` webhook responses with HTML-formatted code blocks. The same hello-world answers now include verification metadata and captured output when local tooling exists.

## Collected Data

Raw evidence is stored in `raw-data/`:

- `issue-8.json`: issue title, body, labels, timestamps, and URL.
- `issue-8-comments.json`: issue comments. The issue had no comments at collection time.
- `pr-9.json`: prepared PR metadata before implementation.
- `pr-9-review-comments.json`: inline PR comments. The PR had none at collection time.
- `pr-9-conversation-comments.json`: PR conversation comments. The PR had none at collection time.
- `pr-9-reviews.json`: PR review records. The PR had none at collection time.
- `recent-merged-prs.json`: recent merged PRs used to match repository style.
- `ci-runs-before-fix.json`: prepared branch CI metadata collected before this implementation.
- `link-assistant-telegram-code-search.txt`: GitHub code search results for Telegram-related code in the owner.
- `link-foundation-start-search.txt`: repository metadata evidence for `link-foundation/start`.
- `local-tool-versions.txt`: compiler, runtime, start-command, and Docker availability.
- `start-command-smoke.txt`: successful start-command smoke run.
- `start-command-help.txt`: failed `--help` invocation captured as evidence that start-command treats arguments as shell commands.

Verification logs are stored in `logs/` after local checks run.

## Requirements Extracted

| Requirement | Evidence | Implemented behavior |
| --- | --- | --- |
| Provide a simple Telegram bot. | Issue title and first sentence. | Added `POST /telegram/webhook` to `formal-ai serve`. |
| Preserve existing library, CLI, API, and web surfaces. | Issue lists interface types and asks for awareness of limitations. | The webhook reuses `FormalAiEngine`; web worker answers now mirror execution metadata. |
| Support private and public Telegram chats. | Issue explicitly asks to test private and public chats. | Unit tests cover a private `chat.id` and a supergroup negative `chat.id`. |
| Execute generated code blocks where practical. | Issue says code blocks should be launched. | Rust, Python, JavaScript, Go, and C hello-world seeds are verified locally. |
| Show output or explain when execution is unavailable. | Issue asks to attach/show output and report launch limitations. | Answers include execution status, commands, output, and a TypeScript compiler warning. |
| Use timeout feedback. | Issue asks to reduce timeout feedback by one minute each iteration. | The verification harness runs with a 60-second command budget and records that no timeout reduction was needed for the seed examples. |
| Use link-foundation/start when testing command execution. | Issue mentions `link-foundation/start` and Docker. | `start-command` was found locally and smoke-tested; Docker was not installed, so code verification used local compilers and runtimes. |
| Keep reasoning bounded. | Issue says to fail when reasoning takes more than 10 minutes. | The current engine has no unbounded reasoning loop; all routes execute deterministic matching synchronously. |
| Collect issue research under `docs/case-studies/issue-8`. | Issue body explicitly requests this folder. | This directory stores raw GitHub data, online research notes, candidate designs, and verification evidence. |

## Online Research

- [Telegram Bot API](https://core.telegram.org/bots/api) supports webhooks and JSON responses that ask Telegram to call a bot method. The chosen design returns `sendMessage` directly from the webhook, so the server does not need to store a Telegram bot token for the prototype.
- [Telegram `sendMessage`](https://core.telegram.org/bots/api#sendmessage) supports HTML parse mode. This avoids Markdown escaping pitfalls for code blocks by rendering fenced markdown as escaped `<pre><code>` content.
- [Telegram `sendDocument`](https://core.telegram.org/bots/api#senddocument) is the right path for large output attachments, but multipart uploads require an outbound token-bearing bot client. This PR documents that boundary instead of storing credentials in the prototype.
- [teloxide](https://github.com/teloxide/teloxide) is a full Rust Telegram bot framework. It is useful for a future long-running bot client but too large for this small webhook adapter.
- [frankenstein](https://github.com/ayrat555/frankenstein) is a Rust Telegram Bot API client. It was considered for future outbound `sendDocument` support; the current direct webhook route needs only `serde`.
- [link-foundation/start](https://github.com/link-foundation/start) provides command execution with logs and optional isolation. The local `start-command` binary exists, but Docker isolation is unavailable in this runtime.
- `link-assistant/hive-mind` has Telegram helpers for markdown escaping, safe fallback replies, and per-command isolation. The useful pattern here is to keep Telegram formatting defensive and avoid assuming one parser mode always succeeds.

## Solution Options

| Option | Benefits | Tradeoff | Decision |
| --- | --- | --- | --- |
| Direct webhook `sendMessage` JSON response. | Small, stateless, no new dependency, no bot token stored. | Cannot upload new attachment files. | Used for the prototype. |
| Add `teloxide` long-polling bot. | Full bot runtime and handlers. | Requires token management, async runtime integration, and more operational state. | Deferred. |
| Add `frankenstein` outbound client. | Enables `sendDocument` and richer Telegram methods. | Adds dependency and credential path before large outputs exist. | Deferred until large-output support is real. |
| Implement Telegram formatting manually. | Keeps dependency graph unchanged. | Must escape HTML carefully. | Used with a small markdown-fence-to-HTML renderer. |

## Root Cause

Before this issue, `formal-ai serve` exposed OpenAI-shaped HTTP routes but no Telegram route. The core engine could answer hello-world requests, but code answers did not say whether the code had been compiled or run. The web demo also had static hello-world answers that would have diverged from the engine once execution metadata was added.

## Implementation Notes

- `src/telegram.rs` parses Telegram update JSON, extracts text or captions from private/group/channel messages, calls `FormalAiEngine`, and returns a direct Telegram `sendMessage` response.
- `src/server.rs` routes `POST /telegram/webhook` beside the OpenAI-compatible routes.
- `src/engine.rs` adds execution metadata to each hello-world seed and exports that metadata in Links Notation.
- `docs/demo/formal_ai_worker.js` mirrors the execution report in the GitHub Pages demo.
- `experiments/verify-hello-world-examples.sh` compiles or runs the seed examples with a one-minute command budget where the local toolchain supports it.

## Regression Coverage

The first unit-test draft failed before implementation because:

```text
/telegram/webhook returned 404
Rust hello-world answers lacked "Execution status: compiled and ran"
```

The fixed unit tests now verify:

- Rust hello-world answers include a code block, execution status, output label, and captured `Hello, world!` output.
- A private Telegram message receives a `sendMessage` response.
- A public supergroup-style chat ID receives a code reply with Telegram HTML code formatting.

## Verification Results

Local verification recorded in `logs/`:

| Check | Result |
| --- | --- |
| `experiments/verify-hello-world-examples.sh` through `start-command` | Rust, Python, JavaScript, Go, and C verified; TypeScript unavailable because `tsc` is not installed. |
| `cargo fmt --all -- --check` | Passed. |
| `cargo clippy --all-targets --all-features` | Passed without warnings. |
| `cargo test --all-features --verbose` | Passed: 39 unit tests and 2 integration tests. |
| `cargo test --doc --verbose` | Passed. |
| `cargo run --quiet -- chat --prompt ... --format chat` | Passed for Rust verified output and TypeScript unavailable output examples. |
| `rust-script scripts/check-file-size.rs` | Passed. |
| `rust-script scripts/check-changelog-fragment.rs` | Passed. |
| `node --check docs/demo/formal_ai_worker.js` | Passed. |
| `npm run test:local` in `tests/e2e` | Passed: 16 Playwright tests. |
| `git diff --check` | Passed. |

## Known Boundaries

- TypeScript is returned with an explicit unavailable status because `tsc` is not installed in this runtime.
- Docker is not installed, so Docker-backed isolation was not used during verification.
- Direct webhook replies are enough for short Telegram messages. Large output attachments need a follow-up outbound bot client that can call `sendDocument` with a token and multipart upload.
