use std::fs;
use std::path::Path;

use formal_ai::seed::client_integrations;

#[test]
fn issue_622_wrapper_docs_match_current_cli_seed_behavior() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let integrations = client_integrations();
    let codex = integrations
        .iter()
        .find(|integration| integration.id == "codex")
        .expect("codex integration");

    let sandbox_window = codex.invocation.args.windows(3).any(|args| {
        args[0] == "--skip-git-repo-check" && args[1] == "--sandbox" && args[2] == "read-only"
    });
    assert!(
        sandbox_window,
        "Codex one-shot wrapper should match the documented git and sandbox flags"
    );

    let readme = read(root.join("README.md"));
    assert_contains_all(
        "README.md",
        &readme,
        &[
            "formal-ai with agent -p \"hi\"",
            "temporary `GEMINI_CLI_HOME`",
            "`LINK_ASSISTANT_AGENT_CONFIG_CONTENT`",
            "`codex exec --skip-git-repo-check --sandbox read-only`",
        ],
    );

    let server_api = read(root.join("docs/desktop/server-api.md"));
    assert_contains_all(
        "docs/desktop/server-api.md",
        &server_api,
        &[
            "with-formal-ai agent -p \"hi\"",
            "`GEMINI_CLI_HOME`",
            "`LINK_ASSISTANT_AGENT_CONFIG_CONTENT`",
            "`codex exec --skip-git-repo-check --sandbox read-only`",
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
