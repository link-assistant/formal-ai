Part of the E127 umbrella (#1183); parity classes **math word problems and data analysis**.

## Evidence

- 0.347.0: "A shop sells pens at 3 dollars each. Maria buys 4 pens and pays with a 20 dollar bill. How much change does she get?" → canned web-search paragraph. "Given the values 4, 8, 15, 16, 23, 42, what are the mean and the median?" → canned web-search paragraph.
- Upstream rows (`docs/status.md`): GSM8K 2/20, MATH 0/20, BIG-bench object counting 0/20.

- "How many kilometers are 26.2 miles?" → canned web-search paragraph.
- "What day of the week will it be 100 days after Monday?" → **wrong**: "The day after Monday is Tuesday. I move Monday by +1" (the 100 is dropped; correct: Wednesday).

## Formal version

Units and calendars: quantities carry units resolved to Wikidata unit items and conversion factors (P2370), converted exactly; date arithmetic over the formalized offset (100 days = 14 weeks + 2 days).


Formalize the problem into quantities, units and relations (price × count, payment − cost), solve with `link-calculator` / the equation solver, and show the derivation as links. Statistics requests map to named operations (mean, median, mode, variance, percentile) defined as meanings with their formulas, applied to the extracted list.

## Root causes (verified on `main` d209aac64)

- **Dates:** `try_calendar_reasoning` (`rust/src/solver_handlers/calendar.rs:206-245`) takes a one-step operation from the word "after"/"before" (`detect_operation`) and the weekday (`detect_weekday`), then `source.shifted(operation)`; the quantity "100 days" is never read, so the answer moves Monday by +1. The template at `calendar.rs:471` prints that step.
- **Units, arithmetic word problems, statistics:** neither `try_verifiable_task` (`solver_dispatch.rs:319`, the plan-08 quantity route in `rust/src/verifiable_task/quantities.rs`) nor `calculation_word_problem.rs` nor `solver_handler_units.rs` (incompatibility only) claimed "How many kilometers are 26.2 miles?", the pens-and-change problem or "mean and median of 4, 8, 15, 16, 23, 42"; they reached the unknown-reasoning fallback of #1173. `median` appears nowhere in `rust/src`: there is no statistics operation.

## Requirements

- **R1** Quantities are formalized with value and unit (unit resolved to a Wikidata item, conversion factor from its P2370 statement or the SI definition), and conversions are exact decimal arithmetic through link-calculator; "26.2 miles in km" → 42.1648128 km (the factor 1.609344 cited).
- **R2** Word problems are formalized into quantities and relations (price × count, payment − cost) and solved through link-calculator / the equation solver; the answer shows the derivation (pens: 20 − 4 × 3 = 8).
- **R3** Descriptive statistics (mean, median, mode, variance, standard deviation, percentile, range) are meanings with formulas in data (new `data/seed/meanings-statistics.lino`), applied to the extracted list; mean(4, 8, 15, 16, 23, 42) = 18, median = 15.5.
- **R4** Calendar arithmetic reads the formalized offset (quantity and unit: days, weeks, months) and computes modulo the cycle: 100 days after Monday = 14 weeks + 2 days → Wednesday; dates use a real calendar (month lengths, leap years).
- **R5** Each answer carries its derivation (#1184) and all three roots answer identically (translated code, parity cases).

## Design

- `rust/src/solver_handlers/calendar.rs`: `detect_operation` returns an offset from the formalized quantity (numeral + unit) when present, falling back to ±1 only for "the day after/before"; `shifted` takes an `i64` offset.
- `rust/src/verifiable_task/quantities.rs`: extend extraction with unit meanings and conversion factors (new `data/seed/meanings-units.lino`, anchored to Wikidata unit items, factors read live and cached).
- New `rust/src/solver_handlers/statistics.rs` reading `meanings-statistics.lino`.
- GSM8K-style problems route through `calculation_word_problem.rs` after formalization (#1175).

## Tests

- `rust/tests/unit/issue_1176_quantities_and_dates.rs`: the four probes above plus held-out sets (`data/benchmarks/math-data/{en,ru,hi,zh}.lino`, at least 25 prompts per language across units, word problems, statistics and dates, each with an exact expected value).
- GSM8K and MATH re-measured on full upstream slices with `formal-ai benchmark run --suite gsm8k` / `--suite math`, recorded in `data/benchmarks/external-results.lino` whatever the numbers are.
- Command: `RUSTUP_TOOLCHAIN=1.98.1 cargo test --test unit issue_1176_quantities_and_dates`.

## Definition of done

- [ ] Requirement shard `docs/requirements/issue-1176-quantities-dates-statistics.md`; `rust-script scripts/assemble-requirements.rs --write`; traceability rows.
- [ ] Case study `docs/case-studies/issue-1176/` with the probes before and after and the new upstream rows.
- [ ] Changelog fragment in `changelog.d/`; `RUSTUP_TOOLCHAIN=1.98.1 rust-script scripts/run-ci-gates.rs --stage rust` green.

## Depends on / blocks

- Depends on #1175 (E140) and #1173 (E138) for these prompts to reach the handlers; uses link-assistant/calculator#222 (exact big integers, linear equations) through #1182.
- Feeds the math, data-analysis, units and dates rows of #1171.

## How to test

GSM8K and MATH re-measured on full upstream slices; a held-out set of 100 descriptive-statistics requests in four languages.



