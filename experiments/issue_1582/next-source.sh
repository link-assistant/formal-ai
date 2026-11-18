#!/usr/bin/env bash
set -eu
sed -n '275,345p' js/worker/formal_ai_worker_solve.js
rg -n 'procedural_how_to|feature_capability|try_feature' rust/src/solver*.rs rust/src/solver_handlers/*.rs data/seed/*.lino | head -90
sed -n '1,115p' js/logging/lino-codec.mjs
sed -n '1,85p' js/logging/typed-codec.mjs
sed -n '760,800p' rust/src/client_integrations.rs
sed -n '195,265p' rust/src/client_integrations/global_config.rs
sed -n '1,110p' .github/actions/dispatch-checks/action.yml
