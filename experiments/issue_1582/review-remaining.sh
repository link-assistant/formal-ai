#!/usr/bin/env bash
set -u
sed -n '190,220p' data/seed/multilingual-responses-capabilities.lino
sed -n '1,155p' js/server/structured-log.mjs
rg -n 'convertLog|convert_log|decodeTyped' js/server/*log* js/*log* rust/src/*log* | head -65
sed -n '1,140p' scripts/translate-js-rust.mjs
git diff HEAD -- rust/tests/unit/issue_1173_fallback_executes_search.rs rust/tests/unit/web-engine-core/issue_1172_factual_qa_subject_match.rs rust/tests/integration/with_formal_ai/fixture_env.rs
gh pr view 1583 --repo link-assistant/formal-ai --json title,body,isDraft,headRefOid,baseRefName,mergeStateStatus
