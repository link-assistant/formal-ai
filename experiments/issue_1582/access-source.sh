#!/usr/bin/env bash
set -eu
sed -n '145,195p' data/seed/meanings-feature-capability.lino
sed -n '213,263p' rust/src/solver_handlers/feature_capability.rs
rg -n 'FeatureCapability|feature_capability|capability_question' data/seed/*routing* data/seed/*registry* js/worker/formal_ai_worker_solve.js js/worker/formal_ai_worker_handler* js/agentic/crate/*route* | head -90
sed -n '300,360p' js/worker/formal_ai_worker_capabilities_and_runtime_rules.js
sed -n '1,70p' data/seed/multilingual-responses-capabilities.lino
sed -n '170,210p' rust/src/client_integrations.rs
sed -n '1,110p' rust/src/client_integrations/global_config.rs
sed -n '1,70p' js/provenance/drive-capture.mjs
