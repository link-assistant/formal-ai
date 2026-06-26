---
bump: patch
---

### Fixed
- **Issue #464 - clock-time duration prompts now route to the calculator.** The web worker and Rust solver now handle `17:30 - 14:00` and elapsed-time wording such as `If a train leaves at 14:00 and arrives at 17:30, how long is the trip?`, returning `3 hours, 30 minutes` instead of `unknown`.
