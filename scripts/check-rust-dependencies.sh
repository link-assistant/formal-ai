#!/usr/bin/env bash
# Audit the committed Cargo.lock, and hold every ignored advisory to a proof.
#
# Issue #1017: the rust pipeline template audits its lockfile on every push;
# this repository audited only its JavaScript locks, so a RustSec advisory in a
# transitive Rust dependency went unreported. An audit is only worth adding if
# it stays honest, and the honest-audit failure mode is a permanent `ignore`
# entry that outlives its reason. So each ignore in `.cargo/audit.toml` carries
# a machine-checkable proof line
#
#     # <ADVISORY-ID> unreachable = "<crate>@<version>"
#
# and this script re-derives that proof with `cargo tree --invert` before
# handing the lockfile to `cargo audit`. The ignore expires by itself: it fails
# the moment the crate enters the build graph, and it fails just as loudly once
# the vulnerable version leaves the lockfile and the entry is merely stale.

set -euo pipefail

config=".cargo/audit.toml"
lock="Cargo.lock"

if [[ ! -f "$config" ]]; then
  echo "::error::$config is missing; cargo audit would silently lose its ignore proofs"
  exit 1
fi

# The ignored advisory IDs, one per line: everything quoted inside the
# `ignore = [...]` array, which may be written on one line or across several.
ignored="$(
  awk '
    /^[[:space:]]*ignore[[:space:]]*=/ { collecting = 1 }
    collecting {
      line = $0
      sub(/#.*/, "", line)
      while (match(line, /"[^"]+"/)) {
        id = substr(line, RSTART + 1, RLENGTH - 2)
        print id
        line = substr(line, RSTART + RLENGTH)
      }
      if (index($0, "]")) { collecting = 0 }
    }
  ' "$config"
)"

status=0
while IFS= read -r advisory; do
  [[ -n "$advisory" ]] || continue

  spec="$(
    sed -n "s/^#[[:space:]]*${advisory}[[:space:]]\+unreachable[[:space:]]*=[[:space:]]*\"\([^\"]\+\)\".*/\1/p" \
      "$config" | head -n 1
  )"
  if [[ -z "$spec" ]]; then
    echo "::error::$config ignores $advisory without a proof line."
    echo "  Add: # $advisory unreachable = \"<crate>@<version>\""
    echo "  An advisory that cannot be proven unreachable must be fixed, not ignored."
    status=1
    continue
  fi

  echo "Proving $advisory is unreachable: $spec"
  if ! tree="$(
    cargo tree --locked --target all --all-features --edges all --invert "$spec" 2> /dev/null
  )"; then
    echo "::error::$config ignores $advisory for \`$spec\`, which is no longer in $lock."
    echo "  The ignore is stale; drop it (and its proof line) from $config."
    status=1
    continue
  fi
  if [[ -n "${tree//[[:space:]]/}" ]]; then
    echo "::error::$spec is now reachable from the build graph, so $advisory applies."
    echo "$tree"
    echo "  Upgrade the dependency instead of ignoring the advisory."
    status=1
  fi
done <<< "$ignored"

if [[ "$status" -ne 0 ]]; then
  exit "$status"
fi

echo "Auditing $lock"
cargo audit --file "$lock"
