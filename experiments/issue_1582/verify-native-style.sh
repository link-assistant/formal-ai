#!/usr/bin/env bash
set -eu
git diff --name-only origin/main -- '*.rs' | while IFS= read -r source; do rustfmt --edition 2024 --check "$source"; done
git diff --check
sed -n '1,135p' data/seed/client-integrations.lino
git log -2 --format='%h %s'
