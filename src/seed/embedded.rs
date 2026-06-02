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
pub const MULTILINGUAL_RESPONSES_LINO: &str =
    include_str!("../../data/seed/multilingual-responses.lino");
pub const CONCEPTS_LINO: &str = include_str!("../../data/seed/concepts.lino");
pub const CONCEPT_CONTEXTS_LINO: &str = include_str!("../../data/seed/concept-contexts.lino");
pub const FACTS_LINO: &str = include_str!("../../data/seed/facts.lino");
pub const BRAINSTORM_SEEDS_LINO: &str = include_str!("../../data/seed/brainstorm-seeds.lino");
pub const PERSONAS_LINO: &str = include_str!("../../data/seed/personas.lino");
pub const SUMMARY_TOPICS_LINO: &str = include_str!("../../data/seed/summary-topics.lino");
pub const COREFERENCE_LINO: &str = include_str!("../../data/seed/coreference.lino");
pub const TOOLS_LINO: &str = include_str!("../../data/seed/tools.lino");
pub const LANGUAGE_DETECTION_LINO: &str = include_str!("../../data/seed/language-detection.lino");
pub const PROMPT_PATTERNS_LINO: &str = include_str!("../../data/seed/prompt-patterns.lino");
pub const INTENT_ROUTING_LINO: &str = include_str!("../../data/seed/intent-routing.lino");
pub const OPERATION_VOCABULARY_LINO: &str =
    include_str!("../../data/seed/operation-vocabulary.lino");
pub const MEANINGS_LINO: &str = include_str!("../../data/seed/meanings.lino");
pub const MEANINGS_UNITS_LINO: &str = include_str!("../../data/seed/meanings-units.lino");
pub const MEANINGS_CALENDAR_LINO: &str = include_str!("../../data/seed/meanings-calendar.lino");
pub const MEANINGS_FACTS_LINO: &str = include_str!("../../data/seed/meanings-facts.lino");
pub const MEANINGS_SOFTWARE_PROJECT_LINO: &str =
    include_str!("../../data/seed/meanings-software-project.lino");
pub const MEANINGS_PROGRAM_SYNTHESIS_LINO: &str =
    include_str!("../../data/seed/meanings-program-synthesis.lino");
pub const MEANINGS_INTENT_LINO: &str = include_str!("../../data/seed/meanings-intent.lino");
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
            "data/seed/multilingual-responses.lino",
            MULTILINGUAL_RESPONSES_LINO,
        ),
        ("data/seed/concepts.lino", CONCEPTS_LINO),
        ("data/seed/concept-contexts.lino", CONCEPT_CONTEXTS_LINO),
        ("data/seed/facts.lino", FACTS_LINO),
        ("data/seed/brainstorm-seeds.lino", BRAINSTORM_SEEDS_LINO),
        ("data/seed/personas.lino", PERSONAS_LINO),
        ("data/seed/summary-topics.lino", SUMMARY_TOPICS_LINO),
        ("data/seed/coreference.lino", COREFERENCE_LINO),
        ("data/seed/tools.lino", TOOLS_LINO),
        ("data/seed/language-detection.lino", LANGUAGE_DETECTION_LINO),
        ("data/seed/prompt-patterns.lino", PROMPT_PATTERNS_LINO),
        ("data/seed/intent-routing.lino", INTENT_ROUTING_LINO),
        (
            "data/seed/operation-vocabulary.lino",
            OPERATION_VOCABULARY_LINO,
        ),
        ("data/seed/meanings.lino", MEANINGS_LINO),
        ("data/seed/meanings-units.lino", MEANINGS_UNITS_LINO),
        ("data/seed/meanings-calendar.lino", MEANINGS_CALENDAR_LINO),
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

/// The ordered set of meaning-lexicon files, concatenated by [`super::lexicon`].
///
/// Split across several `.lino` files so none breaches the seed file-size guard;
/// each wraps its records under a top-level `meanings` node (the loader walks all).
pub const MEANING_FILES: &[&str] = &[
    MEANINGS_LINO,
    MEANINGS_UNITS_LINO,
    MEANINGS_CALENDAR_LINO,
    MEANINGS_FACTS_LINO,
    MEANINGS_SOFTWARE_PROJECT_LINO,
    MEANINGS_PROGRAM_SYNTHESIS_LINO,
    MEANINGS_INTENT_LINO,
];
