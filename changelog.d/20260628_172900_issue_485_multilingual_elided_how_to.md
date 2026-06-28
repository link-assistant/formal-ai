---
bump: patch
---

### Fixed
- **Issue #485 - multilingual elided `how <action> ...` prompts now route to procedural how-to.** The Rust solver and browser worker recognize seeded weak leads such as Russian `как ...` when the following action is approved by the procedural action lexicon, preserving greeting-prefixed compound answers instead of falling through to unknown.
