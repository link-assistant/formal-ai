---
bump: patch
---

### Fixed
- An explicit shell passthrough ("execute cp a.txt b.txt") is no longer re-read
  by the capability table into `cat` of both paths; dictated commands reach the
  shell cascade verbatim (issue #749).
- A protocol-hosted fetch tool now performs the work-item read itself instead of
  planning `gh` first, so server-side clients stop wasting a turn on a command
  their sandbox refuses (issue #904).
- Hindi calculator prompts that spell the operator as a word route to arithmetic
  (the arithmetic promotion now recognizes the `arithmetic_operator_word` role),
  and Hindi verb-final reachability prompts no longer read as web-search
  imperatives.
- Russian compose stems ("материал", "подробн") no longer steal retrieval
  prompts into long-form composition, and the bare English noun "machine" no
  longer turns "Machine learning" into a path-scope request.
- The unknown-reasoning web-search handoff no longer demands a "specific" focus,
  so general research prompts reach their sources instead of the legacy
  fallback.
