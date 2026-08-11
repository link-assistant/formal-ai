use std::{fs, path::PathBuf};

fn repository_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

#[test]
fn default_dependency_lock_has_no_system_openssl_stack() {
    let lock = fs::read_to_string(repository_root().join("Cargo.lock"))
        .expect("the committed Cargo.lock should be readable");

    for package in ["openssl", "openssl-sys", "native-tls", "tokio-native-tls"] {
        assert!(
            !lock.contains(&format!("name = \"{package}\"")),
            "the default dependency lock must not contain {package}; stock Rust images do not provide OpenSSL build prerequisites"
        );
    }
}

#[test]
fn manifests_select_only_transport_independent_web_features() {
    let manifest = fs::read_to_string(repository_root().join("Cargo.toml"))
        .expect("the workspace manifest should be readable");

    assert!(manifest.contains(
        "web-capture = { version = \"0.3.36\", default-features = false, features = [\"search\"] }"
    ));
    assert!(manifest.contains("web-search = { version = \"0.5.0\", default-features = false }"));
}

#[test]
fn stock_rust_ci_installs_and_inspects_the_binary_without_apt() {
    let workflow =
        fs::read_to_string(repository_root().join(".github/workflows/stock-rust-install.yml"))
            .expect("the stock Rust install workflow should be readable");

    for required in [
        "container: rust:1.96-slim-bookworm",
        "cargo tree --locked -i openssl-sys",
        "cargo install --path . --locked",
        "ldd /tmp/formal-ai-install/bin/formal-ai",
        "libssl|libcrypto",
        "/tmp/formal-ai-install/bin/formal-ai --version",
    ] {
        assert!(
            workflow.contains(required),
            "workflow is missing {required}"
        );
    }
    assert!(
        !workflow.contains("apt-get"),
        "the stock-image regression must not install system packages"
    );
}

#[test]
fn same_task_agent_cli_authorship_is_preserved() {
    let root = repository_root();
    let session = "ses_0132073d7ffeHL6POmzfQ29hoH";
    let generated = fs::read_to_string(root.join(
        "docs/case-studies/issue-988/self-hosting-authorship/20260810_988_stock_rust_install.md",
    ))
    .expect("the Agent-CLI-generated changelog artifact should be readable");
    // Towncrier removes the source fragment after publishing it. Compare the
    // authored artifact with its durable destination so release commits keep
    // the provenance check valid.
    let canonical = fs::read_to_string(root.join("CHANGELOG.md"))
        .expect("the canonical changelog should be readable");
    assert!(canonical.contains(generated.trim()));

    let evidence = root.join("docs/case-studies/issue-988/self-hosting-authorship");
    let agent_log = fs::read_to_string(evidence.join("agent-cli.log"))
        .expect("the self-authoring Agent CLI log should be readable");
    assert!(
        agent_log.contains(session),
        "the Agent CLI log must identify {session}"
    );

    let formal_ai_log = fs::read_to_string(evidence.join("formal-ai.log"))
        .expect("the self-authoring Formal AI log should be readable");
    for transition in ["planned ToolCalls", "planned Final"] {
        assert!(
            formal_ai_log.contains(transition),
            "the Formal AI log must preserve the {transition} transition"
        );
    }
    assert!(
        formal_ai_log.matches("POST /v1/chat/completions").count() >= 3,
        "the Formal AI log must preserve the multi-round authoring exchange"
    );
}
