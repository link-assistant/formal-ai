use std::fs;
use std::path::Path;

use formal_ai::seed::{client_integrations, ConfigFormat};

#[test]
fn issue_606_with_formal_ai_docs_and_seed_templates_are_traceable() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));

    let integrations = client_integrations();
    for expected in ["codex", "opencode", "gemini"] {
        assert!(
            integrations
                .iter()
                .any(|integration| integration.id == expected),
            "missing client integration seed for {expected}"
        );
    }
    let codex = integrations
        .iter()
        .find(|integration| integration.id == "codex")
        .expect("codex integration");
    assert_eq!(codex.global_config.format, ConfigFormat::Toml);
    assert!(codex
        .global_config
        .toml_settings
        .iter()
        .any(
            |(key, value)| key == "model_providers.{provider_id}.wire_api" && value == "responses"
        ));
    assert!(codex
        .invocation
        .args
        .iter()
        .any(|arg| arg.contains("wire_api")));

    let opencode = integrations
        .iter()
        .find(|integration| integration.id == "opencode")
        .expect("opencode integration");
    assert_eq!(opencode.global_config.format, ConfigFormat::Json);
    assert_eq!(opencode.invocation.config_env, "OPENCODE_CONFIG");
    assert!(opencode
        .global_config
        .json_settings
        .iter()
        .any(|(key, value)| key == "provider.{provider_id}.npm"
            && value == "@ai-sdk/openai-compatible"));

    let gemini = integrations
        .iter()
        .find(|integration| integration.id == "gemini")
        .expect("gemini integration");
    assert_eq!(gemini.global_config.format, ConfigFormat::ShellEnv);
    assert!(gemini.supported_protocols.contains(&String::from("vertex")));

    let seed = read(root.join("data/seed/client-integrations.lino"));
    assert_contains_all(
        "data/seed/client-integrations.lino",
        &seed,
        &[
            "client_integrations",
            "tool \"codex\"",
            "tool \"opencode\"",
            "tool \"gemini\"",
            "config_json_set",
            "toml_set",
            "json_set",
            "shell_env",
        ],
    );

    let readme = read(root.join("README.md"));
    assert_contains_all(
        "README.md",
        &readme,
        &[
            "### `formal-ai with` / `with-formal-ai`",
            "formal-ai with --start-server codex \"hi\"",
            "formal-ai with opencode run \"hi\"",
            "formal-ai with gemini -p \"hi\"",
            "with-formal-ai -g codex",
            "with-formal-ai -g --all",
            "with-formal-ai -g --undo codex",
            "data/seed/client-integrations.lino",
        ],
    );

    let server_api = read(root.join("docs/desktop/server-api.md"));
    assert_contains_all(
        "docs/desktop/server-api.md",
        &server_api,
        &[
            "with-formal-ai codex \"hi\"",
            "with-formal-ai opencode run \"hi\"",
            "with-formal-ai gemini -p \"hi\"",
            "with-formal-ai -g codex",
            "with-formal-ai -g --all",
            "with-formal-ai -g --undo codex",
            "client-integrations.lino",
        ],
    );
}

fn read(path: impl AsRef<Path>) -> String {
    fs::read_to_string(path.as_ref())
        .unwrap_or_else(|error| panic!("{} should be readable: {error}", path.as_ref().display()))
}

fn assert_contains_all(label: &str, content: &str, expected: &[&str]) {
    for needle in expected {
        assert!(
            content.contains(needle),
            "{label} should contain expected text: {needle}"
        );
    }
}
