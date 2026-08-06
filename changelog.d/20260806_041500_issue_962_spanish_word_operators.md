### Fixed

- Spanish arithmetic prompts (`¿Cuánto es 2 más 2?`) fell to the unknown
  handler: the `arithmetic_operation` meanings carried no Spanish operator
  words and no Spanish `calculation_result_query` cue. Seeded `más`, `menos`,
  `por` / `multiplicado por`, `dividido por` / `entre`, `módulo`, and the
  `cuánto es` / `cuánto da` / `cuántos son` cues.
- The Spanish opening marks `¿` and `¡` are now trimmed as leading prompt
  punctuation. Previously `¿Cuánto es 2 + 2?` reached the typo responder
  ("Interpreted `¿Cuánto es` as `cuánto es`") instead of the calculator, which
  broke even the symbolic form in Spanish.
