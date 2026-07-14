---
bump: minor
---

### Added
- Every formalized statement now carries an explicit, inspectable
  probability weight (issue #661, R384). For a multi-interpretation prompt
  (e.g. the copula-ambiguous "apple is a fruit", which splits into P31
  instance-of vs P279 subclass-of), the formalization step appends a
  `statement_weight` link per accepted interpretation, whose values are the
  softmax posteriors already used for temperature selection and sum to 1
  across candidates. The weights live in the trace (evidence links), never in
  the plain reply, so diagnostics stay default-off.
- Contradictory standing requirements are now detected and warned about
  (issue #661, R384). When a newly formalized directive conflicts with a
  retained one — same subject, opposite polarity, e.g. "always answer in
  Russian" then "never answer in Russian" — the solver appends a
  `requirement_contradiction` event and replies with a warning that names
  both statements, their weights, and a proposed resolution (retract one via a
  superseding requirement, split the meanings, or scope each to a different
  context). The resolution reuses the append-only retraction protocol
  (`policy:add_only_history`), and the warning is rendered in the prompt's
  language (en/ru/hi/zh). Polarity markers are recognised across all four
  languages, so widening coverage is a marker-table edit rather than new
  control flow. The check runs before the contextual handlers so a
  contradictory language directive is flagged instead of being silently
  replayed.
