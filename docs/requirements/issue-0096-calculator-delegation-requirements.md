## Issue #96 Calculator Delegation Requirements

Issue [#96](https://github.com/link-assistant/formal-ai/issues/96) asks formal-ai
to use [`link-assistant/calculator`](https://github.com/link-assistant/calculator)
as the delegated calculator backend for calculator-parsable expressions while
keeping local language processing and local fallbacks for unsupported syntax.

| ID | Requirement | Status |
| --- | --- | --- |
| R120 | Add `link-calculator` as a library dependency and delegate calculator-parsable math, unit, currency, percentage, and datetime expressions to it. | Implemented by `link-calculator = "0.16.0"` in `Cargo.toml`, `evaluate_with_link_calculator` / `evaluate_calculation` in `src/calculation.rs`, and `try_arithmetic` in `src/solver_handlers/mod.rs`. |
| R121 | Keep formal-ai language processing at the prompt boundary, stripping supported natural-language wrappers before delegation. | Implemented by `calculation_expression_candidates` in `src/calculation.rs`, covering English, Russian, Chinese, and Hindi wrappers before passing the expression to `link-calculator`. |
| R122 | Preserve formal-ai fallback behavior for basic calculation syntax that `link-calculator` does not support yet. | Implemented by routing English word-operator arithmetic and binary `%` remainder expressions through the local evaluator first; covered by `local_arithmetic_fallback_keeps_word_operators_and_modulo` in `tests/unit/specification/calculator_delegation.rs`. |
| R123 | Make calculator delegation observable in the answer evidence. | Implemented by `calculation:engine:link-calculator`, `calculation:engine:formal-ai-fallback`, and `calculation:lino:*` evidence links emitted from `src/solver_handlers/mod.rs` and formatted by `src/event_log.rs`. |
| R124 | Cover touched calculator cases with 5-10 natural-language variations in English, Russian, Chinese, and Hindi. | Implemented by `tests/unit/specification/calculator_delegation.rs`, which exercises calculator-backed prompts and fallback prompts across the four supported languages. |
| R125 | Add non-NSFW calculator prompts to examples and the chat demo simulator. | Implemented by `examples/try_arithmetic.rs` and additional calculation dialogs in `data/seed/demo-dialogs.lino`; the browser worker also strips simple multilingual calculation wrappers for simulator prompts. |
| R126 | Register calculator as a visible tool/capability in the seed registry. | Implemented by `tool_calculator` in `data/seed/tools.lino`, exposed through the existing seed/tool-registry surfaces. |
| R127 | Report upstream calculator gaps found while integrating the library. | Implemented by filing [`link-assistant/calculator#158`](https://github.com/link-assistant/calculator/issues/158) for binary modulo / trailing-token handling and documenting the local fallback in `docs/case-studies/issue-96/README.md`. |
| R128 | Compile issue #96 evidence and analysis under `docs/case-studies/issue-96/`. | Implemented in `docs/case-studies/issue-96/README.md` with raw data in `docs/case-studies/issue-96/raw-data/`. |
