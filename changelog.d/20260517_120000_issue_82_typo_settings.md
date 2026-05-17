---
bump: minor
---

### Added
- Added an Assistant behavior settings section to the web sidebar with controls for ambiguity handling, temperature, UI language, theme, location, and greeting variations.
- OpenAI-compatible Chat Completions and Responses requests now accept an optional `temperature` parameter, and `SolverConfig` can read `FORMAL_AI_TEMPERATURE`.

### Fixed
- Russian typo prompts such as `что такое граматика` now resolve through Wikipedia search as the closest match by default, while low-guessing settings ask the user to clarify before using that fuzzy match.
