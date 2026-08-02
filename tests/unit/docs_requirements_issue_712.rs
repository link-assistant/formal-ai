use std::fs;
use std::path::Path;

#[test]
fn issue_712_case_study_and_semantic_routing_contract_are_traceable() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let read = |path: &str| {
        fs::read_to_string(root.join(path)).unwrap_or_else(|error| panic!("{path}: {error}"))
    };

    let case_study = read("docs/case-studies/issue-712/README.md");
    for expected in [
        "tool intent as semantic frames",
        "argument shape",
        "Auto-learning boundary",
        "awaiting_human_review",
        "Agent CLI",
        "formal_ai_worker_17.js",
    ] {
        assert!(case_study.contains(expected), "missing {expected}");
    }

    let requirements = read("docs/case-studies/issue-712/requirements.md");
    for id in ["R712-01", "R712-05", "R712-10", "R712-13", "R712-14"] {
        assert!(requirements.contains(id), "missing {id}");
    }

    let explicit_templates = read("data/seed/meanings-web-search-query.lino");
    for rejected in [
        "google …",
        "what does the web say about …",
        "i need current info from the internet on …",
    ] {
        assert!(
            !explicit_templates.contains(rejected),
            "reported sentence must not survive as an explicit template: {rejected}"
        );
    }

    let semantic_actions = read("data/seed/meanings-web-search.lino");
    assert!(semantic_actions.contains("text \"google …\""));
    let learning = read("data/meta/issue-712-routing-learning.lino");
    assert!(learning.contains("lesson:argument-shape"));
}
