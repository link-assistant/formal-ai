---
bump: patch
---

### Fixed
- Recognize multilingual calendar event requests such as the reported Russian
  "book the 18th at 17:00 Georgia time for a meeting with Levan" prompt and
  return a safe calendar-event draft with `.ics`, browser login, and API-token
  integration paths instead of `unknown`.
