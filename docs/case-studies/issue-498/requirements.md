# Issue 498 Requirements

| ID | Requirement | Implementation |
| --- | --- | --- |
| R498-1 | Preserve issue data, PR data, online research, raw Google Trends evidence, requirements, and solution planning under `docs/case-studies/issue-498`. | This case-study directory stores raw GitHub snapshots, the raw RSS XML, `online-research.md`, this requirement table, `solution-plans.md`, and the Agent CLI session/transcript. |
| R498-2 | Read Google Trends top searches from an automated source instead of hand-copying prompts. | `parse_google_trends_rss` converts the Trends RSS feed into ranked `GoogleTrendTopic` records; `examples/issue_498_parse_google_trends_rss.rs` documents the refresh command. |
| R498-3 | Restrict the first delivered training/test slice to the top 10 trends requested by the issue comments. | `GOOGLE_TRENDS_TOP_LIMIT` caps the generated catalog at 10 topics, and tests assert `topic_count "10"`. |
| R498-4 | Generate prompt variations for every supported language. | `prompt_variants_for_topic` expands each query across `supported_languages()` (`en`, `ru`, `hi`, `zh`) with `tell_me_about` and `trends_context` variants. |
| R498-5 | Answer each generated multilingual request through the existing Formal AI engine. | `google_trends_catalog()` calls `FormalAiEngine::answer` for every prompt and records language, variation, prompt, intent, confidence, answer, and evidence links. |
| R498-6 | Keep CI deterministic even though Google Trends is live and time-sensitive. | The live RSS is saved as `data/seed/google-trends-snapshot.lino`; tests and the Agent CLI recipe load that seed offline. |
| R498-7 | Store the answered Google Trends catalog as reviewable Links Notation. | `src/agentic_coding/google_trends_catalog.rs::render_document` writes `data/meta/google-trends-catalog.lino`, and the unit tests parse it as Links Notation. |
| R498-8 | Execute and pin the solution through Formal AI's Agent CLI. | `GOOGLE_TRENDS_CATALOG_TASK` drives the `google_trends_catalog` recipe through `run_agentic_task`; `agent-cli-session-google-trends.json` is compared byte-for-byte with a fresh run. |

## Scope Boundary

This change does not claim to answer every live Trend with factual background
from the open web. It creates the missing automated ingestion and regression
surface: current Google Trends queries become multilingual Formal AI prompts,
each prompt is answered by the existing deterministic engine, and the result is
reviewable. Improving factual depth for specific trend topics is now visible as
future failing answers in the catalog rather than hidden outside tests.
