---
bump: minor
---

Add support for natural-language calendar event creation (issue #404).

- New data-driven roles + surfaces in `data/seed/meanings-calendar.lino` (schedule verbs "забей"/"поставь", "встреча", clock times, "по грузии" → Asia/Tbilisi aliases).
- New `ROLE_CALENDAR_*` constants (program.rs + mirrors).
- `try_calendar_create_event` (and early guard inside `try_calendar_reasoning`) that parses "NN число", HH:MM / "в 17:00", timezone aliases, title after "на ", applies next-month rollover for past days, and emits `calendar_create_event` intent + `calendar:parsed_*` trace events + localized confirmation proposal.
- Full Rust + WASM (formal_ai_worker.js) parity.
- Intent formalization relevants promotion so bare imperatives route to the handler instead of unknown.
- Unit tests for the exact reproduction prompt and English fallback.

The core remains purely symbolic/deterministic. Real Google Calendar actions (events.insert after confirmation) and list/update/delete are left to surfaces/agent tools (as designed). No existing weekday-relation or "today" calendar behaviour was changed.

Example (ru, Asia/Tbilisi):
  U: Забей мне 18 число в 17:00 по грузии на встречу с Леваном
  A: (non-unknown, calendar_create_event) "Создать событие «встречу с Леваном» на 18 число (...). ... Ответьте «да»..."
