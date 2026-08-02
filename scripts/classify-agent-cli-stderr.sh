#!/usr/bin/env bash
set -euo pipefail

input="${1:?usage: classify-agent-cli-stderr.sh STDERR_LOG}"
system_warning='AI SDK Warning: System messages in the prompt or messages fields can be a security risk because they may enable prompt injection attacks. Use the system option instead when possible. Set allowSystemInMessages to true to suppress this warning, or false to throw an error.'
compat_warning='AI SDK Warning (opencode.chat / big-pickle): The feature "specificationVersion" is used in a compatibility mode. Using v2 specification compatibility mode. Some features may not be available.'
system_count=0
compat_count=0
unexpected=0

while IFS= read -r line || [[ -n "$line" ]]; do
  case "$line" in
    "") ;;
    "$system_warning") system_count=$((system_count + 1)) ;;
    "$compat_warning") compat_count=$((compat_count + 1)) ;;
    *)
      printf '%s\n' "$line" >&2
      unexpected=1
      ;;
  esac
done < "$input"

if (( unexpected != 0 )); then
  echo 'unexpected @link-assistant/agent stderr; refusing to hide a new diagnostic' >&2
  exit 1
fi

if (( system_count + compat_count > 0 )); then
  echo "::notice title=Known Agent CLI 0.25.0 diagnostics::Classified ${system_count} system-message and ${compat_count} compatibility warning(s); tracked upstream at https://github.com/link-assistant/agent/issues/279"
fi
