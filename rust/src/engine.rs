//! Deterministic symbolic engine and Links-Notation knowledge dataset.
//!
//! This module hosts the deterministic answer projection primitives used by the
//! universal solver. Callers should not invoke `FormalAiEngine::answer` to
//! bypass the solver — it delegates to [`crate::solver::UniversalSolver::solve`]
//! so every request walks the same 11-step loop documented in `VISION.md`.

pub(crate) use crate::coding::{
    PROGRAM_LANGUAGES, ProgramSpec, WRITE_PROGRAM_INTENT, program_spec, program_template_count,
    supported_program_languages, supported_program_tasks,
};

use std::sync::OnceLock;

use serde::{Deserialize, Serialize};

use crate::engine_assistant_name::{
    assistant_name_answer, chinese_assistant_name_answer, hindi_assistant_name_answer,
    russian_assistant_name_answer,
};
pub(crate) use crate::engine_responses::{
    assistant_free_time_answer, farewell_answer, greeting_answer, identity_answer, unknown_answer,
    unknown_language_fallback_answer, wellbeing_answer,
};
use crate::engine_responses::{
    chinese_courtesy_response_answer, chinese_farewell_answer, chinese_identity_answer,
    chinese_test_status_answer, chinese_wellbeing_answer, courtesy_response_answer,
    hindi_courtesy_response_answer, hindi_farewell_answer, hindi_identity_answer,
    hindi_test_status_answer, hindi_wellbeing_answer, russian_courtesy_response_answer,
    russian_farewell_answer, russian_identity_answer, russian_test_status_answer,
    russian_wellbeing_answer, test_status_answer,
};
use crate::event_log::EventLog;
use crate::language::Language;
use crate::links_format::{flatten_lino_value, format_lino_record};
use crate::seed;

mod program_answer;
use program_answer::write_program_answer;
pub(crate) use program_answer::{execution_report, write_program_answer_with_output};

pub const DEFAULT_MODEL: &str = "formal-ai";

// Thinking model + deterministic naturalizer live in `crate::thinking` (issue #488),
// re-exported so `crate::engine::{...}` / `formal_ai::{...}` paths stay unchanged.
pub use crate::thinking::{
    ThinkingStep, humanize_meta_identifier, localize_thinking_steps, naturalize_thinking_step,
    naturalize_thinking_step_in, render_thinking_steps, render_thinking_steps_in,
    thinking_answer_language, thinking_language_label, thinking_language_label_in,
    thinking_narrative, thinking_narrative_in, thinking_trace_heading,
};

pub use crate::engine_answer::{ExecutionRecipe, ExecutionRecipeFile, SymbolicAnswer};

#[derive(Debug, Default, Clone, Copy)]
pub struct FormalAiEngine;

impl FormalAiEngine {
    /// Answer a prompt by running it through the universal solver loop.
    #[must_use]
    pub fn answer(&self, prompt: &str) -> SymbolicAnswer {
        self.answer_with_memory(prompt, &[])
    }

    /// Answer with the retained memory events that may change later solving.
    ///
    /// The zero-memory [`Self::answer`] entry point remains source-compatible;
    /// stateful library callers use this method so retained dreaming amendments
    /// reach the same engine surface as the protocol adapters. Both paths share
    /// [`crate::dreaming_application::solve_with_standing_requirements`], which
    /// also owns anticipation-cache fallback and evidence links.
    #[must_use]
    pub fn answer_with_memory(
        &self,
        prompt: &str,
        memory_events: &[crate::memory::MemoryEvent],
    ) -> SymbolicAnswer {
        let solver = crate::solver::UniversalSolver::default();
        let answer = crate::dreaming_application::solve_with_standing_requirements(
            &solver,
            prompt,
            &[],
            memory_events,
        );
        crate::dialog_log::record_native_response_if_enabled(
            "FormalAiEngine::answer_with_memory",
            prompt,
            &[],
            &solver.config,
            || serde_json::json!({"memory-links": crate::memory::export_links_notation(memory_events)}),
            &answer,
        );
        answer
    }
}

pub const KNOWLEDGE_SCHEMA_VERSION: &str = "0.2.0";

#[must_use]
pub fn knowledge_links_notation() -> String {
    let mut records = vec![
        format_lino_record(
            "formal_ai_knowledge",
            &[
                ("model", String::from(DEFAULT_MODEL)),
                ("schema_version", String::from(KNOWLEDGE_SCHEMA_VERSION)),
                ("dataset_version", String::from(KNOWLEDGE_SCHEMA_VERSION)),
                (
                    "policy",
                    String::from("deterministic symbolic rules; no neural network inference"),
                ),
                (
                    "format",
                    String::from(
                        "untyped indented Links Notation via lino-objects-codec format helpers",
                    ),
                ),
                ("rule_count", (1 + 7).to_string()),
            ],
        ),
        format_concept_index_record(),
        format_type_system_record(),
        format_doublet_reduction_record(),
        crate::skill_compiler::natural_language_skill_compiler_record(),
    ];
    records.extend(
        crate::associative_package::default_associative_packages()
            .into_iter()
            .map(|package| package.links_notation()),
    );
    records.extend([
        format_lino_record(
            "rule_greeting",
            &[
                ("intent", String::from("greeting")),
                ("response_link", String::from("response:greeting")),
                ("answer", String::from(greeting_answer())),
                ("examples", seed::intent_examples("greeting")),
                ("source", String::from("local symbolic seed set")),
            ],
        ),
        format_lino_record(
            "rule_courtesy_response",
            &[
                ("intent", String::from("courtesy_response")),
                ("response_link", String::from("response:courtesy_response")),
                ("answer", String::from(courtesy_response_answer())),
                ("examples", seed::intent_examples("courtesy_response")),
                ("source", String::from("local symbolic seed set")),
            ],
        ),
        format_lino_record(
            "rule_assistant_free_time",
            &[
                ("intent", String::from("assistant_free_time")),
                (
                    "response_link",
                    String::from("response:assistant_free_time"),
                ),
                ("answer", String::from(assistant_free_time_answer())),
                ("examples", seed::intent_examples("assistant_free_time")),
                ("source", String::from("local symbolic seed set")),
            ],
        ),
        format_lino_record(
            "rule_identity",
            &[
                ("intent", String::from("identity")),
                ("response_link", String::from("response:identity")),
                ("answer", String::from(identity_answer())),
                ("examples", seed::intent_examples("identity")),
                ("source", String::from("local symbolic seed set")),
            ],
        ),
        format_lino_record(
            "rule_assistant_name",
            &[
                ("intent", String::from("assistant_name")),
                ("response_link", String::from("response:assistant_name")),
                ("answer", String::from(assistant_name_answer())),
                ("examples", seed::intent_examples("assistant_name")),
                ("source", String::from("local symbolic seed set")),
            ],
        ),
        format_lino_record(
            "rule_test_status",
            &[
                ("intent", String::from("test_status")),
                ("response_link", String::from("response:test_status")),
                ("answer", String::from(test_status_answer())),
                ("examples", seed::intent_examples("test_status")),
                ("source", String::from("local symbolic seed set")),
            ],
        ),
    ]);

    records.push(format_write_program_rule_record());
    records.push(format_lino_record(
        "rule_unknown",
        &[
            ("intent", String::from("unknown")),
            ("response_link", String::from("response:unknown")),
            ("answer", String::from(unknown_answer())),
            ("examples", seed::intent_examples("unknown")),
            ("source", String::from("fallback symbolic rule")),
        ],
    ));

    records.join("\n\n")
}

/// Concept index: every named intent is declared once here so consumers can
/// reference concepts by id instead of duplicating them in every rule.
/// The literal `intent: <name>` lines are valid Links Notation values and
/// satisfy the uniqueness invariant from `REQUIREMENTS.md`.
fn format_concept_index_record() -> String {
    format_lino_record(
        "concept_index",
        &[
            ("greeting", String::from("intent: greeting")),
            ("test_status", String::from("intent: test_status")),
            (
                "courtesy_response",
                String::from("intent: courtesy_response"),
            ),
            (
                "assistant_free_time",
                String::from("intent: assistant_free_time"),
            ),
            ("identity", String::from("intent: identity")),
            ("assistant_name", String::from("intent: assistant_name")),
            ("write_program", String::from("intent: write_program")),
            ("translation", String::from("intent: translation")),
            ("algorithm", String::from("intent: algorithm")),
            ("meta_explanation", String::from("intent: meta_explanation")),
            ("unknown", String::from("intent: unknown")),
        ],
    )
}

/// Dynamic type system: every value belongs to a Type → `SubType` → Value
/// chain so the network can grow new categories without schema migrations.
fn format_type_system_record() -> String {
    format_lino_record(
        "type_system",
        &[
            ("Type", String::from("Concept")),
            ("SubType_intent", String::from("Concept -> Intent")),
            ("SubType_language", String::from("Concept -> Language")),
            ("SubType_source", String::from("Concept -> Source")),
            ("SubType_program", String::from("Concept -> Program")),
            ("SubType_meaning", String::from("Concept -> Meaning")),
            ("SubType_trace", String::from("Concept -> Trace")),
            (
                "SubType_skill_package",
                String::from("Concept -> CompiledSkillPackage"),
            ),
            (
                "Value_example",
                String::from("Type Concept; SubType Intent; Value greeting"),
            ),
        ],
    )
}

/// Doublet reduction: every record can be projected to {from -> to} pairs.
/// This declaration is what makes the higher-level Links Notation reducible
/// to doublet links per `VISION.md`.
fn format_doublet_reduction_record() -> String {
    format_lino_record(
        "doublet_reduction",
        &[
            ("doublets", String::from("from -> to pairs")),
            ("from", String::from("any node id")),
            ("to", String::from("any node id")),
            (
                "invariant",
                String::from("every record reduces to doublets"),
            ),
        ],
    )
}

#[must_use]
pub fn estimate_tokens(text: &str) -> u32 {
    u32::try_from(text.chars().count()).unwrap_or(u32::MAX)
}

/// A single node in the network-visualization graph.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GraphNode {
    pub id: String,
    pub label: String,
    pub links_notation: String,
}

/// A doublet-link edge between two nodes.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GraphEdge {
    pub from: String,
    pub to: String,
    pub role: String,
}

/// The graph projection of the engine's knowledge dataset, served from
/// `/v1/graph`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct KnowledgeGraph {
    pub nodes: Vec<GraphNode>,
    pub edges: Vec<GraphEdge>,
}

fn known_intent_slugs() -> &'static [String] {
    static CELL: OnceLock<Vec<String>> = OnceLock::new();
    CELL.get_or_init(|| {
        seed::intent_routing()
            .intents
            .iter()
            .filter(|route| !route.slug.is_empty())
            .map(|route| route.slug.clone())
            .collect()
    })
    .as_slice()
}

fn trace_prefixes() -> &'static [String] {
    static CELL: OnceLock<Vec<String>> = OnceLock::new();
    CELL.get_or_init(|| seed::intent_routing().trace_prefixes.clone())
        .as_slice()
}

#[must_use]
pub fn is_known_trace_id(trace: &str) -> bool {
    if trace_prefixes()
        .iter()
        .any(|prefix| trace.starts_with(prefix.as_str()))
    {
        return true;
    }
    known_intent_slugs()
        .iter()
        .any(|slug| trace.eq_ignore_ascii_case(slug) || trace.contains(slug.as_str()))
}

#[must_use]
pub fn knowledge_graph() -> KnowledgeGraph {
    let mut nodes = vec![
        GraphNode {
            id: String::from("formal_ai_knowledge"),
            label: String::from("formal-ai knowledge root"),
            links_notation: String::from("formal_ai_knowledge"),
        },
        GraphNode {
            id: String::from("rule_greeting"),
            label: String::from("Greeting rule"),
            links_notation: format!("rule_greeting answer={}", greeting_answer()),
        },
        GraphNode {
            id: String::from("rule_identity"),
            label: String::from("Identity rule"),
            links_notation: format!("rule_identity answer={}", identity_answer()),
        },
        GraphNode {
            id: String::from("rule_assistant_name"),
            label: String::from("Assistant name rule"),
            links_notation: format!("rule_assistant_name answer={}", assistant_name_answer()),
        },
        GraphNode {
            id: String::from("rule_courtesy_response"),
            label: String::from("Courtesy response rule"),
            links_notation: format!(
                "rule_courtesy_response answer={}",
                courtesy_response_answer()
            ),
        },
        GraphNode {
            id: String::from("rule_assistant_free_time"),
            label: String::from("Assistant free-time rule"),
            links_notation: format!(
                "rule_assistant_free_time answer={}",
                assistant_free_time_answer()
            ),
        },
        GraphNode {
            id: String::from("rule_test_status"),
            label: String::from("Test status rule"),
            links_notation: format!("rule_test_status answer={}", test_status_answer()),
        },
        GraphNode {
            id: String::from("rule_unknown"),
            label: String::from("Unknown fallback rule"),
            links_notation: format!("rule_unknown answer={}", unknown_answer()),
        },
    ];
    let mut edges = vec![
        GraphEdge {
            from: String::from("formal_ai_knowledge"),
            to: String::from("rule_greeting"),
            role: String::from("contains"),
        },
        GraphEdge {
            from: String::from("formal_ai_knowledge"),
            to: String::from("rule_identity"),
            role: String::from("contains"),
        },
        GraphEdge {
            from: String::from("formal_ai_knowledge"),
            to: String::from("rule_assistant_name"),
            role: String::from("contains"),
        },
        GraphEdge {
            from: String::from("formal_ai_knowledge"),
            to: String::from("rule_courtesy_response"),
            role: String::from("contains"),
        },
        GraphEdge {
            from: String::from("formal_ai_knowledge"),
            to: String::from("rule_assistant_free_time"),
            role: String::from("contains"),
        },
        GraphEdge {
            from: String::from("formal_ai_knowledge"),
            to: String::from("rule_test_status"),
            role: String::from("contains"),
        },
        GraphEdge {
            from: String::from("formal_ai_knowledge"),
            to: String::from("rule_unknown"),
            role: String::from("contains"),
        },
        GraphEdge {
            from: String::from("rule_greeting"),
            to: String::from("response:greeting"),
            role: String::from("response_link"),
        },
        GraphEdge {
            from: String::from("rule_identity"),
            to: String::from("response:identity"),
            role: String::from("response_link"),
        },
        GraphEdge {
            from: String::from("rule_assistant_name"),
            to: String::from("response:assistant_name"),
            role: String::from("response_link"),
        },
        GraphEdge {
            from: String::from("rule_courtesy_response"),
            to: String::from("response:courtesy_response"),
            role: String::from("response_link"),
        },
        GraphEdge {
            from: String::from("rule_assistant_free_time"),
            to: String::from("response:assistant_free_time"),
            role: String::from("response_link"),
        },
        GraphEdge {
            from: String::from("rule_test_status"),
            to: String::from("response:test_status"),
            role: String::from("response_link"),
        },
    ];
    nodes.push(GraphNode {
        id: String::from("rule_write_program"),
        label: String::from("Write-program rule"),
        links_notation: format!(
            "rule_write_program parameters=language,task languages={} tasks={}",
            supported_program_languages(),
            supported_program_tasks()
        ),
    });
    edges.push(GraphEdge {
        from: String::from("formal_ai_knowledge"),
        to: String::from("rule_write_program"),
        role: String::from("contains"),
    });
    edges.push(GraphEdge {
        from: String::from("rule_write_program"),
        to: String::from("response:write_program"),
        role: String::from("response_link"),
    });
    let (package_nodes, package_edges) =
        crate::associative_package::default_package_graph_projection();
    nodes.extend(package_nodes);
    edges.extend(package_edges);
    KnowledgeGraph { nodes, edges }
}

#[must_use]
pub fn knowledge_graph_dot() -> String {
    use std::fmt::Write as _;
    let graph = knowledge_graph();
    let mut dot = String::from("digraph formal_ai_knowledge {\n");
    for node in &graph.nodes {
        let _ = writeln!(dot, "  \"{}\" [label=\"{}\"];", node.id, node.label);
    }
    for edge in &graph.edges {
        let _ = writeln!(
            dot,
            "  \"{}\" -> \"{}\" [label=\"{}\"];",
            edge.from, edge.to, edge.role
        );
    }
    dot.push_str("}\n");
    dot
}

#[must_use]
pub fn stable_id(prefix: &str, text: &str) -> String {
    crate::web_engine_core::stable_id(prefix, text)
}

pub(crate) enum SelectedRule {
    Greeting,
    Wellbeing,
    Farewell,
    TestStatus,
    CourtesyResponse,
    AssistantFreeTime,
    Identity,
    AssistantName,
    WriteProgram(ProgramSpec<'static>),
    UnsupportedWriteProgram {
        task: Option<String>,
        language: Option<String>,
    },
    Unknown,
}

impl SelectedRule {
    pub(crate) fn intent(&self) -> String {
        match self {
            Self::Greeting => String::from("greeting"),
            Self::Wellbeing => String::from("wellbeing"),
            Self::Farewell => String::from("farewell"),
            Self::TestStatus => String::from("test_status"),
            Self::CourtesyResponse => String::from("courtesy_response"),
            Self::AssistantFreeTime => String::from("assistant_free_time"),
            Self::Identity => String::from("identity"),
            Self::AssistantName => String::from("assistant_name"),
            Self::WriteProgram(_) => String::from(WRITE_PROGRAM_INTENT),
            // Issue #906: an unfilled parameter is not a skill gap. Which dead
            // end this is decides both the intent and the wording.
            Self::UnsupportedWriteProgram { task, language } => String::from(
                crate::program_skill_gap::shape(task.as_deref(), language.as_deref()).intent(),
            ),
            Self::Unknown => String::from("unknown"),
        }
    }

    pub(crate) fn response_link(&self) -> String {
        match self {
            Self::Greeting => String::from("response:greeting"),
            Self::Wellbeing => String::from("response:wellbeing"),
            Self::Farewell => String::from("response:farewell"),
            Self::TestStatus => String::from("response:test_status"),
            Self::CourtesyResponse => String::from("response:courtesy_response"),
            Self::AssistantFreeTime => String::from("response:assistant_free_time"),
            Self::Identity => String::from("response:identity"),
            Self::AssistantName => String::from("response:assistant_name"),
            Self::WriteProgram(spec) => spec.response_link(),
            Self::UnsupportedWriteProgram { task, language } => String::from(
                crate::program_skill_gap::shape(task.as_deref(), language.as_deref())
                    .response_link(),
            ),
            Self::Unknown => String::from("response:unknown"),
        }
    }

    pub(crate) fn answer(&self) -> String {
        match self {
            Self::Greeting => String::from(greeting_answer()),
            Self::Wellbeing => String::from(wellbeing_answer()),
            Self::Farewell => String::from(farewell_answer()),
            Self::TestStatus => String::from(test_status_answer()),
            Self::CourtesyResponse => String::from(courtesy_response_answer()),
            Self::AssistantFreeTime => String::from(assistant_free_time_answer()),
            Self::Identity => String::from(identity_answer()),
            Self::AssistantName => String::from(assistant_name_answer()),
            Self::WriteProgram(spec) => {
                crate::coding::catalog_parameters::catalog_parameter_guidance(spec.task.slug)
                    .map_or_else(
                        || write_program_answer(*spec, Language::English, false),
                        |key| crate::coding::guidance::guidance(key, Language::English, &[]),
                    )
            }
            Self::UnsupportedWriteProgram { task, language } => crate::program_skill_gap::render(
                task.as_deref(),
                language.as_deref(),
                Language::English,
            ),
            Self::Unknown => String::from(unknown_answer()),
        }
    }
}

pub(crate) fn language_aware_intent_for(rule: &SelectedRule, _language: Language) -> String {
    rule.intent()
}

pub(crate) fn language_aware_answer_for(
    rule: &SelectedRule,
    language: Language,
    prompt: &str,
    prior_code_response: bool,
    catalog_program: Option<&crate::coding::catalog_parameters::BoundCatalogProgram>,
) -> String {
    match (rule, language) {
        (SelectedRule::AssistantFreeTime, _) => {
            seed::response_variant_for("assistant_free_time", language.slug(), prompt)
                .unwrap_or_else(|| assistant_free_time_answer().to_owned())
        }
        (SelectedRule::Greeting, _) => seed::localized_response("greeting", language.slug())
            .unwrap_or_else(|| greeting_answer().to_owned()),
        (SelectedRule::Wellbeing, Language::Russian) => String::from(russian_wellbeing_answer()),
        (SelectedRule::Wellbeing, Language::Hindi) => String::from(hindi_wellbeing_answer()),
        (SelectedRule::Wellbeing, Language::Chinese) => String::from(chinese_wellbeing_answer()),
        (SelectedRule::Farewell, Language::Russian) => String::from(russian_farewell_answer()),
        (SelectedRule::Farewell, Language::Hindi) => String::from(hindi_farewell_answer()),
        (SelectedRule::Farewell, Language::Chinese) => String::from(chinese_farewell_answer()),
        (SelectedRule::TestStatus, Language::Russian) => String::from(russian_test_status_answer()),
        (SelectedRule::TestStatus, Language::Hindi) => String::from(hindi_test_status_answer()),
        (SelectedRule::TestStatus, Language::Chinese) => String::from(chinese_test_status_answer()),
        (SelectedRule::CourtesyResponse, Language::Russian) => {
            String::from(russian_courtesy_response_answer())
        }
        (SelectedRule::CourtesyResponse, Language::Hindi) => {
            String::from(hindi_courtesy_response_answer())
        }
        (SelectedRule::CourtesyResponse, Language::Chinese) => {
            String::from(chinese_courtesy_response_answer())
        }
        (SelectedRule::Identity, Language::Russian) => String::from(russian_identity_answer()),
        (SelectedRule::Identity, Language::Hindi) => String::from(hindi_identity_answer()),
        (SelectedRule::Identity, Language::Chinese) => String::from(chinese_identity_answer()),
        (SelectedRule::AssistantName, Language::Russian) => {
            String::from(russian_assistant_name_answer())
        }
        (SelectedRule::AssistantName, Language::Hindi) => {
            String::from(hindi_assistant_name_answer())
        }
        (SelectedRule::AssistantName, Language::Chinese) => {
            String::from(chinese_assistant_name_answer())
        }
        (SelectedRule::WriteProgram(spec), language) => {
            if let Some(key) =
                crate::coding::catalog_parameters::catalog_parameter_guidance(spec.task.slug)
            {
                return catalog_program.map_or_else(
                    || crate::coding::guidance::guidance(key, language, &[]),
                    |bound| {
                        write_program_answer_with_output(
                            ProgramSpec {
                                template: &bound.template,
                                ..*spec
                            },
                            language,
                            prior_code_response,
                            &bound.expected_output,
                            None,
                        )
                    },
                );
            }
            if spec.task.slug == "hello_world"
                && let Some(output) = crate::code_editing::inline_hello_world_replacement(prompt)
            {
                let mut template = spec.template.clone();
                template.code = crate::code_editing::apply_inline_hello_world_source_replacement(
                    prompt,
                    &spec.template.code,
                    *spec,
                )
                .into();
                write_program_answer_with_output(
                    ProgramSpec {
                        template: &template,
                        ..*spec
                    },
                    language,
                    prior_code_response,
                    &output,
                    None,
                )
            } else {
                write_program_answer(*spec, language, prior_code_response)
            }
        }
        (
            SelectedRule::UnsupportedWriteProgram {
                task,
                language: program_language,
            },
            language,
        ) => {
            crate::program_skill_gap::render(task.as_deref(), program_language.as_deref(), language)
        }
        (SelectedRule::Unknown, _) => {
            crate::unknown_opener::language_aware_unknown_answer(prompt, language)
        }
        _ => rule.answer(),
    }
}

pub(crate) fn response_link_for_intent(rule: &SelectedRule, _intent: &str) -> String {
    rule.response_link()
}

pub(crate) fn normalize_prompt(prompt: &str) -> String {
    crate::agentic_coding::normalization_cache::normalized(prompt, normalize_prompt_uncached)
}

fn normalize_prompt_uncached(prompt: &str) -> String {
    let canonical: String = prompt
        .chars()
        .flat_map(char::to_lowercase)
        .collect::<String>()
        .replace("c++", " cpp ")
        .replace("c#", " csharp ");

    let mut normalized = String::with_capacity(canonical.len());
    for character in canonical.chars() {
        if character.is_alphanumeric() || is_script_combining_mark(character) {
            normalized.push(character);
        } else {
            normalized.push(' ');
        }
    }

    expand_contractions(&normalized.split_whitespace().collect::<Vec<_>>().join(" "))
}

/// The seeded contracted token pairs, each with the words it stands for.
///
/// Normalization turns an apostrophe into a space, so "you're" reaches every
/// matcher as the pair `you re` and never equals a surface the seed writes out
/// ("you are"), issue #1175 p020. The pairs are the `contraction` /
/// `expansion` records of `data/seed/languages.lino`.
fn contractions() -> &'static [(Vec<String>, String)] {
    static CELL: OnceLock<Vec<(Vec<String>, String)>> = OnceLock::new();
    CELL.get_or_init(|| {
        let ledger = seed::seed_files()
            .into_iter()
            .find(|(path, _)| *path == "data/seed/languages.lino")
            .map_or("", |(_, text)| text);
        let mut pairs = Vec::new();
        let mut contracted: Option<Vec<String>> = None;
        for line in ledger.lines() {
            let Some((key, value)) = line.trim().split_once(' ') else {
                continue;
            };
            let value = value.trim().trim_matches('"');
            match key {
                "contraction" => {
                    contracted = Some(value.split_whitespace().map(ToOwned::to_owned).collect());
                }
                "expansion" => {
                    if let Some(tokens) = contracted.take().filter(|tokens| !tokens.is_empty()) {
                        pairs.push((tokens, value.to_owned()));
                    }
                }
                _ => {}
            }
        }
        pairs
    })
}

/// `normalized` with every seeded contracted pair rewritten into its expansion.
///
/// Mirrors `expandSeededContractions` in `js/worker/formal_ai_worker_seed_responses_and_language.js`.
fn expand_contractions(normalized: &str) -> String {
    let pairs = contractions();
    let tokens: Vec<&str> = normalized.split(' ').collect();
    let mut out: Vec<&str> = Vec::with_capacity(tokens.len());
    let mut index = 0;
    while index < tokens.len() {
        let pair = pairs.iter().find(|(contracted, _)| {
            tokens
                .get(index..index + contracted.len())
                .is_some_and(|window| window.iter().zip(contracted).all(|(a, b)| *a == b.as_str()))
        });
        let (word, step) = pair.map_or((tokens[index], 1), |(contracted, expansion)| {
            (expansion.as_str(), contracted.len())
        });
        out.push(word);
        index += step;
    }
    out.join(" ")
}

const fn is_script_combining_mark(character: char) -> bool {
    let codepoint = character as u32;
    matches!(
        codepoint,
        0x0300..=0x036F
            | 0x0900..=0x094F
            | 0x0951..=0x0957
            | 0x0962..=0x0963
            | 0x0980..=0x09FF
            | 0x0A00..=0x0A7F
            | 0x0A80..=0x0AFF
            | 0x0B00..=0x0B7F
    )
}

pub(crate) fn answer_links_notation(
    prompt: &str,
    intent: &str,
    answer: &str,
    log: &EventLog,
    trace_id: &str,
) -> String {
    let steps = log
        .events()
        .iter()
        .enumerate()
        .map(|(index, event)| {
            format!(
                "step_{index} {} {}",
                event.kind,
                flatten_lino_value(&event.payload)
            )
        })
        .collect::<Vec<_>>()
        .join("; ");
    let thinking_steps = log
        .thinking_steps()
        .iter()
        .map(|step| {
            format!(
                "step_{} {} {} {} {}",
                step.order,
                flatten_lino_value(&step.step),
                flatten_lino_value(&step.level),
                flatten_lino_value(&step.source_event),
                flatten_lino_value(&step.detail)
            )
        })
        .collect::<Vec<_>>()
        .join("; ");
    format_lino_record(
        &format!("answer_{}", stable_id("prompt", prompt)),
        &[
            ("prompt", String::from(prompt)),
            ("intent", String::from(intent)),
            ("answer", String::from(answer)),
            ("trace", String::from(trace_id)),
            ("steps", steps),
            ("thinking_steps", thinking_steps),
        ],
    )
}

fn format_write_program_rule_record() -> String {
    let sample = program_spec("hello_world", "rust").map_or_else(
        || String::from("Write-program template catalog is unavailable."),
        |spec| write_program_answer(spec, Language::English, false),
    );
    format_lino_record(
        "rule_write_program",
        &[
            ("intent", String::from(WRITE_PROGRAM_INTENT)),
            ("parameters", String::from("language, task")),
            ("languages", supported_program_languages()),
            ("tasks", supported_program_tasks()),
            ("template_count", program_template_count().to_string()),
            ("response_link", String::from("response:write_program")),
            ("answer", sample),
            ("examples", seed::intent_examples("write_program")),
            ("source", program_template_sources()),
        ],
    )
}

fn program_template_sources() -> String {
    let mut sources = Vec::new();
    for language in PROGRAM_LANGUAGES {
        if !sources.contains(&language.source) {
            sources.push(language.source);
        }
    }
    sources.join(", ")
}
