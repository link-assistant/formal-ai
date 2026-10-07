//! Issue #1175 R3: claim routing through the capability table.
//!
//! The `claim` rows of `data/seed/capability-routing.lino` are consulted after
//! formalization and before a handler runs: a handler with a row is offered
//! the prompt only when one of its `admits_on` evidence kinds holds in the
//! prompt's structure, so a surface word alone cannot claim it. Mirrored by
//! `rust/tests/web/issue-1175-claim-routing.test.mjs`.

use formal_ai::capability_routing::{
    CLAIM_EVIDENCE_KINDS, ClaimRow, claim_admitted, claim_evidence_holds, claim_rows,
    claim_rows_from,
};

#[test]
fn the_claim_rows_are_read_from_the_capability_table() {
    let rows: Vec<(&str, &str, Vec<&str>)> = claim_rows()
        .iter()
        .map(|row| {
            (
                row.handler.as_str(),
                row.browser_handler.as_str(),
                row.admits_on.iter().map(String::as_str).collect(),
            )
        })
        .collect();
    assert_eq!(
        rows,
        vec![
            (
                "software_project",
                "trySoftwareProjectRequest",
                vec!["object_phrase_artifact", "approval_of_a_proposal"],
            ),
            (
                "terminal_command",
                "tryTerminalCommand",
                vec!["shell_command_shape", "semantic_shell_task"],
            ),
            ("repository_lineage", "", vec!["repository_subject"]),
            ("page_query_text", "tryPageQueryText", vec!["supplied_page"]),
            (
                "javascript_execution",
                "tryJavaScriptExecution",
                vec!["javascript_program"],
            ),
            (
                "incompatible_units",
                "tryIncompatibleUnits",
                vec!["incompatible_unit_pair"],
            ),
            ("http_fetch", "", vec!["fetch_url"]),
            ("url_navigate", "", vec!["navigation_url"]),
            ("calendar_create_event", "", vec!["calendar_date_signal"]),
            ("code_debugging", "tryCodeDebugging", vec!["code_artifact"]),
            (
                "code_explanation",
                "tryCodeExplanation",
                vec!["code_artifact"]
            ),
            ("code_review", "tryCodeReview", vec!["code_artifact"]),
            (
                "summarization_text",
                "trySummarizationText",
                vec!["supplied_text"],
            ),
            ("text_rewrite", "tryTextRewrite", vec!["supplied_text"]),
            ("statistics", "tryStatistics", vec!["stated_number"]),
            ("word_problem", "tryWordProblem", vec!["stated_number"]),
            (
                "arithmetic",
                "tryArithmetic",
                vec!["calculation_expression", "currency_rate_basis"],
            ),
            (
                "compound_interest",
                "tryCompoundInterest",
                vec!["investment_terms", "conversion_target_currency"],
            ),
            (
                "number_constraint_reasoning",
                "tryNumberConstraintReasoning",
                vec!["interval_bounds"],
            ),
            (
                "unit_conversion",
                "tryUnitConversion",
                vec!["measured_quantity"]
            ),
            (
                "calendar_reasoning",
                "tryCalendarReasoning",
                vec!["calendar_date_signal", "calendar_anchor"],
            ),
        ]
    );
}

#[test]
fn every_row_names_a_known_handler_and_known_evidence() {
    let precedence = formal_ai::seed::handler_precedence();
    for row in claim_rows() {
        assert!(
            row.handler == "terminal_command" || precedence.iter().any(|name| name == &row.handler),
            "claim row names a handler the precedence table does not: {row:?}"
        );
        assert!(
            !row.admits_on.is_empty(),
            "a row admits on something: {row:?}"
        );
        for kind in &row.admits_on {
            assert!(
                CLAIM_EVIDENCE_KINDS.contains(&kind.as_str()),
                "unknown evidence kind `{kind}` in {row:?}"
            );
            assert!(claim_evidence_holds(kind, "ls -la", "ls -la").is_some());
        }
        assert!(
            !row.because.is_empty(),
            "every row states its reason: {row:?}"
        );
    }
    assert_eq!(claim_evidence_holds("surface_word", "ls", "ls"), None);
}

#[test]
fn a_surface_word_alone_is_not_admitted() {
    for (handler, prompt) in [
        (
            "terminal_command",
            "Make a 3-day itinerary for a first visit to Rome.",
        ),
        (
            "terminal_command",
            "Find the bug: def average(xs): return sum(xs) / len(xs)",
        ),
        (
            "software_project",
            "Write a regex for a US ZIP code with an optional 4-digit extension",
        ),
        ("repository_lineage", "Which issue introduced this feature?"),
        (
            "page_query_text",
            "What command builds the jar on this page?",
        ),
        ("javascript_execution", "Run this JavaScript"),
        ("incompatible_units", "What does a unit test check?"),
        ("http_fetch", "Fetch me a summary of the news"),
        ("url_navigate", "Navigate the menu to the settings screen"),
        ("calendar_create_event", "Explain how a calendar works."),
        ("code_debugging", "Find the bug in my plan for the trip"),
        ("code_explanation", "Explain how a compiler works"),
        ("code_review", "Review my essay about the ocean"),
        ("summarization_text", "Summarize the rust language"),
        ("text_rewrite", "Make this more formal"),
        ("statistics", "What is the mean of my test scores?"),
        ("statistics", "Is this number prime?"),
        ("word_problem", "How many apples are left if I eat some?"),
        ("arithmetic", "Explain how long division works"),
        (
            "compound_interest",
            "How does compound interest work when you invest?",
        ),
        (
            "number_constraint_reasoning",
            "Guess the hidden number I am thinking of",
        ),
        ("unit_conversion", "Convert this recipe to metric units"),
        ("calendar_reasoning", "Explain how a calendar works."),
    ] {
        assert!(
            !claim_admitted(handler, prompt, &prompt.to_lowercase()),
            "`{handler}` must not be offered `{prompt}`"
        );
    }
}

#[test]
fn the_structural_evidence_admits() {
    for (handler, prompt) in [
        ("terminal_command", "find . -name '*.log' -size +10M"),
        ("software_project", "Build a web app for tracking habits"),
        (
            "repository_lineage",
            "Which issue introduced scripts/check-self-development-release.rs?",
        ),
        (
            "repository_lineage",
            "Why does the self-development status fail?",
        ),
        ("repository_lineage", "What does `evaluate_calculation` do?"),
        (
            "page_query_text",
            "What command builds the jar?\nRun kotlinc hello.kt -include-runtime -d hello.jar.",
        ),
        (
            "javascript_execution",
            "Run this JavaScript: console.log(1 + 2)",
        ),
        ("incompatible_units", "How many meters are in a kilobyte?"),
        ("http_fetch", "Fetch https://example.com"),
        ("url_navigate", "Navigate to github.com"),
        (
            "calendar_create_event",
            "Schedule a meeting with Anna tomorrow at 3pm",
        ),
        (
            "code_debugging",
            "Find the bug: def average(xs): return sum(xs) / len(xs)",
        ),
        (
            "code_explanation",
            "Explain this code: print(sum(range(10)))",
        ),
        (
            "code_review",
            "Review this code: `const total = items.map((item) => item.price)`",
        ),
        (
            "summarization_text",
            "Summarize: The parser reads the file. It builds a tree. The tree is checked.",
        ),
        (
            "text_rewrite",
            "Make this formal: hey can you send me the file",
        ),
        ("statistics", "What is the mean of 3, 5 and 7?"),
        ("statistics", "Is 97 a prime number?"),
        (
            "word_problem",
            "Tom has 5 apples and gets 3 more. How many apples does he have?",
        ),
        ("arithmetic", "What is 2 + 2?"),
        (
            "arithmetic",
            "what dollar exchange rate do you use for calculations?",
        ),
        (
            "compound_interest",
            "If I invest $1000 at 8% annual interest compounded monthly for 5 years, how much will I have?",
        ),
        (
            "compound_interest",
            "convert the final amount to EUR using current exchange rates from the web.",
        ),
        (
            "number_constraint_reasoning",
            "Я загадал число больше 1 но меньше 3. что это за число?",
        ),
        ("unit_conversion", "How many meters are in 3 km?"),
        ("calendar_reasoning", "What day comes after Monday?"),
        ("calendar_reasoning", "What day is today?"),
        ("calendar_reasoning", "What day of the week was 2024-02-29?"),
        (
            "calendar_reasoning",
            "What month is 2 months after January?",
        ),
    ] {
        assert!(
            claim_admitted(handler, prompt, &prompt.to_lowercase()),
            "`{handler}` must be offered `{prompt}`"
        );
    }
}

#[test]
fn a_handler_without_a_row_is_admitted_and_a_fixture_row_routes_without_code() {
    assert!(claim_admitted(
        "concept_lookup",
        "What is a monad?",
        "what is a monad?"
    ));
    let fixture = "capability_routing\n  claim\n    handler concept_lookup\n    admits_on shell_command_shape\n    because \"fixture\"\n";
    assert_eq!(
        claim_rows_from(fixture),
        vec![ClaimRow {
            handler: "concept_lookup".to_owned(),
            browser_handler: String::new(),
            admits_on: vec!["shell_command_shape".to_owned()],
            because: "fixture".to_owned(),
        }]
    );
}
