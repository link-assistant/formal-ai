#!/usr/bin/env bash
# Let Formal AI author a repository change through the real Agent CLI.
#
# Plan 03 L13 reduced this script to a thin wrapper: the loop itself — spawn
# `formal-ai serve`, drive the Agent CLI, harvest the session id, enforce the
# artifact contract, land the four-trailer commit — lives in
# `src/authoring_loop.rs`, where it is unit-testable against fake executables
# (`tests/unit/ci-cd/authoring_effects.rs`). This file keeps only the CLI the
# workflow in `.github/workflows/self-authored-pull-request.yml` calls, and
# translates it onto `formal-ai solve`.
#
# The commit lands on whatever branch is checked out, so Formal AI's work rides
# along inside an ordinary pull request instead of needing a separate one
# (issue #1069). The script opens no pull request and pushes nothing.
#
# Usage:
#   scripts/author-change-with-formal-ai.sh \
#     --task "<prompt>" \
#     --produces <workspace-relative file the CLI must write> \
#     --into <repo-relative destination> \
#     [--produces <file> --into <destination>]...   (repeatable, in pairs) \
#     --evidence <repo-relative evidence directory> \
#     --pull-request <https://github.com/owner/repo/pull/N> \
#     --message "<commit subject>" \
#     [--seed <directory copied into the workspace first>] \
#     [--contains <text the artifact must contain>]... \
#     [--port <port>] [--no-commit]
set -euo pipefail

# The repository this authors in. Derived from the script's own location when
# it runs from the repository, but the composite action stages it into
# `$RUNNER_TEMP` before anything switches branches (so it runs *this* run's
# copy, not the bot branch's), and there `dirname $0/..` is the temp directory.
# `FORMAL_AI_REPO_ROOT` is how the action says where the checkout actually is;
# run 34294396281 died as `--seed is not a directory` without it.
if [[ -n "${FORMAL_AI_REPO_ROOT:-}" ]]; then
  ROOT="$FORMAL_AI_REPO_ROOT"
else
  ROOT="$(cd "$(dirname "$0")/.." && pwd)"
fi
BIN="${BIN:-$ROOT/rust/target/release/formal-ai}"
AGENT="${AGENT:-agent}"

die() {
  echo "author-change-with-formal-ai: $*" >&2
  exit 1
}

args=()
commit=1
while [[ $# -gt 0 ]]; do
  case "$1" in
    --no-commit) commit=0; shift ;;
    --task|--produces|--into|--evidence|--pull-request|--message|--seed|--contains|--port)
      [[ $# -ge 2 ]] || die "$1 needs a value"
      args+=("$1" "$2")
      shift 2
      ;;
    *) die "unknown option: $1" ;;
  esac
done

[[ -x "$BIN" ]] || die "build first: cargo build --release --bin formal-ai"
command -v "$AGENT" >/dev/null || die "the @link-assistant/agent CLI is not on PATH"

# Mutation stays opt-in here (`--no-commit` is the flag the workflow passes for
# a dry run), mirroring `formal-ai solve`'s own default-deny ladder from the
# other side: the loop lands the four-trailer commit only under `--commit`.
if [[ "$commit" -eq 1 ]]; then
  exec "$BIN" solve --repository "$ROOT" --commit "${args[@]}"
else
  exec "$BIN" solve --repository "$ROOT" "${args[@]}"
fi
