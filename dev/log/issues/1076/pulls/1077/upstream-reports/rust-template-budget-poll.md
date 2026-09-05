# Upstream report 1 — `run-with-budget-warning.sh` counts loop iterations, not elapsed time

**Target:** `link-foundation/rust-ai-driven-development-pipeline-template`
**File:** `scripts/run-with-budget-warning.sh` (line 81)
**Severity:** the script silently stops enforcing anything under one documented
configuration, and is systematically late under all others.

---

## Title

`run-with-budget-warning.sh` counts poll iterations instead of elapsed time: a
non-integer `BUDGET_POLL_SECONDS` disables enforcement entirely

## Summary

`scripts/run-with-budget-warning.sh` exists to make a step own its deadline, so
that an overrun is reported as `failure` rather than the `cancelled` that
`timeout-minutes` produces. Its header states this in as many words:

> a job killed by `timeout-minutes` is reported as 'cancelled' and hides the
> failure, while this reports 'failure'

The deadline is tracked by accumulating the *configured* poll interval once per
loop iteration:

```bash
68: elapsed=0
...
80:   sleep "${POLL_SECONDS}"
81:   elapsed=$(( elapsed + POLL_SECONDS ))
```

`$(( ... ))` is integer-only arithmetic. `BUDGET_POLL_SECONDS` is a documented
knob (line 18: `BUDGET_POLL_SECONDS  - poll interval (default 1)`) and a poll
interval is the one setting an operator would naturally set below 1 second, but
any non-integer value makes line 81 a hard bash arithmetic error. Because the
script runs under `set -uo pipefail` — deliberately without `-e` — the error is
printed to stderr and the loop continues with `elapsed` frozen at `0`. The
budget can then never expire, and the guarded command runs to completion.

Separately, and for *every* value of `BUDGET_POLL_SECONDS`, `elapsed` undercounts
real time: it adds the sleep duration but not the cost of the loop body itself
(one `kill -0` fork per iteration, plus scheduling latency). The deadline
therefore fires late, and both the `::warning` (line 78) and the `::error`
(line 93) report the nominal budget rather than the time actually consumed.

## Reproduction

Self-contained; no CI runner required. Only bash and the script are needed.

```bash
curl -fsSLO https://raw.githubusercontent.com/link-foundation/rust-ai-driven-development-pipeline-template/main/scripts/run-with-budget-warning.sh
chmod +x run-with-budget-warning.sh

# A 2-second budget guarding a command that sleeps for 10.
for poll in 1 0.5; do
  echo "--- BUDGET_POLL_SECONDS=$poll ---"
  start=$SECONDS
  BUDGET_POLL_SECONDS="$poll" ./run-with-budget-warning.sh 2 probe sleep 10
  echo "    exit=$? wall=$((SECONDS - start))s"
done
```

Observed:

```
--- BUDGET_POLL_SECONDS=1 ---
::error title=probe exceeded its execution budget::probe was terminated after 2s. ...
    exit=124 wall=3s

--- BUDGET_POLL_SECONDS=0.5 ---
./run-with-budget-warning.sh: line 81: 0.5: syntax error: invalid arithmetic operator (error token is ".5")
    (repeated once per poll)
    exit=0 wall=10s
```

The second case is the defect: a command that blew through a 2-second budget by
5x exits `0`, the step passes, and no annotation is emitted.

Drift, measured separately with the default poll interval:

```bash
for i in 1 2 3; do
  start=$SECONDS; ./run-with-budget-warning.sh 20 probe sleep 60 >/dev/null 2>&1
  echo "run$i exit=$? wall=$((SECONDS - start))s"
done
# run1 exit=124 wall=22s
# run2 exit=124 wall=21s
# run3 exit=124 wall=20s
```

0-2s late on a 20s budget (0-10%), varying with machine load: the three runs
above differ only in what else the box was doing at the time. `kill -0` is a
bash builtin, so the loop body itself is nearly free; the drift is `sleep`'s
overshoot, which is a scheduling effect and therefore grows exactly when the
runner is contended, i.e. in the conditions the budget exists to catch. The
annotation reports "20s" in all three cases regardless of what was actually
consumed.

This half of the report is a papercut, not an outage; see the impact table.


## Impact assessment (deliberately not overstated)

The drift alone does not currently defeat any budget in this template. The
declared budget/cap pairs are all at or below 67% of their job cap:

| Job | Budget | Job `timeout-minutes` | Share | Share at +10% drift |
|---|---|---|---|---|
| `fresh-merge` | `FRESH_MERGE_BUDGET_SECONDS: 1200` (L223) | 30 (L189) | 67% | 73% |
| `test` | `TEST_BUDGET_SECONDS: 900` (L473) + `DOC_TEST_BUDGET_SECONDS: 300` (L481) | 30 (L413) | 67% | 73% |
| `coverage` | `COVERAGE_BUDGET_SECONDS: 600` (L596) | 15 (L548) | 67% | 73% |

So with the default poll interval the step still fails before the job cap, and
the mechanism still yields `failure` rather than `cancelled`. What the drift
costs is part of the safety margin the 67% ratio was chosen to provide, and the
accuracy of the annotation an operator uses to decide whether to raise a budget.
Worth fixing, but not urgent.

The non-integer poll interval, by contrast, is a complete loss of enforcement.

## Workaround

Do not set `BUDGET_POLL_SECONDS` to a non-integer value. There is no way to get
sub-second polling from the current implementation.

## Suggested fix

Read the clock instead of counting iterations. Bash's `SECONDS` is monotonic
within the shell, needs no subprocess, and makes both the enforcement and the
annotations reflect real elapsed time. `POLL_SECONDS` then only feeds `sleep`,
which already accepts fractions, so sub-second polling starts working as
documented.

```diff
@@
-elapsed=0
+started=$SECONDS
 warned=false
 timed_out=false
 while kill -0 "${command_pid}" 2>/dev/null; do
+  elapsed=$(( SECONDS - started ))
   if [ "${elapsed}" -ge "${BUDGET}" ]; then
     timed_out=true
     break
   fi
   if [ "${warned}" = false ] && [ "${elapsed}" -ge "${warn_at}" ]; then
     warned=true
     echo "::warning title=... ${elapsed}s of its ${BUDGET}s budget ..."
   fi
   sleep "${POLL_SECONDS}"
-  elapsed=$(( elapsed + POLL_SECONDS ))
 done
```

and report the measured overrun in the terminal annotation, so the number an
operator acts on is the real one:

```diff
-  echo "::error title=${LABEL} exceeded its execution budget::${LABEL} was terminated after ${BUDGET}s. ..."
+  echo "::error title=${LABEL} exceeded its execution budget::${LABEL} was terminated after $(( SECONDS - started ))s, exceeding its ${BUDGET}s budget. ..."
```

Optionally also validate the knob so a typo fails loudly rather than silently,
mirroring the existing `BUDGET` validation at lines 37-40:

```bash
case "$POLL_SECONDS" in
  '' | *[!0-9.]* | *.*.*) echo "BUDGET_POLL_SECONDS must be a positive number, got: ${POLL_SECONDS}" >&2; exit 2 ;;
esac
```

## Regression test

```bash
# Enforcement must not depend on the poll interval.
for poll in 1 0.5 0.25; do
  BUDGET_POLL_SECONDS="$poll" ./scripts/run-with-budget-warning.sh 2 probe sleep 10
  [ "$?" -eq 124 ] || { echo "FAIL: poll=$poll did not enforce the budget"; exit 1; }
done
```

## Prior art

`link-assistant/formal-ai` carries a variant of this script that reads
`$SECONDS` (`started=$SECONDS` before the loop, `elapsed=$((SECONDS - started))`
inside it) and is not affected: the same reproduction yields `exit=124 wall=2s`
under `BUDGET_POLL_SECONDS=0.5`. Found while comparing the two implementations
for link-assistant/formal-ai#1076.
