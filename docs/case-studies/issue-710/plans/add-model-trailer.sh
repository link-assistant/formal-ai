#!/usr/bin/env bash
# Rebuild the PR #888 branch chain adding `Formal-AI-Model: formal-ai` to the
# two commits that carry a `Formal-AI-Session` trailer without one.
#
# Every tree and every second parent is reused verbatim via `git commit-tree`,
# so no merge is redone and `git diff OLD NEW` must come out empty. Nothing is
# moved: the script only prints the new head, and the caller decides whether to
# point the branch at it.
set -euo pipefail
cd /tmp/wt888

OLD_HEAD=$(git rev-parse HEAD)
BASE=$(git rev-parse de88ca251)
declare -A MAP

for commit in $(git rev-list --reverse --topo-order "$OLD_HEAD" --not "$BASE"); do
  parents=""
  for parent in $(git rev-parse "$commit^@" 2>/dev/null || true); do
    mapped="${MAP[$parent]:-$parent}"
    parents="$parents -p $mapped"
  done

  tree=$(git rev-parse "$commit^{tree}")
  message=$(git log -1 --format='%B' "$commit")

  case "$commit" in
    8a20542453097bed8dce618f451446d58aeee91d|ae1194e7ce085de3fd1d43a585135c30ae37c1e0)
      message="${message}
Formal-AI-Model: formal-ai"
      ;;
  esac

  new=$(printf '%s' "$message" |
    GIT_AUTHOR_NAME="$(git log -1 --format='%an' "$commit")" \
    GIT_AUTHOR_EMAIL="$(git log -1 --format='%ae' "$commit")" \
    GIT_AUTHOR_DATE="$(git log -1 --format='%aD' "$commit")" \
    GIT_COMMITTER_NAME="$(git log -1 --format='%cn' "$commit")" \
    GIT_COMMITTER_EMAIL="$(git log -1 --format='%ce' "$commit")" \
    GIT_COMMITTER_DATE="$(git log -1 --format='%cD' "$commit")" \
    git commit-tree "$tree" $parents)
  MAP[$commit]=$new
done

NEW_HEAD=${MAP[$OLD_HEAD]}
echo "OLD_HEAD=$OLD_HEAD"
echo "NEW_HEAD=$NEW_HEAD"
echo "tree difference between them (must be empty):"
git diff "$OLD_HEAD" "$NEW_HEAD" --stat
echo "commit count: $(git rev-list --count "$NEW_HEAD" --not "$BASE")"
