#!/usr/bin/env bash
set -eu
out=docs/case-studies/issue-1582/raw-data/research
node experiments/issue_1582/gemini-context-probe.mjs > "$out/gemini-context-probe.json"
gh issue create --repo google-gemini/gemini-cli --title 'Support a per-model input context window override for custom gateways' --body-file experiments/issue_1582/gemini-upstream.md > "$out/gemini-upstream-url.txt"
gh issue view "$(cat "$out/gemini-upstream-url.txt")" --json number,title,body,url,state,createdAt > "$out/gemini-upstream-issue.json"
cat "$out/gemini-context-probe.json" "$out/gemini-upstream-url.txt"
