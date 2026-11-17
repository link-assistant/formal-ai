#!/usr/bin/env bash
set -u
git diff HEAD -- rust/src/client_integrations.rs rust/src/client_integrations/global_config.rs data/seed/client-integrations.lino rust/src/structured_log.rs rust/src/main.rs
sed -n '1,120p' rust/src/client_integrations/server.rs
rg -n 'PermissionsExt|set_permissions|\.mode\(' rust/tests/integration/with_formal_ai.rs rust/tests/integration/with_formal_ai/fixture_env.rs
