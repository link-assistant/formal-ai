# Hosted JavaScript scheduling constraints

Run [45240466225](https://github.com/link-assistant/formal-ai/actions/runs/45240466225)
passed on source e46254ac53c360e4c45652fa7abd63fb0d9ecad7 and tested merge
158c4151a2c66ecc5871d0370f11e19ef5e1dd9f. Groups 9 and 7 crossed the
unchanged 270-second warning threshold, taking 306.819501889 and
273.048657371 seconds. Its complete fixture retains all 922 file observations,
both warnings, nine original job identifiers, log line numbers, the run response
and prior scheduling floor. The complete log SHA-256 remains
a0892fbafe15c724eba662993c4d1db18889731e04b06c7d9030182d97750df1.

The original, second and third dialogue owners took 47.474293328,
36.605133486 and 124.65305008 seconds. All 432 maintained observation cases
passed. These results qualify that source, not a later scheduling proposal.

## Rejected uniform reserve

The initial repair refreshed every observation with a uniform 65% reserve.
Eleven historical runtime replays and four dialogue ownership cases passed;
all 143 local gate entries finished, with 89 passes and 54 explicit skips.
Those checks missed existing provenance-ordering and conservative queue
assertions in `rust/tests/web/ci-speed.test.mjs`.

Hosted [Layered CI 45250430676](https://github.com/link-assistant/formal-ai/actions/runs/45250430676)
exposed both failures on source 53816c9eb4bd11729a8be347ccd9497d07042660.
The complete log reports the provenance assertion at line 9314 and the
queue-budget assertion at line 9333; the maintained entry repeats both failures.
The uniform reserve moved Fresh Planner above Full Provenance and produced
conservative queues of approximately 397 seconds against a 360-second budget.
An unchanged local run reproduces 25 passes and two failures. The historical
fixture's `schedulingFactor: 1.65` records this rejected proposal.

## Bounded owner allowance

The corrected probe changes only the three dialogue owners. Every other
explicit weight and the 0.4-second fallback equal the immutable prior floor.
Each owner keeps its 284-second family floor and adds 32% of that owner's
observed duration, rounded upward to a tenth of a second. The resulting
scheduling estimates are 299.2, 295.8 and 323.9 seconds. They are estimates,
not measured child runtimes.

The finite probe evaluates 66 allowance candidates from zero through 65% in
one-percentage-point steps. It selects the largest candidate satisfying every
existing scheduling constraint: exact coverage, deterministic longest-first
ordering, provenance separation and priority, the reasoner's heavy shard,
the behavior suite's first worker wave, three distinct dialogue shards,
four-worker conservative queues below 360 seconds, and all eleven historical
runtime replays at or below their unchanged 270-second warning threshold.
The selected maximum conservative queue is 359.2 seconds. The latest complete e462 cohort
replays at 264.920880627 seconds including its unchanged 30-second allowance;
the largest historical replay is 269.317165196 seconds.

The wider experiments are retained: importing every new duration while keeping
all old floors already requires more than the total worker budget without any
reserve. Replacing ordinary estimates can fit, but would discard existing floor
contracts. The deployed repair preserves those contracts and the original
assertions instead.

## Reproduction and verification

Run these checks sequentially:

- `node --test --test-concurrency=1 rust/tests/web/ci-speed.test.mjs`
- `node --test --test-concurrency=1 experiments/issue-1580/hosted-javascript-scheduling.test.mjs`
- `node --test --test-concurrency=1 experiments/issue-1580/cnl-dialogue-partition.test.mjs`

The first check passes all 28 cases, including a new fallback regression.
A known-file-only or empty duration import previously discarded an explicit
fallback; the importer now preserves it. The new case fails before that fix.
All original scheduling assertions remain unchanged. The twelve historical
checks include a fresh finite probe that reproduces the deployed policy and
weights byte for byte. All four dialogue ownership cases pass.

The failed source also retains 819 observed durations (including the failed
CI-speed file), 103 unexecuted files and the group 5 runtime warning at log line
11211. Its raw Meta Reasoner owner took 254.556480921 seconds; the ladder
case alone took 148.209078777 seconds. Repartitioning files cannot satisfy
the fixed warning-plus-overhead model for that owner.

The original fifteen callbacks now run in two bounded isolated processes:
fourteen learning cases retain their original order and one owns the ladder.
The child runner keeps every original assertion, skip control and capture hash.
Its per-test deadline is 210 seconds, outer deadline 225 seconds and each
process heap is capped at 192 MiB. Four partition regressions verify byte-equal
callback registration, failure propagation, the complete partial-run boundary
and the original over-threshold observation. The actual owner passes locally
in 157.190 seconds: thirteen original cases pass and both existing skips remain.
All 438 maintained observation cases also pass in 215.317 seconds. These runs use
the real committed captures and browser worker; no Rust build runs.

The latest partial timing model uses complete prior observations for the
103 unexecuted files. For the changed Meta Reasoner owner it uses the maximum
of the two original case-group sums, the original setup residual and a separate
five-second process-startup allowance. The original 254.6-second file timing
remains unchanged in the fixture. This explicit changed-source model is
not hosted execution evidence. The finite probe checks it alongside all eleven
historical cohorts.

Moving the case definitions exposes a stale R1017 test-title citation in the
existing requirement-coverage gate. Its unchanged check reports one broken
citation before the documentation repair. The authoritative requirement shards
now cite the actual learning and ladder owners, and the assembled projection is
regenerated with the maintained JavaScript tool. Regenerating the coverage report
then restores its original bytes, preserving all 120 implemented verdicts and
both existing reviewed historical citations. The full gate replay preserves 88 passes,
54 explicit CI/Rust skips and this one failure. After the citation repair, all ten
affected documentation and generation gates pass without changing their checks.
The separate five-command requirement check also detects two stale generated ledger
shards. The maintained generator refreshes only their test-owner references; all
five checks then pass, with no verdict or status-surface changes.

Historical scheduling comments and earlier work-plan observations are retained
in separate readable files. Active data and plan files are below the unchanged
1,400-line warning threshold.

These are local checks and historical models. Exact-source hosted qualification
of the repair remains pending. The nine shards, four workers, warning threshold,
deadline, runner allowance and functional test coverage remain unchanged.
This does not complete broader template adoption, production delivery,
upstream dependency work or the original issue. [PR #1581](https://github.com/link-assistant/formal-ai/pull/1581)
remains draft and unmerged.
