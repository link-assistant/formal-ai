# Historical scheduling comments

These comments were extracted from the complete immutable e462 prior-floor fixture
without changing any scheduling value. The active file keeps the current policy;
the fixture and Git history retain every earlier import and annotation.

```text
# Scheduling weights retain prior floors and reserve 65% over run 45018199621.
# The fallback is rounded upward to the same tenth-second grid; all ten replays fit.
# Scheduling reserve over run 45011615137: 40%; earlier floors and receipt identities are preserved.
# Scheduling weights also reserve 35% over run 45006097030 file timings.
# Exact timings, unchanged limits and the prior floor are retained in
# experiments/issue-1580/fixtures/hosted-javascript-bbc0ac4d/.
# Scheduling weights retain prior floors and reserve 20% over run 44999044965 file timings.
# The allowance is not a measured duration; exact timings and headroom policy are retained in
# experiments/issue-1580/fixtures/hosted-javascript-a558f834/.
# Seconds per JavaScript suite file under rust/tests/web/, the input of the
# longest-first shard plan of the Layered CI js tier (PR #1188). Generated;
# Original records were measured with each file alone. Later records explicitly
# identify shared-shard observations; weights guide scheduling, not latency SLAs.
  # observed-run: Layered CI runs whose js-tier `[test-file]` seconds were folded in (issue #1571).
  # Both split dialogue owners retain the parent aggregate upper bound, not measured child durations.
```
