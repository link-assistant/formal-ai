#!/usr/bin/env bash
set -eu
gh api repos/google-gemini/gemini-cli/commits/main --jq '{sha:.sha,committed:.commit.committer.date}' > /tmp/issue-1582-review/gemini-upstream-head.json
revision=$(jq -r .sha /tmp/issue-1582-review/gemini-upstream-head.json)
for entry in core/tokenLimits config/models core/client context/chatCompressionService services/modelConfigService; do
  name=${entry##*/}
  gh api "repos/google-gemini/gemini-cli/contents/packages/core/src/$entry.ts?ref=$revision" --jq .content | base64 -d > "/tmp/issue-1582-review/gemini-pinned-$name.ts"
done
gh issue view https://github.com/google-gemini/gemini-cli/issues/30333 --json title,body,state,comments > /tmp/issue-1582-review/gemini-upstream-issue.json
node --input-type=module - <<'JS' > /tmp/issue-1582-review/gemini-pinned-probe.log
import fs from 'node:fs';
import assert from 'node:assert/strict';
const source=fs.readFileSync('/tmp/issue-1582-review/gemini-pinned-tokenLimits.ts','utf8');
const names=source.match(/import \{([\s\S]*?)\} from/u)[1].split(',').map(x=>x.trim()).filter(Boolean);
const fn=source.match(/export function tokenLimit\([\s\S]*?\n\}/u)[0].replace('export ','').replace('model: Model','model').replace('): TokenCount',')');
const actual=Function(...names,'DEFAULT_TOKEN_LIMIT','GEMMA_4_TOKEN_LIMIT','return '+fn)(...names,1048576,256000)('formal-ai');
assert.equal(actual,1048576);assert.notEqual(actual,1234567);
console.log(JSON.stringify({source:JSON.parse(fs.readFileSync('/tmp/issue-1582-review/gemini-upstream-head.json','utf8')),kind:'source-only probe; not installed CLI',model:'formal-ai',advertisedContext:1234567,actualInputBudget:actual}));
JS
git fetch origin main > /tmp/issue-1582-review/fetch-final-main.log 2>&1
git merge-base --is-ancestor origin/main HEAD
gh pr diff 1583 --repo link-assistant/formal-ai > /tmp/issue-1582-review/full-pr.diff
git diff --merge-base origin/main -- . ':!docs/case-studies/issue-1582/raw-data' ':!dev/log' > /tmp/issue-1582-review/source-review.diff
wc -l /tmp/issue-1582-review/source-review.diff /tmp/issue-1582-review/full-pr.diff
