#!/usr/bin/env bash
# Trim already-captured `server-tail.log` files to their last 128 KB.
#
# `FORMAL_AI_TRACE_REQUESTS=1` echoes whole request bodies, so an untrimmed
# trace runs to megabytes per prompt. The trim is stated inside each file, so a
# reader never mistakes a trimmed trace for a complete one. The Agent CLI
# transcript (`agent.log`) is the primary record and is never trimmed.
#
# Usage: trim_server_traces.sh <root>

set -uo pipefail
ROOT="${1:-docs/case-studies/issue-1138/self-use}"

find "$ROOT" -name server-tail.log -print0 | while IFS= read -r -d '' file; do
  size="$(wc -c < "$file" | tr -d ' ')"
  [ "${size:-0}" -le 131072 ] && continue
  tmp="$(mktemp)"
  {
    echo "# server trace truncated to its last 128 KB (was $size bytes)"
    tail -c 131072 "$file"
  } > "$tmp"
  mv "$tmp" "$file"
  echo "trimmed $file ($size -> $(wc -c < "$file" | tr -d ' '))"
done
