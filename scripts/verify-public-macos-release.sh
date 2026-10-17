#!/usr/bin/env bash
# Hosted consumer of authenticated public release downloads; never builds or signs.
set -euo pipefail
: "${RELEASE_TAG:?}" "${RELEASE_VERSION:?}" "${RELEASE_SOURCE_SHA:?}"
: "${RELEASE_SOURCE_TREE:?}" "${RELEASE_DOWNLOAD_DIRECTORY:?}" "${MACOS_ARCH:?}"
if [[ "${CI:-}" != true || "$(uname -s)" != Darwin ]]; then
  echo 'Public macOS package execution requires a hosted macOS CI runner.' >&2
  exit 1
fi
script_directory="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
exec node "$script_directory/verify-public-macos-release.mjs"
