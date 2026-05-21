#!/usr/bin/env bash
set -euo pipefail

status=0

require_command() {
  local command_name="$1"
  if ! command -v "$command_name" >/dev/null 2>&1; then
    echo "missing command: $command_name" >&2
    status=1
  fi
}

require_command formal-ai
require_command docker
require_command dockerd
require_command curl
require_command "$"

if [[ "${FORMAL_AI_IMAGE_VARIANT:-}" != "dind" ]]; then
  echo "FORMAL_AI_IMAGE_VARIANT must be dind, got '${FORMAL_AI_IMAGE_VARIANT:-}'" >&2
  status=1
fi

if [[ "${FORMAL_AI_START_ISOLATION:-}" != "docker" ]]; then
  echo "FORMAL_AI_START_ISOLATION must be docker, got '${FORMAL_AI_START_ISOLATION:-}'" >&2
  status=1
fi

if [[ "${FORMAL_AI_START_RUNNER:-}" != *"--isolated docker"* ]]; then
  echo "FORMAL_AI_START_RUNNER must use start-command Docker isolation" >&2
  status=1
fi

if [[ "$status" -ne 0 ]]; then
  exit "$status"
fi

formal-ai --version
"$" --version
docker --version
dockerd --version

if ! docker info >/tmp/formal-ai-docker-info.txt 2>&1; then
  cat /tmp/formal-ai-docker-info.txt >&2
  echo "inner Docker daemon is not reachable; run this through the DinD entrypoint" >&2
  exit 1
fi

"$" --isolated docker --auto-remove-docker-container -- echo formal-ai-dind-ok
