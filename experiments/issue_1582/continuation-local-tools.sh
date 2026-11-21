#!/usr/bin/env bash
# Local-only gate prerequisites, installed outside the repository without scripts.
set -eu
npm install --prefix /tmp/issue-1582-js-tools --ignore-scripts --no-audit --no-fund lino-i18n@0.4.0 yaml@2.8.2 > /tmp/issue-1582-review/local-tools-npm.log 2>&1 &
installation=$!
wait "$installation"
gh api repos/link-foundation/meta-language/tarball/eb0574d5528babd3b4aadece81590d8e2230b99b > /tmp/issue-1582-review/meta-language.tar.gz
mkdir -p .meta-language
tar -xzf /tmp/issue-1582-review/meta-language.tar.gz --strip-components=1 -C .meta-language
printf '%s\n' eb0574d5528babd3b4aadece81590d8e2230b99b > .meta-language/.formal-ai-meta-language-commit
cat .meta-language/js/package.json > /tmp/issue-1582-review/meta-language-package.json
curl --fail --location --silent --show-error https://github.com/koalaman/shellcheck/releases/download/v0.11.0/shellcheck-v0.11.0.linux.x86_64.tar.xz -o /tmp/issue-1582-review/shellcheck.tar.xz
tar -xJf /tmp/issue-1582-review/shellcheck.tar.xz -C /tmp/issue-1582-js-tools
/tmp/issue-1582-js-tools/shellcheck-v0.11.0/shellcheck --version
