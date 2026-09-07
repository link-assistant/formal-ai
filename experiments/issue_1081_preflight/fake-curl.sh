#!/usr/bin/env bash
# A `curl` stand-in for exercising scripts/preflight-credentials.sh offline.
#
# Issue #1081: the preflight's whole job is to interpret registry status codes,
# so its tests have to be able to say "ghcr answers 403" without a network. Each
# line of $FAKE_CURL_ROUTES is "<url-substring> <status> [body]"; the first
# matching line wins, and every invocation is appended to $FAKE_CURL_LOG.
#
# The preflight passes its credentials in a `-K -` config document rather than
# in `-H` arguments, so this stub reads stdin and records it in
# $FAKE_CURL_CONFIG_LOG -- kept apart from $FAKE_CURL_LOG so a test can assert
# both halves of that property: the header was sent, and it was not in argv.
set -uo pipefail

out=""
dump=""
url=""
args=("$@")
i=0
while [ $i -lt ${#args[@]} ]; do
  case "${args[$i]}" in
    -o) i=$((i + 1)); out="${args[$i]}" ;;
    -D) i=$((i + 1)); dump="${args[$i]}" ;;
    http*) url="${args[$i]}" ;;
  esac
  i=$((i + 1))
done

config=""
if [ ! -t 0 ]; then
  config="$(cat)"
fi

if [ -n "${FAKE_CURL_LOG:-}" ]; then
  printf '%s\n' "$*" >> "$FAKE_CURL_LOG"
fi

if [ -n "${FAKE_CURL_CONFIG_LOG:-}" ]; then
  printf '%s\n' "$config" >> "$FAKE_CURL_CONFIG_LOG"
fi

status=000
body=""
while IFS= read -r route; do
  [ -n "$route" ] || continue
  pattern="${route%% *}"
  rest="${route#* }"
  case "$url" in
    *"$pattern"*)
      status="${rest%% *}"
      if [ "$rest" != "$status" ]; then body="${rest#* }"; fi
      break
      ;;
  esac
done <<< "${FAKE_CURL_ROUTES:-}"

if [ -n "$out" ] && [ -n "$body" ]; then printf '%s' "$body" > "$out"; fi
if [ -n "$dump" ]; then printf 'HTTP/1.1 %s\r\nLocation: /v2/uploads/session-1\r\n' "$status" > "$dump"; fi
printf '%s' "$status"
