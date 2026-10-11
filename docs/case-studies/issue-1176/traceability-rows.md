# Issue #1176 Traceability Rows

Rows for the shard `docs/requirements/issue-1176-quantities-dates-statistics.md`,
kept here for the maintainer to merge into
`docs/requirements-traceability.md` (the shared table is not edited by this
change). Honesty rules follow the table's own header: `not yet confirmed`
means no manual confirmation has been recorded for this shard.

| ID | Shard | Delivered | Automated test | Manual confirmation |
| --- | --- | --- | --- | --- |
| R1176-1 | docs/requirements/issue-1176-quantities-dates-statistics.md | 2026-09-30 (this pull request) — exact decimal conversion with seed factors; live P2370 reads and full item resolution not delivered | rust/tests/unit/issue_1176_quantities_dates.rs (post-dispatch-wiring) | not yet confirmed |
| R1176-2 | docs/requirements/issue-1176-quantities-dates-statistics.md | 2026-09-30 (this pull request) — price×count change and total patterns with derivations; general relation extraction not delivered | rust/tests/unit/issue_1176_quantities_dates.rs (post-dispatch-wiring) | not yet confirmed |
| R1176-3 | docs/requirements/issue-1176-quantities-dates-statistics.md | 2026-09-30 (this pull request) — mean, median, mode, variance, standard deviation, range as seed meanings; percentile not delivered | rust/tests/unit/issue_1176_quantities_dates.rs (post-dispatch-wiring) | not yet confirmed |
| R1176-4 | docs/requirements/issue-1176-quantities-dates-statistics.md | 2026-09-30 (this pull request) — days and weeks offsets with the modulo-cycle derivation; month offsets and real-calendar date arithmetic not delivered | rust/tests/unit/issue_1176_quantities_dates.rs | not yet confirmed |
| R1176-5 | docs/requirements/issue-1176-quantities-dates-statistics.md | partial — derivations in every answer body and evidence log; #1184 structured derivations and three-roots parity not delivered | rust/tests/unit/issue_1176_quantities_dates.rs | not yet confirmed |
