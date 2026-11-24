use super::*;

pub(super) fn the_claim_rows_are_read_from_the_capability_table() {
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
            (
                "requirement_listing",
                "requirement_listing",
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
            (
                "learn_from_source",
                "tryLearnFromSource",
                vec!["learnable_source"]
            ),
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
            (
                "conversation_control",
                "",
                vec!["backticked_term", "prior_reply"]
            ),
            (
                "agentic_continuation",
                "tryAgenticContinuation",
                vec!["prior_reply"]
            ),
            ("clarification", "tryClarification", vec!["prior_reply"]),
            (
                "current_dialogue_fact_checking",
                "tryCurrentDialogueFactChecking",
                vec!["prior_user_request"]
            ),
            (
                "historical",
                "tryHistorical",
                vec!["dialogue_turn", "name_assignment", "recall_query_term"]
            ),
            (
                "conversation_memory",
                "",
                vec![
                    "dialogue_turn",
                    "name_assignment",
                    "recall_query_term",
                    "supplied_payload"
                ]
            ),
            ("summarization", "", vec!["summary_topic"]),
            (
                "brainstorming",
                "tryBrainstormingRequest",
                vec!["brainstorm_category"]
            ),
            ("roleplay", "tryRoleplayRequest", vec!["persona_or_topic"]),
            (
                "document_originality_check",
                "tryDocumentOriginalityCheck",
                vec!["document_operand"]
            ),
            ("translation", "tryTranslation", vec!["translation_text"]),
            (
                "triz_resolution",
                "tryTrizResolution",
                vec!["triz_precedent"]
            ),
            ("algorithm", "tryAlgorithm", vec!["algorithm_operation"]),
            (
                "source_refresh",
                "trySourceRefresh",
                vec!["source_reference"]
            ),
            ("proof_request", "tryProofRequest", vec!["stated_claim"]),
            ("capabilities", "tryCapabilities", vec!["assistant_subject"]),
            (
                "meta_explanation",
                "tryMetaExplanation",
                vec!["assistant_subject"]
            ),
            (
                "network_query",
                "tryNetworkSnapshot",
                vec!["assistant_subject"]
            ),
            (
                "exact_memory_query",
                "tryExactMemoryQuery",
                vec!["memory_query_statement"]
            ),
            ("fact_lookup", "", vec!["fact_subject"]),
            (
                "docs_method_explanation",
                "tryDocsMethodExplanation",
                vec!["documented_method"]
            ),
            ("kupi_slona", "tryKupiSlona", vec!["idiom_utterance"]),
            (
                "source_conflict",
                "trySourceConflict",
                vec!["attributed_alternatives"]
            ),
            (
                "github_repository_traffic",
                "tryGithubRepositoryTraffic",
                vec!["named_repository"]
            ),
            (
                "formalization_request",
                "tryFormalizationRequest",
                vec!["formalization_statement"]
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
            ("product_search", vec!["product_search:refusal"]),
            ("conversation_control", vec!["conversation_control:refusal"]),
            ("agentic_continuation", vec!["agentic_continuation:refusal"]),
            ("clarification", vec!["clarification:refusal"]),
            (
                "current_dialogue_fact_checking",
                vec!["current_dialogue_fact_checking:refusal"]
            ),
            ("historical", vec!["conversation_recall:refusal"]),
            ("conversation_memory", vec!["conversation_recall:refusal"]),
            ("summarization", vec!["summarization:refusal"]),
            ("brainstorming", vec!["brainstorming:refusal"]),
            ("roleplay", vec!["roleplay:refusal"]),
            (
                "document_originality_check",
                vec!["document_originality_check:refusal"]
            ),
            ("translation", vec!["translation:refusal"]),
            ("triz_resolution", vec!["triz_resolution:refusal"]),
            ("algorithm", vec!["algorithm:refusal"]),
            ("source_refresh", vec!["source_refresh:refusal"]),
            ("proof_request", vec!["proof_request:refusal"]),
            ("source_conflict", vec!["source_conflict:refusal"]),
            (
                "github_repository_traffic",
                vec!["github_repository_traffic:refusal"]
            ),
            (
                "formalization_request",
                vec!["formalization_request:refusal"]
            ),
        ]
    );
}
