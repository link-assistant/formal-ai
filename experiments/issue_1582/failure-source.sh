#!/usr/bin/env bash
set -eu
for name in check-generated-files check-hardcoded-language check-hardcoded-language-js-twin check-minimal-core-boundary check-notation check-progressive-plan check-language-parity; do
  if test -f "experiments/formal_ai_subagent/sandboxes/gates/$name.log"; then
    head -70 "experiments/formal_ai_subagent/sandboxes/gates/$name.log"
  fi
done
sed -n '1,230p' rust/tests/unit/issue_1173_fallback_executes_search.rs
sed -n '155,205p' rust/tests/unit/web-engine-core/issue_1172_factual_qa_subject_match.rs
rg -n 'LIVE_FETCH|offline|test' scripts/cargo-test.sh | head -45
rg --files rust/tests | rg '(main|mod)\.rs$' | head -20
