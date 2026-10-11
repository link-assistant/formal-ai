# Issue #869 Case Study: The Meeting That Fell Through to Unknown

Issue [#869](https://github.com/link-assistant/formal-ai/issues/869).
Reported on 0.304.1 (wasm) at 2026-07-26T08:16:29Z; case study written on the
`qa-reasoning-coding-bulk-fixes` branch.

## What a user saw

A Russian-language user asked the web app to schedule an evening meeting:

> U: Назначь мне встречу с Александром на 20:00 по Грузии

The reasoning trace shows the request was never routed:

```
intent: unknown
- trace:formalization:(@USER OP:express ?назначь мне встречу с александром на 20 00 по грузии)
- trace:fallback:unknown
```

The formalizer read the whole sentence as one undifferentiated `express`
operation and the unknown fallback answered with the teach-me-a-rule
template — for a request every required capability already existed for.

## Requirements

1. The exact reported prompt must route to calendar-event creation, not
   `unknown` (issue body, "Reproduction of dialog").
2. The participant (`Александр`), the wall-clock time (`20:00`), and the
   zone qualifier (`по Грузии` → Asia/Tbilisi via the entity-grounded
   calendar round) must survive into the answer.
3. The answer stays honest in offline mode: no claim that an invitation was
   sent; the event is created in local memory with its zone.

## Root cause

The pieces existed independently: `calendar_create` recognized `встречу ` /
`meeting with `, extracted participant titles (`с <name>`), parsed `на
HH:MM`, and resolved place surfaces to IANA zones through entity
resolution; the routing layers ahead of it, however, saw a prompt whose
normalized form (`назначь мне встречу … по грузии`) did not match the cue
surfaces the router consulted at 0.304.1, so the request never reached the
handler that could have answered it. The 2026-09 entity-grounded calendar
round (issue #1138 plan 16) and the 2026-09-30 routing-vocabulary batch
(issue #1175) landed the missing surfaces; this case study adds the
regression pin.

## Solution

- **Regression pin (this pull request):**
  `rust/tests/unit/issue_869_meeting_scheduling.rs` replays the exact
  reported string through the hermetic solver and asserts (a) the unknown
  fallback markers are absent, (b) the participant and time are named,
  (c) the zone resolves to Tbilisi or is honestly echoed, and (d) the
  English twin routes equally. The browser worker answers through the same
  seed-driven precedence, so the CLI pin covers the wasm surface.

## Known components surveyed

- `rust/src/solver_handlers/calendar_create.rs` — meeting cues, title
  extraction, place→zone entity resolution (pre-existing).
- `rust/src/solver_handlers/calendar_ics.rs` — ICS export of the created
  event (pre-existing; the offline answer offers it).
- Natural-language date/time libraries (e.g. `chrono-english`) were
  considered and rejected: the repo's convention is seed-data lexemes
  (#386, #659), and the extraction already reads them.

## Verification

- Automated: the four tests in
  `rust/tests/unit/issue_869_meeting_scheduling.rs` (CI; this branch drafts
  them without local execution per the batch's no-build constraint).
- Manual: replay the original dialog on the wasm surface and confirm the
  event card offers `Add to calendar` with Asia/Tbilisi.
