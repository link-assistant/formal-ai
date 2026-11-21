#!/usr/bin/env bash
# The pinned parser needs only its two declared production dependencies.
set -eu
npm ci --prefix .meta-language/js --omit=dev --ignore-scripts --no-audit --no-fund > /tmp/issue-1582-review/meta-language-install.log 2>&1 &
installation=$!
wait "$installation"
sha256sum /tmp/issue-1582-review/meta-language.tar.gz /tmp/issue-1582-review/shellcheck.tar.xz > /tmp/issue-1582-review/tool-archive-sha256.txt
