---
bump: patch
---

### Fixed
- Added browser-worker and Rust-core handling for `Переведи "как у тебя дела?" на английский.` so it returns an English translation instead of the unknown fallback.
- Made the Russian `Что ещё ты умеешь?` follow-up use conversation history and avoid repeating already discussed web-search details.
- Made the left `MENU` sidebar action group collapsible and persistent like the other sidebar sections.
- Improved prefilled issue reports by renaming the dialog section, removing reproduction boilerplate, and preserving more earlier dialog context within the GitHub URL budget.
