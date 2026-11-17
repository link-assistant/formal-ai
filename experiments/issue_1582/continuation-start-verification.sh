#!/usr/bin/env bash
set -eu
setsid bash experiments/issue_1582/continuation-finish-verification.sh > /tmp/issue-1582-review/finish-verification.log 2>&1 < /dev/null &
echo "$!" > /tmp/issue-1582-review/finish-verification.pid
printf 'Verification continues in the background; inspect the exit marker before proceeding.\n'
