#!/usr/bin/env bash
# Audit every committed JavaScript lockfile from one fail-closed CI gate.

set -euo pipefail

audit_locks() {
  local lock_name="$1"
  shift
  local -a audit_command=("$@")
  local lock
  local workspace

  while IFS= read -r lock; do
    # A deleted-but-unstaged file can remain in the local index. CI never has
    # that state, but skipping it keeps this check useful while preparing a
    # commit that removes or renames a lock.
    [[ -f "$lock" ]] || continue
    workspace="$(dirname "$lock")"
    echo "Auditing $lock"
    (
      cd "$workspace"
      "${audit_command[@]}"
    )
  done < <(git ls-files | awk -F/ -v name="$lock_name" '$NF == name')
}

audit_locks "bun.lock" bun audit --audit-level=moderate
audit_locks "package-lock.json" npm audit --package-lock-only --audit-level=moderate
