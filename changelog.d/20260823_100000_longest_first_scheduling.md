---
bump: patch
---

### Changed

- Schedule the longest tests first when splitting work across parallel runners.
  `cargo nextest --partition slice:N/D` splits by test *index*, which is
  uncorrelated with duration: measured across run 32591020809 (2895 tests, 4704
  seconds of work over eight partitions), index order gave an 870-second worst
  partition against a 588-second ideal, so the critical path waited on one
  machine while the other seven idled. The same pattern appeared inside a
  partition — the quarter of tests finishing last averaged 4.31 seconds against
  0.70 for the quarter finishing first. `scripts/plan-test-partition.rs` now
  assigns each recorded test longest-first onto the emptiest partition, reaching
  588 seconds with a 0.2% spread, and the `check_test_partition_balance` gate
  fails if the plan drifts back out of balance. Tests with no recorded duration
  still go through nextest's own index split, so a new test runs exactly once
  without a re-recording.

- Record the rule in `CONTRIBUTING.md` for every future fan-out: start the
  longest work first, and do not throttle CI parallelism. A long task started
  last runs alone on a machine everything else has already finished waiting for.
