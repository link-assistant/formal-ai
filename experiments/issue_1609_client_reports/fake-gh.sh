#!/usr/bin/env bash
set -euo pipefail

case "$1 $2" in
  'gist create')
    test "$3" = --filename
    test "$4" = "$(basename -- "${@: -1}")"
    gist_dir="${FORMAL_AI_TEST_ISSUE_BODY%.md}-gists"
    mkdir -p "$gist_dir"
    cp -- "${@: -1}" "$gist_dir/$4"
    printf '%s\n' 'https://gist.github.com/formal-ai/release-client-report'
    ;;
  'issue create')
    shift 2
    while [ "$#" -gt 0 ]; do
      case "$1" in
        --body-file) cp -- "$2" "$FORMAL_AI_TEST_ISSUE_BODY"; shift 2 ;;
        --repo) test "$2" = link-assistant/formal-ai; shift 2 ;;
        *) shift ;;
      esac
    done
    test -s "$FORMAL_AI_TEST_ISSUE_BODY"
    # A large Markdown preview can omit the assembly trace. The merged
    # attachment retains the source counts even when the preview drops them.
    merged="${FORMAL_AI_TEST_ISSUE_BODY%.md}-gists/context.lino"
    for source in harness server; do
      if ! grep -Eq "^    ${source}_message_count [1-9][0-9]*$" "$merged"; then
        printf 'The merged context is missing a nonempty %s capture: %s\n' "$source" "$merged" >&2
        exit 1
      fi
    done
    printf '%s\n' 'https://github.com/link-assistant/formal-ai/issues/9999'
    ;;
  *) exit 2 ;;
esac
