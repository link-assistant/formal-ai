#!/usr/bin/env bash
set -u
run_case() {
  local script="$1" poll="$2" tag="$3" start e n
  cat > "child-$tag.sh" <<EOF
#!/usr/bin/env bash
# Genuinely ignores SIGTERM: stays in bash so the trap remains installed.
trap 'echo "child-$tag ignored SIGTERM"' TERM
end=\$((SECONDS + 600))
while [ "\$SECONDS" -lt "\$end" ]; do read -r -t 1 _ </dev/null 2>/dev/null || :; done
EOF
  chmod +x "child-$tag.sh"
  start=$SECONDS
  BUDGET_GRACE_SECONDS=3 BUDGET_POLL_SECONDS="$poll" \
    ./"$script" 2 probe "./child-$tag.sh" >"out-$tag.txt" 2>&1
  e=$?
  n=$(pgrep -f "child-$tag" | wc -l)
  printf "%-28s poll=%-4s exit=%-4s wall=%2ss survivors=%s\n" \
    "$script" "$poll" "$e" "$((SECONDS-start))" "$n"
  pkill -9 -f "child-$tag" >/dev/null 2>&1 || true
}
run_case budget-js.sh              1   jsA
run_case budget-js.sh              0.5 jsB
run_case run-with-budget-warning.sh 1  rustA
run_case budget-python.sh          1   pyA
exit 0
