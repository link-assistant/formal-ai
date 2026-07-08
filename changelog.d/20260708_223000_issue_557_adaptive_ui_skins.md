---
bump: patch
---

### Added
- Added a Material UI skin and a Glass opacity slider to the shared web chat settings, with persisted preferences, reset support, localized labels, and Playwright coverage for issue link-assistant/formal-ai#557.
- Added the issue #557 case study with screenshots, raw GitHub metadata, UI-kit research, root-cause analysis, and the implementation plan.

### Changed
- Glass skin and glass composer alpha values now flow through CSS variables driven by the `glassOpacity` preference instead of fixed stylesheet rgba values.
