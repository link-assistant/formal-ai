use std::fs;
use std::path::Path;

#[test]
fn issue_12_vision_documents_are_present_and_traceable() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));

    let vision = read(root.join("VISION.md"));
    assert_contains_all(
        "VISION.md",
        &vision,
        &[
            "# Vision",
            "associative operational space",
            "Links Data Store",
            "Add-only history",
            "dynamic type system",
        ],
    );

    let goals = read(root.join("GOALS.md"));
    assert_contains_all(
        "GOALS.md",
        &goals,
        &[
            "# Goals",
            "smallest useful seed dataset",
            "transparent reasoning",
            "chat-first",
            "isolated execution",
        ],
    );

    let non_goals = read(root.join("NON-GOALS.md"));
    assert_contains_all(
        "NON-GOALS.md",
        &non_goals,
        &[
            "# Non-Goals",
            "memoized answer cache",
            "GPU-required neural inference",
            "Hidden autonomous actions",
        ],
    );

    let case_study = read(root.join("docs/case-studies/issue-12/README.md"));
    assert_contains_all(
        "docs/case-studies/issue-12/README.md",
        &case_study,
        &[
            "# Issue 12 Case Study",
            "## Collected Data",
            "## Holistic Requirements",
            "## Solution Plan",
            "issue #1",
            "issue #4",
            "issue #6",
            "issue #8",
            "issue #10",
        ],
    );
}

#[test]
fn issue_16_followup_documents_capture_universal_seed_and_memory_migration() {
    // Pin the documentation surface that frames R105-R108 so the
    // requirement matrix, the architectural narrative, and the case study
    // cannot silently drift apart.
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));

    let requirements = read(root.join("REQUIREMENTS.md"));
    assert_contains_all(
        "REQUIREMENTS.md",
        &requirements,
        &[
            "| R105 ",
            "| R106 ",
            "| R107 ",
            "| R108 ",
            "src/web/seed/",
            "environments.lino",
            "formal-ai memory",
            "formal-ai bundle",
            "formal_ai_bundle",
        ],
    );

    let vision = read(root.join("VISION.md"));
    assert_contains_all(
        "VISION.md",
        &vision,
        &[
            "Self-Aware Environments",
            "Library-First Availability",
            "environments.lino",
        ],
    );

    let case_study = read(root.join("docs/case-studies/issue-16/README.md"));
    assert_contains_all(
        "docs/case-studies/issue-16/README.md",
        &case_study,
        &[
            "Self-Aware Environments and Cross-Surface Memory Migration",
            "environments.lino",
            "formal-ai environments",
            "demo_memory",
            "formal_ai_bundle",
        ],
    );
}

#[test]
fn issue_103_test_matrix_and_architecture_documents_are_present_and_traceable() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));

    let architecture = read(root.join("ARCHITECTURE.md"));
    assert_contains_all(
        "ARCHITECTURE.md",
        &architecture,
        &[
            "# Architecture",
            "Links Notation",
            "Wikidata",
            "P-id",
            "Q-id",
            "temperature",
            "doublets-rs",
            "doublets-web",
            "Universal Problem Solver",
            "Transformation and Substitution Rules",
            "formal_ai_bundle",
        ],
    );

    let vision = read(root.join("VISION.md"));
    assert_contains_all(
        "VISION.md",
        &vision,
        &[
            "Formalization And Temperature",
            "Wikidata",
            "temperature",
            "doublets-rs",
            "ARCHITECTURE.md",
        ],
    );

    let requirements = read(root.join("REQUIREMENTS.md"));
    assert_contains_all(
        "REQUIREMENTS.md",
        &requirements,
        &[
            "Issue #103 Test-Matrix",
            "| R129 ",
            "| R130 ",
            "| R131 ",
            "| R132 ",
            "| R133 ",
            "| R134 ",
            "| R135 ",
            "| R136 ",
            "prompt_variations.rs",
        ],
    );

    let case_study = read(root.join("docs/case-studies/issue-103/README.md"));
    assert_contains_all(
        "docs/case-studies/issue-103/README.md",
        &case_study,
        &[
            "# Issue 103 Case Study",
            "## Collected Data",
            "## Requirements",
            "competitor-test-research.md",
            "ARCHITECTURE.md",
            "prompt_variations.rs",
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
