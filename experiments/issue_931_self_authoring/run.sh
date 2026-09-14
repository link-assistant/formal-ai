#!/usr/bin/env bash
# Real Formal-AI-server -> Agent-CLI authorship proof for one issue #931 leaf.

set -euo pipefail

ROOT="$(cd "$(dirname "$0")/../.." && pwd)"
ARTIFACT_DIR="$ROOT/docs/case-studies/issue-931/self-hosting-authorship"
CANONICAL="$ROOT/data/meta/local-transport-protocol.lino"
TASK='Finish Formal AI issue #931 by defining the local transport protocol shared by its WebSocket and WebRTC adapters. As one smallest leaf of that same task, create file local-transport-protocol.lino containing exactly
local_transport_protocol
  record_type "local_transport_protocol"
  default_host "127.0.0.1"
  request_shape "api_transport_request"
  response_shape "api_transport_response"
  router "handle_api_request_with_headers"
local_transport_websocket
  record_type "local_transport_binding"
  transport "websocket"
  server_flag "--ws"
  client_value "websocket"
  framing "rfc6455_text"
local_transport_webrtc
  record_type "local_transport_binding"
  transport "webrtc"
  server_flag "--webrtc"
  client_value "webrtc"
  signaling "length_prefixed_json"
  data_channel "formal-ai"
  ice_policy "host_only"
  central_relay "false"
local_transport_storage
  record_type "local_transport_invariant"
  storage_engine "shared_memory_store"
  new_storage_engine "false"
  permission_router "shared_api_router"'

TASK="$TASK" \
EXPECT_FILE="local-transport-protocol.lino" \
EXPECT_TEXT='central_relay "false"' \
MIN_POSTS=3 \
ATTEMPTS=3 \
PORT="${PORT:-8931}" \
ARTIFACT_DIR="$ARTIFACT_DIR" \
"$ROOT/experiments/agent_cli_e2e/run_agent_cli.sh"

cp "$ARTIFACT_DIR/local-transport-protocol.lino" "$CANONICAL"
cmp "$ARTIFACT_DIR/local-transport-protocol.lino" "$CANONICAL"
grep -m1 -o 'ses_[A-Za-z0-9]*' "$ARTIFACT_DIR/agent-cli.log"
