#!/usr/bin/env bash
set -euo pipefail

# Installs node dependencies and classifies npm's stderr so that upstream
# deprecation notices never fail CI, while genuinely unexpected diagnostics
# (network errors, ERESOLVE conflicts, install failures) still do.
#
# Deprecation warnings are matched by PACKAGE NAME, never by exact version.
# Transitive dependencies float without any change on our side, so pinning a
# version here makes CI fail the moment a dependency resolves to a new patch
# (issue #796: glob@7.2.3 was classified, glob@10.5.0 was not, and every
# desktop build plus the .vsix package job failed on a warning we do not own).
#
# Set INSTALL_NODE_DEPENDENCIES_VERBOSE=1 to trace the classification of each
# stderr line. Off by default so normal CI output stays quiet.

directory="${1:?usage: install-node-dependencies.sh DIRECTORY}"
verbose="${INSTALL_NODE_DEPENDENCIES_VERBOSE:-0}"

stderr_log="$(mktemp)"
unexpected="$(mktemp)"
trap 'rm -f "$stderr_log" "$unexpected"' EXIT

trace() {
  [[ "$verbose" == "1" ]] || return 0
  printf 'install-node-dependencies: %s\n' "$*" >&2
}

trace "installing in '$directory' (npm $(npm --version 2>/dev/null || echo unknown))"

if ! npm --prefix "$directory" install --no-audit --no-fund 2>"$stderr_log"; then
  cat "$stderr_log" >&2
  exit 1
fi

trace "npm install succeeded; classifying $(wc -l <"$stderr_log" | tr -d ' ') stderr line(s)"

# Upstream deprecations we have reviewed and cannot fix from this repository.
# Keyed by package name only -- see the note above about version floats.
# "<package>|<title>|<tracking url>". One entry per reviewed package name.
#
# glob is pulled in by two independent chains, so it is attributed to the one
# we can actually act on: @link-assistant/web-capture -> archiver@7.0.1 ->
# archiver-utils@5.0.2 -> glob@10.5.0. (electron-builder separately pulls
# glob@7.2.3.) Upstream deprecated every glob below 12.x, including the very
# releases that fix CVE-2025-64756, so the warning is unavoidable noise rather
# than an actionable vulnerability -- see isaacs/node-glob#644.
reviewed_deprecations=(
  "inflight|electron-builder upstream deprecation|https://github.com/electron-userland/electron-builder/issues/10016"
  "rimraf|electron-builder upstream deprecation|https://github.com/electron-userland/electron-builder/issues/10016"
  "boolean|electron-builder upstream deprecation|https://github.com/electron-userland/electron-builder/issues/10016"
  "lodash.isequal|electron-builder upstream deprecation|https://github.com/electron-userland/electron-builder/issues/10016"
  "prebuild-install|vsce upstream deprecation|https://github.com/microsoft/vscode-vsce/issues/1290"
  "whatwg-encoding|vsce upstream deprecation|https://github.com/microsoft/vscode-vsce/issues/1290"
  "glob|archiver/electron-builder upstream deprecation|https://github.com/isaacs/node-glob/issues/644"
)

# Extracts the package name from "npm warn deprecated <name>@<version>: ...".
# Handles scoped packages (@scope/name@version) by stripping the version only
# from the last '@'.
deprecated_package_name() {
  local line="$1" spec name
  spec="${line#*npm warn deprecated }"
  spec="${spec%%:*}"
  # Strip the trailing @version. Removing the *last* '@' preserves a leading
  # @scope, so "@scope/pkg@1.2.3" yields "@scope/pkg" and "glob@10.5.0" yields
  # "glob". Guard the case where npm printed no version at all.
  name="${spec%@*}"
  printf '%s' "${name:-$spec}"
}

while IFS= read -r line; do
  case "$line" in
    "")
      continue
      ;;
    *"npm warn deprecated "*)
      package="$(deprecated_package_name "$line")"
      classified=0
      for entry in "${reviewed_deprecations[@]}"; do
        if [[ "${entry%%|*}" == "$package" ]]; then
          rest="${entry#*|}"
          trace "classified '$package' as ${rest%%|*}"
          echo "::notice title=${rest%%|*}::${line#npm warn } See ${rest#*|}"
          classified=1
          break
        fi
      done
      if [[ "$classified" == "0" ]]; then
        trace "deprecation for '$package' is not on the reviewed allowlist"
        printf '%s\n' "$line" >>"$unexpected"
      fi
      ;;
    *)
      trace "unrecognized diagnostic: $line"
      printf '%s\n' "$line" >>"$unexpected"
      ;;
  esac
done <"$stderr_log"

if [[ -s "$unexpected" ]]; then
  echo "Unexpected npm stderr; update dependencies or explicitly classify the diagnostic:" >&2
  cat "$unexpected" >&2
  exit 1
fi

trace "all stderr lines classified as reviewed upstream deprecations"
