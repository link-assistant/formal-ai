## Issue #187 Current Day Calendar Prompt

Issue [#187](https://github.com/link-assistant/formal-ai/issues/187)
reported that the Russian prompt "Какой сегодня день?" returned the
unknown-intent fallback in the browser demo. PR review feedback then
required the fix to cover every supported language, not only English and
Russian.

| ID | Requirement | Status |
| --- | --- | --- |
| R216 | Current-day and current-date prompts must route to a typed calendar intent instead of `unknown`. | Implemented by the `calendar_current_day` branch in `try_calendar_reasoning` and mirrored by `tryCalendarReasoning` in `src/web/formal_ai_worker.js`. |
| R217 | Current-day answers must be derived from the runtime clock and must expose date, weekday, and time-zone evidence. | Rust resolves the current UTC date and records `calendar:today`, `calendar:weekday`, and `calendar:time_zone:UTC`; the browser worker resolves the current browser date in the user-context time zone and records the same evidence shape. |
| R218 | Current-day prompts must be supported for every language declared by `agent_info.supported_languages`. | Covered by the English, Russian, Hindi, and Chinese current-day matrix in `tests/unit/specification/reasoning_paths.rs` and the browser e2e matrix in `tests/e2e/tests/multilingual.spec.js`. |
| R219 | CI must fail when a multilingual feature matrix omits one of the supported languages. | Enforced by `tests/e2e/scripts/check-multilingual-intent-coverage.mjs`, which parses `data/seed/agent-info.lino` and validates feature matrices against the supported-language list. |
