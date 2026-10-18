#!/bin/sh
set -eu
source_path='Main.kt'
test -f "$source_path" && test ! -L "$source_path"
verification_dir=$(mktemp -d "${TMPDIR:-/tmp}/formal-ai-verification.XXXXXXXXXX")
trap 'rm -rf -- "$verification_dir"' EXIT
printf '%s' '// kotlinc Main.kt -include-runtime -d Main.jar
// java -jar Main.jar
// Emit the requested text followed by a newline.
fun main() {
    println("Hello, World!")
}
' > "$verification_dir/declared-source"
cmp "$source_path" "$verification_dir/declared-source"
mkdir -p "$verification_dir/$(dirname "$source_path")"
cp -- "$source_path" "$verification_dir/$source_path"
cmp "$source_path" "$verification_dir/$source_path"
cd "$verification_dir"
printf '%s\n' 'Hello, World!' > expected
if kotlinc Main.kt -include-runtime -d Main.jar; then
  :
else
  process_status=$?
  printf '\n%s\n' '__formal_ai_owned_process_failed_0__' >&2
  exit "$process_status"
fi
if java -jar Main.jar > actual; then
  :
else
  process_status=$?
  printf '\n%s\n' '__formal_ai_owned_process_failed_1__' >&2
  exit "$process_status"
fi
cmp expected actual
cat actual
