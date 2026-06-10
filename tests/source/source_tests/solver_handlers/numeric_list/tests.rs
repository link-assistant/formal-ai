use super::*;

#[test]
fn ontology_classifies_every_operation() {
    for (canonical, family, kind) in [
        ("sort", "list_transformation", "list"),
        ("reverse_sort", "list_transformation", "list"),
        ("reverse", "list_transformation", "list"),
        ("sum", "list_reduction", "scalar"),
        ("product", "list_reduction", "scalar"),
        ("minimum", "list_reduction", "scalar"),
        ("maximum", "list_reduction", "scalar"),
    ] {
        assert_eq!(family_for(canonical), family, "family for {canonical}");
        assert_eq!(
            result_kind_for(canonical),
            kind,
            "result_kind for {canonical}"
        );
    }
}

#[test]
fn reduction_results_are_computed() {
    let sum =
        solve_numeric_list("sum the numbers 3, 5, 6, 7, 8 in Python, give me the code").unwrap();
    assert_eq!(sum.result, vec!["29".to_owned()]);
    assert_eq!(sum.result_kind, "scalar");

    let product =
        solve_numeric_list("multiply the numbers 2, 3, 4 in Python, show me the code").unwrap();
    assert_eq!(product.result, vec!["24".to_owned()]);

    let min = solve_numeric_list("find the minimum of 5, 3, 8, 1, 9 in Python code").unwrap();
    assert_eq!(min.result, vec!["1".to_owned()]);

    let max = solve_numeric_list("find the maximum of 5, 3, 8, 1, 9 in Python code").unwrap();
    assert_eq!(max.result, vec!["9".to_owned()]);
}

#[test]
fn reverse_keeps_surface_tokens() {
    let reversed = solve_numeric_list("reverse the numbers 1, 2, 3 in JavaScript").unwrap();
    assert_eq!(reversed.result, vec!["3", "2", "1"]);
    assert_eq!(reversed.result_kind, "list");
    assert!(reversed.syntax_tree.contains("program_syntax_tree"));
    assert!(reversed.syntax_tree.contains("semantic_node reverse_list"));
    assert!(reversed.cst_tree.contains("cst_tree"));
    assert!(reversed.cst_tree.contains("engine meta_language"));
    assert_eq!(reversed.cst_engine, "meta_language");
}

#[test]
fn quoted_string_lists_are_transformed() {
    let sorted = solve_numeric_list(
        "Sort the strings \"pear\", \"apple\", \"banana\" in JavaScript, give me code and result",
    )
    .unwrap();

    assert_eq!(sorted.value_type, "string");
    assert_eq!(sorted.result, vec!["apple", "banana", "pear"]);
    assert!(sorted
        .code
        .contains(r#"const numbers = ["pear", "apple", "banana"];"#));
    assert!(sorted.code.contains("[...numbers].sort()"));
    assert!(sorted.syntax_tree.contains("value_type string"));
    assert!(sorted.cst_tree.contains("cst_tree"));
    assert!(sorted.cst_tree.contains("engine meta_language"));
}

#[test]
fn quoted_string_lists_render_valid_cst_for_every_language() {
    let operation = Operation::Transform(Transform::SortAscending);
    let items = parse_list_items(
        "Sort the strings \"pear\", \"apple\", \"banana\" in every supported language",
        operation,
    );

    for language in crate::coding::PROGRAM_LANGUAGES {
        let program = codegen::build(language, &items, operation, false);
        let code = program
            .render()
            .unwrap_or_else(|| panic!("{} must compose from coding idioms", language.slug));
        let cst = crate::coding::validated_program_cst(language.slug, &code).unwrap_or_else(|| {
            panic!(
                "{} string-list source must parse as a valid CST:\n{}",
                language.slug, code
            )
        });
        assert!(!cst.has_error, "{cst:#?}");
    }
}

#[test]
fn function_synthesis_prompts_are_deferred() {
    assert!(
        solve_numeric_list("write a function that returns the sum of 3 and 5 in Python").is_none(),
        "a function-synthesis prompt must defer to program_synthesis"
    );
}
