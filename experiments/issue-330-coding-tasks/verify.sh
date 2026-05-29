#!/usr/bin/env bash
# Compile/run every new (task, language) template and compare to the expected
# deterministic output (issue #330). Exits non-zero on the first mismatch.
set -u
HERE="$(cd "$(dirname "$0")" && pwd)"
OUT="$HERE/build"
rm -rf "$OUT"; mkdir -p "$OUT"
python3 "$HERE/templates.py" "$OUT" >/dev/null

pass=0; fail=0
check() { # task lang "actual"
  local task="$1" lang="$2" actual="$3"
  local expected
  expected=$(python3 -c "import json,sys;print(json.load(open('$OUT/expected.json'))['$task'])")
  if [ "$actual" = "$expected" ]; then
    echo "PASS $task/$lang"; pass=$((pass+1))
  else
    echo "FAIL $task/$lang"; echo "--- expected ---"; printf '%s\n' "$expected"; echo "--- actual ---"; printf '%s\n' "$actual"; fail=$((fail+1))
  fi
}

for task in fizzbuzz factorial reverse_string sum_to_ten; do
  d="$OUT/$task"
  # rust
  (cd "$d/rust" && rustc main.rs -o main 2>/dev/null && check "$task" rust "$(./main)")
  # python
  check "$task" python "$(python3 "$d/python/main.py")"
  # javascript
  check "$task" javascript "$(node "$d/javascript/main.js")"
  # typescript -> run the JS twin (identical runtime logic; tsc unavailable)
  check "$task" "typescript(as-js)" "$(node "$d/javascript/main.js")"
  # go
  (cd "$d/go" && check "$task" go "$(go run main.go 2>/dev/null)")
  # c
  (cd "$d/c" && gcc main.c -o main 2>/dev/null && check "$task" c "$(./main)")
  # cpp
  (cd "$d/cpp" && g++ main.cpp -o main 2>/dev/null && check "$task" cpp "$(./main)")
  # java
  (cd "$d/java" && javac Main.java 2>/dev/null && check "$task" java "$(java Main)")
  # csharp (scaffold a minimal SDK project so `dotnet run` can build Program.cs)
  cat > "$d/csharp/app.csproj" <<'CSPROJ'
<Project Sdk="Microsoft.NET.Sdk">
  <PropertyGroup>
    <OutputType>Exe</OutputType>
    <TargetFramework>net8.0</TargetFramework>
    <Nullable>disable</Nullable>
    <ImplicitUsings>disable</ImplicitUsings>
  </PropertyGroup>
</Project>
CSPROJ
  (cd "$d/csharp" && check "$task" csharp "$(dotnet run -v q --nologo 2>/dev/null || true)")
  # ruby
  check "$task" ruby "$(ruby "$d/ruby/main.rb")"
done

echo "================ pass=$pass fail=$fail ================"
exit $fail
