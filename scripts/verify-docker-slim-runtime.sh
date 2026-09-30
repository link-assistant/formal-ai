#!/usr/bin/env bash
# Issue #1153: runtime verification for the slim image.
#
# The DinD variant has scripts/verify-docker-runtime.sh; this is its slim
# counterpart, installed as /usr/local/bin/verify-formal-ai-slim. It checks
# exactly what the Hive Mind sidecar contract needs: the binary answers
# --version, serve starts, and /health responds -- with no --privileged flag
# and no Docker engine inside the container.
set -euo pipefail

fail() {
  echo "verify-formal-ai-slim: $*" >&2
  exit 1
}

command -v curl >/dev/null || fail "curl is required for the health check"

formal-ai --version || fail "formal-ai --version did not answer"

port="${VERIFY_PORT:-18080}"
formal-ai serve --agent-mode --host 127.0.0.1 --port "$port" &
serve_pid=$!
trap 'kill "$serve_pid" 2>/dev/null || true' EXIT

for _ in $(seq 1 30); do
  if curl -fsS "http://127.0.0.1:${port}/health" >/dev/null 2>&1; then
    echo "verify-formal-ai-slim: /health answered on ${port}"
    exit 0
  fi
  sleep 1
done

fail "serve did not answer /health on ${port} within 30s"
