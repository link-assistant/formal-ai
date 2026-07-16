//! Deterministic symbolic engine and Links-Notation knowledge dataset.
//!
//! This module hosts the deterministic answer projection primitives used by the
//! universal solver. Callers should not invoke `FormalAiEngine::answer` to
//! bypass the solver — it delegates to [`crate::solver::UniversalSolver::solve`]
//! so every request walks the same 11-step loop documented in `VISION.md`.

pub(crate) use crate::coding::{
    program_language_by_alias, program_spec, program_template_count, supported_program_languages,
    supported_program_tasks, ExecutionStatus, ProgramExecution, ProgramSpec, PROGRAM_LANGUAGES,
    WRITE_PROGRAM_INTENT,
};

use std::sync::OnceLock;

use serde::{Deserialize, Serialize};

use crate::coding::guidance::{program_explanation_section, program_test_instructions};
use crate::engine_assistant_name::{
    assistant_name_answer, chinese_assistant_name_answer, hindi_assistant_name_answer,
    russian_assistant_name_answer, ASSISTANT_NAME_EXAMPLES,
};
pub(crate) use crate::engine_responses::{
    assistant_free_time_answer, chinese_unknown_answer, farewell_answer, greeting_answer,
    hindi_unknown_answer, identity_answer, russian_unknown_answer, unknown_answer,
    unknown_language_fallback_answer,
};
use crate::engine_responses::{
    chinese_assistant_free_time_answer, chinese_courtesy_response_answer, chinese_farewell_answer,
    chinese_greeting_answer, chinese_identity_answer, chinese_test_status_answer,
    courtesy_response_answer, hindi_assistant_free_time_answer, hindi_courtesy_response_answer,
    hindi_farewell_answer, hindi_greeting_answer, hindi_identity_answer, hindi_test_status_answer,
    russian_assistant_free_time_answer, russian_courtesy_response_answer, russian_farewell_answer,
    russian_greeting_answer, russian_identity_answer, russian_test_status_answer,
    test_status_answer, ASSISTANT_FREE_TIME_EXAMPLES, COURTESY_RESPONSE_EXAMPLES,
    GREETING_EXAMPLES, IDENTITY_EXAMPLES, TEST_STATUS_EXAMPLES, UNKNOWN_EXAMPLES,
};
use crate::event_log::EventLog;
use crate::language::Language;
use crate::links_format::{flatten_lino_value, format_lino_record};
use crate::seed;

pub const DEFAULT_MODEL: &str = "formal-ai";

// Thinking model + deterministic naturalizer live in `crate::thinking` (issue #488),
// re-exported so `crate::engine::{...}` / `formal_ai::{...}` paths stay unchanged.
pub use crate::thinking::{
    humanize_meta_identifier, naturalize_thinking_step, thinking_language_label, ThinkingStep,
};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SymbolicAnswer {
    pub intent: String,
    pub answer: String,
    pub confidence: f32,
    pub evidence_links: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub thinking_steps: Vec<ThinkingStep>,
    pub links_notation: String,
}

#[derive(Debug, Default, Clone, Copy)]
pub struct FormalAiEngine;

impl FormalAiEngine {
    /// Answer a prompt by running it through the universal solver loop.
    #[must_use]
    pub fn answer(&self, prompt: &str) -> SymbolicAnswer {
        crate::solver::UniversalSolver::default().solve(prompt)
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
                ("examples", GREETING_EXAMPLES.join(", ")),
                ("source", String::from("local symbolic seed set")),
            ],
        ),
        format_lino_record(
            "rule_courtesy_response",
            &[
                ("intent", String::from("courtesy_response")),
                ("response_link", String::from("response:courtesy_response")),
                ("answer", String::from(courtesy_response_answer())),
                ("examples", COURTESY_RESPONSE_EXAMPLES.join(", ")),
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
                ("examples", ASSISTANT_FREE_TIME_EXAMPLES.join(", ")),
                ("source", String::from("local symbolic seed set")),
            ],
        ),
        format_lino_record(
            "rule_identity",
            &[
                ("intent", String::from("identity")),
                ("response_link", String::from("response:identity")),
                ("answer", String::from(identity_answer())),
                ("examples", IDENTITY_EXAMPLES.join(", ")),
                ("source", String::from("local symbolic seed set")),
            ],
        ),
        format_lino_record(
            "rule_assistant_name",
            &[
                ("intent", String::from("assistant_name")),
                ("response_link", String::from("response:assistant_name")),
                ("answer", String::from(assistant_name_answer())),
                ("examples", ASSISTANT_NAME_EXAMPLES.join(", ")),
                ("source", String::from("local symbolic seed set")),
            ],
        ),
        format_lino_record(
            "rule_test_status",
            &[
                ("intent", String::from("test_status")),
                ("response_link", String::from("response:test_status")),
                ("answer", String::from(test_status_answer())),
                ("examples", TEST_STATUS_EXAMPLES.join(", ")),
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
            ("examples", UNKNOWN_EXAMPLES.join(", ")),
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
    u32::try_from(text.split_whitespace().count()).unwrap_or(u32::MAX)
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
            .into_iter()
            .filter_map(|route| {
                if route.slug.is_empty() {
                    None
                } else {
                    Some(route.slug)
                }
            })
            .collect()
    })
    .as_slice()
}

fn trace_prefixes() -> &'static [String] {
    static CELL: OnceLock<Vec<String>> = OnceLock::new();
    CELL.get_or_init(|| seed::intent_routing().trace_prefixes)
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
    Farewell,
    TestStatus,
    CourtesyResponse,
    AssistantFreeTime,
    Identity,
    AssistantName,
    WriteProgram(ProgramSpec),
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
            Self::Farewell => String::from("farewell"),
            Self::TestStatus => String::from("test_status"),
            Self::CourtesyResponse => String::from("courtesy_response"),
            Self::AssistantFreeTime => String::from("assistant_free_time"),
            Self::Identity => String::from("identity"),
            Self::AssistantName => String::from("assistant_name"),
            Self::WriteProgram(_) => String::from(WRITE_PROGRAM_INTENT),
            Self::UnsupportedWriteProgram { .. } => String::from("write_program_unsupported"),
            Self::Unknown => String::from("unknown"),
        }
    }

    pub(crate) fn response_link(&self) -> String {
        match self {
            Self::Greeting => String::from("response:greeting"),
            Self::Farewell => String::from("response:farewell"),
            Self::TestStatus => String::from("response:test_status"),
            Self::CourtesyResponse => String::from("response:courtesy_response"),
            Self::AssistantFreeTime => String::from("response:assistant_free_time"),
            Self::Identity => String::from("response:identity"),
            Self::AssistantName => String::from("response:assistant_name"),
            Self::WriteProgram(spec) => spec.response_link(),
            Self::UnsupportedWriteProgram { .. } => {
                String::from("response:write_program:unsupported")
            }
            Self::Unknown => String::from("response:unknown"),
        }
    }

    pub(crate) fn answer(&self) -> String {
        match self {
            Self::Greeting => String::from(greeting_answer()),
            Self::Farewell => String::from(farewell_answer()),
            Self::TestStatus => String::from(test_status_answer()),
            Self::CourtesyResponse => String::from(courtesy_response_answer()),
            Self::AssistantFreeTime => String::from(assistant_free_time_answer()),
            Self::Identity => String::from(identity_answer()),
            Self::AssistantName => String::from(assistant_name_answer()),
            Self::WriteProgram(spec) => write_program_answer(*spec, Language::English, false),
            Self::UnsupportedWriteProgram { task, language } => unsupported_write_program_answer(
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
) -> String {
    match (rule, language) {
        (SelectedRule::Greeting, Language::Russian) => String::from(russian_greeting_answer()),
        (SelectedRule::Greeting, Language::Hindi) => String::from(hindi_greeting_answer()),
        (SelectedRule::Greeting, Language::Chinese) => String::from(chinese_greeting_answer()),
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
        (SelectedRule::AssistantFreeTime, Language::Russian) => {
            String::from(russian_assistant_free_time_answer())
        }
        (SelectedRule::AssistantFreeTime, Language::Hindi) => {
            String::from(hindi_assistant_free_time_answer())
        }
        (SelectedRule::AssistantFreeTime, Language::Chinese) => {
            String::from(chinese_assistant_free_time_answer())
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
            let answer = write_program_answer(*spec, language, prior_code_response);
            crate::code_editing::apply_inline_hello_world_output_replacement(prompt, &answer, *spec)
                .unwrap_or(answer)
        }
        (
            SelectedRule::UnsupportedWriteProgram {
                task,
                language: program_language,
            },
            language,
        ) => {
            unsupported_write_program_answer(task.as_deref(), program_language.as_deref(), language)
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

/// Match a default hello-world program from the catalog by language alias.
pub(crate) fn hello_world_program_by_alias(normalized: &str) -> Option<ProgramSpec> {
    let language = program_language_by_alias(normalized)?;
    program_spec("hello_world", language.slug)
}

pub(crate) fn normalize_prompt(prompt: &str) -> String {
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

    normalized.split_whitespace().collect::<Vec<_>>().join(" ")
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
            (
                "examples",
                String::from(
                    "Write me hello world program in Rust; Write a Python program that counts to three",
                ),
            ),
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

fn write_program_answer(
    spec: ProgramSpec,
    language: Language,
    prior_code_response: bool,
) -> String {
    // Issue #324: the natural-language framing around the generated program is
    // localized to the detected (or preferred) response language so a Russian,
    // Hindi, or Chinese request no longer receives an all-English reply. The
    // code itself and the literal shell commands stay in their canonical form
    // because they are the requested artefact, not prose.
    //
    // Issue #330: a code answer must teach a novice — so the program is always
    // accompanied by a plain-language explanation of *how it works* and
    // step-by-step instructions for testing it. When the dialog already walked
    // the user through running code (`prior_code_response`), the verbose setup
    // steps are omitted and replaced by a short "test it the same way" note so
    // follow-up edits stay concise.
    let expected_output = spec.expected_output();
    format!(
        "{}\n\n```{}\n{}\n```\n\n{}\n\n{}\n\n{}",
        write_program_intro(spec.language.name, spec.task.label, language),
        spec.language.code_fence,
        spec.template.code,
        execution_report(&spec.language.execution, &expected_output, language),
        program_explanation_section(spec, language),
        program_test_instructions(spec, language, prior_code_response),
    )
}

fn write_program_intro(language_name: &str, task_label: &str, language: Language) -> String {
    match language {
        Language::Russian => {
            format!("Вот минимальная программа на языке {language_name} ({task_label}):")
        }
        Language::Hindi => {
            format!("यहाँ {language_name} में एक न्यूनतम प्रोग्राम है ({task_label}):")
        }
        Language::Chinese => format!("这是一个最小的 {language_name} 程序（{task_label}）："),
        _ => format!("Here is a minimal {language_name} {task_label} program:"),
    }
}

fn unsupported_write_program_answer(
    task: Option<&str>,
    language: Option<&str>,
    response_language: Language,
) -> String {
    let task = task.unwrap_or("missing");
    let language = language.unwrap_or("missing");
    let languages = supported_program_languages();
    let tasks = supported_program_tasks();
    match response_language {
        Language::Russian => format!(
            "Я могу выполнить `write_program(language, task)`, но у меня нет шаблона для \
             языка `{language}` и задачи `{task}`. Поддерживаемые языки: {languages}. \
             Поддерживаемые задачи: {tasks}."
        ),
        Language::Hindi => format!(
            "मैं `write_program(language, task)` रूट कर सकता हूँ, लेकिन भाषा `{language}` और \
             कार्य `{task}` के लिए मेरे पास कोई टेम्पलेट नहीं है। समर्थित भाषाएँ: {languages}. \
             समर्थित कार्य: {tasks}."
        ),
        Language::Chinese => format!(
            "我可以路由 `write_program(language, task)`，但我没有语言 `{language}` 和任务 \
             `{task}` 的模板。支持的语言：{languages}。支持的任务：{tasks}。"
        ),
        _ => format!(
            "I can route `write_program(language, task)`, but I do not have a template for \
             language `{language}` and task `{task}`. Supported languages: {languages}. \
             Supported tasks: {tasks}."
        ),
    }
}

fn execution_report(execution: &ProgramExecution, output: &str, language: Language) -> String {
    let command_lines = execution_command_lines(execution);
    let verified = matches!(execution.status, ExecutionStatus::Verified);
    let status_phrase = execution_status_phrase(execution.status, language);
    let output_label = execution_output_label(verified, language);
    let status_line = match language {
        Language::Russian => format!(
            "Статус выполнения: {status_phrase} в среде «{}».",
            execution.environment
        ),
        Language::Hindi => format!(
            "निष्पादन स्थिति: {status_phrase} ({} में)।",
            execution.environment
        ),
        Language::Chinese => {
            format!("执行状态：{status_phrase}（{}）。", execution.environment)
        }
        _ => format!(
            "Execution status: {status_phrase} in {}.",
            execution.environment
        ),
    };

    format!(
        "{status_line}\n{command_lines}\n{output_label}:\n```text\n{output}\n```\n{}",
        execution.notes
    )
}

const fn execution_status_phrase(status: ExecutionStatus, language: Language) -> &'static str {
    match (status, language) {
        (ExecutionStatus::Verified, Language::Russian) => "скомпилировано и запущено",
        (ExecutionStatus::Verified, Language::Hindi) => "संकलित और चलाया गया",
        (ExecutionStatus::Verified, Language::Chinese) => "已编译并运行",
        (ExecutionStatus::Unavailable, Language::Russian) => "не скомпилировано и не запущено",
        (ExecutionStatus::Unavailable, Language::Hindi) => "संकलित या चलाया नहीं गया",
        (ExecutionStatus::Unavailable, Language::Chinese) => "未编译或运行",
        (status, _) => status.label(),
    }
}

const fn execution_output_label(verified: bool, language: Language) -> &'static str {
    match (verified, language) {
        (true, Language::Russian) => "Вывод",
        (false, Language::Russian) => "Ожидаемый вывод после проверки",
        (true, Language::Hindi) => "आउटपुट",
        (false, Language::Hindi) => "सत्यापन के बाद अपेक्षित आउटपुट",
        (true, Language::Chinese) => "输出",
        (false, Language::Chinese) => "验证后的预期输出",
        (true, _) => "Output",
        (false, _) => "Expected output after verification",
    }
}

fn execution_command_lines(execution: &ProgramExecution) -> String {
    execution.check_command.map_or_else(
        || format!("Run command: `{}`", execution.run_command),
        |check_command| {
            format!(
                "Check command: `{check_command}`\nRun command: `{}`",
                execution.run_command
            )
        },
    )
}
