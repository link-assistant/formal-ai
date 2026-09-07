#!/usr/bin/env bash
# Print the evidence a failed Agent CLI E2E step left on disk into the job log.
#
# Issue #1079: the `agent-cli-failure-report` job of run 34061511110 failed with
#
#     ##[error]Process completed with exit code 1.
#
# and nothing else -- two seconds of log, no message, no stack, no exit path.
# The diagnosis was in the uploaded artifact:
#
#     {"type":"error","errorType":"UnhandledRejection","message":"Error from
#      provider (Console): Request is missing x-opencode-session ..."}
#
# The harnesses redirect the CLI's streams to files and classify them
# afterwards with `scripts/classify-agent-cli-stderr.sh`, which does print an
# unexpected diagnostic. But `set -e` ends the harness on the CLI's own
# non-zero exit, one line before that classification runs, so the one step that
# would have spoken is unreachable exactly when it has something to say. A red
# check that cannot explain itself costs a reviewer an artifact download to
# learn what a line of log already knew.
#
# Wire this into an `if: failure()` step beside the artifact upload: a green run
# never calls it, so the quiet log stays quiet.
#
# Usage: dump-agent-cli-evidence.sh PATH [PATH ...]
#
# PATH may be a file or a directory; directories are walked. Missing paths are
# reported and skipped -- a diagnostic must never turn one failure into two, so
# this always exits 0 and never runs under `set -e`.

set -uo pipefail

lines="${AGENT_EVIDENCE_TAIL_LINES:-200}"

if [ "$#" -eq 0 ]; then
  echo "usage: dump-agent-cli-evidence.sh PATH [PATH ...]" >&2
  exit 0
fi

collected=()
for path in "$@"; do
  if [ -f "$path" ]; then
    collected+=("$path")
  elif [ -d "$path" ]; then
    while IFS= read -r file; do
      [ -n "$file" ] && collected+=("$file")
    done < <(find "$path" -type f | sort)
  else
    echo "no Agent CLI evidence at $path"
  fi
done

if [ "${#collected[@]}" -eq 0 ]; then
  echo "no Agent CLI evidence files found; the harness may have failed before it wrote any"
  exit 0
fi

# stderr first: the unhandled rejection that explains the exit status lives
# there, and a reviewer scrolling a folded log should meet the cause before the
# transcript it interrupted.
ordered=()
for file in "${collected[@]}"; do
  case "$file" in *stderr*) ordered+=("$file") ;; esac
done
for file in "${collected[@]}"; do
  case "$file" in *stderr*) ;; *) ordered+=("$file") ;; esac
done

printf 'Agent CLI evidence: %s file(s), last %s line(s) of each\n' \
  "${#ordered[@]}" "$lines"

for file in "${ordered[@]}"; do
  size="$(wc -c <"$file" 2>/dev/null || echo 0)"
  echo "::group::${file} (${size} bytes)"
  tail -n "$lines" -- "$file" 2>&1 || echo "could not read $file"
  echo '::endgroup::'
done

exit 0
