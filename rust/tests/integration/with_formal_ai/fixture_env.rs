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
                "{key} must be cleared before a fake client can observe it"
            );
        }
    }
}
