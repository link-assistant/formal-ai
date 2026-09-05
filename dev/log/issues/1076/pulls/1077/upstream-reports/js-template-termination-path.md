# Upstream report 3 - the over-budget termination path never escalates to SIGKILL

**Target:** `link-foundation/js-ai-driven-development-pipeline-template`
**File:** `scripts/run-with-budget-warning.sh` (lines 82-103)
**Severity:** a step that blows its budget can leave the process tree running on
the runner - the exact outcome the script's own header says it exists to prevent.

---

## Title

`run-with-budget-warning.sh`: liveness is tracked on the wrapper subshell, so a
command that ignores SIGTERM is never SIGKILLed and survives the step

## Defect 1 - SIGKILL escalation never fires (at every poll interval)

The command is started inside a brace group so its exit status can be recorded:

```bash
64: {
65:   "$@"
66:   command_status=$?
67:   printf '%s\n' "${command_status}" > "${status_file}.partial"
68:   mv "${status_file}.partial" "${status_file}"
69: } &
70: command_pid=$!
```

so `command_pid` is the **wrapper subshell**, not the user's command. Liveness
is then tested against that pid:

```bash
82: command_is_running() {
83:   [ ! -f "${status_file}" ] && kill -0 "${command_pid}" 2>/dev/null
84: }
```

`signal_command TERM` signals the whole process group. The wrapper subshell
installs no TERM trap, so it dies immediately -- even when the user's command
ignores SIGTERM and keeps running. `kill -0 "${command_pid}"` then fails,
`command_is_running` reports false, and both the grace loop and the escalation
that follows it are skipped:

```bash
91:   while command_is_running && [ "${waited}" -lt "${grace_seconds}" ]; do
...
96:   if command_is_running; then
97:     echo "${label} ignored SIGTERM after ${grace_seconds}s; sending SIGKILL."
98:     signal_command KILL
99:   fi
```

The surviving process tree is never killed. The script's own header states this
is the failure it exists to prevent:

> `npm test` and `bun test` spawn workers, and killing only the direct child
> leaves orphans holding the runner -- which is also why `timeout(1)` is not
> sufficient here.

### Reproduction

Write a child that ignores SIGTERM (staying in bash, so the trap keeps
applying -- an `exec sleep` would drop it):

```bash
cat > child.sh <<'CHILD'
#!/usr/bin/env bash
trap 'echo "child ignored SIGTERM"' TERM
end=$((SECONDS + 600))
while [ "$SECONDS" -lt "$end" ]; do read -r -t 1 _ </dev/null 2>/dev/null || :; done
CHILD
chmod +x child.sh

BUDGET_GRACE_SECONDS=3 ./scripts/run-with-budget-warning.sh 2 probe ./child.sh
echo "exit=$?"
pgrep -f child.sh | wc -l      # survivors
```

Observed:

```
Running probe with a 2s budget (warning at 1s).
::warning title=probe is approaching its execution budget::probe has run for 1s of its 2s budget.
::error title=probe exceeded its execution budget::probe did not finish within its 2s budget and was terminated. ...
child ignored SIGTERM
exit=124
1                              <- still running
```

Note what is *absent*: the `probe ignored SIGTERM after 3s; sending SIGKILL.`
line, and any wait for the grace period -- the whole run takes ~3s, not
2s + `BUDGET_GRACE_SECONDS`.

The rust and python templates carry sibling scripts that do escalate correctly
under the identical test; they track the command itself rather than a wrapper:

| template | exit | wall (2s budget, 3s grace) | survivors |
|---|---|---|---|
| js | 124 | 3s | **1** |
| rust | 124 | 5s | 0 |
| python | 124 | 6s | 0 |

### Suggested fix

Track the command's own pid and keep the status file only as the completion
signal it already is. `set -m` is already in place, so the command can be
backgrounded directly and its status collected with `wait`:

```diff
 set -m
-{
-  "$@"
-  command_status=$?
-  printf '%s\n' "${command_status}" > "${status_file}.partial"
-  mv "${status_file}.partial" "${status_file}"
-} &
-command_pid=$!
+"$@" &
+command_pid=$!
 set +m
```

with `command_is_running` now meaningful:

```bash
command_is_running() { kill -0 "${command_pid}" 2>/dev/null; }
```

and the final status taken from `wait "${command_pid}"`, which is what the rust
template already does. If the status file must be kept for the zombie-reaping
reason given in the comment, then poll the *process group* instead:

```bash
command_is_running() {
  [ ! -f "${status_file}" ] && kill -0 -- "-${command_pid}" 2>/dev/null
}
```

## Defect 2 - `waited` accumulates a possibly non-integer poll interval

```bash
93:     waited=$((waited + poll_seconds))
```

`BUDGET_POLL_SECONDS` is documented as "polling interval while the command runs
(default 1)" -- a knob an operator would naturally set below 1. `$(( ))` is
integer-only, so any fractional value is a bash arithmetic syntax error. The
failed expansion aborts `terminate_over_budget` before its closing `exit 124`,
and the script falls through to the normal completion path:

```
$ BUDGET_POLL_SECONDS=0.5 ./scripts/run-with-budget-warning.sh 2 probe sleep 10
...
./scripts/run-with-budget-warning.sh: line 93: 0.5: syntax error: invalid arithmetic operator (error token is ".5")
probe finished in 2s of its 2s budget (exit 143).
$ echo $?
143
```

The step still fails, so this is a contract violation rather than a loss of
enforcement -- but the header promises "124 on timeout (matching `timeout(1)`)",
and anything downstream that distinguishes a budget overrun from a crash by exit
code gets the wrong answer. The message on the normal path ("finished in 2s of
its 2s budget") also describes a command that was in fact killed.

### Suggested fix

The deadline loop already uses bash's `SECONDS` correctly (line 107 `SECONDS=0`,
line 116 `[ "${SECONDS}" -ge "${budget_seconds}" ]`). Use the same clock in the
grace loop rather than accumulating the poll interval:

```diff
-  local waited=0
-  while command_is_running && [ "${waited}" -lt "${grace_seconds}" ]; do
-    sleep "${poll_seconds}"
-    waited=$((waited + poll_seconds))
-  done
+  local grace_deadline=$(( SECONDS + grace_seconds ))
+  while command_is_running && [ "${SECONDS}" -lt "${grace_deadline}" ]; do
+    sleep "${poll_seconds}"
+  done
```

This is what the python template's sibling script already does
(`grace_deadline=$((SECONDS + grace_seconds))`).

Optionally validate the knob so a typo fails loudly, mirroring the existing
`budget_seconds` validation:

```bash
case "$poll_seconds" in
  '' | *[!0-9.]* | *.*.*) echo "BUDGET_POLL_SECONDS must be a positive number, got: ${poll_seconds}" >&2; exit 2 ;;
esac
```

## Workaround

For defect 2, leave `BUDGET_POLL_SECONDS` at its default. Defect 1 has no
caller-side workaround.

## Regression test

```bash
# A command that ignores SIGTERM must still be killed, and must not outlive the step.
BUDGET_GRACE_SECONDS=2 ./scripts/run-with-budget-warning.sh 2 probe ./child.sh
[ "$?" -eq 124 ] || { echo "FAIL: wrong exit code"; exit 1; }
[ "$(pgrep -cf child.sh)" -eq 0 ] || { echo "FAIL: process survived the step"; exit 1; }

# Enforcement and exit code must not depend on the poll interval.
for poll in 1 0.5 0.25; do
  BUDGET_POLL_SECONDS="$poll" ./scripts/run-with-budget-warning.sh 2 probe sleep 10
  [ "$?" -eq 124 ] || { echo "FAIL: poll=$poll returned $?, expected 124"; exit 1; }
done
```

## Context

Found while comparing the three templates' budget wrappers for
link-assistant/formal-ai#1076. Related: js#137, which introduced the mechanism.
