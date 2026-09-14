## Issue #499 Learn From This Data Source Requirements

Issue [#499](https://github.com/link-assistant/formal-ai/issues/499) reported that
a user teaching the engine where to learn from — the Russian directive
`Обратясь сюда ты узнаешь актуальные темы <Google Trends URL>` — was answered with
`intent: unknown`, and the maintainer asked for automated tools that convert such
a source into the auto-learning process in every supported language. PR
[#641](https://github.com/link-assistant/formal-ai/pull/641) makes the directive a
first-class, language-agnostic `learn_from_source` intent driven from a seed
`learning_sources` registry, routes it into the human-gated Google Trends learning
loop, and drives the *same* directive through the Agent CLI, with a traceable case
study under `docs/case-studies/issue-499`.

| ID | Requirement | Status |
| --- | --- | --- |
| R499-1 | Preserve issue #499 source data, PR #641 discussion data, online research, and the Google Trends raw snapshot under `docs/case-studies/issue-499`. | Implemented by `docs/case-studies/issue-499/raw-data/*`, `README.md`, `requirements.md`, `solution-plans.md`, and `raw-data/online-research.md`; covered by `issue_499_learn_from_source_documents_are_traceable`. |
| R499-2 | The exact reported directive must no longer return `intent: unknown`. | Implemented by `src/solver_handlers/mod.rs::try_learn_from_source`; the reported prompt is pinned in `tests/unit/issue_499_learn_from_source.rs`. |
| R499-3 | Recognition must be language-agnostic, proven with different wording per supported language and localized in the answer. | Implemented by the seed `learning_sources` directive cues and native-language keywords; covered by `learn_from_source_is_recognized_in_every_supported_language` and `the_acknowledgement_is_localized_to_the_prompt_language`. |
| R499-4 | Routing must be data-driven — a directive cue plus a seed-declared source, never a hardcoded URL or phrase. | Implemented by `src/seed.rs::LearningSources::match_directive` and `data/seed/learning-sources.lino`; covered by `routing_requires_both_a_directive_and_a_known_source`. |
| R499-5 | The directive must feed the auto-learning process rather than answer in isolation. | Implemented by routing the Google Trends learning frontier through the human-gated issue-#558 loop; covered by `the_directive_routes_into_the_human_gated_learning_frontier`. |
| R499-6 | The same teaching directive must drive Formal AI's own Agent CLI learning recipe. | Implemented by `src/agentic_coding/google_trends_learning.rs::is_google_trends_learning_task`; covered by `the_reported_directive_drives_the_learning_recipe_via_agent_cli`. |
| R499-7 | Pin the Agent CLI session byte-for-byte and drive the directive through the real Agent CLI in CI. | Implemented by `docs/case-studies/issue-499/agent-cli-session-learn-from-source.json`, the release-workflow E2E step, and `committed_agent_cli_session_matches_a_fresh_learn_from_source_run`. |
| R499-8 | Protect recognition, localization, data-driven routing, the auto-learning route, and artifact reproducibility with regression tests. | Implemented by `tests/unit/issue_499_learn_from_source.rs` and `tests/unit/docs_requirements_issue_499.rs`. |
