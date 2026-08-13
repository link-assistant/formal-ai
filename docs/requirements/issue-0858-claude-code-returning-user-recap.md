## Issue #858 Claude Code Returning-User Recap

Issue [#858](https://github.com/link-assistant/formal-ai/issues/858) reports
that Claude Code's `/recap` command fell through to the unknown-intent answer.
PR [#899](https://github.com/link-assistant/formal-ai/pull/899) adds a semantic
returning-user role, bounded plain recap output, canonical history sanitation,
and browser-worker parity. See `docs/case-studies/issue-858/`.

| ID | Requirement | Status |
| --- | --- | --- |
| R858-1 | Recognize Claude Code's expanded returning-user recap semantically and answer without a tool call or unknown fallback. | Implemented by `conversation_return_recap` seed data and `conversation_memory/conversation_summary.rs`; covered by the exact Anthropic Messages regression. |
| R858-2 | Produce fewer than 40 words in one or two plain sentences without Markdown. | Implemented by `summarize_dialog_plain` with explicit 39-word/two-sentence limits. |
| R858-3 | Lead with the real user goal and current assistant status, never Claude's injected `<system-reminder>` metadata. | Agentic recap and conversation-aware research reuse `protocol::chat_prompt_and_history`; covered by the multi-part reminder fixture and live Claude before/after evidence. |
| R858-4 | Preserve the existing detailed ordinary conversation summary. | The compact formatter is selected only by the returning-user role; covered by `ordinary_summary_keeps_the_existing_detailed_report`. |
| R858-5 | Keep language surfaces in seed data for every supported language. | English, Russian, Hindi, Chinese, and Spanish forms live in `meanings-intent.lino`; generated role registries and the multilingual regression pin them. |
| R858-6 | Keep the Rust core and browser worker behavior aligned. | The worker mirrors semantic routing and the bounded formatter; `browser_worker_matches_the_rust_recap_contract` executes the parity harness. |
