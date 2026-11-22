# Issue 498 Requirements

| ID | Requirement | Implementation |
| --- | --- | --- |
| R498-1 | Preserve issue data, PR data, online research, raw Google Trends evidence, requirements, and solution planning under `docs/case-studies/issue-498`. | This case-study directory stores raw GitHub snapshots, the raw RSS XML, `online-research.md`, this requirement table, `solution-plans.md`, and the Agent CLI session/transcript. |
| R498-2 | Read Google Trends top searches from an automated source instead of hand-copying prompts. | `parse_google_trends_rss` converts the Trends RSS feed into ranked `GoogleTrendTopic` records; `examples/issue_498_parse_google_trends_rss.rs` documents the refresh command. |
| R498-3 | Restrict the first delivered training/test slice to the top 10 trends requested by the issue comments. | `GOOGLE_TRENDS_TOP_LIMIT` caps the generated catalog at 10 topics, and tests assert `topic_count "10"`. |
| R498-4 | Generate prompt variations for every supported language. | `prompt_variants_for_topic` expands each query across `supported_languages()` (`en`, `ru`, `hi`, `zh`) with `tell_me_about` and `trends_context` variants. |
| R498-5 | Answer each generated multilingual request through the existing Formal AI engine. | `google_trends_catalog()` calls `FormalAiEngine::answer` for every prompt and records language, variation, prompt, intent, confidence, answer, and evidence links. |
| R498-6 | Keep CI deterministic even though Google Trends is live and time-sensitive. | Tests inject the checked-in RSS response into the production source client through an explicit test transport. The runtime can fetch each selected region or reuse its verified local capture. |
| R498-7 | Store the answered Google Trends catalog as reviewable Links Notation. | The agentic recipe emits Links Notation into the workspace. The historical output is archived at `docs/case-studies/issue-498/replay/google-trends-catalog.lino`. Tests check the published-source identity and parse the runtime output. |
| R498-8 | Execute and pin the solution through Formal AI's Agent CLI. | The catalog task drives the write, readback, and final response. The historical session records the original tool sequence, and the current regression test supplies the captured RSS and checks the written digest. |
| R498-9 | Route the trending prompts the engine cannot yet resolve into the human-gated auto-learning loop. | The report forwards unknown-intent traces to the issue-#558 proposal-only learner and records the observed split for each captured feed. |
| R498-10 | Drive and pin the learning-frontier report through a differently worded Agent CLI session. | The differently worded task drives the learning recipe. The historical session and explicit replay tests cover route recognition, report delivery and the source digest. |

## Scope Boundary

This change does not claim to answer every live Trend with factual background
from the open web. It creates the missing automated ingestion and regression
surface: current Google Trends queries become multilingual Formal AI prompts,
each prompt is answered by the existing deterministic engine, and the result is
reviewable. The prompts the engine cannot yet resolve are not silently dropped:
`trending_learning_report()` hands that frontier to the human-gated
self-improvement loop (issue #558), which honestly adopts nothing for
open-domain questions — so improving factual depth for specific trend topics is
now visible both as future failing answers in the catalog and as a reviewable
learning frontier, rather than hidden outside tests.
