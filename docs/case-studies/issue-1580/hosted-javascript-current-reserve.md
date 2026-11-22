# Current hosted JavaScript scheduling reserve

[Layered run 45011615137](https://github.com/link-assistant/formal-ai/actions/runs/45011615137)
fails at published 79a024bffdcee2dd94149d929e81b4b65496ed2d: shards one
and four reject stale TypeScript projections. Shards seven and eight separately
take 283 and 302 seconds. The unchanged 270-second warnings occur
at complete raw-log lines 6595 and 3693. All eight wall times and all 915 complete
file observations remain retained. The file without its own summary stays
unobserved; its fallback is not presented as a measured duration.

Replaying these costs against the old plan reproduces the excessive model.
The new ninth scheduling regression fails before repair. Importing these costs
with the prior 35% reserve fixes this cohort but fails the older bbc cohort.
A finite search permits five candidates, stops at the first passing candidate,
and actually tests 36%, 38% and 40%. The first two fail the older replay; 40%
passes all nine retained timing cases and all 27 queue controls.

The maintained duration merger raises 118 floors, lowers none and
preserves every existing receipt/source/run/job field exactly. Eight shards,
four workers, the 360-second execution budget, the 270-second warning and the
separate 30-second runner allowance remain unchanged. Current measured costs
peak at 224.13 modeled seconds plus that allowance.
This is a retained-cost replay; actual wall time of the new plan awaits CI.

The archive under checks/current-javascript-scheduling-repair retains the full
source log, all timing observations, prior data, rejected candidates, regression
failure, selected-case passes and the complete local gate result: 89 pass, 53
explicit hosted-only exclusions, zero failures. Required documentation, Rust
2024 formatting and whitespace checks also pass; no native compilation ran. Failed drafts
remain evidence. Supplied procedures and data imports earn zero independent
authorship credit. Wider acceptance for [PR #1581](https://github.com/link-assistant/formal-ai/pull/1581)
remains unfinished.

The earlier supplied fixture writer incorrectly hardcoded `runConclusion: success`
while already retaining two failed job conclusions. GitHub's complete run response,
job responses, check annotations and raw assertions agree on failure. The correction
retains the old archive, derives the current conclusion from the fresh response,
and preserves all 915 exact measured durations while restoring their actual file
results: 913 completed and two failed. The new provenance assertions fail before
this correction and pass afterward. Both failed test files reproduce before
canonical TypeScript regeneration and pass afterward; the runtime test executes
the actual TypeScript worker/planner and its missing/altered projection controls.
See current-terminal-and-typescript-repair.md for complete terminal collection.
