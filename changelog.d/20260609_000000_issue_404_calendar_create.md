---
bump: minor
---

### Added
- Natural-language calendar event creation that exports a **real, importable
  calendar event** in every supported environment (issue #404). The prompt
  "Забей мне 18 число в 17:00 по грузии на встречу с Леваном" (and its English,
  Hindi, and Chinese equivalents) now resolves to a `calendar_create_event`
  intent instead of `unknown`, and the confirmation proposal carries two
  login-free, portable artifacts:
  - a universal **RFC 5545 `.ics` VEVENT** document (CRLF line endings,
    `DTSTART;TZID=`/`DTEND;TZID=`, escaped `SUMMARY`, stable content-derived
    `UID`) that imports cleanly into Apple Calendar, Outlook, Google Calendar,
    Thunderbird, and any other iCalendar client — the simplest method available
    in the CLI and HTTP environments, where the user can save/import a file; and
  - a **Google Calendar "render" template URL**
    (`calendar.google.com/calendar/render?action=TEMPLATE&text=…&dates=START/END&ctz=…`)
    that pre-fills a new event in the user's browser with no API token or server
    — the simplest method in a browser environment.
- Full multilingual support (en, ru, hi, zh). Surface words — schedule verbs,
  "meeting"/"встреча"/"मीटिंग"/"会议", clock times, and timezone aliases such as
  "по грузии" → `Asia/Tbilisi` — live as self-describing meanings in
  `data/seed/meanings-calendar.lino`; the code knows only roles and English
  slugs. Hindi (verb-final) and Chinese (no word spaces) titles are trimmed of
  trailing/leading schedule-action fragments so the `.ics` SUMMARY keeps only
  the event and its participant.
- Byte-for-byte Rust ↔ WASM parity: the `.ics` builder, Google Calendar URL
  builder, and title tidying are mirrored in the browser worker
  (`src/web/formal_ai_worker.js`), verified to produce identical artifacts
  across all four languages.

### Changed
- Extracted the calendar export logic (the `ScheduledEvent` model, RFC 5545
  `.ics` builder, Google Calendar URL builder, and date/duration helpers) into a
  new `src/solver_handlers/calendar_ics.rs` module, and split the docs-method /
  how-to procedure reasoning-path tests into
  `tests/unit/specification/reasoning_paths_procedures.rs`, keeping every file
  under the repository's 1000-line limit.

The core remains purely symbolic and deterministic: the solver *proposes* the
event and invites confirmation rather than silently mutating a remote calendar.
No existing weekday-relation or "today" calendar behaviour was changed.

Example (ru, Asia/Tbilisi):
  U: Забей мне 18 число в 17:00 по грузии на встречу с Леваном
  A: (calendar_create_event) "Создать событие «Встречу с Леваном» на 18 число
     (2026-06-18). Время: 17:00, часовой пояс: Asia/Tbilisi. … BEGIN:VCALENDAR …
     https://calendar.google.com/calendar/render?action=TEMPLATE… Ответьте «да»…"
