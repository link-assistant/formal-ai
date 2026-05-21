---
bump: minor
---

### Fixed
- Translate single nouns and unquoted prompts correctly: `translate apple to russian` now returns `яблоко` and `переведи «яблоко» на английский` returns `apple` instead of the `[ru]` / `[en]` placeholders (issues #216, #217, umbrella #218). Adds an unquoted-surface fallback to the translation handler, mirrors it in the browser worker, seeds the Wiktionary/Wikidata cache for both nouns, and extends the browser offline registry with the apple meaning so the GitHub Pages demo answers offline.
