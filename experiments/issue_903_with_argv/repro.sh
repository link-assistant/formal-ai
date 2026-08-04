#!/usr/bin/env bash
# Reproduces the five argv-construction defects reported in issue #903.
# Puts a shim that records argv first on PATH, then runs `formal-ai with <tool>`
# in the two shapes a caller realistically uses.
set -u
root="$(cd "$(dirname "$0")/../.." && pwd)"
bin="$root/target/debug/formal-ai"
work="$(mktemp -d)"
shim="$work/bin"
mkdir -p "$shim"
for tool in agent codex claude gemini qwen; do
  cat > "$shim/$tool" <<SHIM
#!/usr/bin/env bash
{ printf '%s argv:' "$tool"; for a in "\$@"; do printf ' [%s]' "\$a"; done; printf '\n'; } >> "$work/argv.log"
exit 0
SHIM
  chmod +x "$shim/$tool"
done
export PATH="$shim:$PATH"
cd "$work"
for tool in agent codex claude gemini qwen; do
  echo "=== $tool shape A (prompt as argument) ===" >> "$work/argv.log"
  "$bin" with --no-start-server "$tool" --model formal-ai -p "Create a file named hello.txt" --verbose </dev/null >/dev/null 2>&1
  echo "=== $tool shape B (prompt piped, qualified model) ===" >> "$work/argv.log"
  echo "Create a file named hello.txt" | "$bin" with --no-start-server --model formalai/formal-ai "$tool" --verbose >/dev/null 2>&1
done
cat "$work/argv.log"
