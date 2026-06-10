//! Issue #395: "У меня есть числа 3, 5, 6, 7, 8 отсортируй их в JavaScript, дай
//! мне код и результат" used to route to the `unknown` intent. The solver must
//! instead recognize the multilingual sort verb, read the given numbers, emit
//! idiomatic code in the requested language, and — because sorting is a pure,
//! decidable function — compute and show the sorted result deterministically.

use formal_ai::UniversalSolver;

/// The exact prompt from the issue must no longer be `unknown`; it must produce
/// a `write_program` answer containing runnable JavaScript and the sorted result.
#[test]
fn issue_395_russian_javascript_prompt_is_not_unknown() {
    let solver = UniversalSolver::default();
    let response = solver.solve(
        "У меня есть числа 3, 5, 6, 7, 8 отсортируй их в JavaScript, дай мне код и результат",
    );

    assert_eq!(
        response.intent, "write_program",
        "the issue prompt must route to write_program, got: {} / {}",
        response.intent, response.answer
    );
    assert!(
        response.answer.contains("```javascript"),
        "answer must contain a JavaScript code fence, got: {}",
        response.answer
    );
    assert!(
        response.answer.contains(".sort((a, b) => a - b)"),
        "answer must contain the ascending JS comparator, got: {}",
        response.answer
    );
    assert!(
        response.answer.contains("Результат: 3, 5, 6, 7, 8"),
        "answer must show the deterministically computed sorted result, got: {}",
        response.answer
    );
}

/// An unsorted English JavaScript request must actually reorder the numbers in
/// the shown result, proving the result is computed rather than echoed.
#[test]
fn issue_395_english_javascript_computes_sorted_result() {
    let solver = UniversalSolver::default();
    let response = solver.solve(
        "I have numbers 5, 3, 8, 1, 9 — sort them in JavaScript, give me the code and the result",
    );

    assert_eq!(response.intent, "write_program", "got: {}", response.answer);
    assert!(
        response.answer.contains("const numbers = [5, 3, 8, 1, 9];"),
        "code must keep the user's given order, got: {}",
        response.answer
    );
    assert!(
        response.answer.contains("Result: 1, 3, 5, 8, 9"),
        "result must be sorted ascending, got: {}",
        response.answer
    );
    assert!(
        response
            .links_notation
            .contains("synthesis:syntax_tree program_syntax_tree"),
        "trace must expose the structural syntax tree, got: {}",
        response.links_notation
    );
    assert!(
        response.links_notation.contains("semantic_node sort_list"),
        "trace must record semantic AST/CST nodes, got: {}",
        response.links_notation
    );
    assert!(
        response
            .links_notation
            .contains("synthesis:cst_tree cst_tree"),
        "trace must expose the CST produced by the meta-language engine, got: {}",
        response.links_notation
    );
    assert!(
        response
            .links_notation
            .contains("synthesis:cst_engine meta_language"),
        "trace must record the primary meta-language CST/AST engine, got: {}",
        response.links_notation
    );
    assert!(
        response.links_notation.contains("component meta-language"),
        "CST trace must identify the meta-language component, got: {}",
        response.links_notation
    );
    assert!(
        response.links_notation.contains("has_error false"),
        "meta-language must parse generated code without errors, got: {}",
        response.links_notation
    );
}

/// The same list coding path must also handle quoted text list data: the
/// operation remains a semantic list transform, but the value domain is
/// `string`, the JavaScript renderer must not use a numeric comparator, and the
/// rendered code must still be validated by the meta-language links network.
#[test]
fn issue_395_string_list_sort_uses_cst_validated_code_path() {
    let solver = UniversalSolver::default();
    let response = solver.solve(
        "Sort the strings \"pear\", \"apple\", \"banana\" in JavaScript, give me the code and result",
    );

    assert_eq!(response.intent, "write_program", "got: {}", response.answer);
    assert!(
        response
            .answer
            .contains(r#"const numbers = ["pear", "apple", "banana"];"#),
        "code must preserve the user's quoted string list, got: {}",
        response.answer
    );
    assert!(
        response.answer.contains("[...numbers].sort()"),
        "string sort must use JavaScript's lexical sort, got: {}",
        response.answer
    );
    assert!(
        response.answer.contains("Result: apple, banana, pear"),
        "result must be sorted lexically, got: {}",
        response.answer
    );
    assert!(
        response.links_notation.contains("value_type=string")
            && response.links_notation.contains("value_type string"),
        "trace must record the string value domain, got: {}",
        response.links_notation
    );
    assert!(
        response.links_notation.contains("component meta-language"),
        "CST trace must identify the meta-language component, got: {}",
        response.links_notation
    );
    assert!(
        response.links_notation.contains("has_error false"),
        "meta-language must parse generated code without errors, got: {}",
        response.links_notation
    );
}

/// "descending order" (the `reverse_sort` operation) must flip the ordering and
/// generate the matching Python `reverse=True` call.
#[test]
fn issue_395_python_descending_uses_reverse_sort() {
    let solver = UniversalSolver::default();
    let response = solver.solve(
        "Sort the numbers 4, 2, 7, 1 in descending order in Python and show me the code and result",
    );

    assert_eq!(response.intent, "write_program", "got: {}", response.answer);
    assert!(
        response.answer.contains("sorted(numbers, reverse=True)"),
        "descending Python code must pass reverse=True, got: {}",
        response.answer
    );
    assert!(
        response.answer.contains("Result: 7, 4, 2, 1"),
        "result must be sorted descending, got: {}",
        response.answer
    );
}

/// The recognizer is seed-driven, so non-English sort verbs work too: a Hindi
/// request for JavaScript and a Chinese request for Python both succeed.
#[test]
fn issue_395_multilingual_sort_verbs_are_recognized() {
    let solver = UniversalSolver::default();

    let hindi = solver
        .solve("मेरे पास संख्याएं 3, 5, 6, 7, 8 हैं, उन्हें JavaScript में क्रमबद्ध करो और मुझे कोड और परिणाम दो");
    assert_eq!(hindi.intent, "write_program", "got: {}", hindi.answer);
    assert!(
        hindi.answer.contains("परिणाम: 3, 5, 6, 7, 8"),
        "Hindi answer must show the localized result, got: {}",
        hindi.answer
    );

    let chinese = solver.solve("我有数字 3, 5, 6, 7, 8，用 Python 排序，给我代码和结果");
    assert_eq!(chinese.intent, "write_program", "got: {}", chinese.answer);
    assert!(
        chinese.answer.contains("结果: 3, 5, 6, 7, 8"),
        "Chinese answer must show the localized result, got: {}",
        chinese.answer
    );
}

/// Guard rails: a sort request without a programming language, or with fewer
/// than two numbers, must not be claimed by the sort-numbers handler. The
/// handler is internal, so we assert through the public solver that neither
/// prompt produces the sort-numbers code/result rendering.
#[test]
fn issue_395_handler_defers_without_language_or_numbers() {
    let solver = UniversalSolver::default();

    let no_language = solver.solve("sort 3, 1, 2 for me please");
    assert!(
        !no_language.answer.contains("console.log(sorted")
            && !no_language.answer.contains("sorted_numbers = sorted("),
        "a sort request with no programming language must not emit sort-numbers code, got: {}",
        no_language.answer
    );

    let single_number = solver.solve("sort 3 in JavaScript");
    assert!(
        !single_number.answer.contains("const sorted = [...numbers]"),
        "a single number is not a sort task and must not emit sort-numbers code, got: {}",
        single_number.answer
    );
}
