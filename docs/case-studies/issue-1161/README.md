# Issue #1161 Case Study: The Config Relocation That Broke `gh`

Issue [#1161](https://github.com/link-assistant/formal-ai/issues/1161) (E126).
Verified on `main` at `d209aac64`; fixed on the `qa-reasoning-coding-bulk-fixes`
branch.

## What a user saw

On 2026-09-27 a Hive Mind session drove the Formal AI agent CLI (its own
product) through a task that needed GitHub. The host had isolated the agent's
config with the only lever the registry seemed to offer — the log of the
reproduced failure (`docs/case-studies/issue-921/hive-mind-to-formal-ai/failure-executor.log:15`)
records what the launcher did:

> 🧠 Formal AI: config XDG_CONFIG_HOME=/home/box/.cache/hive-mind/formal-ai/agent-ixKVC8/.config,
> seeded /home/box/.config/link-assistant-agent → .config/link-assistant-agent

`XDG_CONFIG_HOME` does not relocate one program's config; it relocates the
config of *every* XDG-respecting program in the session. `gh`'s
`hosts.yml` and git's config moved with it, so `gh` lost its authentication
inside the session and every GitHub step failed
([link-assistant/hive-mind#2314](https://github.com/link-assistant/hive-mind/issues/2314)).

The registry had the truth but did not say it: the `agent` entry knows its
config file (`~/.config/link-assistant-agent/opencode.json`) and its one-shot
mechanism (`LINK_ASSISTANT_AGENT_CONFIG_CONTENT`), but the global entry carried
no declaration of *how a host points the client at a different copy of that
file*. A host reading `formal-ai clients` saw a path under `~/.config` and
reached for `XDG_CONFIG_HOME`.

## Root cause

1. **The schema had no place for the answer.**
   `ClientIntegrationGlobalConfig` (`rust/src/seed/client_integrations.rs`)
   declared *where* a global config lives (`path`, `format`,
   `backup_suffix`, settings) but nothing about the mechanism that relocates
   it. The one-shot side had the vocabulary (`config_env`,
   `config_dir_env`, `config_content_env`, `temp_home_env` on the invocation),
   but a host asking about the *global* config got only a `~/.config` path —
   an implied XDG relocation.

2. **The agent's own variables were never named as registry data.**
   The agent CLI reads `LINK_ASSISTANT_AGENT_CONFIG` (file) and
   `LINK_ASSISTANT_AGENT_CONFIG_DIR` (directory), but neither appeared in this
   repository's registry — so a Hive Mind integrator had nothing to read and
   invented the `XDG_CONFIG_HOME` relocation instead.

## The change

| File | Change |
| --- | --- |
| `rust/src/seed/client_integrations.rs` | `ClientIntegrationGlobalConfig` gains `config_env: Vec<String>` with the issue-#1161 doc comment; `parse_global_config` parses `config_env "VAR"` children of a `global` block. |
| `data/seed/client-integrations.lino` (+ its committed `rust/embedded/` byte mirror) | Declarations, each with in-repo evidence (below): `agent` → `LINK_ASSISTANT_AGENT_CONFIG`, `LINK_ASSISTANT_AGENT_CONFIG_DIR`; `codex` → `CODEX_HOME`; `t3code` (openai) → `CODEX_HOME`; `opencode`, `opencode-vscode`, `opencode-desktop` → `OPENCODE_CONFIG`, `OPENCODE_CONFIG_DIR`. |
| `rust/src/cli_clients.rs` | `formal-ai clients --format json`: each `global_configs` entry gains `config_env` (array); the record gains the one-shot `config_env`/`config_dir_env`/`config_content_env`/`temp_home_env` fields. `formal-ai clients` (text): each `global[...]` line appends `via VAR or VAR`. Additive only. |
| `docs/configuration/agentic-clis.md` | New "Host integration contract" section (what a host must set, what it must not touch, the incident) and a *Relocate via* column on the integration table. |
| `docs/testing/agentic-cli-tools.md` | The registry-surface paragraph names `global_configs[].config_env`. |
| `rust/tests/unit/issue_1161_client_registry_env.rs` | New acceptance suite (below). |

### Per-client evidence

No variable is invented; every declaration cites something already in this
repository:

- **agent** — issue #1161 itself names `LINK_ASSISTANT_AGENT_CONFIG_DIR`; the
  wrapper's one-shot path already uses the sibling variable
  `LINK_ASSISTANT_AGENT_CONFIG_CONTENT` (`README.md`,
  `docs/configuration/agentic-clis.md`).
- **codex** — `docs/configuration/output-sessions.md` documents Codex
  sessions at `$CODEX_HOME/sessions/…` (normally `~/.codex/sessions/…`);
  `experiments/agentic_cli_matrix/README.md` resolves `CODEX_HOME` as "a
  directory holding `config.toml`"; `rust/tests/issue_760.rs` reads
  `$CODEX_HOME/config.toml`.
- **t3code** — its own ephemeral block relocates the same config file through
  `temp_home_env "CODEX_HOME"` + `temp_home_config_path "config.toml"`
  (`docs/configuration/t3-code.md`: "T3 uses an isolated `CODEX_HOME` in
  one-shot mode").
- **opencode family** — each block's own ephemeral declaration:
  `config_env "OPENCODE_CONFIG"` and `config_dir_env "OPENCODE_CONFIG_DIR"`
  (also `experiments/agentic_cli_matrix/README.md`: "`OPENCODE_CONFIG` → a
  JSON file").
- **nobody else** — `gemini`/`claude`/`qwen`/`aider` global entries write
  `~/.profile` (a shell profile has no relocation variable) and
  `cursor`/`grok` write fixed paths with no evidenced variable, so they
  declare none. A host must provision those files directly.

## Tests

`rust/tests/unit/issue_1161_client_registry_env.rs`, hermetic (no network;
the registry is seed data compiled into the crate):

1. `agent_global_config_names_its_own_relocation_env` — both agent variables,
   and exactly those two.
2. `codex_family_global_configs_name_codex_home` — codex default global and
   the t3code openai-protocol global.
3. `opencode_family_global_configs_name_both_opencode_variables` — all three
   opencode surfaces, both variables each.
4. `entries_without_a_relocation_variable_declare_none` — the six no-evidence
   clients declare empty lists everywhere.
5. `config_env_is_additive_to_the_existing_file_contract` — path, format,
   backup suffix and settings survive unchanged.
6. `embedded_mirror_of_the_seed_file_stays_in_sync` —
   `data/seed/client-integrations.lino` and its `rust/embedded/` mirror are
   byte-identical (the lib parses the mirror; a hand edit that skips
   `rust-script scripts/mirror-package-data.rs --write` would make every
   other test read stale data).

The JSON/text projection lives in `src/cli_clients.rs`, a bin module the lib
test target cannot import; it is a thin additive mapping reviewed against the
parsed data the tests pin. Command:
`RUSTUP_TOOLCHAIN=1.98.1 cargo test --test unit issue_1161_client_registry_env`.

## Honest boundaries

- The Hive Mind launcher change — actually setting
  `LINK_ASSISTANT_AGENT_CONFIG_DIR` instead of `XDG_CONFIG_HOME` — is
  cross-repo work in link-assistant/hive-mind#2314 and is not delivered or
  confirmed from this repository; what is delivered here is the registry data
  and the documented contract that change consumes.
- `js/src/config/config.ts` (named by the issue) lives in the
  link-assistant/agent repository, not here; the fix on this side is the
  registry-side declaration.
- The `--global` writer still installs into the home-relative path; a host
  that sets a declared variable is responsible for pointing the client at a
  copy it provisioned. Making the writer itself env-aware is not part of
  #1161.
