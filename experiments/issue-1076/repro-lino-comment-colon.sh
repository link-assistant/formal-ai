#!/usr/bin/env bash
#
# Issue #1076, defect D18: Links Notation has no comment syntax. A `#` line is
# an ordinary link, so ordinary English punctuation inside what looks like a
# comment is structural -- one bare colon makes the whole file unparseable.
# Reported upstream as link-foundation/links-notation#301 (the grammar) and
# #302 (the Rust error carries no line or column, while the JavaScript port of
# the same version reports both).
#
# Usage (from anywhere):
#   bash experiments/issue-1076/repro-lino-comment-colon.sh
#
# Needs `rust-script` for the Rust half. The JavaScript half is skipped unless
# `node` can resolve `links-notation`; set NODE_PATH or run
# `npm i links-notation@0.16.1` in a scratch directory and point NODE_PATH at
# its `node_modules`. Exits 0 when the defect reproduces.
set -uo pipefail

work="$(mktemp -d)"
trap 'rm -rf "$work"' EXIT

# The line that turned `Test (ubuntu-latest / full)` red: prose, not data.
broken="$work/broken.lino"
cat > "$broken" <<'LINO'
# One CI gate, one file.
# What this gate holds is the part a commit can break: two of the tests parse
# the repository's real workflows.
ci_gate check_job_headroom
  stage rust
  run "rust-script --test scripts/check-job-headroom.rs"
LINO

# The same file with the colon written as ` -- `, which is the workaround.
fixed="$work/fixed.lino"
sed 's/can break: two/can break -- two/' "$broken" > "$fixed"

# And the same colon inside a backtick span, which the parser reads as one
# reference and therefore accepts.
quoted="$work/quoted.lino"
# shellcheck disable=SC2016  # the backticks are Links Notation, not a subshell
sed 's/can break: two/can break `slice:N` two/' "$broken" > "$quoted"

cat > "$work/probe.rs" <<'RS'
#!/usr/bin/env rust-script
//! ```cargo
//! [dependencies]
//! links-notation = "0.16.1"
//! ```
use links_notation::parse_lino;
fn main() {
    let path = std::env::args().nth(1).expect("usage: probe <file.lino>");
    let text = std::fs::read_to_string(&path).expect("readable file");
    match parse_lino(text.trim()) {
        Ok(_) => println!("OK"),
        Err(error) => println!("ERR {error}"),
    }
}
RS

if ! command -v rust-script > /dev/null 2>&1; then
  echo "rust-script not found: cargo install rust-script" >&2
  exit 2
fi

# Prints one table row and leaves the parser's verdict in `case_result`, so
# the row reaches the terminal instead of a command substitution.
case_result=""
rust_case() {
  # $1 label, $2 file
  case_result="$(rust-script "$work/probe.rs" "$2" 2>&1 | tail -1)"
  printf '  %-26s %s\n' "$1" "$(printf '%s' "$case_result" | cut -c1-96)"
}

echo "rust (links-notation 0.16.1):"
rust_case 'colon in prose' "$broken"
broken_out="$case_result"
rust_case 'colon rewritten as --' "$fixed"
fixed_out="$case_result"
rust_case 'colon inside backticks' "$quoted"
quoted_out="$case_result"

echo
echo "javascript (links-notation 0.16.1):"
if node -e 'require("links-notation")' > /dev/null 2>&1; then
  node - "$broken" <<'JS'
const fs = require('fs')
const { Parser } = require('links-notation')
try {
  new Parser().parse(fs.readFileSync(process.argv[2], 'utf8'))
  console.log('  parsed -- the JavaScript port disagrees with the Rust one')
} catch (error) {
  console.log(`  ${error.message}`)
  const start = error.location && error.location.start
  console.log(`  at line ${start.line}, column ${start.column} -- the position the Rust error omits`)
}
JS
else
  echo "  skipped: node cannot resolve links-notation"
fi

echo
case "$broken_out" in
  ERR*) ;;
  *) echo "NOT REPRODUCED: the colon line parsed."; exit 1 ;;
esac
case "$fixed_out$quoted_out" in
  OKOK) ;;
  *) echo "NOT REPRODUCED: a control case failed, so the colon is not the variable."; exit 1 ;;
esac
echo "REPRODUCED: the same file fails with a colon in its prose and parses without one,"
echo "and the Rust error names no line while the JavaScript one does."
