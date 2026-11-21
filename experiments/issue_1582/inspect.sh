#!/usr/bin/env bash
# Externally authored investigation recipe; executing this earns no authorship credit.
set -eu
case "${1:-runtime}" in
runtime)
  sed -n '165,255p' js/agentic/node-host.mjs
  sed -n '1,230p' js/server/structured-log.mjs
  sed -n '1,180p' rust/src/structured_log.rs
  sed -n '330,470p' rust/src/structured_log.rs
  sed -n '390,490p' rust/src/client_integrations.rs
  sed -n '420,490p' data/seed/client-integrations.lino
  rg -n 'web_search_unavailable|enable internet|LIVE_FETCH|web_search' js/agentic/crate/engine.mjs js/agentic/crate/seed_intents.mjs rust/src/solver_handlers/web_requests.rs data/seed/*web* data/seed/*intent* | head -90
  ;;
tests)
  rg --files rust/tests/web | rg '(client-integrations|discovery|permission|context-capacity|structured-log|web-requests|live-search)'
  sed -n '1,230p' rust/tests/web/server-structured-log.test.mjs
  sed -n '1,240p' rust/tests/web/client-integrations.test.mjs
  ;;
policy)
  cat AGENTS.md
  rg -n 'dogfood|Formal AI|changelog|release|requirements|JS-first' CONTRIBUTING.md | head -100
  cat .github/actions/dispatch-checks/action.yml
  rg -n 'workflow_dispatch|action_required|approval|dispatch-checks' .github/workflows/*.yml | head -90
  ;;
esac
