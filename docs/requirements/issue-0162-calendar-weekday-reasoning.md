## Issue #162 Calendar Weekday Reasoning

Issue [#162](https://github.com/link-assistant/formal-ai/issues/162)
reported that the Russian prompt "какой день недели наступает после
вторника" returned the unknown-intent fallback. The maintainer asked for
date, time, and calendar questions to be handled through actual symbolic
reasoning where possible, not by a one-off memoized answer or tool call.

| ID | Requirement | Status |
| --- | --- | --- |
| R210 | Weekday successor and predecessor prompts must route to a typed calendar intent instead of `unknown`. | Implemented by `try_calendar_reasoning` in `src/solver_handlers/calendar.rs` and mirrored by `tryCalendarReasoning` in `src/web/formal_ai_worker.js`. |
| R211 | The answer must be derived by shifting through the seven-day calendar cycle, not by matching one reported prompt to one fixed string. | Implemented by parsing the source weekday and next/previous operation, applying a `+1` or `-1` cyclic shift, and recording `calendar:cycle`, `calendar:subject_weekday`, `calendar:operation:*`, and `calendar:result_weekday` events. |
| R212 | Russian and English weekday relation variations must be covered by automated tests. | Covered by `calendar_reasoning_answers_russian_weekday_successor` and `calendar_reasoning_answers_weekday_predecessor_and_successor_variations` in `tests/unit/specification/reasoning_paths.rs`. |
