use std::fs;
use std::path::Path;

/// Issue #454: `docs/USER-JOURNEYS.md` must enumerate the pain the project
/// closes and the supported/future user journeys, and `VISION.md` and
/// `README.md` must link to it, so the documentation set cannot silently drift.
#[test]
fn issue_454_user_journeys_document_is_present_and_traceable() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));

    let journeys = read(root.join("docs/USER-JOURNEYS.md"));
    assert_contains_all(
        "docs/USER-JOURNEYS.md",
        &journeys,
        &[
            "# User Journeys",
            "## The Pain Formal AI Closes",
            "## Who This Is For",
            "## Currently Supported Journeys",
            "## Potential Future Journeys",
            "## Journey-To-Surface Coverage",
            "## A Worked Example Journey",
            "What is 8% of $50?",
            "Why did you answer that?",
            "formal_ai_bundle",
            "operation-vocabulary.lino",
        ],
    );

    let vision = read(root.join("VISION.md"));
    assert_contains_all(
        "VISION.md",
        &vision,
        &[
            "Who This Is For And What Pain It Closes",
            "docs/USER-JOURNEYS.md",
            "concrete example user journey",
        ],
    );

    let readme = read(root.join("README.md"));
    assert_contains_all(
        "README.md",
        &readme,
        &["docs/USER-JOURNEYS.md", "what pain it closes"],
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
