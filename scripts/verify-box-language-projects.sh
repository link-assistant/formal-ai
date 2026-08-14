#!/usr/bin/env bash
# Issue #932 (follow-up of PR #119): exercise a Formal-AI-generated language
# project inside the link-foundation/box image that matches the language.
#
# Like `scripts/verify-docker-runtime.sh` this is a contract check that skips
# gracefully when Docker is unavailable, so the same command works on a laptop,
# in the box DinD image and in CI. Set STRICT=1 to turn the skip into a failure.
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
OUT_DIR="${OUT_DIR:-$ROOT/target/box-language-corpus}"
STRICT="${STRICT:-0}"
CONTAINER_WORKDIR=/tmp/box-language-project

usage() {
  cat <<'EOF'
Usage: scripts/verify-box-language-projects.sh [language ...]

With no arguments every language in data/meta/box-language-projects.lino is
verified. Environment:
  OUT_DIR         corpus directory (default: target/box-language-corpus)
  SKIP_GENERATE   1 to reuse an existing corpus instead of regenerating it
  STRICT          1 to fail instead of skipping when Docker is unavailable
  PULL            0 to reuse an already-pulled image without `docker pull`
EOF
}

if [ "${1:-}" = "--help" ] || [ "${1:-}" = "-h" ]; then
  usage
  exit 0
fi

skip() {
  printf 'SKIP: %s\n' "$1"
  if [ "$STRICT" = "1" ]; then
    printf 'error: STRICT=1 turns this skip into a failure\n' >&2
    exit 1
  fi
  exit 0
}

if ! command -v docker >/dev/null 2>&1; then
  skip "docker is not installed, cannot run the box language projects"
fi
if ! docker info >/dev/null 2>&1; then
  skip "the docker daemon is not reachable, cannot run the box language projects"
fi

if [ "$#" -gt 0 ]; then
  languages=("$@")
else
  mapfile -t languages < <(
    awk '
      /^[^ ]/ { current = $1; next }
      $1 == "record_type" && $2 == "\"box_language_project\"" { wanted[current] = 1 }
      $1 == "language" && current in wanted {
        value = $2
        gsub(/"/, "", value)
        print value
      }
    ' "$ROOT/data/meta/box-language-projects.lino"
  )
fi

if [ "${SKIP_GENERATE:-0}" != "1" ]; then
  "$ROOT/scripts/generate-box-language-corpus.sh" "${languages[@]}"
fi

read_meta() {
  local dir="$1" key="$2"
  sed -n "s/^$key=//p" "$dir/meta.env" | head -n 1
}

verify_language() {
  local language="$1"
  local dir="$OUT_DIR/$language"
  local image network
  if [ ! -f "$dir/meta.env" ]; then
    printf 'error: no corpus for %s at %s; drop SKIP_GENERATE\n' "$language" "$dir" >&2
    return 1
  fi
  image="$(read_meta "$dir" IMAGE)"

  if [ "${PULL:-1}" = "1" ]; then
    printf '== pulling %s\n' "$image"
    docker pull --quiet "$image"
  fi

  cp "$ROOT/scripts/run-box-language-project.sh" "$dir/run-box-language-project.sh"

  # The corpus travels as a tar stream on stdin: bind mounts inherit host uids
  # that the unprivileged `box` user inside the image cannot always write to,
  # and they break outright when the daemon runs in a different mount namespace.
  network=(--network none)
  if [ "$(read_meta "$dir" NETWORK_REQUIRED)" = "true" ]; then
    network=()
  fi

  tar -cz -C "$dir" . | docker run --rm -i "${network[@]}" \
    -e HOME=/home/box \
    --workdir /tmp \
    "$image" \
    bash -c "mkdir -p $CONTAINER_WORKDIR \
      && tar -xz -C $CONTAINER_WORKDIR \
      && exec bash $CONTAINER_WORKDIR/run-box-language-project.sh"
}

failures=()
for language in "${languages[@]}"; do
  if ! verify_language "$language"; then
    failures+=("$language")
  fi
done

if [ "${#failures[@]}" -gt 0 ]; then
  printf 'error: box language project check failed for: %s\n' "${failures[*]}" >&2
  exit 1
fi

printf 'ok: box language projects verified for: %s\n' "${languages[*]}"
