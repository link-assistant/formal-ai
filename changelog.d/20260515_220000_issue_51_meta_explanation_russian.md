### Fixed
- Russian prompts such as "покажи как ты работаешь?" now correctly resolve to `intent: meta_explanation` instead of falling back to `intent: unknown` (#51)

### Added
- Multilingual responses for the `meta_explanation` intent (English, Russian, Hindi, Chinese) so the agent explains how it works in the user's language
- Pattern recognition for "how do you work" / "show me how you work" style queries in English, Russian, Hindi, and Chinese
- Prompt patterns for `meta_explanation` intent in the routing seed
