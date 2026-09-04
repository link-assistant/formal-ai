//! Seed-backed canonical source facts used by workspace-inspection planning.

use super::planned_arguments;

#[test]
fn a_plainly_worded_atomic_leaf_invariant_searches_documentation() {
    // Generic words such as `checkable` occur throughout the workspace. The
    // complete invariant must resolve to its canonical documentation sentence.
    let arguments =
        planned_arguments("Verify every tested leaf is atomic and independently checkable.");
    let search = arguments
        .iter()
        .find(|arguments| arguments.get("pattern").is_some())
        .unwrap_or_else(|| panic!("no workspace search was planned: {arguments:?}"));
    assert_eq!(search["query"], "atomic_leaf_invariant");
    assert_eq!(
        search["pattern"],
        "every leaf is atomic and independently checkable"
    );
    assert_eq!(
        search.get("include").and_then(serde_json::Value::as_str),
        Some("docs/**/*")
    );
}

#[test]
fn a_content_addressed_node_identifier_searches_its_source_field() {
    // `content-addressed` describes the identifier's semantics, not a source
    // filename. The requested fact is the field carried by each tree node.
    let arguments = planned_arguments(
        "Inspect how each decomposition node carries its content-addressed stable identifier.",
    );
    let search = arguments
        .iter()
        .find(|arguments| arguments.get("pattern").is_some())
        .unwrap_or_else(|| panic!("no workspace search was planned: {arguments:?}"));
    assert_eq!(search["query"], "decomposition_node_identifier");
    assert_eq!(search["pattern"], "pub id: String");
    assert_eq!(
        search.get("include").and_then(serde_json::Value::as_str),
        Some("src/**/*")
    );
}

#[test]
fn a_dotted_child_path_convention_searches_its_constructor() {
    // The path wording describes a source invariant, even though `recursive`
    // can also introduce a new task decomposition. Inspect the constructor
    // that appends each one-based child number instead of decomposing again.
    let arguments = planned_arguments(
        "Verify recursive child paths append a dot and their one-based numeric ordinal.",
    );
    let search = arguments
        .iter()
        .find(|arguments| arguments.get("pattern").is_some())
        .unwrap_or_else(|| panic!("no workspace search was planned: {arguments:?}"));
    assert_eq!(search["query"], "dotted_child_path");
    assert_eq!(
        search["pattern"],
        r#"format![(]"[{]parent[}][.][{]number[}]"[)]"#
    );
    assert_eq!(
        search.get("include").and_then(serde_json::Value::as_str),
        Some("src/**/*")
    );
}

#[test]
fn a_complete_tree_node_count_searches_its_test_constant() {
    // `depth-five` resembles a module name, but here it qualifies the complete
    // tree count asserted by the ladder regression. Search the test constant
    // rather than filtering the workspace to a guessed depth_five filename.
    let arguments = planned_arguments(
        "Inspect the complete depth-five tree assertion and confirm its root-inclusive node total is 63.",
    );
    let search = arguments
        .iter()
        .find(|arguments| arguments.get("pattern").is_some())
        .unwrap_or_else(|| panic!("no workspace search was planned: {arguments:?}"));
    assert_eq!(search["query"], "complete_tree_node_count");
    assert_eq!(search["pattern"], "const NODE_COUNT: usize = 63");
    assert_eq!(
        search.get("include").and_then(serde_json::Value::as_str),
        Some("tests/**/*")
    );
}

#[test]
fn an_agent_cli_ladder_depth_selector_searches_the_experiment_workflow() {
    // Workflow scripts live outside production source. The final named selector
    // distinguishes the case arm without pipe-packing its alternatives.
    let arguments = planned_arguments(
        "Inspect the Agent-CLI ladder workflow and verify depth selection supports 0 through 5 and all.",
    );
    let search = arguments
        .iter()
        .find(|arguments| arguments.get("pattern").is_some())
        .unwrap_or_else(|| panic!("no workspace search was planned: {arguments:?}"));
    assert_eq!(search["query"], "agent_cli_ladder_depth_selector");
    assert_eq!(search["pattern"], r"all[)] ;;$");
    assert_eq!(
        search.get("include").and_then(serde_json::Value::as_str),
        Some("experiments/**/*")
    );
}

#[test]
fn leaf_first_ladder_execution_searches_the_experiment_ordering() {
    // The atomic-task label belongs to the harness, not to the requested fact.
    // The fact itself is the ordering invariant: leaves execute before their
    // composite parents so every parent can consume verified child effects.
    // Pin both the live failure and wording the ladder never uses so the repair
    // is a semantic source fact rather than a prompt-shaped branch.
    for prompt in [
        "Atomic task L23: Verify the ladder can execute the 32 smallest leaves before moving \
         to larger composite nodes.",
        "Review the recursive Agent workflow and confirm that atomic leaves run before \
         composite parents.",
        "Atomic task L24: Verify the ladder order for all mode is 32, 16, 8, 4, 2, then \
         the root.",
        "Confirm the all-mode traversal visits depth five through depth zero in descending \
         order.",
    ] {
        let arguments = planned_arguments(prompt);
        let search = arguments
            .iter()
            .find(|arguments| arguments.get("pattern").is_some())
            .unwrap_or_else(|| {
                panic!("no workspace search was planned for {prompt:?}: {arguments:?}")
            });
        assert_eq!(search["query"], "leaf_first_ladder_order");
        assert_eq!(search["pattern"], "levels=list[(]range[(]5,-1,-1");
        let pattern = regex::Regex::new(search["pattern"].as_str().unwrap()).unwrap();
        assert!(pattern.is_match("levels=list(range(5,-1,-1)) if mode=='all' else [int(mode)]"));
        assert_eq!(
            search.get("include").and_then(serde_json::Value::as_str),
            Some("experiments/**/*")
        );
    }
}

#[test]
fn remaining_ladder_leaf_invariants_search_their_canonical_artifacts() {
    // These are repository-inspection facts spanning workflow, documentation,
    // test, and production artifacts. Pair each live ladder wording with an
    // unseen equivalent so adding a fact cannot degenerate into an L-number
    // switch in the planner.
    let cases = [
        (
            "Verify every selected node runs in a fresh temporary repository copy.",
            "Confirm each ladder task receives an isolated scratch checkout.",
            "fresh_ladder_repository_copy",
            "work=.*mktemp -d",
            "experiments/**/*",
            "work=$(mktemp -d)",
        ),
        (
            "Verify every selected node uses the real Agent CLI against the real Formal AI server.",
            "Confirm ladder leaves invoke Agent through the served Formal AI model.",
            "real_agent_formal_ai_ladder",
            "AGENT.*formalai/formal-ai",
            "experiments/**/*",
            "\"$AGENT\" --model formalai/formal-ai",
        ),
        (
            "Verify every selected node requires an observable proof file with its exact node path.",
            "Confirm node evidence is rejected unless its proof names the selected path.",
            "exact_node_path_proof",
            "grep -q.*node_path=.*proof",
            "experiments/**/*",
            "grep -q \"^node_path=$id$\" \"$proof\"",
        ),
        (
            "Inspect the committed binary-tree case-study and verify it describes a tree rather than a flat list.",
            "Confirm the decomposition case study explicitly rejects a flat-list interpretation.",
            "complete_binary_tree_not_flat",
            "complete full binary tree, not a flat list",
            "docs/**/*",
            "This is a complete full binary tree, not a flat list.",
        ),
        (
            "Verify the executable ladder formulates exactly 32 distinct atomic leaves.",
            "Confirm the ladder regression fixes the atomic leaf total at thirty-two.",
            "ladder_leaf_count",
            "const LEAF_COUNT: usize = 32",
            "tests/**/*",
            "const LEAF_COUNT: usize = 32;",
        ),
        (
            "Verify generated child paths are required to exist in the complete tree.",
            "Confirm the tree regression checks that every generated left child path exists.",
            "generated_child_path_existence",
            "paths[.]contains[(]&node[.]left[)]",
            "tests/**/*",
            "assert!(paths.contains(&node.left), \"missing {}\", node.left);",
        ),
        (
            "Inspect the decomposition regression matrix and verify requests are not limited to one fixed wording.",
            "Confirm decomposition tests iterate a multilingual prompt matrix.",
            "decomposition_prompt_matrix",
            "for [(]language, prompt[)] in SPLIT_PROMPTS",
            "tests/**/*",
            "for (language, prompt) in SPLIT_PROMPTS {",
        ),
        (
            "Inspect the final evidence-note planner and record the heading used for composed observations.",
            "Confirm the note composer labels observations gathered in the current session.",
            "composed_observations_heading",
            "Observed in this session:",
            "src/**/*",
            "note.push_str(\"\\nObserved in this session:\\n\");",
        ),
    ];

    for (live, unseen, query, expected_pattern, include, source_line) in cases {
        for prompt in [live, unseen] {
            let arguments = planned_arguments(prompt);
            let search = arguments
                .iter()
                .find(|arguments| arguments.get("pattern").is_some())
                .unwrap_or_else(|| {
                    panic!("no workspace search was planned for {prompt:?}: {arguments:?}")
                });
            assert_eq!(search["query"], query, "wrong query for {prompt:?}");
            assert_eq!(
                search["pattern"], expected_pattern,
                "wrong pattern for {prompt:?}"
            );
            assert_eq!(
                search.get("include").and_then(serde_json::Value::as_str),
                Some(include),
                "wrong artifact scope for {prompt:?}"
            );
            let pattern = regex::Regex::new(search["pattern"].as_str().unwrap()).unwrap();
            assert!(
                pattern.is_match(source_line),
                "planned regex {pattern:?} does not match {source_line:?}"
            );
        }
    }
}

#[test]
fn a_focused_dotted_node_filter_searches_the_experiment_selector() {
    // Selecting one dotted path is an observable workflow invariant, not a
    // request to decompose the sentence into another task tree.
    let arguments = planned_arguments(
        "Verify a single node can be selected by dotted binary path for focused debugging.",
    );
    let search = arguments
        .iter()
        .find(|arguments| arguments.get("pattern").is_some())
        .unwrap_or_else(|| panic!("no workspace search was planned: {arguments:?}"));
    assert_eq!(search["query"], "dotted_binary_node_filter");
    assert_eq!(
        search["pattern"],
        r"^        if depth == level and [(]not filt or node == filt[)]:$"
    );
    assert_eq!(
        search.get("include").and_then(serde_json::Value::as_str),
        Some("experiments/**/*")
    );
}

#[test]
fn documented_power_of_two_levels_use_their_literal_invariant() {
    // A prose name such as `power-of-two` resembles an underscored source
    // identifier. Here it names the documented node-count invariant, so an
    // inferred filename must not hide the sentence that contains the counts.
    let arguments = planned_arguments(
        "Atomic task L10: Verify the invariant explicitly names the supported \
         power-of-two levels through 32.",
    );
    let search = arguments
        .iter()
        .find(|arguments| arguments.get("pattern").is_some())
        .unwrap_or_else(|| panic!("no workspace search was planned: {arguments:?}"));
    assert_eq!(search["query"], "power_of_two");
    assert_eq!(search["pattern"], "2, 4, 8, 16, and 32 nodes respectively");
    assert_eq!(
        search.get("include").and_then(serde_json::Value::as_str),
        Some("docs/**/*")
    );
}

#[test]
fn regression_node_counts_use_the_complete_depth_map() {
    // Each requested depth is one projection of the same regression assertion.
    // The prose words `decomposition nodes` must therefore identify that
    // assertion instead of routing the request as a new task to decompose.
    for prompt in [
        "Verify regression coverage includes two decomposition nodes at depth one.",
        "Verify regression coverage includes four decomposition nodes at depth two.",
        "Verify regression coverage includes eight decomposition nodes at depth three.",
        "Verify regression coverage includes sixteen decomposition nodes at depth four.",
        "Verify regression coverage includes thirty-two decomposition nodes at depth five.",
    ] {
        let arguments = planned_arguments(prompt);
        let search = arguments
            .iter()
            .find(|arguments| arguments.get("pattern").is_some())
            .unwrap_or_else(|| {
                panic!("no workspace search was planned for {prompt:?}: {arguments:?}")
            });
        assert_eq!(search["query"], "decomposition_node_counts");
        assert_eq!(
            search["pattern"],
            r"BTreeMap::from[(].*[(]0, 1[)].*[(]1, 2[)].*[(]2, 4[)].*[(]3, 8[)].*[(]4, 16[)].*[(]5, 32[)]"
        );
        let pattern = regex::Regex::new(search["pattern"].as_str().unwrap()).unwrap();
        assert!(
            pattern.is_match("BTreeMap::from([(0, 1), (1, 2), (2, 4), (3, 8), (4, 16), (5, 32)])")
        );
        assert_eq!(
            search.get("include").and_then(serde_json::Value::as_str),
            Some("tests/**/*")
        );
    }
}

#[test]
fn a_named_format_rendering_is_a_workspace_subject() {
    // A source format can have an ordinary multi-word name with no identifier
    // punctuation. The adjacent artifact noun still makes it a local source
    // question, and generated documentation must not become its evidence.
    let arguments = planned_arguments(
        "Inspect the existing Links Notation rendering and record how child relationships are \
         serialized.",
    );
    let search = arguments
        .iter()
        .find(|arguments| arguments.get("pattern").is_some())
        .unwrap_or_else(|| panic!("no workspace search was planned: {arguments:?}"));
    assert_eq!(search["query"], "Notation");
    assert_eq!(
        search["pattern"], r#""child""#,
        "the rendering search did not target the literal relationship key: {search}"
    );
    assert_eq!(
        search.get("include").and_then(serde_json::Value::as_str),
        Some("src/**/*"),
        "an implementation rendering should exclude generated prose: {search}"
    );
}

#[test]
fn a_serialized_relationship_uses_its_literal_links_key() {
    // A source tree can contain thousands of broad prose hits for `record`,
    // `how`, and `child`. The relationship's serialized Links key is the
    // source-level fact: quoting it keeps a client's match cap from being
    // exhausted by unrelated child collections before any renderer appears.
    let arguments = planned_arguments(
        "Atomic task R17: Inspect the existing Wire Notation rendering and record how parent \
         relationships are encoded.\n\nThis is worker 4.2 in a fresh checkout. Record the observed \
         result and do not claim success without evidence.",
    );
    let search = arguments
        .iter()
        .find(|arguments| arguments.get("pattern").is_some())
        .unwrap_or_else(|| panic!("no workspace search was planned: {arguments:?}"));
    assert_eq!(search["pattern"], r#""parent""#);
    assert_eq!(
        search.get("include").and_then(serde_json::Value::as_str),
        Some("src/**/*")
    );
}

#[test]
fn a_plain_adapter_is_a_workspace_subject() {
    // Adapter names are often ordinary prose with no underscore or capital.
    // The artifact noun still makes the adjacent subsystem a local source
    // subject, just as `check` and `rendering` do for their own source kinds.
    let arguments = planned_arguments(
        "Inspect the existing transport adapter and identify where operations are dispatched.",
    );
    let search = arguments
        .iter()
        .find(|arguments| arguments.get("pattern").is_some())
        .unwrap_or_else(|| panic!("no workspace search was planned: {arguments:?}"));
    assert_eq!(search["query"], "transport");
    assert_eq!(
        search.get("include").and_then(serde_json::Value::as_str),
        Some("src/**/*")
    );
}

#[test]
fn recursive_tree_execution_targets_its_conversion_entry_point() {
    // Searching broad words such as `recursive`, `tree`, and `executed` fills
    // a result cap with descriptions of recursion. The adapter entry point is
    // the canonical source fact that reveals how the tree is converted.
    let arguments = planned_arguments(
        "Examine the recursive execution adapter and explain the decomposition tree execution.",
    );
    let search = arguments
        .iter()
        .find(|arguments| arguments.get("pattern").is_some())
        .unwrap_or_else(|| panic!("no workspace search was planned: {arguments:?}"));
    assert_eq!(search["pattern"], "to_recursive_task");
    assert_eq!(
        search.get("include").and_then(serde_json::Value::as_str),
        Some("src/**/*")
    );
}

#[test]
fn approved_task_strategies_target_the_decomposition_entry_point() {
    // A hyphenated subsystem can name a concept without naming a source file.
    // Treating `task-strategy` as a module filter searched only nonexistent
    // `*task_strategy*` files, so the real ledger entry point in
    // `task_decomposition.rs` was excluded before grep could inspect it. The
    // narrower ledger constructor also has several matches, and an unescaped
    // full call is not a regex match for its literal parentheses.
    let arguments = planned_arguments(
        "Inspect the existing task-strategy ledger and record how approved decomposition \
         strategies are selected.",
    );
    let search = arguments
        .iter()
        .find(|arguments| arguments.get("pattern").is_some())
        .unwrap_or_else(|| panic!("no workspace search was planned: {arguments:?}"));
    assert_eq!(search["query"], "task_strategy");
    assert_eq!(
        search["pattern"],
        r"decompose_task_with_ledger[(]task, max_depth, &TaskStrategyLedger::shipped[(][)][)]"
    );
    let pattern = regex::Regex::new(search["pattern"].as_str().unwrap()).unwrap();
    assert!(
        pattern.is_match(
            "decompose_task_with_ledger(task, max_depth, &TaskStrategyLedger::shipped())"
        )
    );
    assert_eq!(
        search.get("include").and_then(serde_json::Value::as_str),
        Some("src/**/*"),
        "a canonical source fact must not be excluded by an inferred module filter: {search}"
    );
}

#[test]
fn a_regression_lower_bound_searches_test_source() {
    // A hyphenated scope such as `issue-scale` is prose, not necessarily a
    // module name. More importantly, this request explicitly names a
    // regression: its canonical fact belongs in test source, where the lower
    // bound is asserted, rather than in the production implementation.
    let arguments = planned_arguments(
        "Review the issue-scale decomposition regression and identify the minimum number of \
         independently verifiable leaves.",
    );
    let search = arguments
        .iter()
        .find(|arguments| arguments.get("pattern").is_some())
        .unwrap_or_else(|| panic!("no workspace search was planned: {arguments:?}"));
    assert_eq!(search["query"], "issue_scale");
    assert_eq!(search["pattern"], "leaves.*len.*>=");
    assert_eq!(
        search.get("include").and_then(serde_json::Value::as_str),
        Some("tests/**/*"),
        "a regression fact must be sought in test source: {search}"
    );
}
