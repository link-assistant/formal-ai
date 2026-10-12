# Issue #1176 Case Study: Four Quantity Questions, Computed Instead of Copied

Issue [#1176](https://github.com/link-assistant/formal-ai/issues/1176) (E141,
part of the #1183 umbrella). Verified on `main` at `d209aac64`; fixed on the
`qa-reasoning-coding-bulk-fixes` branch.

## What a user saw

Four prompts, four failures of the same shape — the engine either described a
search instead of computing, or computed the wrong thing:

1. **"What day of the week will it be 100 days after Monday?"** → **wrong**:
   "The day after Monday is Tuesday. I move Monday by +1" — the stated 100
   was dropped entirely.
2. **"Given the values 4, 8, 15, 16, 23, 42, what are the mean and the
   median?"** → the canned web-search paragraph of #1173. `median` appeared
   nowhere in `rust/src`.
3. **"How many kilometers are 26.2 miles?"** → the same canned paragraph.
4. **"A shop sells pens at 3 dollars each. Maria buys 4 pens and pays with a
   20 dollar bill. How much change does she get?"** → the same paragraph.

All four are pure computation over values the prompt itself states. No
external source, no search, no guessing is needed — only reading the numbers
and doing arithmetic the answer can show.

## Root cause (verified on `main` d209aac64)

- **Dates**: `try_calendar_reasoning` (`rust/src/solver_handlers/calendar.rs`)
  read a one-step direction from "after"/"before" and the source weekday,
  then always shifted by ±1. The quantity "100 days" was never read.
- **Units, statistics, word problems**: no handler claimed any of these
  prompts — the plan-08 quantity route, the word-problem and unit handlers
  that exist all passed — so they reached the unknown-reasoning fallback of
  #1173 and its descriptive paragraph. There was no statistics operation
  anywhere in the solver.

## The change

| File | Change |
| --- | --- |
| `rust/src/solver_handlers/calendar.rs` | `try_calendar_reasoning` reads the stated offset through the new `detect_offset` — a numeral (digits or a spelled cardinal from the quantity extractor) followed by a `calendar_day` / `calendar_week` seed surface; the CJK numeral scan stops at the unit character so unspaced "100天" reads correctly. A bare "the day after/before X" keeps the original ±1. Answers with an offset render through new `calendar_weekday_offset_*` templates (split / exact-weeks / one-week, next and previous) that state the derivation: "100 days = 14 weeks + 2 days, and Monday + 2 days = Wednesday". `shifted_by` takes an `i64`; Russian answers inflect the source weekday (genitive after «после», instrumental after «перед»). |
| `rust/src/solver_handlers/statistics.rs` | New handler file. `handle_statistics` answers any question naming a seed statistics operation over a stated list of ≥2 numbers; `handle_word_problem` answers the price×count pattern (change and total variants). All arithmetic runs on an exact `Decimal` (i128 mantissa, bounded scale) shared with unit conversion: the mean is 108 / 6 = 18, the median (15 + 16) / 2 = 15.5, the variance is the exact integer fraction (n·Σx² − (Σx)²)/n² rendered to seven digits, and only the standard-deviation square root is a marked approximation (≈). |
| `rust/src/solver_handlers/unit_conversion.rs` | New handler file. `handle_unit_conversion` answers "how many X are N Y", "N Y in X" and "convert N Y to X" by parsing `conversion` records from the seed — linear factors (mile→km 1.609344) multiply or divide exactly, temperature formulas (°C→°F ×9/5 +32) apply step by step with each step shown. The gate is a conjunction: question vocabulary, a number, exactly two known units, and a conversion between them. |
| `data/seed/meanings-statistics.lino` | New seed: the six statistics operations as meanings (formula + Wikipedia source + lexemes in en/ru/hi/zh/es) and the four word-problem markers (each / pays with / change / total, all five languages). |
| `data/seed/meanings-units.lino` | Conversion records added to the unit meanings, each grounded in its Wikidata item: mile Q253276 → km 1.609344, foot Q3710 → m 0.3048, inch Q218593 → cm 2.54 (the pre-existing grounding Q174728 was the centimetre item and is corrected), yard Q482798 → m 0.9144, ounce Q48013 → g 28.349523125, US gallon Q23925413 → L 3.785411784, pound Q100995 → kg 0.45359237, °C/°F/K formulas. Plus a `unit_conversion_question` vocabulary meaning ("how many", "convert", "in", "to", … in five languages). |
| `data/seed/meanings-calendar.lino` | Day/week surfaces the offset reader needs: English plurals (`days`, `weeks`) and the Chinese 天. |
| `data/seed/multilingual-responses-quantities.lino` | New seed: 12 answer intents × 5 languages (statistics, three unit-conversion directions, two word-problem variants, six calendar-offset templates). |
| `rust/tests/unit/issue_1176_quantities_dates.rs` | New acceptance suite (below). |

Dispatch wiring (the `HANDLER_FUNCTIONS` table and its ledger) is added by the
pull-request integrator; the handler entry points follow the `handle_`
convention of the native computation primitives.

## Before and after

### Dates

Prompt: `100 days after Monday`

- **Before**: "The day after Monday is Tuesday. I move Monday by +1 in the
  seven-day calendar cycle." (wrong)
- **After**: "100 days after Monday is Wednesday. 100 days = 14 weeks + 2
  days, and Monday + 2 days = Wednesday in the seven-day calendar cycle."
  Evidence: `calendar:offset +100 days`, `calendar:offset_derivation 100 days
  = 14 weeks + 2 days`.

`2 weeks after Monday` now states that the weekday is unchanged ("2 weeks =
14 days exactly") instead of pretending a shift happened; `30 days before
Friday` shifts backwards to Wednesday; `星期一之后100天是星期几？` reads the
unspaced CJK unit and answers 星期三; Russian answers inflect
(«Через 100 дней после понедельника — среда»).

### Statistics

Prompt: `Given the values 4, 8, 15, 16, 23, 42, what are the mean and the median?`

- **Before**: the canned web-search paragraph.
- **After** (en): "The values are 4, 8, 15, 16, 23, 42 (n = 6).
  mean: 18 (108 / 6 = 18)
  median: 15.5 ((15 + 16) / 2 = 15.5)"

Variance and standard deviation come from the exact fraction 5460/36:
`variance: ≈151.6666667 (910 / 6 ≈ 151.6666667)` — the value line and the
derivation agree, and the ≈ marker says which digits are computed. Operation
labels localise from the seed (Russian answers say `среднее: 18`).

### Units

Prompt: `How many kilometers are 26.2 miles?`

- **Before**: the canned paragraph.
- **After**: "26.2 miles is 42.1648128 kilometers. 26.2 × 1.609344 =
  42.1648128, because 1 mile = 1.609344 kilometers."

`10 kilometers in miles` divides by the same factor and marks the
non-terminating quotient (≈6.2137119); `convert 25 celsius to fahrenheit`
shows every formula step (25 × 9/5 + 32 = 77). Furlongs are declined;
"5 feet 9 inches in cm" names three units and is declined rather than guessed.

### Word problems

Prompt: `A shop sells pens at 3 dollars each. Maria buys 4 pens and pays with
a 20 dollar bill. How much change does she get?`

- **Before**: the canned paragraph.
- **After**: "The change is 8: 20 - 4 × 3 = 8." — the price is the number
  stated nearest before the "each" marker, the payment is the last stated
  number, and the count is the remaining one. The total variant ("What is
  the total?") answers "The total is 12: 4 × 3 = 12". Spanish prompts read
  the same pattern through the seed markers (cada, paga con, cambio).

## Tests

`rust/tests/unit/issue_1176_quantities_dates.rs` — 15 engine-level tests:

1. Six calendar tests: stated offset forward (100 days → Wednesday with the
   weeks+days split), the bare day-after ±1 reading preserved, stated offset
   backward, whole-week offsets unchanged, the unspaced CJK unit probe, and
   the Russian inflected answer.
2. Five statistics tests: mean+median exact with derivations, mode+range,
   variance/stddev exactness markers, Russian labels, and the non-claim
   guards.
3. Three unit tests: forward multiply with the factor cited, reverse divide
   with ≈, the temperature formula steps, plus the unknown-unit and
   ambiguous-list declines.
4. Three word-problem tests: change, total, Spanish markers.

The calendar tests run against the already-wired `calendar_reasoning` entry;
the statistics, unit-conversion and word-problem tests assert the
post-wiring state and pass once the integrator adds the three `handle_*`
entries to the dispatch table. Command:
`RUSTUP_TOOLCHAIN=1.98.1 cargo test --test unit issue_1176_quantities_dates`.

## Honest boundaries

- The conversion factors are a hand-grounded starter table in
  `meanings-units.lino` (Wikidata item + factor per unit), not live P2370
  reads — adding a unit is a data edit, but the factor is not fetched.
- Percentile and month-offset arithmetic are not delivered; the word-problem
  pattern is the "each" price×count frame only.
- GSM8K / MATH / BIG-bench re-measurement on the upstream slices is a
  follow-up (the issue asks for it); no benchmark numbers are claimed here.
- The JavaScript worker and TypeScript roots do not carry these handlers;
  three-roots parity is not delivered. The structured-derivation format of
  #1184 is not delivered — derivations are the answer-body strings and
  evidence events described above.
- The unit-conversion gate reads the seed question vocabulary, which includes
  the common words "in", "to", "a"; no single conjunct can claim a prompt
  (a number, exactly two known units and a conversion path are also
  required), but a prompt naming two units and a number inside a larger
  request could in principle be claimed — the dispatch order (after the
  coding handlers) is what the integrator should preserve.
