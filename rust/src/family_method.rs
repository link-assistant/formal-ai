//! Seed-declared generic handler-family interpreter (issue #1138 B9).
//!
//! The migration target is five operation families, not another collection of
//! prompt handlers. Each family declares ordered evidence groups in
//! `data/seed/handler-family-methods.lino`; the one matcher below evaluates all
//! of them. Language surfaces and honest fallback rendering are data, while
//! Rust retains only structural primitives such as counting numeric operands
//! and probing whether the specialist a family names in
//! `declines_when_served_by` already serves the request — an honest-gap family
//! must step aside when the grounded pipeline closes its gap.

use std::sync::OnceLock;

use crate::engine::SymbolicAnswer;
use crate::event_log::EventLog;
use crate::language::detect as detect_language;
use crate::seed::parser::{LinoNode, parse_lino};
use crate::solver::{ConversationTurn, SolverConfig};
use crate::solver_handlers::finalize_simple;

const FAMILY_METHODS_LINO: &str = include_str!("../../data/seed/handler-family-methods.lino");

#[derive(Debug, Clone, PartialEq, Eq)]
struct EvidenceGroup {
    name: String,
    terms: Vec<EvidenceTerm>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum EvidenceTerm {
    Substring(String),
    Word(String),
    Prefix(String),
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct FamilyMethod {
    name: String,
    priority: u32,
    preempts: Vec<String>,
    evidence_groups: Vec<EvidenceGroup>,
    minimum_numbers: usize,
    declines_when_served_by: Option<String>,
    responses: Vec<(String, String)>,
    capture_responses: Vec<(String, String)>,
}

/// The specialists a family may name in `declines_when_served_by`, mapped to
/// their pure serve-probes. A family whose honest-gap response asserts that no
/// grounded renderer exists for the request must step aside when the named
/// specialist demonstrably serves the prompt.
fn specialist_serves(method: &str, prompt: &str) -> bool {
    match method {
        "numeric_list" => {
            crate::solver_handlers::numeric_list::solve_numeric_list(prompt).is_some()
        }
        _ => false,
    }
}

/// The one family whose matched requests run the retrieval procedure
/// (`src/retrieval_method.rs`) instead of the fixed per-language response.
const RETRIEVAL_FAMILY: &str = "retrieval_method";

impl FamilyMethod {
    fn matches(&self, normalized: &str) -> bool {
        let padded = format!(" {normalized} ");
        let lexical = self.evidence_groups.iter().all(|group| {
            group.terms.iter().any(|term| match term {
                EvidenceTerm::Substring(cue) => {
                    if cue.starts_with(' ') || cue.ends_with(' ') {
                        padded.contains(cue)
                    } else {
                        normalized.contains(cue)
                    }
                }
                EvidenceTerm::Word(word) => normalized
                    .split(|character: char| !character.is_alphanumeric())
                    .any(|token| token == word),
                EvidenceTerm::Prefix(prefix) => normalized
                    .split(|character: char| !character.is_alphanumeric())
                    .any(|token| token.starts_with(prefix)),
            })
        });
        lexical && numeric_operands(normalized).len() >= self.minimum_numbers
    }

    fn response(&self, language: &str, operand_count: usize) -> Option<String> {
        self.responses
            .iter()
            .find(|(candidate, _)| candidate == language)
            .or_else(|| {
                self.responses
                    .iter()
                    .find(|(candidate, _)| candidate == "en")
            })
            .map(|(_, text)| {
                text.replace(
                    concat!("{", "operand_count", "}"),
                    &operand_count.to_string(),
                )
            })
    }
}

/// Interpret one request through the first matching seed-declared family.
///
/// `history` is recorded as evidence for dialogue queries. A caller that has no
/// history gets the same honest no-evidence response rather than a fabricated
/// recollection.
#[must_use]
pub fn try_family_method(
    prompt: &str,
    normalized: &str,
    history: &[ConversationTurn],
    log: &mut EventLog,
    config: SolverConfig,
) -> Option<SymbolicAnswer> {
    try_family_method_selected(prompt, normalized, history, log, config, |_| true)
}

/// Interpret a matching family only when its seed record says it outranks the
/// competing dispatcher result.
///
/// Families normally sit at the generic concept/unknown fallback boundary. A
/// few already-grounded family shapes overlap an older specialist (for
/// example, source-backed retrieval also contains the word "summarise"). The
/// precedence relation belongs beside the family's evidence groups in seed,
/// rather than in a Rust list or an unconditional early call that would shadow
/// richer handlers.
#[must_use]
pub fn try_family_method_preempting(
    prompt: &str,
    normalized: &str,
    history: &[ConversationTurn],
    log: &mut EventLog,
    competing_method: &str,
    config: SolverConfig,
) -> Option<SymbolicAnswer> {
    try_family_method_selected(prompt, normalized, history, log, config, |family| {
        family
            .preempts
            .iter()
            .any(|candidate| candidate == competing_method)
    })
}

fn try_family_method_selected(
    prompt: &str,
    normalized: &str,
    history: &[ConversationTurn],
    log: &mut EventLog,
    config: SolverConfig,
    eligible: impl Fn(&FamilyMethod) -> bool,
) -> Option<SymbolicAnswer> {
    let operands = numeric_operands(normalized);
    let family = catalog()
        .iter()
        .filter(|family| eligible(family))
        .filter(|family| family.matches(normalized))
        .filter(|family| {
            // A family that names a specialist in `declines_when_served_by`
            // asserts a grounding gap that specialist would close — an honest
            // gap statement is false when the grounded pipeline actually
            // serves the prompt, so the family steps aside and the walk
            // reaches the specialist (issue #1021).
            family
                .declines_when_served_by
                .as_deref()
                .is_none_or(|method| !specialist_serves(method, prompt))
        })
        .min_by_key(|family| family.priority)?;
    let language = detect_language(prompt).slug();
    let body = family_body(
        family,
        prompt,
        normalized,
        language,
        operands.len(),
        config,
        log,
    );

    log.append("family_method", family.name.clone());
    log.append(
        "family_method:evidence_groups",
        family
            .evidence_groups
            .iter()
            .map(|group| group.name.as_str())
            .collect::<Vec<_>>()
            .join("|"),
    );
    if family.minimum_numbers > 0 {
        log.append("family_method:operand_count", operands.len().to_string());
    }
    log.append("family_method:history_count", history.len().to_string());

    Some(finalize_simple(
        prompt,
        log,
        &family.name,
        &format!("response:{}", family.name),
        &body,
        0.9,
    ))
}

fn catalog() -> &'static [FamilyMethod] {
    static CELL: OnceLock<Vec<FamilyMethod>> = OnceLock::new();
    CELL.get_or_init(|| {
        parse_catalog(FAMILY_METHODS_LINO).unwrap_or_else(|error| panic!("{error}"))
    })
}

/// The body for one matched family: the retrieval procedure's rendering when
/// the family runs the interpreter, the seeded response otherwise.
///
/// The retrieval family renders its walk — captures with provenance, or the
/// seeded no-capture response when nothing was verified. Every other family
/// keeps its fixed per-language response.
fn family_body(
    family: &FamilyMethod,
    prompt: &str,
    normalized: &str,
    language: &str,
    operand_count: usize,
    config: SolverConfig,
    log: &mut EventLog,
) -> String {
    let fallback = family.response(language, operand_count);
    if family.name != RETRIEVAL_FAMILY {
        return fallback.unwrap_or_default();
    }
    let senses = crate::retrieval_method::retrieve(config, prompt, normalized, log);
    let template = family
        .capture_responses
        .iter()
        .find(|(candidate, _)| candidate == language)
        .or_else(|| {
            family
                .capture_responses
                .iter()
                .find(|(candidate, _)| candidate == "en")
        })
        .map(|(_, text)| text.clone());
    crate::retrieval_method::render_answer(
        &senses,
        template.as_deref(),
        fallback.as_deref().unwrap_or_default(),
    )
}

/// The no-capture response one family declares for one language.
#[must_use]
pub fn family_response_for(family_name: &str, language: &str) -> Option<String> {
    catalog()
        .iter()
        .find(|family| family.name == family_name)?
        .response(language, 0)
}

/// The with-capture template one family declares for one language.
#[must_use]
pub fn capture_template_for(family_name: &str, language: &str) -> Option<String> {
    let family = catalog().iter().find(|family| family.name == family_name)?;
    family
        .capture_responses
        .iter()
        .find(|(candidate, _)| candidate == language)
        .or_else(|| {
            family
                .capture_responses
                .iter()
                .find(|(candidate, _)| candidate == "en")
        })
        .map(|(_, text)| text.clone())
}

fn parse_catalog(text: &str) -> Result<Vec<FamilyMethod>, String> {
    let tree = parse_lino(text);
    let root = tree
        .children
        .iter()
        .find(|node| node.name == "handler_family_methods")
        .ok_or_else(|| String::from("handler_family_methods:missing_root"))?;
    let mut families = Vec::new();
    for node in root.children.iter().filter(|node| node.name == "family") {
        let priority = node
            .find_child_value("priority")
            .parse::<u32>()
            .map_err(|_| format!("handler_family_methods:{}:invalid_priority", node.id))?;
        let minimum_numbers = node
            .find_child_value("minimum_numbers")
            .parse::<usize>()
            .unwrap_or(0);
        let evidence_groups = node
            .children
            .iter()
            .filter(|child| child.name == "evidence_group")
            .map(parse_evidence_group)
            .collect::<Result<Vec<_>, _>>()?;
        let responses = node
            .children
            .iter()
            .filter(|child| child.name == "response")
            .filter_map(|child| split_response(&child.id))
            .collect::<Vec<_>>();
        let capture_responses = node
            .children
            .iter()
            .filter(|child| child.name == "response_with_capture")
            .filter_map(|child| split_response(&child.id))
            .collect::<Vec<_>>();
        let declines_when_served_by = node
            .children
            .iter()
            .find(|child| child.name == "declines_when_served_by")
            .map(|child| child.id.clone())
            .filter(|name| !name.is_empty());
        if let Some(method) = &declines_when_served_by
            && !matches!(method.as_str(), "numeric_list")
        {
            return Err(format!(
                "handler_family_methods:{}:unknown_declines_when_served_by:{method}",
                node.id
            ));
        }
        if node.id.is_empty() || evidence_groups.is_empty() || responses.is_empty() {
            return Err(format!(
                "handler_family_methods:{}:incomplete_family",
                node.id
            ));
        }
        families.push(FamilyMethod {
            name: node.id.clone(),
            priority,
            preempts: node
                .children
                .iter()
                .filter(|child| child.name == "preempts")
                .map(|child| child.id.clone())
                .filter(|name| !name.is_empty())
                .collect(),
            evidence_groups,
            minimum_numbers,
            declines_when_served_by,
            responses,
            capture_responses,
        });
    }
    families.sort_by_key(|family| family.priority);
    Ok(families)
}

fn parse_evidence_group(node: &LinoNode) -> Result<EvidenceGroup, String> {
    let terms = node
        .children
        .iter()
        .filter_map(|child| match child.name.as_str() {
            "cue" => Some(EvidenceTerm::Substring(child.id.to_lowercase())),
            "word" => Some(EvidenceTerm::Word(child.id.to_lowercase())),
            "prefix" => Some(EvidenceTerm::Prefix(child.id.to_lowercase())),
            _ => None,
        })
        .filter(|term| match term {
            EvidenceTerm::Substring(value)
            | EvidenceTerm::Word(value)
            | EvidenceTerm::Prefix(value) => !value.is_empty(),
        })
        .collect::<Vec<_>>();
    if node.id.is_empty() || terms.is_empty() {
        return Err(format!(
            "handler_family_methods:{}:empty_evidence_group",
            node.id
        ));
    }
    Ok(EvidenceGroup {
        name: node.id.clone(),
        terms,
    })
}

fn split_response(value: &str) -> Option<(String, String)> {
    let (language, text) = value.split_once(char::is_whitespace)?;
    let text = text.trim().trim_matches('"').replace("\"\"", "\"");
    (!language.is_empty() && !text.is_empty()).then(|| (language.to_owned(), text))
}

fn numeric_operands(text: &str) -> Vec<String> {
    text.split(|character: char| !character.is_numeric() && character != '-' && character != '.')
        .filter(|token| token.chars().any(char::is_numeric))
        .map(str::to_owned)
        .collect()
}
