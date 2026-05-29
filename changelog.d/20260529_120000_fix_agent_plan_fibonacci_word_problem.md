---
bump: minor
---

### Added
- Recursive `fibonacci` coding task in the coding catalog (Rust catalog, `.lino`
  seed, and the WASM/JS worker), so prompts like "Write a Python function that
  calculates the Fibonacci sequence recursively" generate a verified program
  that prints F(10) = 55 (issue #334).
- Natural-language "word problem" normalizer that resolves "(the) N-th Fibonacci
  number" references, rewrites spelled-out operators ("and multiply it by" →
  `*`), and drops trailing instruction sentences, so "calculate the 10th
  Fibonacci number and multiply it by 8% of 500" reduces to `55 * 8% of 500`
  = 2200 (issue #334).

### Fixed
- Coding prompts containing "number" or "program" are no longer misread as a
  unit-incompatibility conversion: unit tokens such as "mb" and "gram" now match
  only on word boundaries instead of as substrings of "nu**mb**er" /
  "pro**gram**" (issue #334).
