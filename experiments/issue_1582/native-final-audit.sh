#!/usr/bin/env bash
set -eu
exec > /tmp/issue-1582-native-final-audit.log 2>&1
rg -n 'FORMAL_AI_LIVE_FETCH|LIVE_FETCH_ENV_VAR|with_online\(|online: false|offline: true' rust/src js/server --glob '*.rs' --glob '*.mjs'
git diff --name-only origin/main -- '*.rs' ':!rust/tests/fixtures/js-rust-translation/**' | while IFS= read -r source; do rustfmt --edition 2024 --check "$source"; done
git diff --check
