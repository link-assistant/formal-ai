//! Deterministic symbolic engine and Links-Notation knowledge dataset.
//!
//! This module hosts the rule-matching primitives (`select_rule_for`,
//! `intent_for_rule`, `language_aware_answer_for`, …) used by the universal
//! solver. Callers should not invoke `FormalAiEngine::answer` to bypass the
//! solver — it delegates to [`crate::solver::UniversalSolver::solve`] so every
//! request walks the same 11-step loop documented in `VISION.md`.

pub(crate) use crate::engine_hello_world::{
    ExecutionStatus, HelloWorldProgram, ProgramExecution, HELLO_WORLD_PROGRAMS,
};

use std::sync::OnceLock;

use lino_objects_codec::format::format_indented_ordered;
use serde::{Deserialize, Serialize};

use crate::event_log::EventLog;
use crate::language::Language;
use crate::seed;

pub const DEFAULT_MODEL: &str = "formal-symbolic-production";

/// Hardcoded English fallbacks used only when `data/seed/multilingual-responses.lino`
/// cannot be parsed (which would be a build-time bug since the file is
/// embedded via `include_str!`). All real reads come from [`crate::seed`].
const FALLBACK_GREETING_ANSWER: &str = "Hi, how may I help you?";
const FALLBACK_FAREWELL_ANSWER: &str = "Goodbye! Feel free to return any time.";
const FALLBACK_COURTESY_RESPONSE_ANSWER: &str = "Glad to hear it. What would you like to do next?";
const FALLBACK_IDENTITY_ANSWER: &str = "I am formal-ai, a deterministic symbolic AI implementation that answers from local Links Notation rules and OpenAI-compatible API shapes. I do not perform neural inference in this demo.";
const FALLBACK_UNKNOWN_ANSWER: &str = "I cannot answer that from local Links Notation rules yet. Please add a fact or add a rule in Links Notation, then run the request again.";
const FALLBACK_UNKNOWN_LANGUAGE_ANSWER: &str = concat!(
    "I detected an unsupported language. Falling back to English: I cannot ",
    "answer that from local Links Notation rules yet. Please add a fact or ",
    "add a rule in Links Notation, then run the request again."
);

fn cached_response(
    cell: &'static OnceLock<String>,
    intent: &str,
    language: &str,
    fallback: &str,
) -> &'static str {
    cell.get_or_init(|| {
        seed::response_for(intent, language).unwrap_or_else(|| String::from(fallback))
    })
    .as_str()
}

pub(crate) fn greeting_answer() -> &'static str {
    static CELL: OnceLock<String> = OnceLock::new();
    cached_response(&CELL, "greeting", "en", FALLBACK_GREETING_ANSWER)
}

pub(crate) fn farewell_answer() -> &'static str {
    static CELL: OnceLock<String> = OnceLock::new();
    cached_response(&CELL, "farewell", "en", FALLBACK_FAREWELL_ANSWER)
}

pub(crate) fn courtesy_response_answer() -> &'static str {
    static CELL: OnceLock<String> = OnceLock::new();
    cached_response(
        &CELL,
        "courtesy_response",
        "en",
        FALLBACK_COURTESY_RESPONSE_ANSWER,
    )
}

fn russian_farewell_answer() -> &'static str {
    static CELL: OnceLock<String> = OnceLock::new();
    cached_response(&CELL, "farewell", "ru", FALLBACK_FAREWELL_ANSWER)
}

fn hindi_farewell_answer() -> &'static str {
    static CELL: OnceLock<String> = OnceLock::new();
    cached_response(&CELL, "farewell", "hi", FALLBACK_FAREWELL_ANSWER)
}

fn chinese_farewell_answer() -> &'static str {
    static CELL: OnceLock<String> = OnceLock::new();
    cached_response(&CELL, "farewell", "zh", FALLBACK_FAREWELL_ANSWER)
}

fn russian_courtesy_response_answer() -> &'static str {
    static CELL: OnceLock<String> = OnceLock::new();
    cached_response(
        &CELL,
        "courtesy_response",
        "ru",
        FALLBACK_COURTESY_RESPONSE_ANSWER,
    )
}

fn hindi_courtesy_response_answer() -> &'static str {
    static CELL: OnceLock<String> = OnceLock::new();
    cached_response(
        &CELL,
        "courtesy_response",
        "hi",
        FALLBACK_COURTESY_RESPONSE_ANSWER,
    )
}

fn chinese_courtesy_response_answer() -> &'static str {
    static CELL: OnceLock<String> = OnceLock::new();
    cached_response(
        &CELL,
        "courtesy_response",
        "zh",
        FALLBACK_COURTESY_RESPONSE_ANSWER,
    )
}

pub(crate) fn identity_answer() -> &'static str {
    static CELL: OnceLock<String> = OnceLock::new();
    cached_response(&CELL, "identity", "en", FALLBACK_IDENTITY_ANSWER)
}

pub(crate) fn unknown_answer() -> &'static str {
    static CELL: OnceLock<String> = OnceLock::new();
    cached_response(&CELL, "unknown", "en", FALLBACK_UNKNOWN_ANSWER)
}

fn russian_greeting_answer() -> &'static str {
    static CELL: OnceLock<String> = OnceLock::new();
    cached_response(&CELL, "greeting", "ru", FALLBACK_GREETING_ANSWER)
}

fn hindi_greeting_answer() -> &'static str {
    static CELL: OnceLock<String> = OnceLock::new();
    cached_response(&CELL, "greeting", "hi", FALLBACK_GREETING_ANSWER)
}

fn chinese_greeting_answer() -> &'static str {
    static CELL: OnceLock<String> = OnceLock::new();
    cached_response(&CELL, "greeting", "zh", FALLBACK_GREETING_ANSWER)
}

fn russian_identity_answer() -> &'static str {
    static CELL: OnceLock<String> = OnceLock::new();
    cached_response(&CELL, "identity", "ru", FALLBACK_IDENTITY_ANSWER)
}

fn hindi_identity_answer() -> &'static str {
    static CELL: OnceLock<String> = OnceLock::new();
    cached_response(&CELL, "identity", "hi", FALLBACK_IDENTITY_ANSWER)
}

fn chinese_identity_answer() -> &'static str {
    static CELL: OnceLock<String> = OnceLock::new();
    cached_response(&CELL, "identity", "zh", FALLBACK_IDENTITY_ANSWER)
}

fn russian_unknown_answer() -> &'static str {
    static CELL: OnceLock<String> = OnceLock::new();
    cached_response(&CELL, "unknown", "ru", FALLBACK_UNKNOWN_ANSWER)
}

fn hindi_unknown_answer() -> &'static str {
    static CELL: OnceLock<String> = OnceLock::new();
    cached_response(&CELL, "unknown", "hi", FALLBACK_UNKNOWN_ANSWER)
}

fn chinese_unknown_answer() -> &'static str {
    static CELL: OnceLock<String> = OnceLock::new();
    cached_response(&CELL, "unknown", "zh", FALLBACK_UNKNOWN_ANSWER)
}

const fn unknown_language_fallback_answer() -> &'static str {
    FALLBACK_UNKNOWN_LANGUAGE_ANSWER
}

const GREETING_EXAMPLES: &[&str] = &["Hi", "Hello", "Hey"];
const COURTESY_RESPONSE_EXAMPLES: &[&str] = &["I am fine, thank you", "thanks"];
const IDENTITY_EXAMPLES: &[&str] = &[
    "Who are you?",
    "What are you?",
    "Tell me about yourself",
    "What is formal-ai?",
];
const UNKNOWN_EXAMPLES: &[&str] = &["Any prompt without a matching symbolic rule"];

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SymbolicAnswer {
    pub intent: String,
    pub answer: String,
    pub confidence: f32,
    pub evidence_links: Vec<String>,
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
                ("rule_count", (HELLO_WORLD_PROGRAMS.len() + 4).to_string()),
            ],
        ),
        format_concept_index_record(),
        format_type_system_record(),
        format_doublet_reduction_record(),
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
            "rule_identity",
            &[
                ("intent", String::from("identity")),
                ("response_link", String::from("response:identity")),
                ("answer", String::from(identity_answer())),
                ("examples", IDENTITY_EXAMPLES.join(", ")),
                ("source", String::from("local symbolic seed set")),
            ],
        ),
    ];

    records.extend(
        HELLO_WORLD_PROGRAMS
            .iter()
            .map(format_hello_world_rule_record),
    );
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
            (
                "courtesy_response",
                String::from("intent: courtesy_response"),
            ),
            ("identity", String::from("intent: identity")),
            ("hello_world", String::from("intent: hello_world")),
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
            id: String::from("rule_courtesy_response"),
            label: String::from("Courtesy response rule"),
            links_notation: format!(
                "rule_courtesy_response answer={}",
                courtesy_response_answer()
            ),
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
            to: String::from("rule_courtesy_response"),
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
            from: String::from("rule_courtesy_response"),
            to: String::from("response:courtesy_response"),
            role: String::from("response_link"),
        },
    ];
    for program in HELLO_WORLD_PROGRAMS {
        let rule_id = format!("rule_hello_world_{}", program.slug);
        nodes.push(GraphNode {
            id: rule_id.clone(),
            label: format!("Hello-world rule ({})", program.language),
            links_notation: format!("{rule_id} language={}", program.language),
        });
        edges.push(GraphEdge {
            from: String::from("formal_ai_knowledge"),
            to: rule_id.clone(),
            role: String::from("contains"),
        });
        edges.push(GraphEdge {
            from: rule_id,
            to: String::from(program.response_link),
            role: String::from("response_link"),
        });
    }
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
    let mut hash = 0xcbf2_9ce4_8422_2325_u64;
    for byte in text.as_bytes() {
        hash ^= u64::from(*byte);
        hash = hash.wrapping_mul(0x0000_0100_0000_01b3);
    }

    format!("{prefix}_{hash:016x}")
}

pub(crate) enum SelectedRule {
    Greeting,
    Farewell,
    CourtesyResponse,
    Identity,
    HelloWorld(&'static HelloWorldProgram),
    Unknown,
}

impl SelectedRule {
    pub(crate) fn intent(&self) -> String {
        match self {
            Self::Greeting => String::from("greeting"),
            Self::Farewell => String::from("farewell"),
            Self::CourtesyResponse => String::from("courtesy_response"),
            Self::Identity => String::from("identity"),
            Self::HelloWorld(program) => format!("hello_world_{}", program.slug),
            Self::Unknown => String::from("unknown"),
        }
    }

    pub(crate) const fn response_link(&self) -> &'static str {
        match self {
            Self::Greeting => "response:greeting",
            Self::Farewell => "response:farewell",
            Self::CourtesyResponse => "response:courtesy_response",
            Self::Identity => "response:identity",
            Self::HelloWorld(program) => program.response_link,
            Self::Unknown => "response:unknown",
        }
    }

    pub(crate) fn answer(&self) -> String {
        match self {
            Self::Greeting => String::from(greeting_answer()),
            Self::Farewell => String::from(farewell_answer()),
            Self::CourtesyResponse => String::from(courtesy_response_answer()),
            Self::Identity => String::from(identity_answer()),
            Self::HelloWorld(program) => hello_world_answer(program),
            Self::Unknown => String::from(unknown_answer()),
        }
    }
}

pub(crate) fn select_rule_for(prompt: &str) -> SelectedRule {
    let normalized = normalize_prompt(prompt);
    if is_greeting(&normalized) {
        SelectedRule::Greeting
    } else if is_farewell(&normalized) {
        SelectedRule::Farewell
    } else if is_courtesy_response(&normalized) {
        SelectedRule::CourtesyResponse
    } else if is_identity_question(&normalized) {
        SelectedRule::Identity
    } else if let Some(program) = hello_world_program(&normalized) {
        SelectedRule::HelloWorld(program)
    } else {
        SelectedRule::Unknown
    }
}

pub(crate) fn language_aware_intent_for(rule: &SelectedRule, _language: Language) -> String {
    rule.intent()
}

pub(crate) fn language_aware_answer_for(
    rule: &SelectedRule,
    language: Language,
    _prompt: &str,
) -> String {
    match (rule, language) {
        (SelectedRule::Greeting, Language::Russian) => String::from(russian_greeting_answer()),
        (SelectedRule::Greeting, Language::Hindi) => String::from(hindi_greeting_answer()),
        (SelectedRule::Greeting, Language::Chinese) => String::from(chinese_greeting_answer()),
        (SelectedRule::Farewell, Language::Russian) => String::from(russian_farewell_answer()),
        (SelectedRule::Farewell, Language::Hindi) => String::from(hindi_farewell_answer()),
        (SelectedRule::Farewell, Language::Chinese) => String::from(chinese_farewell_answer()),
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
        (SelectedRule::Unknown, Language::Russian) => String::from(russian_unknown_answer()),
        (SelectedRule::Unknown, Language::Hindi) => String::from(hindi_unknown_answer()),
        (SelectedRule::Unknown, Language::Chinese) => String::from(chinese_unknown_answer()),
        (SelectedRule::Unknown, Language::Unknown) => {
            String::from(unknown_language_fallback_answer())
        }
        _ => rule.answer(),
    }
}

pub(crate) fn response_link_for_intent(rule: &SelectedRule, _intent: &str) -> String {
    String::from(rule.response_link())
}

fn intent_route(id: &str) -> Option<&'static seed::IntentRoute> {
    static CELL: OnceLock<Vec<seed::IntentRoute>> = OnceLock::new();
    let routes = CELL.get_or_init(|| seed::intent_routing().intents);
    routes.iter().find(|route| route.id == id)
}

fn matches_intent_route(normalized_prompt: &str, id: &str) -> bool {
    let Some(route) = intent_route(id) else {
        return false;
    };
    if route
        .keywords
        .iter()
        .any(|kw| normalized_prompt == kw.as_str())
    {
        return true;
    }
    if route
        .phrases
        .iter()
        .any(|phrase| normalized_prompt == phrase.as_str())
    {
        return true;
    }
    if route
        .tokens
        .iter()
        .any(|token| contains_token(normalized_prompt, token))
    {
        return true;
    }
    route.combos.iter().any(|combo| {
        !combo.is_empty()
            && combo
                .iter()
                .all(|token| contains_token(normalized_prompt, token))
    })
}

fn is_greeting(normalized_prompt: &str) -> bool {
    matches_intent_route(normalized_prompt, "intent_greeting")
}

fn is_farewell(normalized_prompt: &str) -> bool {
    matches_intent_route(normalized_prompt, "intent_farewell")
}

fn is_courtesy_response(normalized_prompt: &str) -> bool {
    matches_intent_route(normalized_prompt, "intent_courtesy_response")
}

fn is_identity_question(normalized_prompt: &str) -> bool {
    matches_intent_route(normalized_prompt, "intent_identity")
}

fn hello_world_program(normalized_prompt: &str) -> Option<&'static HelloWorldProgram> {
    let has_hello =
        contains_token(normalized_prompt, "hello") || contains_token(normalized_prompt, "хелло");
    let has_world =
        contains_token(normalized_prompt, "world") || contains_token(normalized_prompt, "ворлд");
    if !has_hello || !has_world {
        return None;
    }

    HELLO_WORLD_PROGRAMS.iter().find(|program| {
        program
            .aliases
            .iter()
            .any(|alias| contains_token(normalized_prompt, alias))
    })
}

/// Match a program from the catalog by language alias or Russian colloquial name.
pub(crate) fn hello_world_program_by_alias(normalized: &str) -> Option<&'static HelloWorldProgram> {
    const RU: &[(&str, &str)] = &[
        ("питоне", "python"),
        ("питон", "python"),
        ("расте", "rust"),
        ("раст", "rust"),
        ("джаваскрипт", "javascript"),
        ("тайпскрипт", "typescript"),
        ("джава", "java"),
        ("руби", "ruby"),
        ("го ", "go"),
    ];
    for (ru, slug) in RU {
        if normalized.contains(ru) {
            return HELLO_WORLD_PROGRAMS.iter().find(|p| p.slug == *slug);
        }
    }
    HELLO_WORLD_PROGRAMS
        .iter()
        .find(|p| p.aliases.iter().any(|a| normalized.contains(a)))
}

fn contains_token(normalized_prompt: &str, expected: &str) -> bool {
    normalized_prompt
        .split_whitespace()
        .any(|token| token == expected)
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
                sanitize_lino_value(&event.payload)
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
        ],
    )
}

fn format_hello_world_rule_record(program: &HelloWorldProgram) -> String {
    let answer = hello_world_answer(program);
    format_lino_record(
        &format!("rule_hello_world_{}", program.slug),
        &[
            ("intent", format!("hello_world_{}", program.slug)),
            ("language", String::from(program.language)),
            ("aliases", program.aliases.join(", ")),
            ("response_link", String::from(program.response_link)),
            ("answer", answer),
            (
                "execution_status",
                String::from(program.execution.status.label()),
            ),
            (
                "execution_environment",
                String::from(program.execution.environment),
            ),
            ("execution_output", String::from(program.execution.output)),
            (
                "examples",
                format!(
                    "Write me hello world program in {}; hello world in {}",
                    program.language, program.language
                ),
            ),
            ("source", String::from(program.source)),
        ],
    )
}

fn hello_world_answer(program: &HelloWorldProgram) -> String {
    format!(
        "Here is a minimal {} hello world program:\n\n```{}\n{}\n```\n\n{}",
        program.language,
        program.code_fence,
        program.code,
        execution_report(&program.execution)
    )
}

fn execution_report(execution: &ProgramExecution) -> String {
    let command_lines = execution_command_lines(execution);
    let output_label = if matches!(execution.status, ExecutionStatus::Verified) {
        "Output"
    } else {
        "Expected output after verification"
    };

    format!(
        "Execution status: {} in {}.\n{}\n{}:\n```text\n{}\n```\n{}",
        execution.status.label(),
        execution.environment,
        command_lines,
        output_label,
        execution.output,
        execution.notes
    )
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

fn format_lino_record(id: &str, pairs: &[(&str, String)]) -> String {
    let sanitized = pairs
        .iter()
        .map(|(key, value)| (*key, sanitize_lino_value(value)))
        .collect::<Vec<_>>();
    let borrowed = sanitized
        .iter()
        .map(|(key, value)| (*key, value.as_str()))
        .collect::<Vec<_>>();

    format_indented_ordered(id, &borrowed, "  ")
        .expect("static Links Notation records should be valid")
}

fn sanitize_lino_value(value: &str) -> String {
    value
        .replace('\r', "\\r")
        .replace('\n', "\\n")
        .replace('\t', "\\t")
}
