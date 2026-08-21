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

    // The version is deliberately not part of the assertion. What keeps a stock
    // Rust image building is the *feature* selection -- these crates pull a
    // native TLS transport in by default -- and pinning the exact release here
    // only meant that every routine dependency bump failed this test for a
    // reason that has nothing to do with OpenSSL.
    for crate_name in ["web-capture", "web-search"] {
        let declaration = manifest
            .lines()
            .find(|line| line.trim_start().starts_with(&format!("{crate_name} = ")))
            .unwrap_or_else(|| panic!("the manifest should declare {crate_name}"));
        assert!(
            declaration.contains("default-features = false"),
            "{crate_name} must opt out of default features, which carry a native TLS transport: {declaration}"
        );
    }
    let web_capture = manifest
        .lines()
        .find(|line| line.trim_start().starts_with("web-capture = "))
        .expect("the manifest should declare web-capture");
    assert!(
        web_capture.contains("features = [\"search\"]"),
        "web-capture is here for search, and that feature has to be asked for once defaults are off: {web_capture}"
    );
}

#[test]
fn stock_rust_ci_installs_and_inspects_the_binary_without_apt() {
    let workflow =
        fs::read_to_string(repository_root().join(".github/workflows/stock-rust-install.yml"))
            .expect("the stock Rust install workflow should be readable");

    // The container tag is derived, not spelled: the image has to be the floor
    // the manifest declares, or the job proves a stock install works on a
    // compiler the crate no longer claims to support.
    let manifest = fs::read_to_string(repository_root().join("Cargo.toml"))
        .expect("the workspace manifest should be readable");
    let rust_version = manifest
        .lines()
        .find_map(|line| line.trim().strip_prefix("rust-version = "))
        .expect("the manifest should declare a rust-version")
        .trim()
        .trim_matches('"')
        .to_owned();
    let container = format!("container: rust:{rust_version}-slim-bookworm");

    for required in [
        container.as_str(),
        "cargo tree --locked --prefix none --format '{p}'",
        "> /tmp/formal-ai-dependency-tree.txt",
        "grep -Eq '^openssl-sys v' /tmp/formal-ai-dependency-tree.txt",
        "CARGO_INSTALL_ROOT: /tmp/formal-ai-install",
        "export PATH=\"$CARGO_INSTALL_ROOT/bin:$PATH\"",
        "cargo install --path . --locked",
        "ldd \"$(command -v formal-ai)\"",
        "libssl|libcrypto",
        "formal-ai --version",
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
    assert!(
        !workflow.contains("<<<"),
        "the stock container uses POSIX sh, so the probe must not use Bash here-strings"
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
