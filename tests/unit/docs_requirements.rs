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
