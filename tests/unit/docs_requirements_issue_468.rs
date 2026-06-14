use std::fs;
use std::path::Path;

#[test]
fn issue_468_agentic_coding_case_study_is_traceable() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));

    // R306-R319: the requirements rows cite the shipped src/agentic_coding/ loop.
    let requirements = read(root.join("REQUIREMENTS.md"));
    assert_contains_all(
        "REQUIREMENTS.md",
        &requirements,
        &[
            "## Issue #468 Agentic-Coding Mode",
            "everything is a link",
            "core task",
            "| R306 ",
            "| R307 ",
            "| R311 ",
            "| R312 ",
            "| R314 ",
            "| R319 ",
            "formalize_text_to_links",
            "PRIMITIVE_KINDS",
            "covers_all_nine()",
            "37 records",
            "plan_chat_step",
            "pkg_agentic_coding",
            "Сказка о рыбаке и рыбке",
        ],
    );

    // The case study describes the agentic loop, not the removed typed-struct draft.
    let case_study = read(root.join("docs/case-studies/issue-468/README.md"));
    assert_contains_all(
        "docs/case-studies/issue-468/README.md",
        &case_study,
        &[
            "# Issue 468 Case Study",
            "## 1. Summary",
            "## 2. Collected Data",
            "## 3. Holistic Requirements",
            "## 4. Deep Analysis",
            "## 5. Agentic Mode",
            "## 6. Solution Plans",
            "## 7. Existing Components",
            "plan_chat_step",
            "formalize_text_to_links",
            "pkg_agentic_coding",
            "MAX_TURNS",
            "37 records",
            "everything is a link",
            "src/agentic_coding/",
        ],
    );

    // R311: the nine primitives are mapped onto Links Notation records.
    let mapping = read(root.join("docs/case-studies/issue-468/formal-protocol-mapping.md"));
    assert_contains_all(
        "docs/case-studies/issue-468/formal-protocol-mapping.md",
        &mapping,
        &[
            "# The Nine Primitives as Links",
            "R311",
            "format_lino_record",
            "knowledge_base",
            "primitive_scheme",
            "### 1. Concept",
            "### 9. Annotation",
            "total records: 37",
            "covers all nine: true",
            "everything is a link",
        ],
    );

    // R314: the multi-surface agentic tool-calling loop is documented for operators.
    let server_api = read(root.join("docs/desktop/server-api.md"));
    assert_contains_all(
        "docs/desktop/server-api.md",
        &server_api,
        &[
            "Agentic mode: the server drives a tool-calling loop",
            "src/agentic_coding/planner.rs",
            "pkg_agentic_coding",
            "tool_use",
            "function_call",
            "tool_calls",
            "formal-ai agent",
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
