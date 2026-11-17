#!/usr/bin/env bash
# Supplied investigation, no independent authorship credit.
set -eu
case "${1:-routing}" in
routing)
 sed -n '1,200p' rust/src/solver_handlers/feature_capability.rs
 sed -n '1,130p' rust/tests/web/issue-1395-practical-help.test.mjs
 sed -n '1,160p' data/seed/feature-capabilities.lino
 sed -n '1,100p' data/seed/meanings-tool-access.lino
 sed -n '1,140p' rust/tests/web/server-structured-log-cli.test.mjs
 ;;
integration)
 rg --files rust/tests | rg 'client.integration'
 sed -n '1,170p' data/seed/client-integrations.lino
 sed -n '475,600p' data/seed/client-integrations.lino
 rg -n 'context_window|max_output|model_context|token_limit|num_ctx' data/seed/client-integrations.lino rust/src/client_integrations*.rs rust/src/client_integrations/*.rs
 sed -n '715,775p' rust/src/client_integrations.rs
 sed -n '1,105p' rust/src/structured_log.rs
 ;;
esac
