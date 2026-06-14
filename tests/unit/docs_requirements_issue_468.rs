use std::fs;
use std::path::Path;

/// Issue #468: pins the case-study deliverables (collected data, deep analysis
/// with online research, the full requirement enumeration, solution plans, and
/// the prior-art survey) so they remain present and traceable.
#[test]
fn issue_468_text_formalization_case_study_is_present_and_traceable() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let issue_url = "https://github.com/link-assistant/formal-ai/issues/468";
    let article_url =
        "https://telegra.ph/Formalnyj-protokol-dlya-perevoda-tekstov-v-bazu-znanij-06-10";

    // R317: every issue requirement is enumerated in the matrix, R306..R319.
    let requirements = read(root.join("REQUIREMENTS.md"));
    assert_contains_all(
        "REQUIREMENTS.md",
        &requirements,
        &[
            "Issue #468 Text-To-Knowledge Formalization",
            issue_url,
            "| R306 ",
            "| R307 ",
            "| R308 ",
            "| R309 ",
            "| R310 ",
            "| R311 ",
            "| R312 ",
            "| R313 ",
            "| R314 ",
            "| R315 ",
            "| R316 ",
            "| R317 ",
            "| R318 ",
            "| R319 ",
            "src/text_formalization/",
        ],
    );

    // R315/R316/R317/R318: the case study with collected data, analysis,
    // requirements, solution plans, prior art, and risks.
    let case_study = read(root.join("docs/case-studies/issue-468/README.md"));
    assert_contains_all(
        "docs/case-studies/issue-468/README.md",
        &case_study,
        &[
            "# Issue 468 Case Study",
            "## 2. Collected Data",
            "## 3. Holistic Requirements",
            "## 5. Agentic Mode",
            "## 6. Solution Plans",
            "## 7. Existing Components / Prior Art Surveyed",
            "## 8. Risks",
            "Сказка о рыбаке и рыбке",
            "everything is a link",
            "Abstract Meaning Representation",
            "OpenIE",
            "RDF-star",
            "Wikidata",
            "FrameNet",
            "115",
            "R306",
            "R319",
            issue_url,
        ],
    );

    // R314: the agentic-mode flow is documented while honoring the explicit
    // do-not-wire-external-CLIs constraint.
    assert_contains_all(
        "docs/case-studies/issue-468/README.md",
        &case_study,
        &[
            "link-assistant/agent",
            "gemini-cli",
            "don't use claude or codex",
            "formal-ai serve",
        ],
    );

    // R311: the protocol-to-links mapping shows every primitive reduces to
    // plain doublets.
    let mapping = read(root.join("docs/case-studies/issue-468/formal-protocol-mapping.md"));
    assert_contains_all(
        "docs/case-studies/issue-468/formal-protocol-mapping.md",
        &mapping,
        &[
            "The Nine Primitives as Links",
            "everything is a link",
            "lnk:concept:greed:type",
            "source → target",
            "115",
        ],
    );

    // R316: the cited online research backing the analysis.
    let research = read(root.join("docs/case-studies/issue-468/raw-data/online-research.md"));
    assert_contains_all(
        "docs/case-studies/issue-468/raw-data/online-research.md",
        &research,
        &[
            "Abstract Meaning Representation",
            "Open Information Extraction",
            "RDF reification",
            "RDF-star",
            "Wikidata",
            "PropBank",
            "FrameNet",
            "associative data model",
            "The Tale of the Fisherman and the Fish",
            "https://en.wikipedia.org/wiki/Abstract_Meaning_Representation",
        ],
    );

    // R315: the summarized-and-cited source protocol, including the worked
    // example sentence and the declarative query shape.
    let summary = read(root.join("docs/case-studies/issue-468/raw-data/article-summary.md"));
    assert_contains_all(
        "docs/case-studies/issue-468/raw-data/article-summary.md",
        &summary,
        &[
            article_url,
            "Пётр открыл магазин в Москве в 2019 году",
            "ent:petrov_petr",
            "pred:open",
            "doc-0001",
            "SELECT ?shop",
        ],
    );

    // The raw-data captures of the issue and pull request are archived.
    for capture in [
        "docs/case-studies/issue-468/raw-data/issue-468.json",
        "docs/case-studies/issue-468/raw-data/issue-468-comments.json",
        "docs/case-studies/issue-468/raw-data/pr-469.json",
    ] {
        assert!(
            root.join(capture).exists(),
            "{capture} should be archived under raw-data/"
        );
    }
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
