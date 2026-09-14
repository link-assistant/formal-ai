## Issue #498 Google Trends Requirements

Issue [#498](https://github.com/link-assistant/formal-ai/issues/498) asks
Formal AI to learn from popular Google searches visible in Google Trends. The
delivered slice converts the U.S. top 10 Trends RSS snapshot into multilingual
Formal AI prompts, answers every prompt, and records the catalog through the
Agent CLI.

| ID | Requirement | Status |
| --- | --- | --- |
| R498-1 | Issue data, PR data, online research, requirements, solution planning, and raw Google Trends evidence must be compiled under `docs/case-studies/issue-498`. | Implemented by `docs/case-studies/issue-498/{README,requirements,solution-plans}.md`, `raw-data/online-research.md`, raw GitHub snapshots, and `raw-data/google-trends-us-rss.xml`. |
| R498-2 | The Google Trends top searches must be converted from an automated source into ranked topics. | Implemented by `parse_google_trends_rss` and `render_google_trends_snapshot_lino`, with the deterministic seed stored at `data/seed/google-trends-snapshot.lino`. |
| R498-3 | The first catalog must cover the top 10 Google Trends topics. | Implemented by `GOOGLE_TRENDS_TOP_LIMIT = 10`; `data/meta/google-trends-catalog.lino` records `topic_count "10"`. |
| R498-4 | Every supported language must receive Google Trends prompt variations. | Implemented by `google_trends_catalog()` expanding each topic across `supported_languages()` with `tell_me_about` and `trends_context` variants for English, Russian, Hindi, and Chinese. |
| R498-5 | Every generated prompt must be answered through the existing Formal AI engine. | Implemented by `google_trends_catalog`, which calls `FormalAiEngine::answer` for each prompt and records the answer metadata in the `google_trends_catalog` artifact. |
| R498-6 | The live Google Trends dependency must not make CI flaky. | Implemented by checking in the parsed Trends seed and keeping the live RSS fetch in `examples/issue_498_parse_google_trends_rss.rs` for explicit refreshes. |
| R498-7 | The generated Google Trends catalog must be reviewable Links Notation. | Implemented by `src/agentic_coding/google_trends_catalog.rs::render_document` and committed at `data/meta/google-trends-catalog.lino`, with parser coverage in `tests/unit/issue_498_google_trends_catalog.rs`. |
| R498-8 | The work must be driven and reproducible through Formal AI's Agent CLI. | Implemented by the `google_trends_catalog` agentic recipe, `GOOGLE_TRENDS_CATALOG_TASK`, and the pinned `docs/case-studies/issue-498/agent-cli-session-google-trends.json`. |
| R498-9 | The trending prompts the engine cannot yet resolve must feed the human-gated auto-learning loop instead of being dropped. | Implemented by `src/google_trends_learning.rs::trending_learning_report`, which routes every unresolved prompt through the issue-#558 learner (`learn_rules_from_unknown_traces`) and records the honest, proposal-only result at `data/meta/google-trends-learning.lino` (nothing is auto-adopted). |
| R498-10 | The learning-frontier report must be driven and pinned through a differently worded Agent CLI session. | Implemented by the `google_trends_learning` agentic recipe, `GOOGLE_TRENDS_LEARNING_TASK`, and the pinned `docs/case-studies/issue-498/agent-cli-session-google-trends-learning.json`. |
