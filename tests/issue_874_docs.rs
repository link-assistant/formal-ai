use std::fs;
use std::path::PathBuf;

const DIRECT_COMPONENTS: &[(&str, &str)] = &[
    ("link-cli", "https://github.com/link-foundation/link-cli"),
    (
        "links-notation",
        "https://github.com/link-foundation/links-notation",
    ),
    (
        "lino-objects-codec",
        "https://github.com/link-foundation/lino-objects-codec",
    ),
    (
        "meta-language",
        "https://github.com/link-foundation/meta-language",
    ),
    (
        "link-calculator",
        "https://github.com/link-assistant/calculator",
    ),
    (
        "lino-arguments",
        "https://github.com/link-foundation/lino-arguments",
    ),
    ("lino-i18n", "https://github.com/link-foundation/lino-i18n"),
];

const RELATED_COMPONENTS: &[&str] = &[
    "https://github.com/linksplatform/doublets-rs",
    "https://github.com/linksplatform/mem-rs",
    "https://github.com/linksplatform/doublets-web",
    "https://github.com/link-foundation/relative-meta-logic",
    "https://github.com/link-foundation/meta-theory",
    "https://github.com/link-foundation/transformer",
    "https://github.com/link-assistant/agent",
    "https://github.com/link-assistant/hive-mind",
];

fn root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn read(path: &str) -> String {
    let path = root().join(path);
    fs::read_to_string(&path).unwrap_or_else(|error| panic!("read {}: {error}", path.display()))
}

fn guide() -> String {
    read("docs/associative-tech-stack.md")
}

fn assert_contains_all(label: &str, text: &str, needles: &[&str]) {
    for needle in needles {
        assert!(text.contains(needle), "{label} is missing {needle:?}");
    }
}

#[test]
fn associative_stack_guide_is_a_separate_discoverable_document() {
    let readme = read("README.md");
    assert!(
        readme.contains("docs/associative-tech-stack.md"),
        "README must link the standalone associative stack guide"
    );

    assert_contains_all(
        "associative stack guide",
        &guide(),
        &[
            "# Associative technology stack",
            "Direct runtime components",
            "Architecture and protocol components",
            "Development and orchestration components",
            "How data moves through the stack",
        ],
    );
}

#[test]
fn every_direct_associative_dependency_has_a_repository_and_usage_boundary() {
    let cargo_manifest = read("Cargo.toml");
    let package_manifest = read("package.json");
    let guide = guide();

    for (dependency, repository) in DIRECT_COMPONENTS {
        assert!(
            cargo_manifest.contains(dependency) || package_manifest.contains(dependency),
            "{dependency} must be grounded in a current direct manifest"
        );
        assert!(
            guide.contains(repository),
            "guide is missing the repository for direct component {dependency}"
        );
        assert!(
            guide.contains(&format!("`{dependency}`")),
            "guide is missing a named usage explanation for {dependency}"
        );
    }

    assert_contains_all(
        "direct component evidence",
        &guide,
        &[
            "Cargo.toml",
            "package.json",
            "src/link_store.rs",
            "src/coding/cst.rs",
            "src/document_formats.rs",
            "src/calculation.rs",
            "src/main.rs",
            "src/web/i18n.js",
        ],
    );
}

#[test]
fn related_repositories_are_linked_without_overstating_runtime_integration() {
    let guide = guide();
    for repository in RELATED_COMPONENTS {
        assert!(
            guide.contains(repository),
            "guide is missing related component repository {repository}"
        );
    }
    assert_contains_all(
        "integration-status boundaries",
        &guide,
        &[
            "direct dependency",
            "compatibility target",
            "in-repository implementation",
            "conceptual foundation",
            "development-time",
            "not linked into the Formal AI runtime",
        ],
    );
}

#[test]
fn whole_guide_explains_the_associative_stack_end_to_end() {
    let guide = guide();
    assert_contains_all(
        "whole associative stack guide",
        &guide,
        &[
            "links network",
            "Links Notation",
            "doublets-rs",
            "native",
            "browser",
            "parse",
            "serialize",
            "CST",
            "AST",
            "calculation",
            "configuration",
            "internationalization",
            "substitution",
            "reasoning",
            "Agent CLI",
            "Hive Mind",
        ],
    );

    let agent_authored_leaf = read(
        "docs/case-studies/issue-874/agent-cli-evidence/explicit-containing/agent-authored-summary.md",
    );
    assert!(
        guide.contains(agent_authored_leaf.trim()),
        "guide must include the documentation leaf authored through Formal AI and Agent CLI"
    );
}
