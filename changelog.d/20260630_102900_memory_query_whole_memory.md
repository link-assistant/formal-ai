---
bump: minor
---

### Added
- Natural-language access to the entire associative memory (issue #529). Queries now read across all stored memory events and projected memory links, and `formal-ai memory query` performs Turing-complete read+write control: appending new memory and applying substitutions that rewrite every matching stored value in place (not just recording intent). Both paths are driven by the multilingual seed lexicon across English, Russian, Hindi, and Chinese.

### Fixed
- Asking "what was written in the previous message?" (and its Russian, Hindi, and Chinese equivalents) now recalls the previous message instead of returning an unknown intent, in both the Rust runtime and the browser JS worker (issue #529).
