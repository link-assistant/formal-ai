#!/usr/bin/env bash
# Shared loopback publisher and explicit teaching input for the #707 harnesses.

setup_issue_707_fixture() {
  local asset_dir="$ROOT/rust/tests/fixtures/computer-use"
  python3 "$asset_dir/serve.py" >"$WORKDIR/publisher.log" &
  PUBLISHER_PID=$!
  for _ in {1..100}; do
    [[ ! -s "$WORKDIR/publisher.log" ]] || break
    kill -0 "$PUBLISHER_PID" 2>/dev/null || return 1
    sleep 0.05
  done
  IFS= read -r FORMAL_AI_COMPUTER_USE_HTTP_ORIGIN <"$WORKDIR/publisher.log" || return 1
  [[ "$FORMAL_AI_COMPUTER_USE_HTTP_ORIGIN" =~ ^http://127[.]0[.]0[.]1:[0-9]+$ ]] || return 1
  export FORMAL_AI_COMPUTER_USE_HTTP_ORIGIN
  COMPUTER_USE_CORPUS="$WORKDIR/computer-use-corpus.lino"
  sed "s|fixture://|$FORMAL_AI_COMPUTER_USE_HTTP_ORIGIN/|g" "$asset_dir/tasks.lino" >"$COMPUTER_USE_CORPUS"
}
