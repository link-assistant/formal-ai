#!/usr/bin/env bash
set -euo pipefail

printf '%s\n' '{"type":"error","sessionID":"ses_issue921_failure","error":"issue 921 injected agent failure"}'
exit 23
