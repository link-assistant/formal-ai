---
bump: minor
---

### Added
- Configurable controlled natural language (CNL) output in English, Russian, Hindi, Chinese and Spanish. Independent output-standard (cnl-default, asd-ste100, ace, gellish, sbvr, plain-language, minimal-english, e-prime, basic-english, special-english, globish) and lexical-density (standard, primes, top-N, max-precision) settings are available in the CLI, OpenAI-compatible API, browser worker and desktop preferences, and can be switched during a dialogue. One declarative grammar per language (`data/seed/cnl-language-<id>.lino`) drives realization, parsing and validation, in both the JavaScript and Rust engines.
- Whole-prompt controlled dialogue. Declared fact questions are answered in the selected standard. An ambiguous question gets a numbered CNL clarification, and a numbered reply picks one reading. A task paraphrase that the solver did not understand is solved through its canonical controlled prompt.
- An exhaustive matrix over languages, standards, densities and meaning graphs, with a reviewed infeasibility ledger and per-language JavaScript/Rust parity fixtures.
