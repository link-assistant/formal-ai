#!/usr/bin/env bash
set -eu
git add docs/status.md experiments/issue_1582/commit-status.sh
git diff --cached --check
git commit -m "docs: retain regenerated native capability status"
git status --short
