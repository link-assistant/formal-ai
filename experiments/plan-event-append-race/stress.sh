#!/bin/sh
# Runs the generated plan-event append command ($1, a file holding it) N times
# ($2, default 200) as two simultaneous processes in a fresh directory, and
# counts how often the event record ends up in the log more than once.
command_file=$1; runs=${2:-200}; shell=${3:-/bin/sh}
command=$(cat "$command_file")
duplicates=0; failures=0
i=0
while [ "$i" -lt "$runs" ]; do
  i=$((i + 1))
  directory=$(mktemp -d)
  mkdir "$directory/.formal-ai"; printf 'prior-event' > "$directory/.formal-ai/general-change-plan.lino"
  (cd "$directory" && "$shell" -c "$command" >/dev/null 2>&1) & first=$!
  (cd "$directory" && "$shell" -c "$command" >/dev/null 2>&1) & second=$!
  wait "$first" || failures=$((failures + 1)); wait "$second" || failures=$((failures + 1))
  count=$(grep -c '^general_change_plan$' "$directory/.formal-ai/general-change-plan.lino")
  [ "$count" -eq 1 ] || duplicates=$((duplicates + 1))
  rm -rf "$directory"
done
echo "runs=$runs duplicates=$duplicates failures=$failures shell=$shell"
