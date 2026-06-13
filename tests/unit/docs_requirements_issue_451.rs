use std::fs;
use std::path::Path;

#[test]
fn issue_451_symbolic_ai_reference_documents_are_present_and_traceable() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let symbolic_ai_url = "https://en.wikipedia.org/wiki/Symbolic_artificial_intelligence";

    // R298: the article is referenced from the three reader-facing docs.
    let readme = read(root.join("README.md"));
    assert_contains_all(
        "README.md",
        &readme,
        &[
            symbolic_ai_url,
            "semantic network",
            "docs/case-studies/issue-451/README.md",
        ],
    );

    let vision = read(root.join("VISION.md"));
    assert_contains_all(
        "VISION.md",
        &vision,
        &[
            symbolic_ai_url,
            "https://en.wikipedia.org/wiki/Physical_symbol_system",
            "symbolic-ai-best-practices.md",
        ],
    );

    let architecture = read(root.join("ARCHITECTURE.md"));
    assert_contains_all(
        "ARCHITECTURE.md",
        &architecture,
        &[
            "Domain background (symbolic AI)",
            symbolic_ai_url,
            "https://en.wikipedia.org/wiki/Neuro-symbolic_AI",
            "R1 ... R304",
        ],
    );

    // R302: every issue requirement is enumerated in the matrix.
    let requirements = read(root.join("REQUIREMENTS.md"));
    assert_contains_all(
        "REQUIREMENTS.md",
        &requirements,
        &[
            "Issue #451 Symbolic AI Reference And Best Practices",
            "| R298 ",
            "| R299 ",
            "| R300 ",
            "| R301 ",
            "| R302 ",
            "| R303 ",
            "| R304 ",
            "docs/case-studies/issue-451/symbolic-ai-best-practices.md",
        ],
    );

    // R300, R301, R302, R303, R304: the case study with analysis, requirements,
    // solution plans, prior art, and risks.
    let case_study = read(root.join("docs/case-studies/issue-451/README.md"));
    assert_contains_all(
        "docs/case-studies/issue-451/README.md",
        &case_study,
        &[
            "# Issue 451 Case Study",
            "## 2. Collected Data",
            "## 3. Holistic Requirements",
            "## 6. Solution Plans",
            "## 7. Existing Components / Prior Art Surveyed",
            "## 8. Risks",
            "physical symbol system hypothesis",
            "Newell & Simon",
            "Knowledge-acquisition bottleneck",
            "WordNet",
            "ConceptNet",
            "Cyc",
            "DeepProbLog",
            symbolic_ai_url,
            "R298",
            "R304",
        ],
    );

    // R299: the best-practice audit maps each technique to the associative stack.
    let best_practices =
        read(root.join("docs/case-studies/issue-451/symbolic-ai-best-practices.md"));
    assert_contains_all(
        "docs/case-studies/issue-451/symbolic-ai-best-practices.md",
        &best_practices,
        &[
            "# Symbolic AI best practices, expressed in the associative stack",
            "Automated theorem proving",
            "Explainability / glass box",
            "Knowledge-acquisition bottleneck mitigation",
            "15 applied, 4 partial, 1 proposed",
            "splr",
            "varisat",
            symbolic_ai_url,
        ],
    );

    // R301: the cited online research backing the analysis.
    let research = read(root.join("docs/case-studies/issue-451/raw-data/online-research.md"));
    assert_contains_all(
        "docs/case-studies/issue-451/raw-data/online-research.md",
        &research,
        &[
            "GOFAI",
            "physical symbol system hypothesis",
            "Henry Kautz",
            "Semantic network",
            "IEEE TPAMI",
            symbolic_ai_url,
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
