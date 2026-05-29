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
- The shared `no_std` arithmetic evaluator (used by the CLI fallback, the
  compiled WASM worker, and the JS worker fallback) now understands "N% of M"
  percentage-of phrases, rewriting `8% of 500` to `( 8 * 500 / 100 )` so the
  GitHub Pages WASM demo evaluates `55 * 8% of 500` to 2200 instead of returning
  "unparseable". A bare `%` not followed by "of" still parses as modulo
  (issue #334).
- Coding prompts containing "number" or "program" are no longer misread as a
  unit-incompatibility conversion: unit tokens such as "mb" and "gram" now match
  only on word boundaries instead of as substrings of "nu**mb**er" /
  "pro**gram**" (issue #334).
