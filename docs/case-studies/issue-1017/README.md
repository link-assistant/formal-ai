# Issue #1017 — a deadline nobody owned

Issue: <https://github.com/link-assistant/formal-ai/issues/1017>

Pull request: <https://github.com/link-assistant/formal-ai/pull/1018>

## 1. Collected data

The canonical [evidence archive](../../../dev/log/issues/1017/pulls/1018/README.md)
holds complete logs for all ten workflow runs at `main` head `1858b338`, their
API metadata, the two jobs that decided the outcome isolated for line-level
citation, every annotation GitHub attached to any job (`all-annotations.tsv`),
every warning- or error-shaped line across all ten logs
(`analysis/soft-warnings.txt`, 20,766 lines), per-file diffs against the Rust
pipeline template, the Hive Mind CI/CD guidance, and immutable copies of all
three template trees.

The ten runs are the complete set for that head, taken from the API rather than
from the issue text, so a run the issue did not mention could not be missed.

## 2. Timeline

All ten runs were triggered by the same push: the merge of PR #1016 into `main`
at 08:47:28Z. Nine of them succeeded between 08:49:55Z and 09:13:08Z. `CI/CD
Pipeline` concluded `cancelled` at 09:18:39Z and the gated `Desktop Release`
reported `skipped` four seconds later.

Inside the cancelled run, one job decided everything —
`macOS Core Tests / Run macOS core slice 10/12`:

| Time (UTC) | Event |
| --- | --- |
| 08:59:31.7 | Job starts; the 600-second `timeout-minutes` clock starts here. |
| 08:59:31 → 09:01:44 | Checkout, toolchain, `nextest` install, archive download — **133 seconds outside any budget**. |
| 09:01:44.9 | The budgeted step starts. Its 480-second budget would expire at 09:09:44.9. |
| 09:09:43.6 | The runner kills the job at its cap — **1.3 seconds earlier**. |

## 3. Requirements

R1017-1 through R1017-12 cover root-cause repair of the cancelled run; making
"the budget expires before the cap" a checked invariant; classifying every
annotation and warning-shaped line; removing the security false negatives and
the security false positive; removing diagnostics manufactured by a run's own
cancellation; universal concurrency grouping; the full template and Hive Mind
comparison; upstream reporting; an off-by-default verbose mode; repository-wide
application of every fix; and delivery through PR #1018.

## 4. Root causes

**The deadline belonged to nobody.** `timeout-minutes` is a property of the
*job*, and it includes checkout, toolchain installation and artifact download.
The step's own budget started 133 seconds later and therefore could never
expire first. Because GitHub reports a `timeout-minutes` kill as `cancelled`
rather than `failed`, the failure mode silently degraded: on the default branch
`scripts/check-pipeline-status.sh` still turned it red, but on any other ref it
would have produced only a "superseded run" warning. That is the same false
negative as issue #977, one level down — and the general rule it establishes is
that **`timeout-minutes` is a backstop, never the deadline**.

The overrun itself came from partition skew: `cargo nextest --partition
slice:N/M` is round-robin by test *index* and never by duration, so slice 10
carried 467 seconds against a 185-second minimum across the twelve slices.

Two further defects of the same class were found by sweeping rather than by the
incident: a job at 1,415 seconds of a 1,500-second cap, and a job with no
`timeout-minutes` at all, silently inheriting GitHub's 360-minute default.

Beyond the timeout class, the sweep found a missing `cargo audit` on the
default branch (an advisory against an unchanged lockfile was invisible until
the next dependency bump), a `cargo audit` false positive caused by
`Cargo.lock` recording optional dependencies that no feature activates, a
broken-link error guarded by `always()` so a cancelled run could report links
it never checked, two jobs in no concurrency group at any level, and a
`.gitignore` negation that reached only one directory level and therefore
silently dropped every nested evidence log while `git add` reported success.

**What the fix uncovered underneath itself.** With the deadline structural, the
next run failed *cleanly* — and on something else entirely. Two macOS slices
panicked at `tests/integration/http_server.rs:185` with
`Os { code: 35, kind: WouldBlock }` after 30.08 s and 30.27 s: the harness's own
30-second per-request limit, to the millisecond. Both failing tests send the
**first** request to a freshly spawned server, and that request reached rule
recall, which built the canonical learning ledger before asking whether the
ledger could answer the prompt at all. Building it round-trips a 39 195-byte
module through `meta-language`'s parser, whose `point_at_byte` rescans the
source from byte 0 for every span — `O(nodes × bytes)`. Measured cost: over ten
seconds on the `dev` profile CI runs under, with 12 of 12 `gdb` samples inside
that one function. On a 3-core Intel runner with four tests in flight, that
constant crossed 30 s; which slices it hits depends on how the round-robin
partitioner happens to group the heavy tests, which is why it looked like
flakiness. It is the same false-negative shape as the rest of this issue — a
real ten-second first response that a "retry and it passes" reading would have
buried.

## 5. Research and prior art

GitHub documents the `cancelled`-not-`failed` behaviour as design; there is no
setting that changes it, which is why the deadline has to move into the step.
`timeout(1)` was considered and rejected as the implementation — it signals
only its direct child, and `cargo nextest` spawns a tree — so the wrapper uses
`set -m` for a dedicated process group with a SIGTERM → grace → SIGKILL
sequence, keeping exit code 124 to match `timeout`'s convention. `cargo
nextest` has no duration-aware partitioner (`hash:N/M` hashes the name), so
raising the slice count is the available lever.

All three `link-foundation` pipeline templates were compared file by file.
None has any step-execution-budget concept: every long step runs unbounded
under its job clock, so each is exposed to exactly this failure. That shared
defect was reported to all three with reproductions, workarounds and code-level
suggestions —
[rust#135](https://github.com/link-foundation/rust-ai-driven-development-pipeline-template/issues/135),
[js#137](https://github.com/link-foundation/js-ai-driven-development-pipeline-template/issues/137),
[python#60](https://github.com/link-foundation/python-ai-driven-development-pipeline-template/issues/60).

The 20,725 `macro expansion failed` diagnostics turned out not to be this
repository's defect at all. Every one of the 25 distinct failing macros is
defined in `std`/`core` or in a dependency; none is defined here. The CodeQL
Rust extractor resolves `std` from the runner's ambient toolchain using the
rust-analyzer it vendors, and cannot parse a `std` that new — the open upstream
issue `github/codeql#19982` (this repository's measurements are
[comment 5309221141](https://github.com/github/codeql/issues/19982#issuecomment-5309221141)),
whose query-side consequence is `github/codeql#22244`. This is an analysis-coverage false negative rather than
noise: a file whose macros do not expand is extracted with errors and its
bodies are not analysed, yet the run still reports success.

The quadratic parse is also someone else's defect, and the function is
byte-for-byte identical in the pinned 0.54.0 and the latest 0.58.1, so it was
reported with a standalone reproducer crate, both scaling tables, the `gdb`
attribution and a line-start-table patch that turns each lookup into
`O(log lines)`:
[meta-language#193](https://github.com/link-foundation/meta-language/issues/193).
No prior issue existed for it. Nothing in this repository can change the
algorithm — only stop calling it on a request path, which is what the fix does.

## 6. Tests-first reproduction

Thirteen tests in `tests/unit/ci-cd/issue_1017.rs` pin every fix, and each one
sweeps *all* workflows rather than the single file that failed — that is the
mechanism by which the same defect cannot survive elsewhere.

The runtime timeout is pinned separately by `tests/issue_1017_ledger_recall.rs`,
which counts whole-source parses through a process-wide counter and asserts an
ordinary request performs none. It is its own test binary precisely because the
counter is global. With the guard removed it fails (`left: 1, right: 0`), so it
reproduces the defect rather than describing it.

## 7. Implemented fix

`scripts/run-with-budget-warning.sh` now owns the deadline: it runs the command
in its own process group, warns at 70 % of the budget while the warning is
still actionable, then terminates the group and exits 124 with an
`::error … exceeded its execution budget` annotation, so an overrun reports
`failure` naming the budget it blew. `MAX_BUDGET_SHARE_PERCENT = 70` makes the
relationship between every budget and the cap above it a checked invariant.

The macOS core lane runs 16 slices with a 600-second budget under a 900-second
cap; the near-miss job's cap rose to 35 minutes so its 1,200-second budget
expires first; the untimed job received a 5-minute cap. `security.yml` gained a
`cargo audit` job on push, pull request and the weekly schedule, a manual
trigger, a CodeQL config excluding archived evidence, and the extractor sysroot
pin that restores macro expansion. `.cargo/audit.toml` ignores the one false
positive with a proof line that `scripts/check-rust-dependencies.sh` re-derives
from `cargo tree --invert` on every run, so the ignore expires by itself. The
link check tests its report parser before trusting it and no longer reports
broken links for a run it did not finish. Every read-only job now belongs to a
concurrency group that never cancels `main`.

On the runtime side, `learning_ledger::approved_lesson_for` now derives the
prompts the canonical ledger can possibly answer from the same canonical failure
trace the ledger is promoted from, and refuses a miss before building anything;
the match uses the same normalised form the ledger itself uses, so recall
behaviour is unchanged and no promotion gate is relaxed. The one round-trip
whose inputs are compile-time constants is memoised per process. A cold
`plan_chat_step` fell from 9.96–12.6 s to 579 ms and a cold POST from ~13 s to
274 ms. `FORMAL_AI_TRACE_SLOW_INIT=1`, off by default, reports each
whole-source parse with its size and duration so a regression names itself.

## 8. Verification

`cargo fmt --check`, `cargo clippy --lib --bins --tests --all-features`,
`cargo check --examples --all-features`, `rust-script scripts/check-file-size.rs`,
`rust-script scripts/check-hardcoded-language.rs`,
`bash scripts/lint-shell-scripts.sh`, and the full `ci_cd::` suite. The
budget wrapper's termination path is exercised directly rather than asserted
about: the test runs a command that outlives its budget and checks for exit
code 124 and the `::error` annotation.
