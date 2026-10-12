---
bump: minor
---

### Added

- `formal-ai explain <answer-id>` prints the white-box derivation of a
  previously returned answer: the search queries issued, every fetched URL
  with its SHA-256 hash and fetch timestamp, the formalized fragments, the
  decomposed parts, the recomposition, the rendering, and the verification
  output — read from a durable Links Notation record persisted under
  `data/cache/derivations/<answer_id>.lino`, so explaining a past answer
  never replays the request (`--format links` prints the canonical record).
  The record is a projection of the same append-only event log the
  `--thinking` trace narrates, so the two surfaces cannot disagree, and a
  stage the route never populated is reported as "not recorded", never
  fabricated. The subcommand is disjoint from the in-chat "explain how
  Formal AI works" self-explanation recipe (issue #1184).

### Notes

- The derivation stage vocabulary (`formalize:fragment`, `decompose:part`,
  `recompose:bind`, `render:emit`, `verify:evidence`) is declared as seed
  data in `data/seed/derivation-schema.lino`; the E128/E129/E131/E132
  landings append their stages to it.
