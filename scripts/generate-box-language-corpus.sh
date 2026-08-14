#!/usr/bin/env bash
# Issue #932: build the corpus of Formal-AI-generated language projects that
# `scripts/verify-box-language-projects.sh` exercises inside the matching
# link-foundation/box image.
#
# Everything in the corpus comes out of the solver itself:
#   * the program is the `write program` answer for the language, asked once per
#     locale in `prompt_locales` (en/ru/hi/zh) and required to be byte-identical
#     across all four;
#   * the init script is the `installation_conversion` answer for the README
#     step list recorded in `data/meta/box-language-projects.lino`, so the
#     traditional `cargo new` / `npm init` / `go mod init` commands the container
#     runs are the ones the solver emitted, not ones this script hard-codes.
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
CONTRACT="${CONTRACT:-$ROOT/data/meta/box-language-projects.lino}"
OUT_DIR="${OUT_DIR:-$ROOT/target/box-language-corpus}"

usage() {
  cat <<'EOF'
Usage: scripts/generate-box-language-corpus.sh [language ...]

With no arguments every language in data/meta/box-language-projects.lino is
generated. Environment:
  FORMAL_AI_BIN  path to the formal-ai binary (default: target/release/formal-ai
                 then target/debug/formal-ai, built on demand)
  OUT_DIR        corpus directory (default: target/box-language-corpus)
EOF
}

if [ "${1:-}" = "--help" ] || [ "${1:-}" = "-h" ]; then
  usage
  exit 0
fi

# --- contract reader -------------------------------------------------------
# The contract is flat Links Notation: a top-level record id at column 0 and
# `key "value"` children indented by two spaces.
contract_field() {
  awk -v record="$1" -v key="$2" '
    /^[^ ]/ { current = $1; next }
    current != record { next }
    {
      line = $0
      sub(/^[ ]+/, "", line)
      name = line
      sub(/[ ].*$/, "", name)
      if (name != key) next
      value = line
      sub(/^[^ ]+[ ]+/, "", value)
      gsub(/^"|"$/, "", value)
      print value
      exit
    }
  ' "$CONTRACT"
}

contract_records() {
  awk -v wanted="$1" '
    /^[^ ]/ { current = $1; next }
    $1 == "record_type" {
      value = $2
      gsub(/"/, "", value)
      if (value == wanted) print current
    }
  ' "$CONTRACT"
}

contract_languages() {
  local record
  for record in $(contract_records box_language_project); do
    contract_field "$record" language
  done
}

record_for_language() {
  local wanted="$1" record language
  for record in $(contract_records box_language_project); do
    language="$(contract_field "$record" language)"
    if [ "$language" = "$wanted" ]; then
      printf '%s\n' "$record"
      return 0
    fi
  done
  printf 'error: language %s is not in %s\n' "$wanted" "$CONTRACT" >&2
  return 1
}

locales() {
  contract_field box_language_projects prompt_locales | tr -d '()"'
}

prompt_for() {
  local locale="$1" display_name="$2" template
  template="$(contract_field "box_language_prompt_$locale" template)"
  if [ -z "$template" ]; then
    printf 'error: no prompt template for locale %s\n' "$locale" >&2
    return 1
  fi
  printf '%s\n' "${template//\{display_name\}/$display_name}"
}

# --- solver ----------------------------------------------------------------
resolve_binary() {
  if [ -n "${FORMAL_AI_BIN:-}" ]; then
    printf '%s\n' "$FORMAL_AI_BIN"
    return 0
  fi
  local candidate
  for candidate in "$ROOT/target/release/formal-ai" "$ROOT/target/debug/formal-ai"; do
    if [ -x "$candidate" ]; then
      printf '%s\n' "$candidate"
      return 0
    fi
  done
  printf 'Building formal-ai (release) for the corpus...\n' >&2
  (cd "$ROOT" && cargo build --release --bin formal-ai >&2)
  printf '%s\n' "$ROOT/target/release/formal-ai"
}

# Prints the first fenced block whose opening tag matches the requested fence.
# The answer also carries a ```text expected-output block, so the tag match
# matters.
extract_fence() {
  awk -v fence="$1" '
    BEGIN { opening = "```" fence }
    !inside && $0 == opening { inside = 1; next }
    inside && $0 == "```" { exit }
    inside { print }
  '
}

ask() {
  "$BIN" chat --prompt "$1" 2>/dev/null
}

# --- generation ------------------------------------------------------------
generate_language() {
  local language="$1" record display_name fence program_file guide_prompt
  record="$(record_for_language "$language")"
  display_name="$(contract_field "$record" display_name)"
  fence="$(contract_field "$record" code_fence)"
  program_file="$(contract_field "$record" program_file)"

  local dir="$OUT_DIR/$language"
  rm -rf "$dir"
  mkdir -p "$dir/prompts" "$dir/answers" "$dir/locales"

  local locale prompt answer program reference=""
  for locale in $(locales); do
    prompt="$(prompt_for "$locale" "$display_name")"
    printf '%s\n' "$prompt" >"$dir/prompts/$locale.txt"
    answer="$(ask "$prompt")"
    printf '%s\n' "$answer" >"$dir/answers/$locale.md"
    program="$(printf '%s\n' "$answer" | extract_fence "$fence")"
    if [ -z "$program" ]; then
      printf 'error: %s/%s answer carried no ```%s block\n' "$language" "$locale" "$fence" >&2
      return 1
    fi
    mkdir -p "$dir/locales/$locale"
    printf '%s\n' "$program" >"$dir/locales/$locale/$program_file"
    if [ -z "$reference" ]; then
      reference="$locale"
    elif ! cmp -s "$dir/locales/$reference/$program_file" "$dir/locales/$locale/$program_file"; then
      printf 'error: %s program differs between locales %s and %s\n' \
        "$language" "$reference" "$locale" >&2
      diff -u "$dir/locales/$reference/$program_file" "$dir/locales/$locale/$program_file" >&2 || true
      return 1
    fi
  done
  cp "$dir/locales/$reference/$program_file" "$dir/$program_file"

  # The init script is generated the same way: a README step list goes in,
  # a runnable shell script comes out of the installation-conversion handler.
  local guide="## Installation" step command index=1
  while true; do
    step="$(contract_field "$record" "init_step_$index")"
    command="$(contract_field "$record" "init_command_$index")"
    if [ -z "$step" ] || [ -z "$command" ]; then
      break
    fi
    guide="$guide
$index. $step
   \`$command\`"
    index=$((index + 1))
  done
  if [ "$index" -eq 1 ]; then
    printf 'error: %s has no init steps in the contract\n' "$language" >&2
    return 1
  fi
  printf '%s\n' "$guide" >"$dir/install-guide.md"

  guide_prompt="Convert this README.md installation guide into a sh script:

\`\`\`markdown
$guide
\`\`\`"
  printf '%s\n' "$guide_prompt" >"$dir/prompts/install-guide.txt"
  answer="$(ask "$guide_prompt")"
  printf '%s\n' "$answer" >"$dir/answers/install-guide.md"
  printf '%s\n' "$answer" | extract_fence bash >"$dir/init.sh"
  if [ ! -s "$dir/init.sh" ]; then
    printf 'error: %s install-guide answer carried no ```bash block\n' "$language" >&2
    return 1
  fi
  # Every contract command must survive the conversion: the container has to run
  # the traditional init commands, not a paraphrase of them.
  index=1
  while true; do
    command="$(contract_field "$record" "init_command_$index")"
    [ -n "$command" ] || break
    if ! grep -Fqx "$command" "$dir/init.sh"; then
      printf 'error: %s init.sh dropped the command: %s\n' "$language" "$command" >&2
      return 1
    fi
    index=$((index + 1))
  done

  {
    printf 'LANGUAGE=%s\n' "$language"
    printf 'IMAGE=%s:%s\n' "$(contract_field "$record" image)" \
      "$(contract_field box_language_projects image_tag)"
    printf 'PROGRAM_FILE=%s\n' "$program_file"
    printf 'PROJECT_DIRECTORY=%s\n' "$(contract_field box_language_projects project_directory)"
    printf 'EXPECTED_OUTPUT=%s\n' "$(contract_field box_language_projects expected_output)"
    printf 'PROMPT_LOCALE=%s\n' "$(contract_field "$record" prompt_locale)"
    printf 'SHELL_PRELUDE=%s\n' "$(contract_field "$record" shell_prelude)"
    printf 'NETWORK_REQUIRED=%s\n' "$(contract_field "$record" network_required)"
  } >"$dir/meta.env"

  printf 'corpus: %s -> %s\n' "$language" "$dir"
}

BIN="$(resolve_binary)"
if [ ! -x "$BIN" ]; then
  printf 'error: formal-ai binary not found at %s\n' "$BIN" >&2
  exit 1
fi

mkdir -p "$OUT_DIR"
if [ "$#" -gt 0 ]; then
  requested=("$@")
else
  mapfile -t requested < <(contract_languages)
fi

for language in "${requested[@]}"; do
  generate_language "$language"
done
