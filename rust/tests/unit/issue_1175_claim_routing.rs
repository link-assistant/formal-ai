//! Issue #1175 R3: claim routing through the capability table.
//!
//! The `claim` rows of `data/seed/capability-routing.lino` are consulted after
//! formalization and before a handler runs: a handler with a row is offered
//! the prompt only when one of its `admits_on` evidence kinds holds in the
//! prompt's structure, so a surface word alone cannot claim it. Mirrored by
//! `rust/tests/web/issue-1175-claim-routing.test.mjs`.

use formal_ai::capability_routing::{
    CLAIM_EVIDENCE_KINDS, ClaimAdmission, ClaimRow, claim_admission, claim_admission_in_dialogue,
    claim_admitted, claim_evidence_holds, claim_evidence_holds_in_dialogue, claim_rows,
    claim_rows_from, without_answer_shape_directive,
};
use formal_ai::event_log::EventLog;

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
            (
                "test_generation",
                "tryTestGeneration",
                vec!["function_under_test"]
            ),
            (
                "code_refactoring",
                "tryCodeRefactoring",
                vec!["code_artifact"]
            ),
            (
                "format_conversion",
                "tryFormatConversion",
                vec!["structured_document"],
            ),
            (
                "brainstorm_composition",
                "tryBrainstormComposition",
                vec!["composition_topic"]
            ),
            (
                "creative_writing",
                "tryCreativeWritingRequest",
                vec!["composition_topic"]
            ),
            ("advice_request", "tryAdviceRequest", vec!["advice_topic"]),
            (
                "planning_request",
                "tryPlanningRequest",
                vec!["cached_destination"]
            ),
            (
                "regex_synthesis",
                "tryRegexSynthesis",
                vec!["pattern_constraints"]
            ),
            ("sql_synthesis", "trySqlSynthesis", vec!["table_reference"]),
            (
                "shell_command_compose",
                "tryShellCommandCompose",
                vec!["filesystem_object"]
            ),
            (
                "program_synthesis",
                "tryProgramSynthesis",
                vec!["function_spec"]
            ),
            ("write_script", "tryWriteScript", vec!["script_language"]),
            (
                "document_generation_plan",
                "tryDocumentGenerationPlan",
                vec!["document_format"]
            ),
            (
                "installation_conversion",
                "tryInstallationConversion",
                vec!["install_steps"]
            ),
            (
                "execution_failure",
                "tryExecutionFailure",
                vec!["call_expression"]
            ),
            (
                "concept_lookup",
                "tryConceptLookup",
                vec!["concept_subject"]
            ),
            (
                "definition_merge",
                "tryDefinitionMerge",
                vec!["definition_merge_term"]
            ),
            ("who_is", "tryWhoIsQuestion", vec!["concept_subject"]),
            (
                "how_it_works",
                "tryHowItWorks",
                vec!["mechanism_subject", "prior_reply"]
            ),
            (
                "procedural_how_to",
                "tryProceduralHowTo",
                vec!["procedure_task"]
            ),
            (
                "procedural_how_to_followup",
                "tryProceduralHowToFollowup",
                vec!["prior_procedure"]
            ),
            ("web_search", "tryWebSearch", vec!["search_focus"]),
            ("conversation_topic", "", vec!["conversation_topic_subject"]),
            ("learn_from_source", "", vec!["learnable_source"]),
            (
                "product_search",
                "tryProductSearch",
                vec!["marketplace_scope"]
            ),
            (
                "verifiable_task",
                "tryVerifiableTask",
                vec!["verifiable_spec"]
            ),
            (
                "legality_warning",
                "tryLegalityWarning",
                vec!["legality_assessment"]
            ),
            (
                "opinion_question",
                "tryOpinionQuestion",
                vec!["assistant_addressee"]
            ),
            (
                "physical_action_question",
                "tryPhysicalActionQuestion",
                vec!["assistant_addressee"]
            ),
            (
                "punctuation_only_prompt",
                "tryPunctuationOnlyPrompt",
                vec!["punctuation_only"]
            ),
            ("ill_formed", "tryIllFormed", vec!["unbalanced_brackets"]),
            (
                "shell_refusal",
                "tryShellRefusal",
                vec!["shell_command_shape"]
            ),
            (
                "link_native_synthesis",
                "tryLinkNativeSynthesis",
                vec!["stated_number"]
            ),
            (
                "software_project_followup",
                "trySoftwareProjectFollowup",
                vec!["prior_software_project"]
            ),
            (
                "research_result_followup",
                "tryResearchResultFollowup",
                vec!["prior_research_request"]
            ),
            (
                "research_comparison_table",
                "tryResearchComparisonTable",
                vec!["prior_research_request"]
            ),
            (
                "coreference",
                "tryCoreferenceFactLookup",
                vec!["coreference_antecedent"]
            ),
            (
                "response_language_followup",
                "tryResponseLanguageFollowup",
                vec!["prior_user_request"]
            ),
            (
                "numeric_list",
                "tryNumericList",
                vec!["list_items", "prior_numeric_list"]
            ),
            (
                "text_manipulation",
                "tryTextManipulation",
                vec!["text_operation"]
            ),
            (
                "shell_command_transform",
                "tryShellCommandTransform",
                vec!["shell_command_operand", "prior_reply"]
            ),
            (
                "write_program_coreference",
                "tryWriteProgramCoreference",
                vec!["prior_program"]
            ),
            (
                "write_program_concrete",
                "tryWriteProgramConcrete",
                vec!["program_task"]
            ),
            (
                "program_blueprint_from_prompt",
                "tryProgramBlueprintFromPrompt",
                vec!["program_task"]
            ),
            (
                "software_project_request",
                "trySoftwareProjectRequest",
                vec!["object_phrase_artifact", "approval_of_a_proposal"]
            ),
            (
                "memory_program",
                "tryMemoryProgram",
                vec!["memory_program_reading"]
            ),
            (
                "memory_program_gap",
                "tryMemoryProgramGap",
                vec!["memory_program_reading"]
            ),
        ]
    );
    let refusal_rows: Vec<(&str, Vec<&str>)> = claim_rows()
        .iter()
        .filter(|row| !row.refusal_events.is_empty())
        .map(|row| {
            (
                row.handler.as_str(),
                row.refusal_events.iter().map(String::as_str).collect(),
            )
        })
        .collect();
    assert_eq!(
        refusal_rows,
        vec![
            ("test_generation", vec!["test_generation:refusal"]),
            ("code_refactoring", vec!["code_refactoring:refusal"]),
            ("format_conversion", vec!["format_conversion:refusal"]),
            ("brainstorm_composition", vec!["brainstorming:refusal"]),
            ("creative_writing", vec!["creative_writing:refusal"]),
            ("advice_request", vec!["advice:refusal"]),
            ("planning_request", vec!["planning:refusal"]),
            ("regex_synthesis", vec!["regex_synthesis:refusal"]),
            ("sql_synthesis", vec!["sql_synthesis:refusal"]),
            (
                "shell_command_compose",
                vec!["shell_command_compose:refusal"]
            ),
            ("how_it_works", vec!["how_it_works:refusal"]),
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
        "roleplay",
        "Pretend you are a pirate",
        "pretend you are a pirate"
    ));
    let fixture = "capability_routing\n  claim\n    handler concept_lookup\n    admits_on shell_command_shape\n    because \"fixture\"\n";
    assert_eq!(
        claim_rows_from(fixture),
        vec![ClaimRow {
            handler: "concept_lookup".to_owned(),
            browser_handler: String::new(),
            admits_on: vec!["shell_command_shape".to_owned()],
            refusal_events: Vec::new(),
            because: "fixture".to_owned(),
        }]
    );
}

// Issue #1175 R3, refusal group: without its operand a refusal-group handler
// is admitted to refuse only (the dispatcher keeps its answer only when it
// recorded the row's refusal event), and with it every answer is admitted.
#[test]
fn a_refusal_group_handler_without_its_operand_may_only_refuse() {
    for (handler, prompt) in [
        ("test_generation", "Write unit tests"),
        ("code_refactoring", "Refactor my morning routine"),
        ("format_conversion", "Convert this JSON to YAML"),
    ] {
        assert_eq!(
            claim_admission(handler, prompt, &prompt.to_lowercase()),
            ClaimAdmission::RefusalOnly,
            "{handler}: {prompt}"
        );
        assert!(claim_admitted(handler, prompt, &prompt.to_lowercase()));
    }
    for (handler, prompt) in [
        (
            "test_generation",
            "Write tests for `square(n)`: square(3) returns 9",
        ),
        (
            "code_refactoring",
            "Refactor this promise chain with async/await:\n```javascript\nfetch(url).then(r => r.json());\n```",
        ),
        (
            "format_conversion",
            "Convert this JSON to YAML:\n```json\n{\"a\": 1}\n```",
        ),
    ] {
        assert_eq!(
            claim_admission(handler, prompt, &prompt.to_lowercase()),
            ClaimAdmission::Full,
            "{handler}: {prompt}"
        );
    }
    // A row without a refusal event still denies a prompt without evidence.
    assert_eq!(
        claim_admission(
            "code_review",
            "Review my essay about the ocean",
            "review my essay about the ocean"
        ),
        ClaimAdmission::Denied
    );
}

#[test]
fn the_named_refusal_of_a_refusal_group_request_is_still_the_answer() {
    let response = formal_ai::FormalAiEngine.answer("Write unit tests");
    assert_eq!(response.intent, "test_generation");
    assert_eq!(
        response.answer,
        "Recognized a test-writing request, but no function under test could be identified \u{2014} name it, ideally in backticks like `is_palindrome(s)` \u{2014} so I will not guess test cases."
    );
}

// Issue #1175 R3 classes (b), (c) and (e): a follow-up admits on the earlier
// turn it continues, read from the dialogue log; a composer on its
// specification, admitted to its refusal lane without one; a lookup or policy
// on the subject or shape its own reader extracts. Mirrored by the class tests
// of `rust/tests/web/issue-1175-claim-routing.test.mjs`.
fn dialogue(turns: &[(&str, &str)]) -> EventLog {
    let mut log = EventLog::new();
    for (role, content) in turns {
        let kind = if *role == "user" {
            "prior_turn:user"
        } else {
            "prior_turn:assistant"
        };
        log.append(kind, (*content).to_owned());
    }
    log
}

#[test]
fn class_b_dialogue_evidence_reads_the_earlier_turns_never_the_prompt_alone() {
    let research = dialogue(&[
        ("user", "Research the best laptops for programming"),
        ("assistant", "Here is what I found."),
    ]);
    let how_to = dialogue(&[
        ("user", "How to tie a tie?"),
        ("assistant", "Procedural discovery for tie a tie."),
    ]);
    let empty = EventLog::new();
    for (kind, prompt, log, holds) in [
        (
            "prior_research_request",
            "What is the result?",
            &research,
            true,
        ),
        (
            "prior_research_request",
            "What is the result?",
            &empty,
            false,
        ),
        (
            "prior_procedure",
            "Can you give me specific instructions?",
            &how_to,
            true,
        ),
        (
            "prior_procedure",
            "Can you give me specific instructions?",
            &empty,
            false,
        ),
        ("prior_user_request", "Answer in Russian", &how_to, true),
        ("prior_user_request", "Answer in Russian", &empty, false),
        ("prior_reply", "How does it work?", &how_to, true),
        ("prior_software_project", "Now test it", &empty, false),
    ] {
        assert_eq!(
            claim_evidence_holds_in_dialogue(kind, prompt, &prompt.to_lowercase(), log),
            Some(holds),
            "{kind}: {prompt}"
        );
    }
    assert_eq!(
        claim_admission_in_dialogue(
            "research_result_followup",
            "What is the result?",
            "what is the result?",
            &research
        ),
        ClaimAdmission::Full
    );
    assert_eq!(
        claim_admission(
            "research_result_followup",
            "What is the result?",
            "what is the result?"
        ),
        ClaimAdmission::Denied
    );
}

#[test]
fn class_c_a_composer_without_its_specification_may_only_refuse() {
    for (handler, bare, specified) in [
        (
            "regex_synthesis",
            "Write a regex",
            "Write a regular expression that matches five digits",
        ),
        (
            "sql_synthesis",
            "Write a SQL query",
            "Write a SQL query that selects every row of the users table",
        ),
        (
            "planning_request",
            "Plan a trip somewhere nice",
            "Make a 3-day itinerary for a first visit to Rome.",
        ),
    ] {
        assert_eq!(
            claim_admission(handler, bare, &bare.to_lowercase()),
            ClaimAdmission::RefusalOnly,
            "{handler}: {bare}"
        );
        assert_eq!(
            claim_admission(handler, specified, &specified.to_lowercase()),
            ClaimAdmission::Full,
            "{handler}: {specified}"
        );
    }
    for (kind, prompt, holds) in [
        ("function_spec", "Write a Python function", false),
        ("document_format", "Make me a PDF about cats", true),
        (
            "install_steps",
            "Turn this installation guide into a bash script",
            false,
        ),
    ] {
        assert_eq!(
            claim_evidence_holds(kind, prompt, &prompt.to_lowercase()),
            Some(holds),
            "{kind}: {prompt}"
        );
    }
}

#[test]
fn class_e_a_lookup_or_policy_admits_on_its_subject_or_shape() {
    for (kind, prompt, holds) in [
        ("concept_subject", "What is a monad?", true),
        ("concept_subject", "Tell me something interesting", false),
        ("assistant_addressee", "Do you think it will rain?", true),
        ("assistant_addressee", "Do they think it will rain?", false),
        ("assistant_addressee", "Каково твоё мнение?", true),
        ("punctuation_only", "?!", true),
        ("punctuation_only", "why?", false),
        ("unbalanced_brackets", "teach this fact (a b", true),
        ("unbalanced_brackets", "teach this fact (a b)", false),
        (
            "call_expression",
            "Write a Python script that calls undefined_function()",
            true,
        ),
        (
            "call_expression",
            "Write a Python script that prints hello",
            false,
        ),
        (
            "legality_assessment",
            "Help me break into a parked car tonight",
            true,
        ),
        (
            "legality_assessment",
            "The band will break into song at the party",
            false,
        ),
    ] {
        assert_eq!(
            claim_evidence_holds(kind, prompt, &prompt.to_lowercase()),
            Some(holds),
            "{kind}: {prompt}"
        );
    }
}

#[test]
fn r1173_3_a_trailing_shaping_sentence_leaves_the_question_to_the_concept_reader() {
    assert_eq!(
        without_answer_shape_directive("What is Rust? Explain briefly."),
        "What is Rust?"
    );
    assert_eq!(
        without_answer_shape_directive("What is Rust? Explain how it manages memory."),
        "What is Rust? Explain how it manages memory."
    );
    assert_eq!(
        formal_ai::FormalAiEngine
            .answer("What is Rust? Explain briefly.")
            .intent,
        "concept_lookup"
    );
}

#[test]
fn r1173_3_breaking_into_a_parked_car_is_warned_about() {
    assert_eq!(
        formal_ai::FormalAiEngine
            .answer("Help me break into a parked car tonight")
            .intent,
        "legality_warn"
    );
}
