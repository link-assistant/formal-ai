#!/usr/bin/env bash
# Supplied exploration for reusable request recognition.
set -eu
sed -n '540,575p' data/seed/roles.lino
sed -n '1,110p' rust/src/solver_handlers/feature_capability.rs
rg -n 'requests_role|mentions_role|role.*clause|clause.*role' rust/src/seed/lexicon*.rs | head -70
rg -n 'lexiconRequestsRole|lexiconMentionsRole|tryProceduralHowTo|internet-access-help' js/worker/formal_ai_worker*js | head -70
rg -n 'requests_role|request_role|query.*role' data/seed/meanings-feature-capability.lino data/seed/meanings*permission* data/seed/roles.lino || true
sed -n '1,220p' scripts/translate-js-rust.mjs
rg -n 'capabilities_and_runtime_rules|feature_capability.rs' data/meta/*lino data/meta/**/*lino | head -70
sed -n '1,170p' rust/tests/unit/feature_capabilities.rs 2>/dev/null || true
rg -n 'internet-access-help|feature_capability' rust/tests/unit | head -40
