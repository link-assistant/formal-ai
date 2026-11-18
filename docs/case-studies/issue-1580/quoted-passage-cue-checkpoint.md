# Quoted-passage cue checkpoint

This is an unfinished checkpoint for [PR #1581](https://github.com/link-assistant/formal-ai/pull/1581).
The wider issue remains open.

## Reproduction and change

`Read this and answer: "Marisol owns pears and plums." Which fruit?` splits into
`Read this` and the remaining answer request. The quotation itself is protected,
but the cue's conjunction is treated as another task. Russian, Hindi and Spanish
cues have the same defect. Genuine preceding or following tasks expose an extra split.

Add 12 literal splitting/tree controls in five languages. Before the change,
the focused executor file has 18 passes and seven failures; the Chinese control
already passes. Preserve the entire 219-line failing payload and its hashes.
Escaped quote openers, word boundaries, cues inside quotations and genuinely
unquoted commands are negative controls.

Mask only seeded cue surfaces that directly introduce an actual quotation,
separated by whitespace or declared colon characters. Both runtimes reuse the
existing prompt-text-question vocabulary and the quote mask's escape/word rules.
Genuine tasks before and after the cue remain independent. A separate shared
12-case fixture records complete result and event parity. The original 25-case
and four-case fixtures remain byte-identical, for 41 replay cases in total.

## Verification and retained failures

All 25 executor tests and 15 adjacent factual-QA tests pass. Read the full failed
before payload, both full passing payloads, the handwritten source diff and all
477 lines of the new shared fixture. Preserve hashes and complete archived logs.
Rust 2024 formatting, fixture currency and maintained worker, TypeScript, seed
and translation generation pass. No local Rust build runs; native cue execution
passes on published head `7152daeb`, as recorded below.

The initial worker writer rejects the existing executor ceiling. Extract a small
helper; the next writer correctly requires its explicit budget. Register its
measured 46-line factory without raising any existing module ceiling. The next
translation writer refuses the new untracked helper; register the new sources
with intent-to-add, then run the maintained pinned translator successfully.
Preserve all three failed generation attempts separately from the corrections.
Projection verification reports 337 translated items, 37 workarounds, 2,592 carried
items and 112/162 translating modules (one fully), with 50 refused modules. The
new helper accounts for one additional refusal; this is not complete translation.

The full 138-entry local inventory records 83 passes, 52 explicit native/CI
exclusions and three failures: notation and package checks find the stale embedded
seed copy, while the status check finds a stale generated worker-budget count.
Preserve the aggregate and all 86 executable gate payloads. A correction guard
stops before mutation when the third failure expands its expected failure set.
Regenerate the package mirror and status surfaces with their maintained JavaScript
writers, then pass all three failed gates and the adjacent status twin. The 86
executable inventory entries therefore pass after correction; the original failed
aggregate remains unchanged. Memory events show zero OOMs or OOM kills.

The final whitespace check finds an extra EOF blank line in the helper and its
generated TypeScript copy. Preserve that failed receipt, remove only the source
EOF blank line and refresh maintained projections. The remeasured helper factory
contains 46 lines; retain its exact ceiling without raising any existing budget.
All five affected generated/translation/budget/status checks and whitespace pass.

The preceding synthesis correction on `c8fb9ad1` passes all 23 hosted workflows.
Its actual native log passes five executor tests, calendar/currency and forced
functions, plus both HTTP modes. Those executor results replay the previous 29
fixture cases; they do not execute this new 12-case cue fixture.

## Limits

These controls establish task boundaries using a deterministic table host. They
do not validate factual answers, bind passage references to later questions,
establish complete semantic parent coverage, or validate multi-passage evidence.
Those requirements and the wider issue remain unfinished. Supplied scripts,
commands, patches and conclusions receive zero independent authorship credit;
independent acceptance remains 0/20.

## Hosted cue execution

Published head `7152daeb69796ca0213df51b9dfc6a03460c2e80` passes all 23 workflows.
The [actual native job](https://github.com/link-assistant/formal-ai/actions/runs/44287034899/job/134743168987)
passes five executor tests, including all 41 shared cases and complete event
sequences, plus calendar/currency and forced functions: seven Rust tests and both
HTTP modes. JavaScript shards pass 1,440 and 1,268 tests, with four and 13 skips,
zero failures and zero cancellations. Complete downloaded logs, hashes and exact
literal result ranges remain under `experiments/issue-1580/evidence/ci/quoted-cues-7152daeb/`.
The fixture hashes match the published head. Full logs are retained; literal
review covers the recorded result windows, not every log line. This verifies
cue decomposition and replay, without completing the wider semantic contracts.
