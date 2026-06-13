---
bump: minor
---

### Added
- Relative-date calendar scheduling (issue #435). The prompt
  "Можешь поставить мне созвон в кальндарь на завтра?" — which carries no day
  number and no clock time, only a relative-date word ("на завтра") and an event
  noun ("созвон") — now resolves to a `calendar_create_event` instead of
  `unknown`. The solver recognizes relative-date words as a date anchor, resolves
  "завтра"/"tomorrow"/"कल"/"明天" to **tomorrow** (and "послезавтра"/"day after
  tomorrow"/"परसों"/"后天" to the day after), and titles the draft from the matched
  event noun when no explicit subject is given.
- New `calendar_relative_date` role with the `calendar_tomorrow` and
  `calendar_day_after_tomorrow` meanings in `data/seed/meanings-calendar.lino`,
  grounded in Wikidata and surfaced in en/ru/hi/zh. As with the rest of the
  lexicon-driven design, the code knows only the role and English slugs; adding a
  language never touches code.
- A new `calendar:parsed_relative_offset` evidence link records the resolved
  day offset, alongside the existing `calendar:parsed_*` trace, in both the Rust
  engine and the byte-for-byte browser worker mirror (`src/web/formal_ai_worker.js`).

The core remains purely symbolic and deterministic: the solver *proposes* the
tomorrow event with an importable RFC 5545 `.ics` VEVENT and a login-free Google
Calendar render URL, and invites confirmation rather than silently writing the
calendar. A bare relative-date mention with no schedule verb or event noun is
not hijacked into a create request.

Example (ru):
  U: Можешь поставить мне созвон в кальндарь на завтра?
  A: (calendar_create_event) "Создать событие «Созвон» на 14 число (2026-06-14).
     … BEGIN:VCALENDAR … calendar.google.com/calendar/render… Ответьте «да»…"
