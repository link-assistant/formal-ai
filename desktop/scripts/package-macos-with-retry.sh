#!/usr/bin/env bash
# Retry only the transient hdiutil failures observed on GitHub-hosted macOS
# runners. See https://github.com/actions/runner-images/issues/7522.
set -euo pipefail

readonly max_attempts=3
retry_delay_seconds="${FORMAL_AI_MACOS_PACKAGE_RETRY_DELAY_SECONDS:-5}"

if ! [[ "$retry_delay_seconds" =~ ^[0-9]+$ ]]; then
  echo "FORMAL_AI_MACOS_PACKAGE_RETRY_DELAY_SECONDS must be a non-negative integer" >&2
  exit 2
fi
if [ "$#" -eq 0 ]; then
  echo "usage: $0 <electron-builder arguments...>" >&2
  exit 2
fi

package_log="$(mktemp "${RUNNER_TEMP:-${TMPDIR:-/tmp}}/formal-ai-macos-package.log.XXXXXX")"
cleanup_log() {
  rm -f -- "$package_log"
}
trap cleanup_log EXIT

attempt=1
while [ "$attempt" -le "$max_attempts" ]; do
  set +e
  npx --no-install electron-builder "$@" 2>&1 | tee "$package_log"
  package_status="${PIPESTATUS[0]}"
  set -e

  if [ "$package_status" -eq 0 ]; then
    exit 0
  fi

  # Do not hide signing, notarization, dependency, or arbitrary builder
  # failures. These create/attach errors are the runner disk-image service
  # signatures documented in actions/runner-images#7522; everything else fails
  # on its first attempt.
  if ! grep -Eq \
    'hdiutil: (create|attach) failed - (Device not configured|Resource busy|No child processes)' \
    "$package_log"; then
    exit "$package_status"
  fi
  if [ "$attempt" -eq "$max_attempts" ]; then
    echo "macOS DMG creation still failed after ${max_attempts} attempts" >&2
    exit "$package_status"
  fi

  echo "::warning title=Transient macOS DMG creation failure::Retrying electron-builder after hdiutil failed on attempt ${attempt}/${max_attempts}"
  # Preserve completed ZIP/app output while removing only an incomplete
  # top-level disk image that would otherwise collide with the next attempt.
  if [ -d release ]; then
    find release -maxdepth 1 -type f -name '*.dmg' -delete
  fi
  sync
  sleep_seconds=$((retry_delay_seconds * attempt))
  if [ "$sleep_seconds" -gt 0 ]; then
    sleep "$sleep_seconds"
  fi
  attempt=$((attempt + 1))
done
