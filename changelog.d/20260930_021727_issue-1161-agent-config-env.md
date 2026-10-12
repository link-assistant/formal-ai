---
bump: patch
---

### Fixed

- The client registry now declares *how* a client is pointed at its config
  instead of implying an `XDG_CONFIG_HOME` relocation that also moves `gh`'s
  and git's config and drops the session's GitHub authentication (the
  2026-09-27 Hive Mind incident, link-assistant/hive-mind#2314). Every
  `global_configs` entry in `data/seed/client-integrations.lino` may carry
  `config_env "VAR"` declarations — parsed into
  `ClientIntegrationGlobalConfig::config_env`, surfaced additively as
  `global_configs[].config_env` in `formal-ai clients --format json` and as
  `via VAR` on each `global[...]` line of the text output. Filled only where
  in-repo evidence exists: `agent` → `LINK_ASSISTANT_AGENT_CONFIG` /
  `LINK_ASSISTANT_AGENT_CONFIG_DIR`, `codex` and the `t3code` OpenAI entry →
  `CODEX_HOME`, the opencode family → `OPENCODE_CONFIG` /
  `OPENCODE_CONFIG_DIR`; profile-writing and fixed-path entries declare none.
  A documented host integration contract (`docs/configuration/agentic-clis.md`)
  requires hosts to use the declared variables and never relocate
  `XDG_CONFIG_HOME`/`HOME` (issue #1161).
