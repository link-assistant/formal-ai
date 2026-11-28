//! Fake clients must never capture a developer's credentials or launch overrides.
use std::process::Command;

pub(super) fn clear_client_environment(command: &mut Command) {
    // These are the two expansions of the catalog's protocol-base placeholder.
    for key in ["GOOGLE_GEMINI_BASE_URL", "GOOGLE_VERTEX_BASE_URL"] {
        command.env_remove(key);
    }
    for integration in formal_ai::seed::client_integrations() {
        let invocation = &integration.invocation;
        for key in [
            integration.api_key_env.as_str(),
            integration.command_env.as_str(),
            invocation.config_content_env.as_str(),
            invocation.config_env.as_str(),
            invocation.config_dir_env.as_str(),
            invocation.temp_home_env.as_str(),
        ]
        .into_iter()
        .chain(invocation.env.iter().map(|entry| entry.key.as_str()))
        {
            // HOME is explicitly redirected by each fixture, not a credential.
            if !key.is_empty() && !key.contains(['{', '}']) && key != "HOME" && key != "USERPROFILE"
            {
                command.env_remove(key);
            }
        }
    }
}

#[test]
fn all_seeded_credentials_and_command_overrides_are_removed_from_fixtures() {
    for integration in formal_ai::seed::client_integrations() {
        for key in [&integration.api_key_env, &integration.command_env] {
            if key.is_empty() {
                continue;
            }
            let mut command = Command::new("unused-fixture");
            command.env(key, "synthetic-test-value");
            clear_client_environment(&mut command);
            assert!(
                command
                    .get_envs()
                    .any(|(name, value)| name == key.as_str() && value.is_none()),
                "client credentials and command overrides must be cleared before fake client execution"
            );
        }
    }
}

use std::os::unix::fs::PermissionsExt;
use std::path::Path;

pub fn write_fake_cli(bin_dir: &Path, name: &str) {
    let path = bin_dir.join(name);
    std::fs::write(
        &path,
        r#"#!/bin/sh
{
  echo "tool=$0"
  i=0
  for arg in "$@"; do
    echo "arg[$i]=$arg"
    i=$((i + 1))
  done
  echo "FORMAL_AI_API_KEY=$FORMAL_AI_API_KEY"
  echo "LINK_ASSISTANT_AGENT_CONFIG_CONTENT=$LINK_ASSISTANT_AGENT_CONFIG_CONTENT"
  echo "OPENCODE_CONFIG=$OPENCODE_CONFIG"
  echo "OPENCODE_CONFIG_DIR=$OPENCODE_CONFIG_DIR"
  echo "OPENCODE_ENABLE_EXA=$OPENCODE_ENABLE_EXA"
  echo "GEMINI_API_KEY=$GEMINI_API_KEY"
  echo "GEMINI_DEFAULT_AUTH_TYPE=$GEMINI_DEFAULT_AUTH_TYPE"
  echo "GEMINI_CLI_TRUST_WORKSPACE=$GEMINI_CLI_TRUST_WORKSPACE"
  echo "GEMINI_CLI_HOME=$GEMINI_CLI_HOME"
  echo "GOOGLE_GEMINI_BASE_URL=$GOOGLE_GEMINI_BASE_URL"
  echo "GOOGLE_VERTEX_BASE_URL=$GOOGLE_VERTEX_BASE_URL"
  echo "ANTHROPIC_AUTH_TOKEN=$ANTHROPIC_AUTH_TOKEN"
  echo "ANTHROPIC_API_KEY=$ANTHROPIC_API_KEY"
  echo "ANTHROPIC_BASE_URL=$ANTHROPIC_BASE_URL"
  echo "CLAUDE_CODE_MAX_CONTEXT_TOKENS=$CLAUDE_CODE_MAX_CONTEXT_TOKENS"
  echo "CLAUDE_CODE_MAX_OUTPUT_TOKENS=$CLAUDE_CODE_MAX_OUTPUT_TOKENS"
  echo "OPENAI_API_KEY=$OPENAI_API_KEY"
  echo "OPENAI_BASE_URL=$OPENAI_BASE_URL"
  echo "OPENAI_API_BASE=$OPENAI_API_BASE"
  echo "OPENAI_MODEL=$OPENAI_MODEL"
  echo "GROK_API_KEY=$GROK_API_KEY"
  echo "GROK_BASE_URL=$GROK_BASE_URL"
  if [ -f "$HOME/.codex/config.toml" ]; then
    echo "---CODEX_CONFIG---"
    cat "$HOME/.codex/config.toml"
  fi
  model_catalog_path=""
  for arg in "$@"; do
    case "$arg" in
      model_catalog_json=*)
        model_catalog_path=${arg#model_catalog_json=}
        model_catalog_path=${model_catalog_path#\"}
        model_catalog_path=${model_catalog_path%\"}
        ;;
    esac
  done
  if [ -z "$model_catalog_path" ] && [ -f "$HOME/.codex/config.toml" ]; then
    model_catalog_path=$(sed -n 's/^model_catalog_json = "\(.*\)"$/\1/p' "$HOME/.codex/config.toml")
  fi
  if [ -n "$model_catalog_path" ] && [ -f "$model_catalog_path" ]; then
    echo "---CODEX_MODEL_CATALOG---"
    cat "$model_catalog_path"
  fi
  if [ -n "$OPENCODE_CONFIG" ] && [ -f "$OPENCODE_CONFIG" ]; then
    echo "---OPENCODE_CONFIG---"
    cat "$OPENCODE_CONFIG"
  fi
  if [ -n "$GEMINI_CLI_HOME" ] && [ -f "$GEMINI_CLI_HOME/.gemini/settings.json" ]; then
    echo "---GEMINI_CLI_SETTINGS---"
    cat "$GEMINI_CLI_HOME/.gemini/settings.json"
  fi
  if [ -n "$CLAUDE_CONFIG_DIR" ] && [ -f "$CLAUDE_CONFIG_DIR/.claude.json" ]; then
    echo "---CLAUDE_CONFIG---"
    cat "$CLAUDE_CONFIG_DIR/.claude.json"
  fi
  if [ -f "$HOME/.gemini/settings.json" ]; then
    echo "---HOME_GEMINI_SETTINGS---"
    cat "$HOME/.gemini/settings.json"
  fi
} > "$FORMAL_AI_CAPTURE"
printf 'Hi, how may I help you?\n'
"#,
    )
    .expect("write fake cli");
    let mut permissions = std::fs::metadata(&path).expect("metadata").permissions();
    permissions.set_mode(0o755);
    std::fs::set_permissions(&path, permissions).expect("chmod fake cli");
}
