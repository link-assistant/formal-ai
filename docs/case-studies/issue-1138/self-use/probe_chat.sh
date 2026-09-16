#!/usr/bin/env bash
# In-process probe: run each prompt of a cases.tsv through `formal-ai chat`
# (the `solver::solve` entry point the HTTP surface uses) and record the answer
# next to the Agent CLI transcript. This is the library-level half of a wave F
# observation; `run_self_use_batch.sh` is the client-level half.
#
# Usage: probe_chat.sh <cases.tsv> <out-root>

set -uo pipefail

BIN="${BIN:-/Users/konard/Code/Archive/link-assistant/formal-ai/target/release/formal-ai}"
CASES="$1"
OUT_ROOT="$2"

while IFS=$'\t' read -r slug lang prompt; do
  [ -z "${slug:-}" ] && continue
  case "$slug" in \#*) continue ;; esac
  dest="$OUT_ROOT/$slug/$lang"
  mkdir -p "$dest"
  printf '%s' "$prompt" > "$dest/prompt.txt"
  echo "== chat $slug/$lang =="
  timeout 180 "$BIN" chat --silent --prompt "$prompt" < /dev/null \
    > "$dest/chat-answer.txt" 2>&1
  echo "chat_exit=$?" >> "$dest/run.env"
done < "$CASES"
