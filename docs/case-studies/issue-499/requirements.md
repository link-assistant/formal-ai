# Issue 499 Requirements

The reported directive
`Обратясь сюда ты узнаешь актуальные темы https://trends.google.com/trending?hl=ru&&geo=US`
must be recognized as a **learn from this data source** directive and routed into
the auto-learning process, in every supported language, and driven through the
Agent CLI — not answered with `intent: unknown`.

| ID | Requirement | Acceptance |
| --- | --- | --- |
| R499-1 | Preserve the issue, PR, comments, related work, Google Trends source snapshot, and online research under `docs/case-studies/issue-499`. | `tests/unit/docs_requirements_issue_499.rs` asserts the case-study files and raw data exist. |
| R499-2 | The exact reported prompt must no longer return `intent: unknown`. | `try_learn_from_source` routes it to `learn_from_source`; the reported prompt is pinned in `tests/unit/issue_499_learn_from_source.rs`. |
| R499-3 | Recognition is language-agnostic and proven with different wording per supported language. | English, Russian, Hindi, and Chinese directives all route to `learn_from_source`, and the acknowledgement is rendered in the prompt's language. |
| R499-4 | Routing is data-driven — a directive cue plus a seed-declared source, never a hardcoded URL or phrase. | `LearningSources::match_directive` reads `data/seed/learning-sources.lino`; production code branches only on the declared `capability` slug. |
| R499-5 | The directive feeds the auto-learning process instead of answering in isolation. | Both entry points route the Google Trends learning frontier (60 of 80 prompts) through the human-gated issue-#558 loop; nothing is auto-adopted. |
| R499-6 | The same teaching directive drives the Agent CLI learning recipe. | `is_google_trends_learning_task` recognizes the seed-driven directive; `run_agentic_task(<reported prompt>)` writes `data/meta/google-trends-learning.lino`. |
| R499-7 | Pin the Agent CLI session byte-for-byte and drive it live in CI. | `tests/unit/issue_499_learn_from_source.rs` compares a fresh run with `agent-cli-session-learn-from-source.json`; `.github/workflows/release.yml` drives the directive through the real Agent CLI. |
| R499-8 | Add regression coverage so future changes cannot silently drop recognition, localization, data-driven routing, the auto-learning route, or artifact reproducibility. | `tests/unit/issue_499_learn_from_source.rs` and `tests/unit/docs_requirements_issue_499.rs` pin routing, localization, recipe driving, the pinned session, and traceability. |
