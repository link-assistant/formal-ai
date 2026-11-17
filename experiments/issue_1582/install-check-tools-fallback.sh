#!/usr/bin/env bash
set -eu
mkdir -p /tmp/issue-1582-shellcheck
gh release view --repo koalaman/shellcheck --json tagName,assets > /tmp/issue-1582-shellcheck/release.json
gh release download --repo koalaman/shellcheck --pattern 'shellcheck-v*.linux.x86_64.tar.xz' --dir /tmp/issue-1582-shellcheck
for archive in /tmp/issue-1582-shellcheck/*.tar.xz; do
  tar -xJf "$archive" -C /tmp/issue-1582-shellcheck
done
if ! test -d .meta-language/.git; then
  git init .meta-language
  git -C .meta-language remote add origin https://github.com/link-foundation/meta-language.git
fi
git -C .meta-language fetch --depth 1 origin eb0574d5528babd3b4aadece81590d8e2230b99b
git -C .meta-language checkout --detach FETCH_HEAD
