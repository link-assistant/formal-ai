#!/usr/bin/env sh
# Exit 0 only when a ref mentioning `issue-<N>` appeared DURING the current task.
#
# The obvious check, `git branch -a | grep -q "issue-846"`, is a false green in
# any clone that has fetched the remote: `-a` includes remote-tracking refs, so
# a branch someone else pushed months ago scores as work this agent just did.
# On the machine that first exposed this, seven of sixteen L1 tasks "passed"
# that way -- including the very branch the measurement was being run from.
#
# run_coding_ladder.sh snapshots every ref into .branches-before immediately
# before launching a task; a pass therefore requires a ref that is not in that
# snapshot.
set -u
here=$(dirname "$0")
before="$here/.branches-before"
[ -s "$before" ] || exit 1
git for-each-ref --format='%(refname)' \
  | grep "issue-$1" \
  | grep -qvxF -f "$before"
