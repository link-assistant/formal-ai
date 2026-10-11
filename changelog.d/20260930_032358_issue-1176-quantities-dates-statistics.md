---
bump: patch
---

### Added

- Quantity questions are now computed exactly instead of answered with the
  canned search paragraph: statistics over a stated list ("what are the mean
  and the median of 4, 8, 15, 16, 23, 42?" → mean 108 / 6 = 18, median
  (15 + 16) / 2 = 15.5, variance from the exact fraction with an ≈ marker on
  the rendered digits), unit conversion from seed factors and formulas
  ("26.2 miles" → 42.1648128 km with the multiplication shown; °C → °F shows
  25 × 9/5 + 32 = 77), and the price×count word-problem pattern ("buys 4
  pens at 3 dollars each and pays with 20" → the change is 8: 20 - 4 × 3 =
  8). Operation names, markers and answer prose are seed data in five
  languages (issue #1176).

### Fixed

- "What day of the week will it be 100 days after Monday?" no longer drops
  the stated offset: the calendar handler reads the numeral and unit the
  prompt states (days or weeks, digits or spelled cardinals, unspaced CJK
  "100天" included) and shifts by it — 100 days after Monday is Wednesday,
  derived as 100 days = 14 weeks + 2 days; a whole number of weeks states
  that the weekday is unchanged; a bare "the day after Monday" keeps its
  original ±1 reading (issue #1176).
