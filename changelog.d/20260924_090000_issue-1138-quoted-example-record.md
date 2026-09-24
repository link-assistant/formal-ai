---
bump: patch
---

### Fixed
- The wave-T corpus gate (`issue_1138_no_silent_unknown`) still failed assertion
  (a) for the six quoted-example lipogram paraphrases (ru, hi, zh, each carried
  by both held-out corpora). The consult walk recorded attributed misses for
  every variant, but the record arm rejected ru/hi/zh on focus specificity
  while en passed only through the implementation-language modifier scan
  stripping ` in e` — a letter it mistook for a language; `на букву e` is
  correctly rejected and head-final hi/zh carry no preposition to scan. A
  quoted-example question (quoted span plus question shape) now closes on the
  consulted-source record in every language: the predicate it turns on lives
  outside the quotes, `unknown_surfaces` skips quoted spans for exactly that
  reason, and every alternative terminal embedded a seeded unknown opener. A
  question that quotes nothing keeps the issue-#44 teaching ladder. The Spanish
  pin moved from the localized unresolved body to the same record (still
  Spanish, still never the unsupported-language fallback); en is byte-identical
  to its pin.
