---
bump: minor
---

### Added
- An iterative repository-summarization validation protocol with a published quality
  metric and an 80% ratchet (`src/summarization/validation/`). A seeded
  `splitmix64` Fisher-Yates permutation draws repository files reproducibly, two per
  iteration, and the loop keeps going until three consecutive iterations sit within
  five points of one another above the ratchet — never before twelve iterations have
  run, because three perfect iterations are six files and six files say nothing about
  a corpus of ten thousand — or it stops at the iteration bound and reports
  `bound_reached` instead of claiming a stability it never observed. The
  ten criteria are scored as an exact integer `passed/applicable` ratio, floored, with
  criteria that cannot apply to a file dropped from that file's denominator rather
  than counted as free passes (issue #893, re-opening issue #563).
- `formal-ai summarization criteria | validate | ratchet` — the operator surface for
  the metric. `validate --append` writes the measured run to
  `data/summarization/quality-baseline.lino`; `ratchet` re-measures and fails when the
  score drops below the published 80% minimum or below whatever the repository last
  committed (issue #893).

### Fixed
- Markdown embedded grammars are now exercised through the production summarizer on
  every validation run, and counted against an *independent* CommonMark fence scanner
  so the summarizer cannot grade itself. A run that recorded no embedded grammar block
  may not declare stability and is rejected by the ratchet — and because fenced
  Markdown is rare enough that a uniform draw of the affordable size can miss it
  entirely (it failed one CI run at 100% measured quality), the draw is stratified:
  the seeded permutation is computed as before, then one fence-carrying Markdown file
  is promoted into iteration 0 (issue #893).
