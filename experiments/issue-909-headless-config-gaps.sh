#!/usr/bin/env bash
# Issue #909: `formal-ai with <tool> --global` wrote only shell exports, which is
# not enough for gemini (no *selected* auth type) or qwen (incomplete OpenAI
# triple) to start headlessly.
#
# This script reproduces the check offline: it runs `--global` into a throwaway
# HOME — the operator's real profile is never touched — and reports, per tool,
# whether every piece the client needs for a headless start is present.
#
# Usage: experiments/issue-909-headless-config-gaps.sh [path-to-formal-ai]
set -euo pipefail

repository_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
formal_ai="${1:-${repository_root}/target/debug/formal-ai}"
if [[ ! -x "${formal_ai}" ]]; then
  echo "build the binary first: cargo build --bin formal-ai" >&2
  exit 1
fi

throwaway_home="$(mktemp -d)"
trap 'rm -rf "${throwaway_home}"' EXIT
base_url="http://127.0.0.1:18080"

failures=0

report() {
  local tool="$1" description="$2" found="$3"
  if [[ "${found}" == "yes" ]]; then
    printf '      present: %s\n' "${description}"
  else
    printf '      MISSING: %s\n' "${description}"
    failures=$((failures + 1))
  fi
}

for tool in gemini qwen; do
  printf '########## %s\n' "${tool}"
  # The flags come before the tool name: everything after it is forwarded to
  # the client itself.
  HOME="${throwaway_home}" "${formal_ai}" with --global --base-url "${base_url}" "${tool}" \
    | sed 's/^/  /'

  printf '  --- what --global wrote to ~/.profile\n'
  sed -n "/^# >>> formal-ai ${tool}\$/,/^# <<< formal-ai ${tool}\$/p" \
    "${throwaway_home}/.profile" | sed 's/^/      /'

  case "${tool}" in
    gemini)
      settings="${throwaway_home}/.gemini/settings.json"
      printf '  --- companion settings file\n'
      if [[ -f "${settings}" ]]; then
        sed 's/^/      /' "${settings}"
        selected="$(python3 -c 'import json,sys; print(json.load(open(sys.argv[1])).get("security",{}).get("auth",{}).get("selectedType",""))' "${settings}")"
        [[ -n "${selected}" ]] && found=yes || found=no
      else
        printf '      (no %s)\n' "${settings}"
        found=no
      fi
      report "${tool}" "security.auth.selectedType in ~/.gemini/settings.json" "${found}"
      ;;
    qwen)
      for name in OPENAI_API_KEY OPENAI_BASE_URL OPENAI_MODEL; do
        if grep -q "^export ${name}=" "${throwaway_home}/.profile"; then
          found=yes
        else
          found=no
        fi
        report "${tool}" "export ${name} in ~/.profile" "${found}"
      done
      ;;
  esac
done

printf '\n'
if (( failures == 0 )); then
  printf 'every headless requirement is written by --global\n'
else
  printf '%d headless requirement(s) missing\n' "${failures}"
  exit 1
fi
