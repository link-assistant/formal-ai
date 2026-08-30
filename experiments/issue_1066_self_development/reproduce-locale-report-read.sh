#!/usr/bin/env bash
# Reproduce, then falsify, the locale-dependent read of an incremental dispatch
# report.
#
# `agent dispatch --incremental` writes UTF-8: the task text, the verifier
# output, and every seed-authored message can carry non-ASCII bytes. Ruby's
# `File.read` decodes with the locale's default external encoding, so on a host
# whose locale is `POSIX`/`C` — the default in a bare container, and what
# `experiments/issue_924_self_authoring/run.sh` met on the release server — the
# first non-ASCII byte raises `Encoding::InvalidByteSequenceError`. The harness
# then reports a failed self-authoring run for a dispatch that actually solved
# its task.
#
# This script asserts both directions: the unfixed expression must fail under a
# POSIX locale, and the committed expression must succeed on the same bytes.

set -euo pipefail

ROOT="$(cd "$(dirname "$0")/../.." && pwd)"
WORK="$(mktemp -d)"
trap 'rm -rf "$WORK"' EXIT

command -v ruby >/dev/null || { echo "ruby is required" >&2; exit 2; }

# An em dash is what the real report carried; any non-ASCII byte does it.
cat > "$WORK/dispatch-report.json" <<'JSON'
{"mode":"incremental","incremental":{"solved":true,"steps":[{"task":"write a file — exactly","passed":false}]}}
JSON

unfixed='JSON.parse(File.read(ARGV.fetch(0)))'
fixed='JSON.parse(File.read(ARGV.fetch(0), encoding: Encoding::UTF_8))'

run_expression() {
  LC_ALL=POSIX LANG= ruby -rjson -e "report = $1; abort 'unexpected report' unless report.fetch('incremental').fetch('solved')" \
    "$WORK/dispatch-report.json" 2>"$WORK/stderr.log"
}

if run_expression "$unfixed"; then
  echo "FAIL: the unfixed read parsed UTF-8 under a POSIX locale; the bug is not reproduced here" >&2
  exit 1
fi
grep -q 'Encoding::InvalidByteSequenceError' "$WORK/stderr.log" || {
  echo "FAIL: the unfixed read failed for some other reason:" >&2
  cat "$WORK/stderr.log" >&2
  exit 1
}
echo "reproduced: File.read raises Encoding::InvalidByteSequenceError under LC_ALL=POSIX"

run_expression "$fixed" || {
  echo "FAIL: the committed read still cannot parse the same bytes:" >&2
  cat "$WORK/stderr.log" >&2
  exit 1
}
echo "fixed: File.read(..., encoding: Encoding::UTF_8) parses the same bytes under the same locale"

# The harnesses this protects must actually carry the fixed form.
for harness in \
  "$ROOT/experiments/issue_924_self_authoring/run.sh" \
  "$ROOT/experiments/issue_933_self_authoring/run.sh"
do
  grep -q "File.read(ARGV.fetch(0), encoding: Encoding::UTF_8)" "$harness" || {
    echo "FAIL: $harness does not name the report's encoding" >&2
    exit 1
  }
done
echo "both incremental self-authoring harnesses name the report's encoding"

# Each harness embeds the Ruby program inside a single-quoted bash string, so a
# `'UTF-8'` literal would close that string and Ruby would see a bare `UTF`
# constant. Spelling the encoding as `Encoding::UTF_8` needs no quote at all;
# assert the whole embedded program stays quote-free so the next edit cannot
# reintroduce the split.
for harness in \
  "$ROOT/experiments/issue_924_self_authoring/run.sh" \
  "$ROOT/experiments/issue_933_self_authoring/run.sh"
do
  program="$(awk "/^ruby -rjson -e '$/{flag=1;next}/^' /{flag=0}flag" "$harness")"
  [ -n "$program" ] || { echo "FAIL: no embedded ruby program found in $harness" >&2; exit 1; }
  case "$program" in
    *"'"*) echo "FAIL: $harness embeds a single quote inside its single-quoted ruby program" >&2; exit 1 ;;
  esac
done
echo "no embedded ruby program contains a single quote"
