#!/usr/bin/env bash
# Supplied finite source review, output retained separately.
set -eu
git diff origin/main -- rust/src/client_integrations.rs rust/src/client_integrations/global_config.rs rust/src/client_integrations/global_config_settings.rs rust/src/client_integrations/global_config_toml.rs > /tmp/issue-1582-review/native-client.diff
git diff origin/main -- rust/src/structured_log.rs rust/src/cli_logs.rs rust/src/cli_env.rs rust/src/main.rs rust/src/source_fetch.rs > /tmp/issue-1582-review/native-runtime.diff
rg -n 'context_window_tokens|model_limits|generate|RenderContext' rust/src/client_integrations.rs rust/src/client_integrations/*.rs > /tmp/issue-1582-review/client-context-index.txt
rg -n '^  (tool|language)|^    (default_protocol|supported_protocols)|internet.*permission|permission.*internet' data/seed/client-integrations.lino data/seed/languages.lino data/seed/meanings-feature-capability.lino data/seed/multilingual-responses-capabilities.lino > /tmp/issue-1582-review/support-index.txt
node experiments/issue_1582/continuation-behavior-probe.mjs > /tmp/issue-1582-review/language-detection.jsonl
wc -l /tmp/issue-1582-review/native-client.diff /tmp/issue-1582-review/native-runtime.diff /tmp/issue-1582-review/client-context-index.txt
