# Issue #700 Case Study: Conversions the Pairwise Seed Never Stated (E58)

Issue [#700](https://github.com/link-assistant/formal-ai/issues/700)
(epic E58). Case study written on the `qa-reasoning-coding-bulk-fixes`
branch.

## What a user saw

> U: сколько миль в 5 километрах?
> U: how many watts is 3 horsepower?
> U: 32 psi в килопаскалях?
> A: (research suggestion or the unknown fallback — no arithmetic)

Issue #1176's engine (rust/src/unit_conversion.rs) converts pairs that
have an explicit `conversion` record in data/seed/meanings-units.lino
and inverts linear ones. The seed states dozens of pairs — but the
combinatorics win: 60 units are ~1 800 ordered pairs, and any pair the
seed does not state falls through to a search. Users asked for
arithmetic and got a reading list.

## Root cause

Conversion was modeled pairwise when it is actually an algebra. Every
physical unit is a point in the seven-dimensional SI quantity system
(length L, mass M, time T, electric current I, thermodynamic temperature
K, amount of substance N, luminous intensity J) with one factor to the
coherent SI unit of its dimension. Two units of the same dimension
always convert through SI — `value × factor(from) / factor(to)` — and
two units of different dimensions never do. A pairwise table cannot
express that; a dimension table makes the pairwise records a convenience
for taught surfaces, not a requirement for arithmetic.

## Solution (this pull request)

- `data/seed/si-unit-dimensions.lino` (mirrored at
  rust/embedded/data/seed/): 7 `base_dimension` records defining the
  system, and 66 `si_unit` records — each a `dimension` exponent string
  (`L2MT-3` is the watt's kg·m²/s³), an exact decimal or `num/den`
  `si_factor` (5/18 for km/h, 101325/760 for the torr), and plain
  `surface` recognition rows across English, Russian, Hindi, and Chinese
  (plain data rows, so no language-parity lexeme gaps open; the units
  that also live in meanings-units.lino keep their five-language
  lexemes there).
- `rust/src/si_units.rs`: `Dimension` (parse/render/multiply/inverse —
  the algebra), `parse_factor` (exact rationals, no floats),
  `si_units()`/`base_dimensions()` (seed loaders), `unit_named_by`
  (multilingual surface → canonical unit), and
  `convert_through_si(value_num, value_den, from, to) ->
  SiConversion` — `Converted` as a reduced i128 rational,
  `Incompatible { from, to }` when the dimensions differ (meters to
  seconds refuses instead of fabricating), `UnknownUnit(String)` with
  `note_unknown_unit` logging a `si_units:unknown_unit` event so the gap
  is named, and `Overflow` rather than silent precision loss.
  `ENGINE_NAME = "si-dimension-algebra"` is the evidence-link token the
  dispatch wiring should attach.
- `rust/tests/unit/issue_700_si_units.rs`: the seed loads completely;
  every dimension round-trips through its string form; the algebra
  holds (speed × time = length, energy = power × time, frequency =
  inverse time); **40 conversions** across every dimension family with
  expected values computed from the defining constants (international
  yard and pound agreements, standard atmosphere, IT calorie,
  mechanical horsepower 745.699872 W); exact-rational checks (3 hp =
  69909363/31250 W, 90 km/h = exactly 25 m/s); honest failures
  (dimension mismatch by name, unknown unit as a logged event); 25
  surfaces across en/ru/hi/zh resolve to canonical units; and four full
  prompts — Russian, English, Hindi, Chinese — resolve both surfaces
  they contain and convert.

The example from the issue body: 3 hp → W is two units of dimension
`L2MT-3` with factors 745.699872 and 1 — `3 × 745.699872 / 1 =
2237.099616` — no pairwise record involved.

## Gap matrix

| Family | Covered | Gap / note |
| --- | --- | --- |
| length, mass, time | meter…light year, pound, stone, both tons; second…julian year | complete for common use |
| speed | m/s, km/h, mph, knot | mach (altitude-dependent) deliberately out |
| power | W, kW, hp (mechanical 745.699872, metric 735.49875) | electrical apparent power (VA) pending |
| energy | J…kWh, cal, BTu | therm pending |
| force, pressure | N, kN, dyne, kgf, lbf; Pa…torr | complete for common use |
| area, volume | m²…acre; liter, both gallons, cup | dry measures pending |
| frequency, data, angle | Hz…MHz; bit…GiB; radian, degree | neper pending |
| **temperature** | **not covered** | °C/°F need affine offsets (×5/9 **+32**), not factors — a factor-only algebra would answer 1 °C = 0.55 K-style questions wrongly; requires an `offset` field and affine conversion, filed as the follow-up below |
| molar, optical | mole, candela as base only | mol/L, lumen, lux pending |
| currency | out of scope | market data, not a quantity system |

Unknown units fail honestly with a named gap event
(`si_units:unknown_unit`), never a guess.

## Known components surveyed

- rust/src/unit_conversion.rs (#1176, not this change's file): keeps the
  pairwise engine and its taught surfaces; the integration site is one
  line in its miss path — when no stated pair matches, call
  `convert_through_si` and render the result (or the named gap) before
  falling through to research. Left for main per ownership.
- data/seed/meanings-units.lino: 48 grounded units with Wikidata Q-ids
  and five-language lexemes; this table deliberately reuses those unit
  names (meter, kilometer, inch, pound…) so the two seeds compose.
- The upstream extraction ask (a shared Rust+JS si-units package) is
  drafted at `upstream-filing.md` in this directory — for main to file
  against link-foundation/si-units; **not filed here**.

## Integration site left for main

- `rust/src/lib.rs`: `pub mod si_units;`.
- `rust/tests/unit/mod.rs`: register `issue_700_si_units`.
- Seed registry row for `data/seed/si-unit-dimensions.lino` (embedded
  mirror already byte-identical, `cmp`-verified).
- unit_conversion.rs miss path: one-line fallback call (above).
- Optional: multilingual response template
  (`si_conversion_result`) so the rendered answer localizes like the
  other handlers.

No Cargo.toml changes: std only.

## Verification

- Automated: the eight tests in
  `rust/tests/unit/issue_700_si_units.rs` (CI).
- Manual: none required beyond CI; spot-check a live prompt once the
  dispatch wiring lands.

## Residuals

- Affine temperature conversion (°C/°F/K with offsets) — the follow-up
  above, needs an `offset` field and an affine path in the algebra.
- The benchmark slice (≥30 prompts) is pinned as the 40-row conversion
  table plus the four multilingual prompt cases; a live-prompt corpus
  run belongs to the dispatch integration, not this change.
