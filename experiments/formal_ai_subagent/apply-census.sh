#!/usr/bin/env bash
# Apply the self-AST census that CI regenerated for a commit.
#
# The census (data/meta/self-ast/ and its companions) is derived from the
# Rust sources, and only CI builds Rust. Any change under rust/src makes the
# committed census drift, so after a push the "Regenerate self-AST census"
# workflow uploads the regenerated files and a census-only commit lands them.
# The paths come from the workflow's upload step, so this script and CI
# cannot disagree about what the census is.
#
# Usage:
#   bash experiments/formal_ai_subagent/apply-census.sh            # census of HEAD
#   bash experiments/formal_ai_subagent/apply-census.sh <commit>   # census of a commit
#   bash experiments/formal_ai_subagent/apply-census.sh <run-id>   # census of a run
set -euo pipefail

workflow=.github/workflows/regenerate-self-ast-census.yml
root=$(git rev-parse --show-toplevel)
cd "$root"

target=${1:-$(git rev-parse HEAD)}
if [[ $target =~ ^[0-9]+$ ]]; then
  run=$target
else
  run=$(gh run list --workflow "$(basename "$workflow")" --commit "$(git rev-parse "$target")" \
    --status success --limit 1 --json databaseId --jq '.[0].databaseId // empty')
fi
if [ -z "$run" ]; then
  echo "No successful census run for $target yet; wait for the workflow." >&2
  exit 1
fi

# The artifact name and the `path: |` block of the upload step.
artifact=$(awk '/upload-artifact/ {up=1} up && /name:/ {print $2; exit}' "$workflow")
paths=()
while IFS= read -r path; do paths+=("$path"); done < <(awk '
  /upload-artifact/ {up=1}
  up && /path: \|/ {inside=1; indent=-1; next}
  inside {
    match($0, /^ */); if (indent < 0) indent = RLENGTH
    if (RLENGTH < indent || $0 ~ /^ *$/ || $0 ~ /:/) exit
    sub(/^ +/, ""); print
  }' "$workflow")

download=experiments/formal_ai_subagent/sandboxes/census-$run
rm -rf "$download"
gh run download "$run" --name "$artifact" --dir "$download"
for path in "${paths[@]}"; do
  if [ -d "$download/$path" ]; then
    rsync -a --delete "$download/$path/" "$path/"
  elif [ -f "$download/$path" ]; then
    mkdir -p "$(dirname "$path")"
    cp "$download/$path" "$path"
  else
    echo "The artifact of run $run has no $path." >&2
    exit 1
  fi
done
rm -rf "$download"
echo "Applied the census of run $run:"
git status --short -- "${paths[@]}"
