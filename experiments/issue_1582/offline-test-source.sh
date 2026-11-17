#!/usr/bin/env bash
set -u
sed -n '1,160p' rust/src/solver_config.rs
sed -n '1,100p' rust/src/solver_handlers/web_requests/live_search.rs
rg -n 'offline|with_.*client|cfg\(test\)|default' rust/src/solver_unknown_reasoning.rs rust/src/concept_lookup.rs scripts/cargo-test.sh rust/tests/unit/web-engine-core/issue_1172_factual_qa_subject_match.rs
sed -n '1,90p' rust/tests/unit/web-engine-core/issue_1172_factual_qa_subject_match.rs
sed -n '60,100p' js/server/structured-log.mjs
