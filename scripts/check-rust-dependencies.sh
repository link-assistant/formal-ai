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
#
# Issue #1079 added a second proof form, because not every advisory this
# repository must live with is unreachable. `fxhash` is genuinely compiled into
# this binary, through `web-capture -> scraper -> selectors`, and no change in
# this repository can take it out of the graph -- the fix is a version bump in
# a dependency's manifest. Ignoring it under the `unreachable` form would have
# meant writing a proof that is false. The honest form names the crate, names
# the upstream report that will remove it, and expires the same way:
#
#     # <ADVISORY-ID> blocked-upstream = "<crate>@<version>" report = "<url>"
#
# `unreachable` fails when the crate *is* in the build graph; `blocked-upstream`
# fails when it is *not*, which is exactly the moment the upstream fix has
# landed and the entry should go. Neither can be written to mean "ignore this
# forever".
#
# Issue #1079, defect D2: the audit itself now runs with `--deny warnings`.
# `cargo audit` classifies `unmaintained`, `unsound` and `yanked` findings as
# warnings, and a warning does not change its exit status -- it prints
# "warning: N allowed warnings found" and exits 0. This workflow was green for
# the whole time `Cargo.lock` pinned `chacha20 0.10.1`, a version its authors
# had yanked. Reproduce the difference with
# `experiments/issue-1079-cargo-audit-warning-exit-status.sh`.

set -euo pipefail

config=".cargo/audit.toml"
lock="rust/Cargo.lock"

if [[ ! -f "$config" ]]; then
  echo "::error::$config is missing; cargo audit would silently lose its ignore proofs"
  exit 1
fi

# Complete the NUL-delimited inventory before any Cargo command. A temporary
# file preserves exact Git paths and the producer status without loading the
# complete inventory into shell memory or losing errors in process substitution.
lock_inventory="$(mktemp)"
audit_log_dir="$(mktemp -d)"
trap 'rm -f -- "$lock_inventory"; rm -rf -- "$audit_log_dir"' EXIT

# cargo-audit 0.22.2 can print registry errors and still exit zero. Keep the
# complete original stream and require a complete observation. Retry only the
# positively identified transient yank-check failure, at most three times;
# ordinary findings retain their status and are never retried as transport errors.
run_audit() {
  local attempt observed_status errors
  for attempt in 1 2 3; do
    observed_status=0
    timeout 120 "$@" > "$audit_log_dir/output" 2>&1 || observed_status=$?
    cat "$audit_log_dir/output"
    sed -E $'s/\\x1b\\[[0-9;]*[[:alpha:]]//g' "$audit_log_dir/output" > "$audit_log_dir/plain"
    errors="$(grep -E '^[[:space:]]*error:' "$audit_log_dir/plain" || true)"
    if [[ -z "$errors" ]]; then return "$observed_status"; fi
    if [[ "$observed_status" -eq 0 ]]; then observed_status=1; fi
    if grep -Evq "^[[:space:]]*error: couldn't check if the package is yanked:" <<< "$errors" ||
       ! grep -Eq 'connection reset|connection refused|timed out|timeout|temporarily unavailable' "$audit_log_dir/plain"; then
      echo "::error::Cargo audit emitted an error; the dependency observation is incomplete." >&2
      return "$observed_status"
    fi
    if [[ "$attempt" -eq 3 ]]; then
      echo "::error::Cargo audit registry observation incomplete after 3 attempts." >&2
      return "$observed_status"
    fi
    echo "Cargo audit registry observation incomplete; retrying after attempt $attempt."
    sleep "$((attempt * 2))"
  done
}

git ls-files -z -- 'Cargo.lock' '*/Cargo.lock' > "$lock_inventory" || {
  status=$?
  echo "::error::Unable to enumerate Cargo lockfiles (git ls-files exited $status)." >&2
  exit "$status"
}
primary_tracked=0
each=''
while IFS= read -r -d '' each; do
  if [[ -z "$each" ]]; then
    echo "::error::invalid Cargo lockfile inventory: empty path record." >&2
    exit 1
  fi
  if [[ "$each" == "$lock" ]]; then primary_tracked=1; fi
done < "$lock_inventory"
if [[ -n "$each" ]]; then
  echo "::error::invalid Cargo lockfile inventory: missing NUL terminator." >&2
  exit 1
fi
if [[ "$primary_tracked" -ne 1 ]]; then
  echo "::error::$lock is not in the committed lockfile inventory." >&2
  exit 1
fi

# The index also contains newly staged files. A binary lock is reproducible
# from this revision only when it is already part of the HEAD commit.
if ! git cat-file -e "HEAD:$lock"; then
  echo "::error::$lock is not committed at HEAD." >&2
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
    sed -E -n "s/^#[[:space:]]*${advisory}[[:space:]]+unreachable[[:space:]]*=[[:space:]]*\"([^\"]+)\".*/\1/p" \
      "$config" | head -n 1
  )"
  blocked_spec="$(
    sed -E -n "s/^#[[:space:]]*${advisory}[[:space:]]+blocked-upstream[[:space:]]*=[[:space:]]*\"([^\"]+)\".*/\1/p" \
      "$config" | head -n 1
  )"
  report="$(
    sed -E -n "s/^#[[:space:]]*${advisory}[[:space:]]+blocked-upstream[[:space:]]*=.*[[:space:]]report[[:space:]]*=[[:space:]]*\"([^\"]+)\".*/\1/p" \
      "$config" | head -n 1
  )"

  if [[ -n "$spec" && -n "$blocked_spec" ]]; then
    echo "::error::$config proves $advisory both ways."
    echo "  An advisory is either unreachable or blocked upstream, never both."
    status=1
    continue
  fi

  if [[ -z "$spec" && -z "$blocked_spec" ]]; then
    echo "::error::$config ignores $advisory without a proof line."
    echo "  Add one of:"
    echo "    # $advisory unreachable = \"<crate>@<version>\""
    echo "    # $advisory blocked-upstream = \"<crate>@<version>\" report = \"<url>\""
    echo "  An advisory that can be neither proven unreachable nor traced to a"
    echo "  filed upstream report must be fixed, not ignored."
    status=1
    continue
  fi

  if [[ -n "$blocked_spec" ]]; then
    if [[ ! "$report" =~ ^https://github\.com/[^/]+/[^/]+/(issues|pull)/[0-9]+$ ]]; then
      echo "::error::$config ignores $advisory as blocked upstream without a filed report."
      echo "  Add: report = \"https://github.com/<owner>/<repo>/issues/<number>\""
      echo "  \"Someone should fix this upstream\" is not a report; a URL is."
      status=1
      continue
    fi
    echo "Proving $advisory is still blocked upstream: $blocked_spec ($report)"
    if ! tree="$(
      cargo tree --manifest-path rust/Cargo.toml --locked --target all --all-features --edges all --invert "$blocked_spec" 2> /dev/null
    )" || [[ -z "${tree//[[:space:]]/}" ]]; then
      echo "::error::$config ignores $advisory for \`$blocked_spec\`, which no longer reaches the build graph."
      echo "  The upstream fix has landed; drop the ignore (and its proof line) from $config."
      status=1
    fi
    continue
  fi

  echo "Proving $advisory is unreachable: $spec"
  if ! tree="$(
    cargo tree --manifest-path rust/Cargo.toml --locked --target all --all-features --edges all --invert "$spec" 2> /dev/null
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

# `--deny warnings` covers `unmaintained`, `unsound` and `yanked`, none of which
# move the exit status on their own (issue #1079, defect D2). A yanked crate in
# particular carries no advisory ID, so it cannot be silenced by an `ignore`
# entry at all: the only way past this line is to change the lockfile.
audit_status=0
if [[ ! -f "$lock" ]]; then
  echo "::error::$lock is committed but not checked out, so its dependencies would go unaudited."
  audit_status=1
else
  echo "Auditing $lock"
  run_audit cargo audit --file "$lock" --deny warnings || audit_status=$?
fi

# Issue #1571: the audit read only rust/Cargo.lock, so the committed lockfile
# of a standalone experiment crate kept rustls 0.23.43 (four advisories), the
# yanked chacha20 0.10.1 and five more advisories while this job stayed green;
# only Dependabot reported the rustls ones. Like check-javascript-dependencies.sh,
# discover every committed lockfile, and fail when one is committed but not
# checked out instead of skipping it silently. Those lockfiles ship nothing:
# they pin historical versions on purpose (the issue #1017 reproducer pins
# meta-language 0.58.1), so a vulnerability, an unsound crate or a yanked
# release fails here, and an `unmaintained` notice with no fixed release is
# reported but does not fail.
while IFS= read -r -d '' each; do
  [[ "$each" != "$lock" ]] || continue
  if [[ ! -f "$each" ]]; then
    annotation_path=${each//%/%25}
    annotation_path=${annotation_path//$'\r'/%0D}
    annotation_path=${annotation_path//$'\n'/%0A}
    echo "::error::$annotation_path is committed but not checked out, so its dependencies would go unaudited."
    echo "  Add it to the security workflow's sparse checkout."
    if [[ "$audit_status" -eq 0 ]]; then audit_status=1; fi
    continue
  fi
  echo "Auditing $each"
  run_audit cargo audit --file "$each" --deny unsound --deny yanked || {
    observed_status=$?
    if [[ "$audit_status" -eq 0 ]]; then audit_status=$observed_status; fi
  }
done < "$lock_inventory"
exit "$audit_status"
