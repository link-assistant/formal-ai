#!/usr/bin/env bash
set -euo pipefail

directory="${1:?usage: install-node-dependencies.sh DIRECTORY}"
stderr_log="$(mktemp)"
trap 'rm -f "$stderr_log"' EXIT

if ! npm --prefix "$directory" install --no-audit --no-fund 2>"$stderr_log"; then
  cat "$stderr_log" >&2
  exit 1
fi

unexpected="$(mktemp)"
trap 'rm -f "$stderr_log" "$unexpected"' EXIT
while IFS= read -r line; do
  case "$line" in
    *"npm warn deprecated inflight@1.0.6:"*|\
    *"npm warn deprecated glob@7.2.3:"*|\
    *"npm warn deprecated rimraf@2.6.3:"*|\
    *"npm warn deprecated boolean@3.2.0:"*|\
    *"npm warn deprecated lodash.isequal@4.5.0:"*)
      echo "::notice title=electron-builder upstream deprecation::${line#npm warn } See https://github.com/electron-userland/electron-builder/issues/10016"
      ;;
    *"npm warn deprecated prebuild-install@7.1.3:"*|\
    *"npm warn deprecated whatwg-encoding@3.1.1:"*)
      echo "::notice title=vsce upstream deprecation::${line#npm warn } See https://github.com/microsoft/vscode-vsce/issues/1290"
      ;;
    "") ;;
    *) printf '%s\n' "$line" >>"$unexpected" ;;
  esac
done <"$stderr_log"

if [[ -s "$unexpected" ]]; then
  echo "Unexpected npm stderr; update dependencies or explicitly classify the diagnostic:" >&2
  cat "$unexpected" >&2
  exit 1
fi
