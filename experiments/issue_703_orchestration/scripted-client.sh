#!/usr/bin/env sh

set -eu

case "$(basename "$0")" in
  codex)
    printf '# Formal AI\n' > README.md
    ;;
  claude)
    printf '# Formal AI\n\nScripted comparison candidate.\n' > README.md
    ;;
  *)
    printf 'unsupported scripted client: %s\n' "$0" >&2
    exit 2
    ;;
esac

printf 'scripted client completed: %s\n' "$(basename "$0")"
