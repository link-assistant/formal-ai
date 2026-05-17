# Issue 96 Case Study: Delegate Calculator Expressions to link-calculator

## Summary

Issue [#96](https://github.com/link-assistant/formal-ai/issues/96) asks
formal-ai to stop expanding its own calculator surface when
[`link-assistant/calculator`](https://github.com/link-assistant/calculator) can
already parse the expression. The boundary is deliberate:

- formal-ai keeps natural-language prompt processing, language detection, and
  evidence projection;
- calculator-parsable expressions are delegated to `link-calculator`;
- unsupported math-like syntax remains on formal-ai's local evaluator until it
  is supported upstream;
- gaps found during integration are reported back to the calculator project.

## Collected Data

Fresh evidence is preserved in `raw-data/`:

- `issue-96.json` and `issue-96-comments.json` — source issue and comments.
- `pr-97.json`, `pr-97-conversation-comments.json`,
  `pr-97-review-comments.json`, and `pr-97-reviews.json` — prepared PR state
  and review surfaces.
- `calculator-repo.json` and `calculator-release-v0.16.0.json` — upstream
  repository and latest release metadata captured on May 16, 2026.
- `link-calculator-cargo-info.txt`, `link-calculator-readme-excerpt.md`, and
  `link-calculator-lib-excerpt.rs` — crate-level integration evidence.
- `link-calculator-probe.txt` — local probe output showing supported
  percentage/currency handling and the binary `%` gap.
- `calculator-issue-158.json` — upstream report filed for the gap.

## Requirements

| ID | Requirement | Source | Solution in this PR |
| --- | --- | --- | --- |
| R120 | Add `link-assistant/calculator` as a library and calculator tool. | Issue body | Add `link-calculator = "0.16.0"` and the seeded `tool_calculator` entry in `data/seed/tools.lino`. |
| R121 | Delegate everything that is calculator-parsable while keeping language processing in formal-ai. | Issue body | Strip English, Russian, Chinese, and Hindi calculation wrappers in `calculation_expression_candidates`, then call `link_calculator::Calculator::calculate_with_value`. |
| R122 | Keep support for cases calculator does not yet support. | Issue body | Route English word operators and binary `%` remainder syntax to the existing local evaluator first. |
| R123 | Add broad multilingual tests for touched cases. | Issue body | Add `tests/unit/mvp/calculator_delegation.rs` with 5-6 prompt variations per supported language plus explicit fallback assertions. |
| R124 | Add safe calculator examples to examples and the chat demo simulator. | Issue body | Extend `examples/try_arithmetic.rs` and `data/seed/demo-dialogs.lino`; update the browser worker's simple arithmetic extraction for multilingual simulator wrappers. |
| R125 | Run calculator-style checks and report upstream gaps. | Issue body | Probe `link-calculator` v0.16.0 locally and file `link-assistant/calculator#158` for binary modulo / trailing-token behavior. |
| R126 | Compile raw data and analysis under `docs/case-studies/issue-96`. | Issue body | This directory contains the raw GitHub, crate, release, and probe artifacts used for the implementation decision. |

## Online And Upstream Research

The latest usable crate release found during implementation was
`link-calculator` v0.16.0. Its public crate metadata describes a Rust
calculator with grammar-based expression parsing, Links Notation output,
datetime handling, currency support, unit support, and a Rust 1.70 minimum.
The crate exposes a `Calculator` API that returns a displayable value, a LINO
representation, and step-by-step calculation explanations, which maps directly
to formal-ai's event-log evidence model.

The integration probe confirmed that calculator handles expressions formal-ai
previously could not:

- `8% of $50` becomes `4 USD`;
- `sqrt(16)` becomes `4`;
- duration/unit prompts such as `300000 ms in seconds` are calculator-shaped;
- currency and date arithmetic can be delegated after formal-ai strips the
  surrounding natural-language request.

The same probe found an upstream gap. `100 - 25 % 7` is parsed as postfix
percent (`100 - 25 / 100`) and returns `99.75`, with the trailing `7` absent
from the LINO and steps. That is unsafe for formal-ai because the previous
contract treated `%` as binary remainder and returned `96`. The PR therefore
keeps that syntax on the local evaluator and reports the upstream issue:
[`link-assistant/calculator#158`](https://github.com/link-assistant/calculator/issues/158).

## Root Cause

Before this change, formal-ai owned every arithmetic detail itself. That was
reasonable for issue #14's MVP arithmetic requirement, but it created three
problems:

1. the local parser only understood a small arithmetic grammar;
2. adding units, currencies, dates, percentages, and math functions would
   duplicate work already happening in `link-calculator`;
3. the solver trace could only say a calculation occurred, not which engine
   produced it or which formal representation the calculation used.

The right split is to keep formal-ai responsible for deciding whether a prompt
is a calculation request, then delegate the expression body to the calculator
library when it is safe to do so.

## Design Decisions

- **Boundary before delegation.** Natural-language wrappers stay in formal-ai.
  The calculator receives expression text, not full chat prompts.
- **Calculator first, except known gaps.** The new `evaluate_calculation`
  function calls `link-calculator` unless the expression contains English
  word operators or binary `%` remainder syntax.
- **Fallback remains exact.** Existing integer and decimal arithmetic behavior
  remains in place for unsupported syntax, including large integer arithmetic
  and division-by-zero reporting.
- **Evidence names the engine.** Answers now emit `calculation:engine:*`.
  Calculator-backed answers also preserve `calculation:lino:*` so reviewers
  can inspect the delegated formal expression.
- **Browser simulator stays honest.** The web worker is still a lightweight
  JavaScript mirror, so simulator additions use simple multilingual arithmetic
  wrappers it can solve. Rich calculator delegation is tested through the Rust
  pipeline.

## Verification Plan

- Focused regression: `cargo test --test unit calculator_delegation -- --nocapture`.
- Existing arithmetic regression: `cargo test --test unit arithmetic -- --nocapture`.
- Full local CI before PR finalization:
  `cargo fmt --all -- --check`, `cargo clippy --all-targets --all-features`,
  `cargo test --all-features --verbose`, `cargo test --doc --verbose`, and
  `rust-script scripts/check-file-size.rs`.
