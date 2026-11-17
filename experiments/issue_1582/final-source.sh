#!/usr/bin/env bash
set -eu
sed -n '310,435p' js/server/worker-host.mjs
rg -n 'WorkerHost|sourceFetch|nodeSource|worker =' js/server/main.mjs js/agentic/node-host.mjs
sed -n '150,240p' rust/src/client_integrations.rs
sed -n '430,500p' rust/src/client_integrations.rs
sed -n '730,840p' rust/src/client_integrations.rs
sed -n '1,100p' rust/src/client_integrations/global_config.rs
rg -n -A 65 'fn merge_json_settings|fn set_json_setting' rust/src/client_integrations/global_config.rs
sed -n '155,430p' data/seed/client-integrations.lino
sed -n '1,230p' rust/tests/integration/with_formal_ai.rs
