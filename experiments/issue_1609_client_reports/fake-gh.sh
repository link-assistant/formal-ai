#!/usr/bin/env bash
set -euo pipefail

case "$1 $2" in
  'gist create')
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
    printf '%s\n' 'https://github.com/link-assistant/formal-ai/issues/9999'
    ;;
  *) exit 2 ;;
esac
