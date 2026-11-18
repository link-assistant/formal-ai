#!/usr/bin/env bash
# Temporary dependencies for existing checks; no project dependency changes.
set -eu
npm install --prefix /tmp/issue-1582-js-tools --ignore-scripts --no-audit --no-fund yaml lino-i18n
mkdir -p /tmp/issue-1582-shellcheck
(cd /tmp/issue-1582-shellcheck && apt-get download shellcheck)
for package in /tmp/issue-1582-shellcheck/*.deb; do
  dpkg-deb --extract "$package" /tmp/issue-1582-shellcheck/root
done
if ! test -d .meta-language/.git; then
  git init .meta-language
  git -C .meta-language remote add origin https://github.com/link-foundation/meta-language.git
fi
git -C .meta-language fetch --depth 1 origin eb0574d5528babd3b4aadece81590d8e2230b99b
git -C .meta-language checkout --detach FETCH_HEAD
