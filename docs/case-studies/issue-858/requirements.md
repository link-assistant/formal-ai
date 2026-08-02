# Issue #858 requirements

| ID | Requirement | Evidence | Regression test |
| --- | --- | --- | --- |
| R858-01 | Claude Code's expanded returning-user recap request must resolve as `summarize_conversation`, never as the unknown fallback and never as a tool call. | `data/seed/meanings-intent.lino`; `src/solver_handlers/conversation_memory/conversation_summary.rs` | `claude_code_away_recap_returns_a_bounded_plain_summary` |
| R858-02 | The recap must contain fewer than 40 words in one or two plain sentences with no Markdown. | `summarization::summarize_dialog_plain`; explicit recap budgets | `claude_code_away_recap_returns_a_bounded_plain_summary`, `plain_dialog_summary_removes_markdown_and_honors_budgets` |
| R858-03 | The user's overall goal and current assistant status must outrank client-injected reminder metadata. | canonical `protocol::chat_prompt_and_history`; exact Anthropic reminder fixture | `claude_code_away_recap_returns_a_bounded_plain_summary` |
| R858-04 | The existing ordinary conversation-summary report must remain detailed and unchanged. | separate ordinary branch in `try_summarize_conversation` | `ordinary_summary_keeps_the_existing_detailed_report` |
| R858-05 | Returning-user recognition must be semantic seed data for English, Russian, Hindi, Chinese, and Spanish, without the full Claude prompt in production Rust. | `conversation_return_recap` seed records; generated role registry | `returning_user_recap_is_a_multilingual_semantic_role` |
| R858-06 | Rust and the browser worker must share routing, content, budget, and ordinary-summary behavior. | `src/web/worker/formal_ai_worker_{05,13,16}.js`; executable worker harness | `browser_worker_matches_the_rust_recap_contract` |
