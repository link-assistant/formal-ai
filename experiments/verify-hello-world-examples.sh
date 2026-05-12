#!/usr/bin/env bash
set -euo pipefail

workdir="$(mktemp -d "${TMPDIR:-/tmp}/formal-ai-hello-world.XXXXXX")"
trap 'rm -rf "$workdir"' EXIT

run_with_timeout() {
  if command -v timeout >/dev/null 2>&1; then
    timeout 60s "$@"
  else
    "$@"
  fi
}

expect_output() {
  local language="$1"
  local actual="$2"

  if [[ "$actual" != "Hello, world!" ]]; then
    printf '%s: expected "Hello, world!", got "%s"\n' "$language" "$actual" >&2
    return 1
  fi

  printf '%s: verified output "%s"\n' "$language" "$actual"
}

verify_rust() {
  local dir="$workdir/rust"
  mkdir -p "$dir"
  cat >"$dir/main.rs" <<'RS'
fn main() {
    println!("Hello, world!");
}
RS
  run_with_timeout rustc "$dir/main.rs" -o "$dir/main"
  expect_output rust "$(run_with_timeout "$dir/main")"
}

verify_python() {
  local dir="$workdir/python"
  mkdir -p "$dir"
  cat >"$dir/main.py" <<'PY'
print("Hello, world!")
PY
  run_with_timeout python3 -m py_compile "$dir/main.py"
  expect_output python "$(run_with_timeout python3 "$dir/main.py")"
}

verify_javascript() {
  local dir="$workdir/javascript"
  mkdir -p "$dir"
  cat >"$dir/main.js" <<'JS'
console.log("Hello, world!");
JS
  run_with_timeout node --check "$dir/main.js" >/dev/null
  expect_output javascript "$(run_with_timeout node "$dir/main.js")"
}

verify_typescript() {
  local dir="$workdir/typescript"
  mkdir -p "$dir"
  cat >"$dir/hello.ts" <<'TS'
console.log("Hello, world!");
TS

  if ! command -v tsc >/dev/null 2>&1; then
    printf 'typescript: unavailable (tsc is not installed in this runtime)\n'
    return 0
  fi

  run_with_timeout tsc "$dir/hello.ts" --outDir "$dir"
  expect_output typescript "$(run_with_timeout node "$dir/hello.js")"
}

verify_go() {
  local dir="$workdir/go"
  mkdir -p "$dir"
  cat >"$dir/main.go" <<'GO'
package main

import "fmt"

func main() {
    fmt.Println("Hello, world!")
}
GO
  expect_output go "$(cd "$dir" && run_with_timeout go run main.go)"
}

verify_c() {
  local dir="$workdir/c"
  mkdir -p "$dir"
  cat >"$dir/main.c" <<'C'
#include <stdio.h>

int main(void) {
    puts("Hello, world!");
    return 0;
}
C
  run_with_timeout gcc "$dir/main.c" -o "$dir/main"
  expect_output c "$(run_with_timeout "$dir/main")"
}

verify_rust
verify_python
verify_javascript
verify_typescript
verify_go
verify_c
