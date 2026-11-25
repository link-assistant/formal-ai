# Corpus gate OOM and full-test shard timeouts

Runs analysed: Benchmark Corpus Gate 45636781444 and release (full tests)
45636781871, both on `main` 513e4d9. The binaries come from run 45636781871's
artifacts. Every local probe ran under `ulimit -v` and `timeout`. Peak RSS was
read from `getrusage(RUSAGE_CHILDREN)`, which reports the largest single
descendant.

## A. Benchmark Corpus Gate: runner out of memory

### Timeline (`ci-logs/corpus-jobs/Corpus_shard_{1,2,6}.log`)

All three shards were killed while answering the same coding prompt, `near_pair`
("Return whether at least one distinct pair has a difference that is less than
the threshold"), each in its own language.

| Shard | Prompt | Started | Memory before | Memory last seen | Killed (exit 143) |
|---|---|---|---|---|---|
| 1 | 77/234, ru `near_pair` | 15:19:04 | 1169 MiB (15:19:25) | 15421 MiB used, 280 MiB available (15:19:55) | 15:19:56 |
| 6 | 76/233, en `near_pair` | 15:18:57 | 1157 MiB (15:19:12) | 15683 MiB used (15:19:42) | 15:20:01 |
| 2 | 77/233, hi `near_pair` | 15:10:56 | — | 1503 MiB (15:11:45), then no more samples | 15:12:11 |

At 15:58:09 the `corpus-gate` aggregate reported "not every shard succeeded".

### Reproduction

Network denied against live network, through the CLI (`formal-ai chat --prompt <near_pair en>`):

| Mode | Elapsed | Peak RSS | Answer |
|---|---|---|---|
| `FORMAL_AI_NETWORK=denied` | 5.96 s | 112.2 MiB | skill-gap answer; concept discovery finds nothing, so no draft is verified |
| default (live) | 23.96 s | 113.8 MiB | skill-gap answer |

The CLI stays small because it does not verify drafts the way the gate's engine
path does.

To reproduce the gate's own path, I ran the `integration` binary with its baked
repository path byte-patched to a scratch root. That root held only the
`near_pair` corpus plus padding:

```
(ulimit -v 2000000; FORMAL_AI_CORPUS_TRACE=1 timeout 150 \
  patched/integration issue_1138_no_silent_unknown --nocapture --test-threads=1)
```

- Result: killed by `timeout` (exit 124) at 100–150 s, peak 1927–1956 MiB.
- The `python3 solution.py` children of `compose_with_ir` grew from 309 to 1161
  MiB before the address-space limit stopped them.
- I captured two of the drafts being verified:

```python
def near_pair(numbers, threshold):
    return itertools.combinations(itertools.combinations(itertools.combinations(
        itertools.combinations(itertools.combinations(itertools.combinations(numbers, 2), 2), 2), 2), 2), 2)
```

The other draft nests one level deeper, seven in total.

### Root cause

1. **Network decides whether anything is verified.** With network allowed,
   concept discovery yields the structures `quantifier_any`, `pairwise_distinct`
   and `predicate_abs_diff_lt`. These derive four examples, so `compose` verifies
   every draft that `search_with_structures` (`SearchBounds::default()`, depth 8)
   produces. With network denied there are no candidates and nothing runs.
2. **The search nests a size-amplifying fragment inside itself.**
   `pairwise_distinct` (`itertools.combinations({items}, 2)`) has the type
   `sequence<T> -> sequence<pair<T,T>>`. The result is again a sequence, so the
   type checker lets the search apply it to its own output.
   - Each level squares the element count: over 4 inputs the levels hold 6, 15,
     105, 5460 and 14.9 M elements, then about 1e14.
   - The 6-deep draft needs about 1.15 GB (measured: 1151 MiB over 9.34 s).
   - The 7-deep draft is unbounded. Several shards do this at once, which is the
     15 GB seen on CI.
3. **Verification had no memory bound.** `compose_with_ir` had a 60 s time floor
   but no address-space cap. One bad candidate therefore takes the whole runner
   down instead of failing that one attempt.

This is a false negative of the gate: the job dies before it measures anything.
It is not a prompt that reaches the unknown opener.

### Fix (working tree)

- `rust/src/coding/composition_search/arguments.rs` adds
  `amplifies_its_own_output`. `rust/src/coding/composition_search.rs` skips any
  argument tuple for which it returns true.
  - It rejects a fragment applied, anywhere inside a sequence-typed argument, to
    its own output, but only when the fragment's result type is strictly larger
    than that slot's type (`type_size`). That is the doubly exponential case.
  - Size-preserving self-nesting is still searched, for example
    `list(list(x))` or `tuple(...)`. So is self-nesting through scalar slots.
- `data/seed/coding-discovery-runtime.lino` adds a `python_memory_guard`
  template.
  - It is a guarded `resource.setrlimit(RLIMIT_AS, ...)`, grounded in the
    Python `resource` documentation, licensed PSF-2.0.
  - It has no `fragment_signature`, so the search never treats it as a fragment.
- `rust/src/coding/composition/verification.rs` inserts the guard right after
  the `if __name__ == '__main__':` entrypoint of every Function-shaped draft,
  with `VERIFICATION_ADDRESS_SPACE_BYTES = 512 MiB`.
  - An exploding draft now fails its own attempt with `MemoryError`.
  - On platforms without `resource` the guard does nothing.

Guard check: I rendered the scripts exactly as `verify` builds them and ran each
under `ulimit -v 1500000`.

| Draft | Result | Time | Peak RSS |
|---|---|---|---|
| a correct solution | exit 0 | 0.04 s | 12.5 MiB |
| 6-deep `pairwise_distinct` | `MemoryError` | 1.62 s | 495.8 MiB |
| 7-deep `pairwise_distinct` | `MemoryError` | 1.59 s | 499.0 MiB |

Before the guard, the 6-deep draft reached 1151 MiB and the 7-deep draft was
unbounded.

### Regression tests

- `rust/tests/unit/coding_discovery/composition_search.rs` adds
  `an_amplifying_fragment_is_never_applied_to_its_own_output`.
  - It runs `search_with_structures` on the `near_pair` spec with the three
    discovered structures and the default bounds.
  - It asserts `pairwise_distinct` is used somewhere, so the test is not
    vacuous, and that no program nests it inside itself (walking every
    `IrNode` variant).
  - Before the fix, the search emitted the 6- and 7-deep nestings captured
    above, so this assertion fails.
- `rust/tests/unit/coding_discovery/composition/rejected_ir.rs` adds the
  Linux-only test
  `a_draft_that_outgrows_the_verification_memory_cap_fails_with_memory_error`.
  - It verifies the 6-deep IR through `compose_with_ir` and asserts the attempt
    fails with `MemoryError` in its detail.
  - Before the fix, the same script fails with `AssertionError` after mapping
    about 1.15 GB, which I measured with the Python simulation above.

## B. Full-test shards timing out (`TEST_BUDGET_SECONDS=1080`)

### What ran too long (run 45636781871 shard logs)

These figures are libtest's 60 s warnings: time since start, or time until
termination for tests still running.

| Shard | Still running when the budget killed it | Recorded weight |
|---|---|---|
| 1/8 | `coding_research_production::…cold_held_out_typescript…` 1055 s+, `meta_executor_census::every_specification_prompt…` 160 s+ | unrecorded under its current name |
| 3/8 | `issue_1562_cited_programs::the_three_reports…` 1025 s+ | unrecorded |
| 6/8 | `issue_1138_family_migration::held_out_family_paraphrases…` 602 s+ | 51 s |
| 6/8 | `issue_701_learning_adoption::the_adoption_ledger_records…` 418 s+ | 11.3 s |
| 6/8 | `issue_1138_intent_phrase_migration::held_out_web_search_paraphrases…` 335 s+ | 1.0 s |
| 6/8 | `issue_699_handler_migration::held_out_number_constraint_paraphrases…` 327 s+ | 1.1 s |

On shard 6 no test finished between 16:17:54 and the kill at 16:23:19. The
planner had predicted 846 s for every shard.

### Root cause: stale shard weights, not the Task A explosion

The planner (`scripts/plan-test-shards.mjs`) orders tests longest-first by
`data/meta/test-durations.lino`. The weights were recorded on 2026-10-08 from
run 37802763478 and have not been regenerated since, because `--report-time`
recording is off.

Since then:
- 50 entries went stale when the `agentic_coding` module was flattened
  (`agentic_coding::agentic_coding::X` became `agentic_coding::X`).
- Another 50 entries name tests that no longer exist. For example, the 101 s
  `repository_workflows_include_runtime_setup_and_output_verification` was
  split into one test per language.
- The heaviest new agentic tests are unrecorded and weigh 0.092 s.
- Tests that took about 1 s now take 15–44 s.

The planner therefore puts the long tests last and badly underpredicts the
shards.

Measured locally with the CI binary (`unit --exact`, 4 threads, bounded):

| Test | Default network | `FORMAL_AI_NETWORK=denied` |
|---|---|---|
| number_constraint paraphrases | 16.82 s | 15.37 s |
| web_search paraphrases | 43.74 s | 24.41 s |

- The four shard-6 tail tests together took 56.5 s at a peak of 144 MiB, with
  no deadlock.
- Live network adds about 19 s to web_search.
- The Task A explosion is not involved: the composition tests run at their
  October times.
- On CI the same tests ran 327 s+, which a local run does not reproduce. Runner
  contention, together with the poor placement, is the likely multiplier. This
  part is unverified.

### Fix (working tree): `data/meta/test-durations.lino`

- I renamed the 50 entries for moved tests and dropped the 50 that name no
  listed test.
- For every test that run 45636781871 reported running past 60 s, the weight
  is now max(recorded, observed lower bound): 31 entries added and 25 raised,
  653 in total.
- The header states the source (`lower_bound_run 45636781871`) and the caveat:
  these are lower bounds under 4-thread contention, and `regenerate` replaces
  them. `data/meta/coverage-test-weights.lino` already handles 60 s warnings
  the same way.
- Checks:
  - The planner now predicts 2504 s of serial weight per shard over 8 shards,
    which shows the 1080 s budget is too tight at 4 threads.
  - `node scripts/plan-test-shards.mjs --check`, `coverage-execution.test.mjs`
    (15/15) and `check-ci-speed.mjs` pass.
  - `ci-speed.test.mjs` has 4 failures from
    `experiments/issue-1580/javascript-capacity-replay.test.mjs`. They come from
    the untracked `rust/tests/web/issue-1639-e2e-harness-contracts.test.mjs`
    being absent from the JS test inventory, not from this change.

## Verification status

- **Rust edits:** `rustfmt --edition 2024 --check` is clean. I did not run
  `cargo build` or `cargo test`, because cargo runs out of memory in the 3 GB
  sandbox. CI has to compile them.
- **Python guard:** verified by running the rendered scripts (table above).
- **Durations file:** verified by the planner and coverage checks above.
- **"Fails before the fix":** argued for the MemoryError test from the
  measurement. For the search test, argued from the captured drafts. Neither has
  been run against the unpatched binary.

## Narrowest PR-lane check

```
cargo test --test unit -- --exact \
  coding_discovery::composition_search::an_amplifying_fragment_is_never_applied_to_its_own_output \
  coding_discovery::composition::rejected_ir::a_draft_that_outgrows_the_verification_memory_cap_fails_with_memory_error
```

Both modules are also compiled into `--test coding_contracts`.

End to end, the Benchmark Corpus Gate already runs on this PR because it
touches `rust/src/**`. It is the check that must turn green: every shard,
including the `near_pair` prompts at position 76–77 of each shard.

## Proposed workflow changes

Applied later in this PR: item 2's network half (13d08a486, `FORMAL_AI_NETWORK:
denied` in the full and specification shards) and a durations refresh
(b17063b44). The corpus gate keeps its network access: the product fix
(0e5ed502a) bounds the search and the verification memory, so the gate keeps
measuring the discovery path users get. Items 1 and 3 are not applied.

1. `benchmark-corpus-gate.yml`, corpus-shard step:
   - Set `FORMAL_AI_NETWORK: denied`, or use a pinned discovery cache, so the
     gate measures the repository rather than what the network returned that
     day.
   - Wrap the test in a memory cap, for example
     `prlimit --as=$((6*1024**3)) --` or `systemd-run --user --scope -p
     MemoryMax=6G`. A runaway then fails one shard with a named prompt instead
     of killing the runner silently.
2. `release.yml`, full tests:
   - Turn on `FORMAL_AI_RECORD_TEST_TIMES` periodically on `main` (for example
     weekly) and commit the regenerated `test-durations.lino`.
   - Until then, raise the shard count or the `TEST_BUDGET_SECONDS` headroom.
     The serial prediction is now about 2500 s per shard.
3. Make `plan-test-shards.mjs --check` fail when:
   - a durations entry names no listed test, or
   - a listed test warned past 60 s on `main` but weighs the default.

   Either way, staleness then fails the check itself rather than turning up as
   a shard timeout.
