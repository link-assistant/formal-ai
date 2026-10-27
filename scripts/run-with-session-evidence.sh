#!/usr/bin/env bash
# Streaming supervisor retains complete independent stdout/stderr and process custody.
set -euo pipefail
exec node "$(dirname "$0")/capture-draft-command.mjs" "$@"
