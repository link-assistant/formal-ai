//! Issue #1558: reference extraction and ranking, against the recorded issue texts.
//! Twin of the extraction checks in `rust/tests/web/issue-1558-reference-resolution.test.mjs`.
use super::{
    WorkItemReference, not_found_read, percent_decoded, product_name, ranked_candidates, same_path,
    search_matches, work_item_references, workspace_roots,
};

fn recorded(issue: u32) -> &'static str {
    match issue {
        1484 => include_str!("../../../../data/benchmarks/fixtures/issue-1558/issue-1484.txt"),
        1485 => include_str!("../../../../data/benchmarks/fixtures/issue-1558/issue-1485.txt"),
        1487 => include_str!("../../../../data/benchmarks/fixtures/issue-1558/issue-1487.txt"),
        1488 => include_str!("../../../../data/benchmarks/fixtures/issue-1558/issue-1488.txt"),
        1507 => include_str!("../../../../data/benchmarks/fixtures/issue-1558/issue-1507.txt"),
        _ => panic!("no recorded issue {issue}"),
    }
}

fn references_of(issue: u32) -> Vec<WorkItemReference> {
    work_item_references(
        recorded(issue),
        &format!("https://github.com/link-assistant/formal-ai/issues/{issue}"),
    )
}

#[test]
fn link_text_resolves_to_the_linked_path() {
    let r1485 = references_of(1485);
    let delivery = r1485
        .iter()
        .find(|row| row.name == "hive-delivery-attempt.mjs")
        .expect("the linked name");
    assert_eq!(
        delivery.path.as_deref(),
        Some("scripts/hive-delivery-attempt.mjs")
    );
    assert!(!delivery.external);

    let r1487 = references_of(1487);
    let formalization = r1487
        .iter()
        .filter(|row| row.name == "text_formalization.mjs")
        .collect::<Vec<_>>();
    assert_eq!(
        formalization.len(),
        1,
        "link text and link are one reference"
    );
    assert_eq!(
        formalization[0].path.as_deref(),
        Some("js/agentic/crate/text_formalization.mjs")
    );
}

#[test]
fn files_of_another_repository_are_external() {
    for issue in [1484, 1488] {
        let references = references_of(issue);
        assert!(!references.is_empty(), "issue {issue}");
        for row in references {
            assert!(row.external, "issue {issue}: {}", row.name);
            assert_eq!(row.path, None);
            assert_eq!(row.repository.as_deref(), Some("link-assistant/hive-mind"));
        }
    }
}

#[test]
fn runtime_names_are_terms_before_files() {
    assert!(product_name("Node.js"));
    assert!(product_name("Express.js"));
    assert!(!product_name("js/agentic/node-host.mjs"));
    assert!(!product_name("package.json"));
    assert!(!product_name("lib/Node.js"));
    assert!(
        references_of(1507)
            .iter()
            .any(|row| row.name == "Node.js" && product_name(&row.target))
    );
}

#[test]
fn search_results_are_made_relative_and_ranked_by_context() {
    let roots = workspace_roots(Some("/ws".to_owned()), Some("/ws".to_owned()));
    assert_eq!(roots, ["/ws"]);
    let reference = &work_item_references(
        "The runner script `helper.mjs` exits early; see the scripts directory.",
        "https://github.com/example/project/issues/7",
    )[0];
    let matches = search_matches(
        "/ws/docs/examples/helper.mjs\n/ws/scripts/runner/helper.mjs\n/ws/node_modules/x/helper.mjs\n/ws/other.mjs\n",
        reference,
        &roots,
    );
    assert_eq!(
        matches,
        ["docs/examples/helper.mjs", "scripts/runner/helper.mjs"]
    );
    assert_eq!(
        ranked_candidates(&matches, reference),
        ["scripts/runner/helper.mjs", "docs/examples/helper.mjs"]
    );
    assert!(same_path(
        "/ws/scripts/runner/helper.mjs",
        "scripts/runner/helper.mjs",
        &roots
    ));
    assert!(same_path("./a.mjs", "a.mjs", &roots));
    assert!(!same_path("/elsewhere/a.mjs", "a.mjs", &roots));
}

#[test]
fn not_found_answers_are_failed_reads() {
    assert!(not_found_read("Error: File not found: /ws/a.mjs"));
    assert!(not_found_read(
        "ENOENT: no such file or directory, open 'a'"
    ));
    assert!(not_found_read("cat: a.mjs: No such file or directory"));
    assert!(!not_found_read("<file>\n00001| File not found\n</file>"));
}

#[test]
fn link_paths_are_percent_decoded() {
    assert_eq!(percent_decoded("a%20b.md").as_deref(), Some("a b.md"));
    assert_eq!(percent_decoded("bad%2"), None);
    assert_eq!(percent_decoded("bad%zz"), None);
}
