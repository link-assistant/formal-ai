### Fixed
- Route enumeration-style research prompts such as "list all Genshin characters
  with off-field DMG" to deterministic web search instead of the unknown
  fallback.
- Cover enumeration-style web-search prompts across English, Russian, Hindi,
  and Chinese, and add a CI guard for language-resource changes that omit
  Hindi or Chinese updates.
