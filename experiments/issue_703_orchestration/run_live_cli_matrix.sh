#!/usr/bin/env bash
# Exercise the public issue-703 command through every installed real client.
#
# This is intentionally opt-in: it launches authenticated third-party CLIs and
# may consume vendor quota. Recorded scripted sessions cover deterministic CI;
# maintainers set FORMAL_AI_ISSUE_703_LIVE=1 for the real-TUI compatibility gate.

set -euo pipefail

if [ "${FORMAL_AI_ISSUE_703_LIVE:-0}" != "1" ]; then
  echo "skip: set FORMAL_AI_ISSUE_703_LIVE=1 to launch the six real clients"
  exit 0
fi

ROOT="$(cd "$(dirname "$0")/../.." && pwd)"
BIN="${BIN:-$ROOT/target/debug/formal-ai}"
KEEP_MATRIX_ROOT="${KEEP_MATRIX_ROOT:-0}"
CLIS=(agent claude codex gemini qwen opencode)
created_matrix_root=0
if [ -z "${MATRIX_ROOT:-}" ]; then
  MATRIX_ROOT="$(mktemp -d "${TMPDIR:-/tmp}/formal-ai-issue703-live.XXXXXX")"
  created_matrix_root=1
fi

cleanup() {
  if [ "$created_matrix_root" = "1" ] && [ "$KEEP_MATRIX_ROOT" != "1" ]; then
    case "$MATRIX_ROOT" in
      */formal-ai-issue703-live.*) rm -rf -- "$MATRIX_ROOT" ;;
      *) echo "refusing to remove unexpected matrix root: $MATRIX_ROOT" >&2 ;;
    esac
  fi
}
trap cleanup EXIT

mkdir -p "$MATRIX_ROOT"
if [ "${FORMAL_AI_ISSUE_703_CODEX_UNSANDBOXED:-0}" = "1" ]; then
  # Codex's workspace-write sandbox requires user namespaces. Some CI
  # containers block them even though the whole job is already externally
  # sandboxed. This explicit gate keeps the product default intact while still
  # exercising the real Codex binary in that environment.
  export FORMAL_AI_REAL_CODEX
  FORMAL_AI_REAL_CODEX="$(command -v codex)"
  shim_dir="$MATRIX_ROOT/client-shims"
  mkdir -p "$shim_dir"
  printf '%s\n' \
    '#!/usr/bin/env bash' \
    'set -euo pipefail' \
    'args=()' \
    'while [ "$#" -gt 0 ]; do' \
    '  if [ "$1" = "--sandbox" ] && [ "${2:-}" = "workspace-write" ]; then' \
    '    args+=(--sandbox danger-full-access)' \
    '    shift 2' \
    '  else' \
    '    args+=("$1")' \
    '    shift' \
    '  fi' \
    'done' \
    'exec "$FORMAL_AI_REAL_CODEX" "${args[@]}"' \
    > "$shim_dir/codex"
  chmod +x "$shim_dir/codex"
  PATH="$shim_dir:$PATH"
  export PATH
  echo "note codex: native sandbox disabled inside the externally sandboxed live gate"
fi

for cli in "${CLIS[@]}"; do
  command -v "$cli" >/dev/null || {
    echo "fail $cli: executable not found" >&2
    exit 1
  }
  workspace="$MATRIX_ROOT/$cli"
  mkdir -p "$workspace"
  printf '# Live orchestration fixture\n' > "$workspace/README.md"

  "$BIN" agent run \
    --cli "$cli" \
    --task "add a README badge" \
    --workspace "$workspace" \
    --session "$workspace/session.json"

  grep -F '"status": "succeeded"' "$workspace/session.json" >/dev/null
  grep -F '"path": "README.md"' "$workspace/session.json" >/dev/null
  grep -F 'img.shields.io' "$workspace/README.md" >/dev/null
  echo "ok $cli: real client changed README.md and the session recorded the effect"
done

echo "ok live matrix: ${CLIS[*]}"
if [ "$KEEP_MATRIX_ROOT" = "1" ]; then
  echo "artifacts: $MATRIX_ROOT"
fi
