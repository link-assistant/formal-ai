#!/usr/bin/env bash
# Issue #932: in-container half of the box language check. It is copied into the
# corpus directory and executed inside the link-foundation/box image that
# matches the language, where it runs the generated init script (the traditional
# `cargo new` / `npm init` / `go mod init` commands) and asserts the project
# really builds and prints the expected line.
set -euo pipefail

cd "$(dirname "$0")"

LANGUAGE=""
IMAGE=""
PROGRAM_FILE=""
PROJECT_DIRECTORY=""
EXPECTED_OUTPUT=""
PROMPT_LOCALE=""
SHELL_PRELUDE=""
while IFS='=' read -r key value; do
  case "$key" in
    LANGUAGE) LANGUAGE="$value" ;;
    IMAGE) IMAGE="$value" ;;
    PROGRAM_FILE) PROGRAM_FILE="$value" ;;
    PROJECT_DIRECTORY) PROJECT_DIRECTORY="$value" ;;
    EXPECTED_OUTPUT) EXPECTED_OUTPUT="$value" ;;
    PROMPT_LOCALE) PROMPT_LOCALE="$value" ;;
    SHELL_PRELUDE) SHELL_PRELUDE="$value" ;;
    *) ;;
  esac
done <meta.env

printf '== %s project generated from the %s prompt, image %s\n' \
  "$LANGUAGE" "$PROMPT_LOCALE" "$IMAGE"
printf '== program (%s):\n' "$PROGRAM_FILE"
cat "$PROGRAM_FILE"
printf '== traditional init script:\n'
cat init.sh

if [ -n "$SHELL_PRELUDE" ]; then
  # Toolchain managers (nvm, rvm) are not `set -u` clean, so relax it while the
  # prelude runs. The PATH it exports is inherited by the init script below.
  set +u
  # shellcheck disable=SC1090
  eval "$SHELL_PRELUDE"
  set -u
fi

printf '== running the init script\n'
status=0
bash ./init.sh >run-output.txt 2>&1 || status=$?
sed 's/^/   | /' run-output.txt
if [ "$status" -ne 0 ]; then
  printf 'error: the init script failed with exit code %s\n' "$status" >&2
  exit "$status"
fi

if [ ! -d "$PROJECT_DIRECTORY" ]; then
  printf 'error: the init script did not create %s\n' "$PROJECT_DIRECTORY" >&2
  exit 1
fi

if ! grep -Fqx "$EXPECTED_OUTPUT" run-output.txt; then
  printf 'error: the project never printed the expected line: %s\n' "$EXPECTED_OUTPUT" >&2
  exit 1
fi

printf 'ok: %s project built and printed %s inside %s\n' \
  "$LANGUAGE" "$EXPECTED_OUTPUT" "$IMAGE"
