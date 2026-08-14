## Issue #8 Telegram Bot Requirements

Issue [#8](https://github.com/link-assistant/formal-ai/issues/8) adds a Telegram chat surface and asks the assistant to report execution evidence for code answers.

| ID | Requirement | Status |
| --- | --- | --- |
| R31 | Provide a simple Telegram bot interface. | Implemented as `POST /telegram/webhook` on the existing Rust HTTP server. |
| R32 | Route Telegram prompts through the same symbolic engine used by other interfaces. | Implemented by calling `FormalAiEngine` from the Telegram webhook handler. |
| R33 | Support private Telegram chats. | Implemented and tested with a private `chat.id`. |
| R34 | Support public Telegram chats. | Implemented and tested with a supergroup-style negative `chat.id`. |
| R35 | Preserve code formatting in Telegram replies. | Implemented by converting markdown code fences to escaped Telegram HTML `<pre><code>` blocks. |
| R36 | Compile or run generated code blocks where the environment supports it. | Implemented for Rust, Python, JavaScript, Go, and C hello-world seeds through the issue-8 verification harness. |
| R37 | Report output and execution status to users. | Implemented by appending execution status, check/run commands, and output to hello-world answers. |
| R38 | Report environment limitations instead of silently claiming execution. | Implemented for TypeScript, which is marked unavailable because `tsc` is not configured in this runtime. |
| R39 | Keep timeout behavior bounded and visible. | Implemented for seed verification with a 60-second command budget; all verified seeds completed in one iteration without timeout reduction. |
| R40 | Keep the web interface aware of execution limitations. | Implemented by updating the GitHub Pages worker fallback answers to include the same execution metadata. |
| R41 | Preserve issue research and raw evidence under `docs/case-studies/issue-8`. | Implemented with raw GitHub data, online research, local tool records, solution options, and verification notes. |
| R42 | Run the Telegram bot from the CLI using long polling by default. | Implemented as `formal-ai telegram` defaulting to `--mode=polling` and calling Telegram's `getUpdates` with offset, timeout, limit, and `allowed_updates` controls. |
| R43 | Keep the webhook server available as an opt-in CLI mode. | Implemented as `formal-ai telegram --mode=webhook`, which delegates to the existing `serve` HTTP route on the same host/port flags. |
| R44 | Publish the Telegram CLI in a Cargo crate. | Implemented by keeping the `formal-ai` binary and library in the published `formal-ai` crate; `cargo install formal-ai` exposes the `telegram` subcommand. |
| R45 | Use `lino-arguments` with clap-style configuration for all CLIs. | Implemented through `lino_arguments::Parser` derive and `lino_arguments::init()` in `main`, so flags read from CLI, environment variables, and `.lenv`/`.env` files consistently. |
| R46 | Make the Telegram polling loop testable without a network. | Implemented through the `TelegramTransport` trait and a `ScriptedTransport` test double covering offset advancement and invalid-payload recovery. |
| R47 | Log polling lifecycle events for operators. | Implemented through `eprintln!` lines that announce loop start, transport failures, invalid payloads, successful sends, send failures, and loop stop. |
