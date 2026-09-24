---
bump: patch
---

### Added
- `formal-ai translate --from rust --to meta --input PATH` (plus `--list` for
  every direction): any-direction translation through the meta language as the
  single pivot — `translate(X → Y)` is always `extract(X → meta)` then
  `render(meta → Y)`, and the dispatcher holds no per-pair code. The first
  materialized leg is the Rust extractor into the pivot; every leg not built
  yet answers with the exact plan-16 leaf that owes it (js/ts extractors and
  the `meta → rust` inverse by L5, the js → ts dogfood by L3), the same
  stated-gap honesty the capability table practices.
