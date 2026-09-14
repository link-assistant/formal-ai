#!/usr/bin/env bash
# Issue #863 / #862 / #1021: do what the `copy stdin to stdout` answer tells the
# reader to do, in every language Formal AI catalogues, and check that what
# comes out is the output the answer promised.
#
# No template is duplicated here: the harness reads the program, the file name,
# the check command, the run command and the expected output out of the rendered
# answer itself.
#
# The workspace defaults to a temporary directory *outside* the repository. The
# repository's `package.json` sets `"type": "module"`, which Node applies to
# every `.js` file below it, so a `main.js` saved inside the tree would fail on
# `require` for a reason the answer has no part in.
set -euo pipefail
HERE="$(cd "$(dirname "$0")" && pwd)"
ROOT="$(cd "$HERE/../.." && pwd)"
OUT="${1:-$(mktemp -d -t issue-1021-copy-stdin-XXXXXX)}"
mkdir -p "$OUT"
echo "workspace: $OUT"
cd "$ROOT"
cargo run --quiet --example issue_1021_copy_stdin_harness -- "$OUT"
