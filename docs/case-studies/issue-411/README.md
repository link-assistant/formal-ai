# Case study: issue #411 `Покажи правила` answered `unknown`

> Source issue: <https://github.com/link-assistant/formal-ai/issues/411>
> Branch: `issue-411-1e4094a180fd` · PR: #415
> Raw data: [`raw-data/`](./raw-data) (issue JSON, PR JSON, comments/reviews, issue body, online research)

## 1. Summary

The reported wasm dialog contained two symptoms:

1. A bare numeric-list follow-up, `Отсортируй 4, 3, 1, 17, 8, 9, 15`, answered `unknown` after a previous JavaScript sort turn. This class is already covered in this checkout by issue #412: `tests/integration/issue_412_followup_sort.rs`, `examples/repro_issue_412.rs`, and `docs/case-studies/issue-412/`.
2. The direct Russian behavior-rule request `Покажи правила` answered `unknown`, even though the assistant's Russian help text told users to inspect rules and the rule-detail explanation already mentioned the shorter `Покажи правила` form.

This PR fixes the remaining issue-411 behavior-rule-list gap. The phrase is now a seed-backed `behavior_rules_list` prompt, with balanced short-form variants for every supported language:

| Language | New short form |
|---|---|
| English | `Show rules` |
| Russian | `Покажи правила` |
| Hindi | `नियम दिखाओ` |
| Chinese | `显示规则` |

## 2. Timeline

| # | Event |
|---|---|
| 1 | User asks for JavaScript code and result for sorting `3, 5, 6, 7, 8`; assistant answers correctly. |
| 2 | User sends a bare Russian sort follow-up; older builds routed it to `unknown`. |
| 3 | User sends `Покажи правила`; older builds routed it to `unknown` instead of behavior-rule listing. |
| 4 | Issue #412 resolves the numeric-list coreference branch. |
| 5 | Issue #411 remains reproducible as a missing behavior-rule-list surface: `cargo test behavior_rules_list_works_for_russian_speakers -- --nocapture` failed with `expected behavior_rules_list for "Покажи правила", got unknown`. |

## 3. Root cause

`src/solver_handlers/behavior_rules.rs` recognizes behavior-rule list requests through three seed-backed routes:

- exact prompt patterns from `data/seed/prompt-patterns.lino`;
- standalone rule-set phrases from `data/seed/meanings-behavior-rules.lino`;
- a compositional rule that requires subject + list request + assistant/ruleset scope in one supported language.

Before this PR, Russian seed coverage included:

- `покажи правила поведения`;
- `покажи список своих правил`;
- `перечисли свои правила`;
- compositional matches that include an explicit scope such as `поведения`, `свои`, or `список правил`.

The shorter `Покажи правила` has the subject (`правила`) and request (`покажи`) but not an explicit scope term, so it failed the compositional gate and was not present as a standalone phrase or exact prompt pattern.

## 4. Requirements

| Requirement | Resolution |
|---|---|
| Reproduce the bug before fixing it. | Added the exact prompt to native regression coverage and confirmed it failed with `intent: unknown`. |
| Make chat configuration/rule inspection more user-friendly. | `Покажи правила` now lists behavior rules directly; equivalent short forms were added for all supported languages. |
| Formalize variants in the meta language rather than hardcoding one prompt. | Added canonical seed entries in `data/seed/prompt-patterns.lino` and `data/seed/meanings-behavior-rules.lino`. |
| Apply the fix across the codebase. | Updated native tests, e2e intent-coverage metadata, browser e2e prompts, browser local fallback patterns, and the worker's embedded seed fallback. |
| Preserve total seed closure. | Added closure definitions for the new pattern ids and `short_rule_list` coverage group. |
| Build the case study and raw data folder. | Added this document and raw GitHub/online-research artifacts under `docs/case-studies/issue-411/`. |
| Check existing components/libraries for comparable approaches. | See [`raw-data/online-research.md`](./raw-data/online-research.md): Rasa rules, Dialogflow training phrases, Bot Framework dialog state, and Botpress intents. |
| File upstream issues if another project is at fault. | Not applicable. The failure is local seed vocabulary/mirror coverage. |

## 5. Fix

- Added a balanced `short_rule_list` coverage group to `data/seed/prompt-patterns.lino`.
- Added the same short forms to the `rule_listing_phrase` role in `data/seed/meanings-behavior-rules.lino`.
- Mirrored the fallback vocabulary in `src/web/app.js`, `src/web/formal_ai_worker.js`, and `experiments/issue-386-js-behavior-rules.mjs`.
- Added native regression coverage in `tests/unit/specification/chat_surface.rs` and `tests/unit/specification/behavior_rules.rs`.
- Added browser/e2e coverage through `tests/e2e/scripts/check-multilingual-intent-coverage.mjs` and `tests/e2e/tests/multilingual.spec.js`.

## 6. Verification plan

- `cargo test behavior_rules_list_works_for_russian_speakers -- --nocapture`
- `cargo test behavior_rules_short_list_phrase_covers_supported_languages -- --nocapture`
- `node tests/e2e/scripts/check-multilingual-intent-coverage.mjs`
- `node experiments/issue-386-js-behavior-rules.mjs`
- Full local quality checks before finalizing the PR.
