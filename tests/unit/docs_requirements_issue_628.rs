use std::fs;
use std::path::Path;

#[test]
fn issue_628_agentic_cli_testing_guide_is_traceable() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));

    let guide = read(root.join("docs/testing/agentic-cli-tools.md"));
    assert_contains_all(
        "docs/testing/agentic-cli-tools.md",
        &guide,
        &[
            "# Testing Agentic CLI Tools",
            "cargo build --bin with-formal-ai --bin formal-ai",
            "formal-ai serve --agent-mode --host 127.0.0.1 --port 8080",
            "with-formal-ai --base-url http://127.0.0.1:<proxyport>",
            "opencode run",
            "agent --model formalai/formal-ai --permission-mode auto",
            "--skip-git-repo-check",
            "GEMINI_DEFAULT_AUTH_TYPE=gemini-api-key",
            "ALPHA_MARKER_11111",
            "BETA_MARKER_22222",
            "GAMMA_33333",
            "NESTED_MARKER_44444",
            "phrasing matrix",
            "marker",
            "OpenAI `chat/completions`",
            "OpenAI `responses`",
            "Gemini `streamGenerateContent`",
            "#624",
            "#625",
            "#626",
            "#627",
        ],
    );

    let contributing = read(root.join("CONTRIBUTING.md"));
    assert_contains_all(
        "CONTRIBUTING.md",
        &contributing,
        &[
            "docs/testing/agentic-cli-tools.md",
            "Testing external agentic CLIs",
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
