#!/usr/bin/env bash
set -euo pipefail

# Hive Mind 2.12.2 still requires repository write permission before its
# prepare-only exit. Let a read-only CI token cross only that preflight while
# refusing any later command that could mutate GitHub state.
if [[ "${1:-}" == "api" ]] && \
  [[ "${2:-}" == "repos/link-assistant/formal-ai" ]] && \
  [[ " $* " == *" --jq .permissions "* ]]; then
  printf '%s\n' '{"admin":false,"maintain":false,"push":true,"triage":false,"pull":true}'
  exit 0
fi

refuse_mutation() {
  printf '%s\n' 'read-only prepare wrapper refused GitHub mutation' >&2
  exit 77
}

case "${1:-}" in
  "issue" | "pr")
    case "${2:-}" in
      close | comment | create | delete | develop | edit | lock | merge | pin | ready | \
        reopen | review | transfer | unlock | unpin)
        refuse_mutation
        ;;
    esac
    ;;
  "api")
    # `gh api --method=POST` and field/input flags can mutate state even when
    # the endpoint itself looks like an ordinary repository URL.
    previous=''
    for argument in "$@"; do
      if [[ "${previous}" == "-X" || "${previous}" == "--method" ]]; then
        case "${argument^^}" in
          POST | PUT | PATCH | DELETE) refuse_mutation ;;
        esac
      fi
      case "${argument^^}" in
        -XPOST | -XPUT | -XPATCH | -XDELETE | --METHOD=POST | --METHOD=PUT | \
          --METHOD=PATCH | --METHOD=DELETE)
          refuse_mutation
          ;;
      esac
      case "${argument}" in
        -f | -F | -f?* | -F?* | --field | --field=* | --raw-field | --raw-field=* | \
          --input | --input=*)
          refuse_mutation
          ;;
      esac
      previous="${argument}"
    done
    ;;
esac

exec "${REAL_GH:?REAL_GH must point to the authenticated gh executable}" "$@"
