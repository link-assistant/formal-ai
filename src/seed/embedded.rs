//! Embedded Links Notation seed files and the file registry.
//!
//! Every `data/seed/*.lino` file is compiled into the binary with
//! [`include_str!`] so even offline builds expose the same data the browser
//! fetches at runtime. [`seed_files`] returns them in declaration order so
//! callers can render the merged bundle deterministically, and
//! [`MEANING_FILES`] names the subset that make up the language-independent
//! meaning lexicon (see [`super::lexicon`]). Keeping this registry in its own
//! module leaves the rest of `seed.rs` room to grow under the file-size guard.

/// Raw embedded contents (used by `merged_bundle` and by tests).
pub const AGENT_INFO_LINO: &str = include_str!("../../data/seed/agent-info.lino");
pub const AGENTIC_TOOL_CAPABILITIES_LINO: &str =
    include_str!("../../data/seed/agentic-tool-capabilities.lino");
pub const INTERFACE_CAPABILITIES_LINO: &str =
    include_str!("../../data/seed/interface-capabilities.lino");
pub const MULTILINGUAL_RESPONSES_LINO: &str =
    include_str!("../../data/seed/multilingual-responses.lino");
pub const MULTILINGUAL_RESPONSES_AGENTIC_LINO: &str =
    include_str!("../../data/seed/multilingual-responses-agentic.lino");
pub const CONCEPTS_LINO: &str = include_str!("../../data/seed/concepts.lino");
pub const CONCEPT_CONTEXTS_LINO: &str = include_str!("../../data/seed/concept-contexts.lino");
pub const FACTS_LINO: &str = include_str!("../../data/seed/facts.lino");
pub const MODEL_ALIASES_LINO: &str = include_str!("../../data/seed/model-aliases.lino");
pub const MARKET_PRICE_REFERENCES_LINO: &str =
    include_str!("../../data/seed/market-price-references.lino");
pub const CLIENT_INTEGRATIONS_LINO: &str = include_str!("../../data/seed/client-integrations.lino");
pub const BRAINSTORM_SEEDS_LINO: &str = include_str!("../../data/seed/brainstorm-seeds.lino");
pub const PERSONAS_LINO: &str = include_str!("../../data/seed/personas.lino");
pub const SUMMARY_TOPICS_LINO: &str = include_str!("../../data/seed/summary-topics.lino");
pub const COREFERENCE_LINO: &str = include_str!("../../data/seed/coreference.lino");
pub const TOOLS_LINO: &str = include_str!("../../data/seed/tools.lino");
pub const LANGUAGE_DETECTION_LINO: &str = include_str!("../../data/seed/language-detection.lino");
pub const PROMPT_PATTERNS_LINO: &str = include_str!("../../data/seed/prompt-patterns.lino");
pub const INTENT_ROUTING_LINO: &str = include_str!("../../data/seed/intent-routing.lino");
pub const HANDLER_PRECEDENCE_LINO: &str = include_str!("../../data/seed/handler-precedence.lino");
pub const LEARNING_SOURCES_LINO: &str = include_str!("../../data/seed/learning-sources.lino");
pub const OPERATION_VOCABULARY_LINO: &str =
    include_str!("../../data/seed/operation-vocabulary.lino");
pub const TERMINAL_COMMANDS_LINO: &str = include_str!("../../data/seed/terminal-commands.lino");
pub const SHELL_INTENTS_LINO: &str = include_str!("../../data/seed/shell-intents.lino");
pub const NUMERIC_LIST_OPERATIONS_LINO: &str =
    include_str!("../../data/seed/numeric-list-operations.lino");
pub const CODING_IDIOMS_LINO: &str = include_str!("../../data/seed/coding-idioms.lino");
pub const PROGRAM_CST_GRAMMARS_LINO: &str =
    include_str!("../../data/seed/program-cst-grammars.lino");
pub const MEANINGS_LINO: &str = include_str!("../../data/seed/meanings.lino");
pub const MEANINGS_UNITS_LINO: &str = include_str!("../../data/seed/meanings-units.lino");
pub const MEANINGS_CALENDAR_LINO: &str = include_str!("../../data/seed/meanings-calendar.lino");
pub const MEANINGS_CALCULATOR_LINO: &str = include_str!("../../data/seed/meanings-calculator.lino");
pub const MEANINGS_FACTS_LINO: &str = include_str!("../../data/seed/meanings-facts.lino");
pub const MEANINGS_SOFTWARE_PROJECT_LINO: &str =
    include_str!("../../data/seed/meanings-software-project.lino");
pub const MEANINGS_PROGRAM_SYNTHESIS_LINO: &str =
    include_str!("../../data/seed/meanings-program-synthesis.lino");
pub const MEANINGS_INTENT_LINO: &str = include_str!("../../data/seed/meanings-intent.lino");
pub const MEANINGS_HOW_LINO: &str = include_str!("../../data/seed/meanings-how.lino");
pub const MEANINGS_META_LINO: &str = include_str!("../../data/seed/meanings-meta.lino");
pub const MEANINGS_SEARCH_LINO: &str = include_str!("../../data/seed/meanings-search.lino");
pub const MEANINGS_WEB_NAVIGATION_LINO: &str =
    include_str!("../../data/seed/meanings-web-navigation.lino");
pub const MEANINGS_WEB_SEARCH_LINO: &str = include_str!("../../data/seed/meanings-web-search.lino");
pub const MEANINGS_WEB_SEARCH_QUERY_LINO: &str =
    include_str!("../../data/seed/meanings-web-search-query.lino");
pub const MEANINGS_WEB_RESEARCH_LINO: &str =
    include_str!("../../data/seed/meanings-web-research.lino");
pub const MEANINGS_WEB_FOLLOWUP_LINO: &str =
    include_str!("../../data/seed/meanings-web-followup.lino");
pub const MEANINGS_TRANSLATION_LINO: &str =
    include_str!("../../data/seed/meanings-translation.lino");
pub const MEANINGS_ONTOLOGY_LINO: &str = include_str!("../../data/seed/meanings-ontology.lino");
pub const MEANINGS_SEMANTIC_META_LINO: &str =
    include_str!("../../data/seed/meanings-semantic-meta.lino");
pub const MEANINGS_LEXICAL_META_LINO: &str =
    include_str!("../../data/seed/meanings-lexical-meta.lino");
pub const MEANINGS_LINKS_ROOT_LINO: &str = include_str!("../../data/seed/meanings-links-root.lino");
pub const MEANINGS_WIKIDATA_LINO: &str = include_str!("../../data/seed/meanings-wikidata.lino");
pub const MEANINGS_BEHAVIOR_RULES_LINO: &str =
    include_str!("../../data/seed/meanings-behavior-rules.lino");
pub const MEANINGS_PROOF_LINO: &str = include_str!("../../data/seed/meanings-proof.lino");
pub const MEANINGS_POLICY_LINO: &str = include_str!("../../data/seed/meanings-policy.lino");
pub const MEANINGS_DOCS_LINO: &str = include_str!("../../data/seed/meanings-docs.lino");
pub const MEANINGS_SKILL_COMPILER_LINO: &str =
    include_str!("../../data/seed/meanings-skill-compiler.lino");
pub const MEANINGS_FINANCE_LINO: &str = include_str!("../../data/seed/meanings-finance.lino");
pub const MEANINGS_DEFINITION_MERGE_LINO: &str =
    include_str!("../../data/seed/meanings-definition-merge.lino");
pub const MEANINGS_TOOL_ACCESS_LINO: &str =
    include_str!("../../data/seed/meanings-tool-access.lino");
pub const MEANINGS_FEATURE_CAPABILITY_LINO: &str =
    include_str!("../../data/seed/meanings-feature-capability.lino");
pub const MEANINGS_FILE_WRITE_LINO: &str = include_str!("../../data/seed/meanings-file-write.lino");
pub const MEANINGS_FILE_EDIT_LINO: &str = include_str!("../../data/seed/meanings-file-edit.lino");
pub const MEANINGS_AGENT_ACTIONS_LINO: &str =
    include_str!("../../data/seed/meanings-agent-actions.lino");
pub const MEANINGS_PLAYWRIGHT_LINO: &str = include_str!("../../data/seed/meanings-playwright.lino");
pub const MEANINGS_RESEARCH_TABLE_LINO: &str =
    include_str!("../../data/seed/meanings-research-table.lino");
pub const MEANINGS_CONVERSATION_LINO: &str =
    include_str!("../../data/seed/meanings-conversation.lino");
pub const MEANINGS_SUMMARY_LINO: &str = include_str!("../../data/seed/meanings-summary.lino");
pub const MEANINGS_CODING_CATALOG_LINO: &str =
    include_str!("../../data/seed/meanings-coding-catalog.lino");
pub const MEANINGS_LEXICON_IMPORT_01_LINO: &str =
    include_str!("../../data/seed/meanings-lexicon-import-01.lino");
pub const MEANINGS_LEXICON_IMPORT_02_LINO: &str =
    include_str!("../../data/seed/meanings-lexicon-import-02.lino");
pub const MEANINGS_LEXICON_IMPORT_03_LINO: &str =
    include_str!("../../data/seed/meanings-lexicon-import-03.lino");
pub const MEANINGS_LEXICON_IMPORT_04_LINO: &str =
    include_str!("../../data/seed/meanings-lexicon-import-04.lino");
pub const GREETINGS_LINO: &str = include_str!("../../data/seed/greetings.lino");
pub const IDENTITY_LINO: &str = include_str!("../../data/seed/identity.lino");
pub const HELLO_WORLD_PROGRAMS_LINO: &str =
    include_str!("../../data/seed/hello-world-programs.lino");
pub const PROGRAM_PLAN_RULES_LINO: &str = include_str!("../../data/seed/program-plan-rules.lino");
pub const SELF_IMPROVEMENT_LOOP_LINO: &str =
    include_str!("../../data/seed/self-improvement-loop.lino");
pub const DEMO_DIALOGS_LINO: &str = include_str!("../../data/seed/demo-dialogs.lino");
pub const ENVIRONMENTS_LINO: &str = include_str!("../../data/seed/environments.lino");
pub const PROJECTS_LINO: &str = include_str!("../../data/seed/projects.lino");

/// Embedded copy of every Links Notation seed file. Returned in declaration
/// order so callers can render the merged bundle deterministically.
#[must_use]
pub fn seed_files() -> Vec<(&'static str, &'static str)> {
    vec![
        ("data/seed/agent-info.lino", AGENT_INFO_LINO),
        (
            "data/seed/agentic-tool-capabilities.lino",
            AGENTIC_TOOL_CAPABILITIES_LINO,
        ),
        (
            "data/seed/interface-capabilities.lino",
            INTERFACE_CAPABILITIES_LINO,
        ),
        (
            "data/seed/multilingual-responses.lino",
            MULTILINGUAL_RESPONSES_LINO,
        ),
        (
            "data/seed/multilingual-responses-agentic.lino",
            MULTILINGUAL_RESPONSES_AGENTIC_LINO,
        ),
        ("data/seed/concepts.lino", CONCEPTS_LINO),
        ("data/seed/concept-contexts.lino", CONCEPT_CONTEXTS_LINO),
        ("data/seed/facts.lino", FACTS_LINO),
        ("data/seed/model-aliases.lino", MODEL_ALIASES_LINO),
        (
            "data/seed/market-price-references.lino",
            MARKET_PRICE_REFERENCES_LINO,
        ),
        (
            "data/seed/client-integrations.lino",
            CLIENT_INTEGRATIONS_LINO,
        ),
        ("data/seed/brainstorm-seeds.lino", BRAINSTORM_SEEDS_LINO),
        ("data/seed/personas.lino", PERSONAS_LINO),
        ("data/seed/summary-topics.lino", SUMMARY_TOPICS_LINO),
        ("data/seed/coreference.lino", COREFERENCE_LINO),
        ("data/seed/tools.lino", TOOLS_LINO),
        ("data/seed/language-detection.lino", LANGUAGE_DETECTION_LINO),
        ("data/seed/prompt-patterns.lino", PROMPT_PATTERNS_LINO),
        ("data/seed/intent-routing.lino", INTENT_ROUTING_LINO),
        ("data/seed/handler-precedence.lino", HANDLER_PRECEDENCE_LINO),
        ("data/seed/learning-sources.lino", LEARNING_SOURCES_LINO),
        (
            "data/seed/operation-vocabulary.lino",
            OPERATION_VOCABULARY_LINO,
        ),
        ("data/seed/terminal-commands.lino", TERMINAL_COMMANDS_LINO),
        ("data/seed/shell-intents.lino", SHELL_INTENTS_LINO),
        (
            "data/seed/numeric-list-operations.lino",
            NUMERIC_LIST_OPERATIONS_LINO,
        ),
        ("data/seed/coding-idioms.lino", CODING_IDIOMS_LINO),
        (
            "data/seed/program-cst-grammars.lino",
            PROGRAM_CST_GRAMMARS_LINO,
        ),
        ("data/seed/meanings.lino", MEANINGS_LINO),
        ("data/seed/meanings-units.lino", MEANINGS_UNITS_LINO),
        ("data/seed/meanings-calendar.lino", MEANINGS_CALENDAR_LINO),
        (
            "data/seed/meanings-calculator.lino",
            MEANINGS_CALCULATOR_LINO,
        ),
        ("data/seed/meanings-facts.lino", MEANINGS_FACTS_LINO),
        (
            "data/seed/meanings-software-project.lino",
            MEANINGS_SOFTWARE_PROJECT_LINO,
        ),
        (
            "data/seed/meanings-program-synthesis.lino",
            MEANINGS_PROGRAM_SYNTHESIS_LINO,
        ),
        ("data/seed/meanings-intent.lino", MEANINGS_INTENT_LINO),
        ("data/seed/meanings-how.lino", MEANINGS_HOW_LINO),
        ("data/seed/meanings-meta.lino", MEANINGS_META_LINO),
        (
            "data/seed/meanings-web-navigation.lino",
            MEANINGS_WEB_NAVIGATION_LINO,
        ),
        (
            "data/seed/meanings-web-search.lino",
            MEANINGS_WEB_SEARCH_LINO,
        ),
        (
            "data/seed/meanings-web-search-query.lino",
            MEANINGS_WEB_SEARCH_QUERY_LINO,
        ),
        (
            "data/seed/meanings-web-research.lino",
            MEANINGS_WEB_RESEARCH_LINO,
        ),
        (
            "data/seed/meanings-web-followup.lino",
            MEANINGS_WEB_FOLLOWUP_LINO,
        ),
        (
            "data/seed/meanings-translation.lino",
            MEANINGS_TRANSLATION_LINO,
        ),
        ("data/seed/meanings-ontology.lino", MEANINGS_ONTOLOGY_LINO),
        (
            "data/seed/meanings-semantic-meta.lino",
            MEANINGS_SEMANTIC_META_LINO,
        ),
        (
            "data/seed/meanings-lexical-meta.lino",
            MEANINGS_LEXICAL_META_LINO,
        ),
        (
            "data/seed/meanings-links-root.lino",
            MEANINGS_LINKS_ROOT_LINO,
        ),
        ("data/seed/meanings-wikidata.lino", MEANINGS_WIKIDATA_LINO),
        (
            "data/seed/meanings-behavior-rules.lino",
            MEANINGS_BEHAVIOR_RULES_LINO,
        ),
        ("data/seed/meanings-proof.lino", MEANINGS_PROOF_LINO),
        ("data/seed/meanings-policy.lino", MEANINGS_POLICY_LINO),
        ("data/seed/meanings-docs.lino", MEANINGS_DOCS_LINO),
        (
            "data/seed/meanings-skill-compiler.lino",
            MEANINGS_SKILL_COMPILER_LINO,
        ),
        ("data/seed/meanings-finance.lino", MEANINGS_FINANCE_LINO),
        (
            "data/seed/meanings-definition-merge.lino",
            MEANINGS_DEFINITION_MERGE_LINO,
        ),
        (
            "data/seed/meanings-tool-access.lino",
            MEANINGS_TOOL_ACCESS_LINO,
        ),
        (
            "data/seed/meanings-feature-capability.lino",
            MEANINGS_FEATURE_CAPABILITY_LINO,
        ),
        (
            "data/seed/meanings-file-write.lino",
            MEANINGS_FILE_WRITE_LINO,
        ),
        ("data/seed/meanings-file-edit.lino", MEANINGS_FILE_EDIT_LINO),
        (
            "data/seed/meanings-agent-actions.lino",
            MEANINGS_AGENT_ACTIONS_LINO,
        ),
        (
            "data/seed/meanings-playwright.lino",
            MEANINGS_PLAYWRIGHT_LINO,
        ),
        (
            "data/seed/meanings-research-table.lino",
            MEANINGS_RESEARCH_TABLE_LINO,
        ),
        (
            "data/seed/meanings-conversation.lino",
            MEANINGS_CONVERSATION_LINO,
        ),
        ("data/seed/meanings-summary.lino", MEANINGS_SUMMARY_LINO),
        (
            "data/seed/meanings-coding-catalog.lino",
            MEANINGS_CODING_CATALOG_LINO,
        ),
        (
            "data/seed/meanings-lexicon-import-01.lino",
            MEANINGS_LEXICON_IMPORT_01_LINO,
        ),
        (
            "data/seed/meanings-lexicon-import-02.lino",
            MEANINGS_LEXICON_IMPORT_02_LINO,
        ),
        (
            "data/seed/meanings-lexicon-import-03.lino",
            MEANINGS_LEXICON_IMPORT_03_LINO,
        ),
        (
            "data/seed/meanings-lexicon-import-04.lino",
            MEANINGS_LEXICON_IMPORT_04_LINO,
        ),
        ("data/seed/greetings.lino", GREETINGS_LINO),
        ("data/seed/identity.lino", IDENTITY_LINO),
        (
            "data/seed/hello-world-programs.lino",
            HELLO_WORLD_PROGRAMS_LINO,
        ),
        ("data/seed/program-plan-rules.lino", PROGRAM_PLAN_RULES_LINO),
        (
            "data/seed/self-improvement-loop.lino",
            SELF_IMPROVEMENT_LOOP_LINO,
        ),
        ("data/seed/demo-dialogs.lino", DEMO_DIALOGS_LINO),
        ("data/seed/environments.lino", ENVIRONMENTS_LINO),
        ("data/seed/projects.lino", PROJECTS_LINO),
    ]
}

/// The ordered set of multilingual-response files, walked by
/// [`super::multilingual_responses`].
///
/// Split so neither breaches the seed file-size guard; each wraps its records
/// under a top-level `multilingual_responses` node and the parser walks all of
/// them, so an intent may live in whichever file keeps the sizes balanced.
pub const RESPONSE_FILES: &[&str] = &[
    MULTILINGUAL_RESPONSES_LINO,
    MULTILINGUAL_RESPONSES_AGENTIC_LINO,
];

/// The ordered set of meaning-lexicon files, concatenated by [`super::lexicon`].
///
/// Split across several `.lino` files so none breaches the seed file-size guard;
/// each wraps its records under a top-level `meanings` node (the loader walks all).
pub const MEANING_FILES: &[&str] = &[
    MEANINGS_LINO,
    MEANINGS_UNITS_LINO,
    MEANINGS_CALENDAR_LINO,
    MEANINGS_CALCULATOR_LINO,
    MEANINGS_FACTS_LINO,
    MEANINGS_SOFTWARE_PROJECT_LINO,
    MEANINGS_PROGRAM_SYNTHESIS_LINO,
    MEANINGS_INTENT_LINO,
    MEANINGS_HOW_LINO,
    MEANINGS_META_LINO,
    MEANINGS_SEARCH_LINO,
    MEANINGS_WEB_NAVIGATION_LINO,
    MEANINGS_WEB_SEARCH_LINO,
    MEANINGS_WEB_SEARCH_QUERY_LINO,
    MEANINGS_WEB_RESEARCH_LINO,
    MEANINGS_WEB_FOLLOWUP_LINO,
    MEANINGS_TRANSLATION_LINO,
    MEANINGS_ONTOLOGY_LINO,
    MEANINGS_SEMANTIC_META_LINO,
    MEANINGS_LEXICAL_META_LINO,
    MEANINGS_LINKS_ROOT_LINO,
    MEANINGS_WIKIDATA_LINO,
    MEANINGS_BEHAVIOR_RULES_LINO,
    MEANINGS_PROOF_LINO,
    MEANINGS_POLICY_LINO,
    MEANINGS_DOCS_LINO,
    MEANINGS_SKILL_COMPILER_LINO,
    MEANINGS_FINANCE_LINO,
    MEANINGS_DEFINITION_MERGE_LINO,
    MEANINGS_TOOL_ACCESS_LINO,
    MEANINGS_FEATURE_CAPABILITY_LINO,
    MEANINGS_FILE_WRITE_LINO,
    MEANINGS_FILE_EDIT_LINO,
    MEANINGS_AGENT_ACTIONS_LINO,
    MEANINGS_PLAYWRIGHT_LINO,
    MEANINGS_RESEARCH_TABLE_LINO,
    MEANINGS_CONVERSATION_LINO,
    MEANINGS_SUMMARY_LINO,
    MEANINGS_CODING_CATALOG_LINO,
    MEANINGS_LEXICON_IMPORT_01_LINO,
    MEANINGS_LEXICON_IMPORT_02_LINO,
    MEANINGS_LEXICON_IMPORT_03_LINO,
    MEANINGS_LEXICON_IMPORT_04_LINO,
];
