use std::fs;
use std::path::Path;

#[test]
fn issue_602_codex_docs_are_copy_pasteable_and_responses_only() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let readme = read(root.join("README.md"));
    assert_contains_all(
        "README.md",
        &readme,
        &[
            "### Codex CLI",
            "model_provider = \"formalai\"",
            "model = \"formal-ai\"",
            "[model_providers.formalai]",
            "base_url = \"http://127.0.0.1:8080/api/openai/v1\"",
            "env_key = \"FORMAL_AI_API_KEY\"",
            "wire_api = \"responses\"",
            "Codex 0.142+",
            "`wire_api = \"chat\"`",
            "always streams",
            "codex exec \\",
            "--skip-git-repo-check --sandbox read-only \\",
            "\"hi\"",
            "# Hi, how may I help you?",
        ],
    );

    let server_api = read(root.join("docs/desktop/server-api.md"));
    assert_contains_all(
        "docs/desktop/server-api.md",
        &server_api,
        &[
            "### 4a. `codex` (OpenAI Codex CLI) - Responses API",
            "model_provider = \"formalai\"",
            "model = \"formal-ai\"",
            "[model_providers.formalai]",
            "base_url = \"http://127.0.0.1:8080/api/openai/v1\"",
            "env_key = \"FORMAL_AI_API_KEY\"",
            "wire_api = \"responses\"",
            "Codex 0.142+",
            "`wire_api = \"chat\"`",
            "always streams",
            "codex exec \\",
            "--skip-git-repo-check --sandbox read-only \\",
            "\"hi\"",
            "# Hi, how may I help you?",
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
