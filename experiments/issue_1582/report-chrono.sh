#!/usr/bin/env bash
set -eu
root=docs/case-studies/issue-1582/raw-data/research
for repository in link-foundation/command-stream link-assistant/calculator; do
  name=${repository##*/}
  target="$root/$name-chrono-report-url.txt"
  if [ ! -s "$target" ]; then
    gh issue create --repo "$repository" --title "Replace unmaintained chrono dependency (RUSTSEC-2026-0543)" --body-file experiments/issue_1582/chrono-report.md > "$target"
  fi
  cat "$target"
  gh issue view "$(cat "$target")" --json url,number,title,body,state,createdAt > "$root/$name-chrono-report.json"
done
