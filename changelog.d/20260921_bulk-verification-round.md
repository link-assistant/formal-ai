---
bump: patch
---

### Fixed

- Derivations record their composed structure instead of a content hash:
  search drafts now log `typed_search(reduce_count(...))`-style notation
  with meaning fragments inline, scope fillers omitted, and runtime
  materialization templates rendered as what they wrap (issue #326).
- Multilingual synthesis benchmarks assert the same verified artifact
  behind a wrapper rendered in the prompt's own language; only the
  English lead case pins the full rendering byte for byte.
- Elliptical clock-hour prompts create calendar events without a web
  search detour (issue #595), shell-process prompts keep their terminal
  suggestion across languages (issue #870), and commutative idioms emit
  operands in the order the task names them (issue #315).
- Offline concept misses record their consulted-source trail instead of
  skipping it; Spanish fallback-script markers no longer vote inside
  English words.
