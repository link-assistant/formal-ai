#!/usr/bin/env bash
# Re-check every committed transcript under `recorded/`, offline.
#
#   experiments/agentic_cli_matrix/replay.sh            # all recorded clients
#   experiments/agentic_cli_matrix/replay.sh claude     # one client
#
# This needs no CLI installed, no server, no network and no vendor credentials —
# only `jq`. It is what makes the recorded sessions issue #671 asks for actually
# *replayable* rather than merely archived: the transcript-level invariants each
# case asserts live are re-asserted here against the committed evidence, so a
# change that would break a leg is caught even on a machine where that leg's CLI
# cannot be installed at all.
#
# What it cannot do is re-derive the CLI's own rendered output; those assertions
# only exist in the live leg. The split is deliberate and stated so nobody reads
# a green replay as a green matrix.

source "$(cd "$(dirname "$0")" && pwd)/lib.sh"

command -v jq > /dev/null 2>&1 || {
  echo "!! replay needs jq" >&2
  exit 1
}

RECORDED_DIR="${RECORDED_DIR:-$MATRIX_DIR/recorded}"
MODEL="${MODEL:-formal-ai}"
FAILED=0

# The same bounds the live cases use. A recorded transcript that would fail its
# own case must fail here too, otherwise committing it would launder the failure.
replay_bound() {
  case "$1" in
    greeting) echo 4 ;;
    read-file) echo 12 ;;
    summarize) echo 6 ;;
    interactive) echo 12 ;;
    constraints) echo 12 ;;
    *) echo 20 ;;
  esac
}

replay_fail() {
  echo "!! $*" >&2
  FAILED=1
}

# What a transcript must contain depends on the *shape* of the leg that recorded
# it, and the case name is that shape: only a `cli`-shape case puts a prompt
# through our server, so only its transcript can carry model rounds.
#
# The distinction is read from the file name rather than from the client
# registry on purpose — replay must work with jq alone, on a machine where
# neither the binary nor any CLI is installed.
replay_shape() {
  case "$1" in
    launch) echo launch ;;
    mcp) echo mcp ;;
    *) echo cli ;;
  esac
}

# Every exchange must carry the model *as provenance*, not merely somewhere in
# the URL. Gemini names the model in the path rather than the body
# (`/v1beta/models/formal-ai:streamGenerateContent`), which is why `formal-ai
# proxy` recovers it from there; asserting on `request_model` alone is therefore
# also a regression test for that recovery, and matching the path here would
# hide it.
replay_names_model() {
  jq -es --arg model "$1" 'any(.[]; .request_model == $model)' "$2" > /dev/null
}

replay_transcript() {
  local file="$1" client="$2" case_name="$3" shape rows all bad
  shape="$(replay_shape "$case_name")"
  all="$(wc -l < "$file" | tr -d ' ')"
  rows="$(jq -r 'select(.path != "/health") | .path' "$file" | wc -l | tr -d ' ')"

  # Common to every shape: nothing our server answered may have failed, and no
  # bodies may be committed — bodies carry the run's temp paths and session ids,
  # so a transcript holding them churns on every re-record.
  bad="$(jq -r 'select(.status >= 400) | "\(.status) \(.path)"' "$file" | head -3)"
  [ -z "$bad" ] || {
    replay_fail "$client/$case_name: transcript records failing exchanges: $bad"
    return
  }
  jq -es 'any(.[]; has("request_body") or has("response_body"))' "$file" > /dev/null \
    && {
      replay_fail "$client/$case_name: transcript still carries request/response bodies"
      return
    }

  case "$shape" in
    cli)
      [ "$rows" -ge 1 ] || {
        replay_fail "$client/$case_name: transcript has no model rounds"
        return
      }
      [ "$rows" -le "$(replay_bound "$case_name")" ] || {
        replay_fail "$client/$case_name: $rows rounds exceed the $(replay_bound "$case_name")-round bound"
        return
      }
      replay_names_model "$MODEL" "$file" || {
        replay_fail "$client/$case_name: no exchange named model '$MODEL'"
        return
      }
      echo "ok $client/$case_name: $rows rounds replayed"
      ;;
    mcp)
      # We are the tool server here, so there is no model round to replay: what
      # the transcript proves is that the client's JSON-RPC surface was reached
      # through the proxy. The handshake and the tool call themselves are
      # re-asserted live in `case_mcp`; only their reachability is archived.
      local mcp_rows
      mcp_rows="$(jq -r 'select(.path == "/mcp") | .path' "$file" | wc -l | tr -d ' ')"
      [ "$mcp_rows" -ge 3 ] || {
        replay_fail "$client/$case_name: only $mcp_rows /mcp exchanges recorded — expected the initialize, tools/list and tools/call round trips"
        return
      }
      replay_names_model "$MODEL" "$file" && {
        replay_fail "$client/$case_name: an MCP transcript named model '$MODEL' — this client is supposed to drive its own model, so the leg is no longer testing the MCP surface"
        return
      }
      echo "ok $client/$case_name: $mcp_rows /mcp exchanges replayed"
      ;;
    launch)
      # A launch leg starts a GUI or a server and proves — live, from `/proc` —
      # that the wrapper's configuration reached the application. Nothing about
      # that is replayable offline: a GUI issues no model request until a human
      # types, so this transcript is a record of what the client touched on
      # startup, which is legitimately just our readiness probe.
      #
      # Asserting "at least one model round" here is what made every launch
      # transcript red; asserting nothing would be a green that means nothing.
      # What is genuinely checkable is that the stack came up and nothing the client
      # touched on startup failed (checked above).
      [ "$all" -ge 1 ] || {
        replay_fail "$client/$case_name: empty transcript — the stack never came up"
        return
      }
      jq -es 'any(.[]; .path == "/health")' "$file" > /dev/null || {
        replay_fail "$client/$case_name: no /health row — the recorded run never reached a ready server"
        return
      }
      echo "ok $client/$case_name: startup replayed ($rows non-probe exchanges; live leg asserts the process configuration)"
      ;;
  esac
}

targets=("$@")
if [ ${#targets[@]} -eq 0 ]; then
  mapfile -t targets < <(ls "$RECORDED_DIR" 2> /dev/null)
fi
[ ${#targets[@]} -gt 0 ] || {
  echo "!! no recorded transcripts under $RECORDED_DIR" >&2
  exit 1
}

for client in "${targets[@]}"; do
  found=0
  for file in "$RECORDED_DIR/$client"/*.jsonl; do
    [ -f "$file" ] || continue
    found=1
    name="$(basename "$file" .jsonl)"
    replay_transcript "$file" "$client" "$name"
  done
  [ "$found" -eq 1 ] || replay_fail "$client: no recorded transcripts"
done

[ "$FAILED" -eq 0 ] || {
  echo "!! replay failed" >&2
  exit 1
}
echo "== replay OK =="
