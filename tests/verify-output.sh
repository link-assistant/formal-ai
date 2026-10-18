#!/bin/sh
set -eu
source_path='main.rs'
test -f "$source_path" && test ! -L "$source_path"
verification_dir=$(mktemp -d "${TMPDIR:-/tmp}/formal-ai-verification.XXXXXXXXXX")
trap 'rm -rf -- "$verification_dir"' EXIT
printf '%s' '// rustc main.rs -o main
// ./main
// Emit the requested text followed by a newline.
fn main() {
    println!("{}", "Hello, World!");
}
' > "$verification_dir/declared-source"
cmp "$source_path" "$verification_dir/declared-source"
mkdir -p "$verification_dir/$(dirname "$source_path")"
cp -- "$source_path" "$verification_dir/$source_path"
cmp "$source_path" "$verification_dir/$source_path"
cd "$verification_dir"
printf '%s\n' 'Hello, World!' > expected
if rustc main.rs -o main; then
  :
else
  process_status=$?
  printf '\n%s\n' '__formal_ai_owned_process_failed_0__' >&2
  exit "$process_status"
fi
if ./main > actual; then
  :
else
  process_status=$?
  printf '\n%s\n' '__formal_ai_owned_process_failed_1__' >&2
  exit "$process_status"
fi
cmp expected actual
cat actual
