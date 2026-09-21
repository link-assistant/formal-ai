//! Follow-up comparison-table handler for research workflows.
//!
//! Agent mode may split "search for X; then create a comparison table" into a
//! search step followed by a bare table-construction step. This handler keeps
//! that second step bound to the prior research prompt instead of letting it
//! fall through to the unknown opener.

use std::fmt::Write as _;
use std::sync::OnceLock;

use crate::engine::{SymbolicAnswer, normalize_prompt};
use crate::event_log::EventLog;
use crate::language::detect as detect_language;
use crate::seed;
use crate::seed::parser::{LinoNode, parse_lino};
use crate::solver_helpers::{last_assistant_turn, last_user_turn};

use super::finalize_simple;

const RESEARCH_TABLE_PROCEDURE_LINO: &str =
    include_str!("../../../data/seed/research-table-procedure.lino");

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Criterion {
    KeyDifferences,
    UseCases,
    Advantages,
    Disadvantages,
}

impl Criterion {
    const fn slug(self) -> &'static str {
        match self {
            Self::KeyDifferences => "key_differences",
            Self::UseCases => "use_cases",
            Self::Advantages => "advantages",
            Self::Disadvantages => "disadvantages",
        }
    }

    /// The criterion whose [`slug`](Self::slug) equals `slug`, or `None`.
    ///
    /// The inverse of [`slug`](Self::slug): it keys a column off a
    /// `research_criterion` meaning's slug, so the comparison-table handler turns
    /// a matched meaning into its column without naming a surface word in code.
    fn from_slug(slug: &str) -> Option<Self> {
        match slug {
            "key_differences" => Some(Self::KeyDifferences),
            "use_cases" => Some(Self::UseCases),
            "advantages" => Some(Self::Advantages),
            "disadvantages" => Some(Self::Disadvantages),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ResearchResultStatus {
    NoResults,
    AllProvidersDisabled,
    SearchPlanOnly,
    Open,
}

impl ResearchResultStatus {
    const fn slug(self) -> &'static str {
        match self {
            Self::NoResults => "no_results",
            Self::AllProvidersDisabled => "all_providers_disabled",
            Self::SearchPlanOnly => "search_plan_only",
            Self::Open => "open_research",
        }
    }

    fn from_slug(slug: &str) -> Option<Self> {
        match slug {
            "no_results" => Some(Self::NoResults),
            "all_providers_disabled" => Some(Self::AllProvidersDisabled),
            "search_plan_only" => Some(Self::SearchPlanOnly),
            "open_research" => Some(Self::Open),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Default)]
struct LocalizedTableText {
    language: String,
    intro: String,
    topic_label: String,
}

#[derive(Debug, Clone, Default)]
struct CriterionText {
    slug: String,
    labels: Vec<(String, String)>,
    instructions: Vec<(String, String)>,
}

#[derive(Debug, Clone, Default)]
struct ResearchTableProcedure {
    statuses: Vec<(ResearchResultStatus, Vec<String>)>,
    table_texts: Vec<LocalizedTableText>,
    criteria: Vec<CriterionText>,
}

impl ResearchTableProcedure {
    fn table_text(&self, language: &str) -> Option<&LocalizedTableText> {
        self.table_texts
            .iter()
            .find(|text| text.language == language)
            .or_else(|| self.table_texts.iter().find(|text| text.language == "en"))
    }

    fn criterion_text(&self, criterion: Criterion) -> Option<&CriterionText> {
        self.criteria
            .iter()
            .find(|text| text.slug == criterion.slug())
    }
}

pub fn try_research_comparison_table(
    prompt: &str,
    normalized: &str,
    log: &mut EventLog,
) -> Option<SymbolicAnswer> {
    if !is_comparison_table_request(normalized) {
        return None;
    }

    let prior_search = last_user_turn(log)?;
    if !looks_like_research_prompt(prior_search) {
        return None;
    }

    let topics = extract_research_topics(prior_search);
    if topics.len() < 2 {
        return None;
    }
    let criteria = extract_criteria(prompt);
    if criteria.is_empty() {
        return None;
    }

    log.append(
        "research_table:prior_search",
        compact_log_value(prior_search),
    );
    for topic in &topics {
        log.append("research_table:topic", topic.clone());
    }
    for criterion in &criteria {
        log.append("research_table:criterion", criterion.slug());
    }

    let language = detect_language(prompt).slug();
    let body = render_comparison_table(&topics, &criteria, language);
    Some(finalize_simple(
        prompt,
        log,
        "research_comparison_table",
        "response:research_comparison_table",
        &body,
        0.78,
    ))
}

pub fn try_research_result_followup(
    prompt: &str,
    normalized: &str,
    log: &mut EventLog,
) -> Option<SymbolicAnswer> {
    if !is_research_result_followup(normalized) {
        return None;
    }

    let prior_search = last_user_turn(log)?.to_owned();
    if !looks_like_research_prompt(&prior_search) {
        return None;
    }

    let status = classify_prior_research_answer(last_assistant_turn(log));
    let prior_search_log = compact_log_value(&prior_search);
    let body = render_research_result_followup(&prior_search, status);
    log.append("research_result_followup:prior_search", prior_search_log);
    log.append("research_result_followup:status", status.slug());

    Some(finalize_simple(
        prompt,
        log,
        "research_result_followup",
        "response:research_result_followup",
        &body,
        0.76,
    ))
}

/// True when the prompt asks for a comparison drawn as a table: either a strong
/// `comparison_table_trigger` ('comparison table', 'compare', …) occurs, or the
/// weak pair of a `comparison_table_noun` ('table') and a
/// `comparison_difference_cue` ('differences') co-occur — each recognized
/// token-bounded across every supported language via the seed lexicon.
fn is_comparison_table_request(normalized: &str) -> bool {
    let lexicon = seed::lexicon();
    lexicon.mentions_role(seed::ROLE_COMPARISON_TABLE_TRIGGER, normalized)
        || (lexicon.mentions_role(seed::ROLE_COMPARISON_TABLE_NOUN, normalized)
            && lexicon.mentions_role(seed::ROLE_COMPARISON_DIFFERENCE_CUE, normalized))
}

/// True when `prompt` was itself a research request — the prior turn a
/// comparison-table follow-up reuses for its topics. The `research_prompt_signal`
/// meaning carries both bare markers ('web search', 'research', …), matched
/// token-bounded anywhere, and prefix surfaces ('search …', 'find information …',
/// …), matched when the prompt opens with the literal before the `…` slot. Both
/// the markers and the prefixes live in the seed data, not in a
/// `starts_with`/`contains` list in the code.
fn looks_like_research_prompt(prompt: &str) -> bool {
    let normalized = normalize_prompt(prompt);
    let lexicon = seed::lexicon();
    lexicon.mentions_role(seed::ROLE_RESEARCH_PROMPT_SIGNAL, &normalized)
        || lexicon
            .role_word_forms(seed::ROLE_RESEARCH_PROMPT_SIGNAL)
            .iter()
            .filter(|form| form.slot() == seed::Slot::Prefix)
            .any(|form| normalized.starts_with(form.before_slot()))
}

fn is_research_result_followup(normalized: &str) -> bool {
    let cleaned = normalize_prompt(normalized);
    matches!(
        cleaned.as_str(),
        "result"
            | "the result"
            | "what result"
            | "what is result"
            | "what s the result"
            | "what is the result"
            | "what was the result"
            | "what are the results"
            | "what were the results"
            | "show the result"
            | "show the results"
            | "give me the result"
            | "give me the results"
            | "what is the answer"
            | "what was the answer"
            | "what did you find"
            | "what did we find"
            | "what is the outcome"
            | "what was the outcome"
    )
}

fn classify_prior_research_answer(answer: Option<&str>) -> ResearchResultStatus {
    let Some(answer) = answer else {
        return ResearchResultStatus::Open;
    };
    let normalized = normalize_prompt(answer);
    for (status, markers) in &research_table_procedure().statuses {
        if markers
            .iter()
            .any(|marker| normalized.match_indices(marker).next().is_some())
        {
            return *status;
        }
    }
    ResearchResultStatus::Open
}

fn render_research_result_followup(prior_search: &str, status: ResearchResultStatus) -> String {
    let preview = research_prompt_preview(prior_search);
    match status {
        ResearchResultStatus::NoResults => format!(
            "The result of the previous research step is: no CORS-readable web search results were returned. I do not have verified source data to complete the requested analysis, calculation, table, or sources list yet.\n\nPrior research task: `{preview}`\n\nNext step: rerun the search with narrower queries or provide source links; then I can calculate the requested impact from those sources."
        ),
        ResearchResultStatus::AllProvidersDisabled => format!(
            "The result of the previous research step is: web search could not run because the CORS-readable search providers were disabled. No verified research result was produced yet.\n\nPrior research task: `{preview}`\n\nNext step: enable a search provider or provide source links; then I can complete the requested analysis from those sources."
        ),
        ResearchResultStatus::SearchPlanOnly => format!(
            "The previous turn only set up the research/search step. It did not produce a final result, calculation, or sourced executive summary yet.\n\nPrior research task: `{preview}`\n\nNext step: run the source search and use the returned sources to finish the calculation."
        ),
        ResearchResultStatus::Open => format!(
            "There is no verified final research result in the conversation yet. The prior turn was a research request, but I do not see a completed source-backed answer to report.\n\nPrior research task: `{preview}`\n\nNext step: run the search or provide source links; then I can produce the requested result."
        ),
    }
}

fn research_prompt_preview(value: &str) -> String {
    let compact = compact_log_value(value);
    if compact.chars().count() <= 240 {
        return compact;
    }
    let mut preview = compact.chars().take(237).collect::<String>();
    preview.push_str("...");
    preview
}

fn extract_research_topics(prompt: &str) -> Vec<String> {
    let mut topics = Vec::new();
    for line in prompt.lines() {
        let Some(topic) = clean_topic_line(line) else {
            continue;
        };
        if !topics
            .iter()
            .any(|existing: &String| existing.eq_ignore_ascii_case(&topic))
        {
            topics.push(topic);
        }
        if topics.len() >= 8 {
            break;
        }
    }
    if topics.is_empty()
        && let Some(after_colon) = prompt.split_once(':').map(|(_, tail)| tail)
    {
        let topic = clean_search_text(after_colon);
        if !topic.is_empty() {
            topics.push(topic);
        }
    }
    topics
}

fn clean_topic_line(line: &str) -> Option<String> {
    let stripped = strip_list_marker(line.trim())?;
    let topic = clean_search_text(stripped);
    if topic.is_empty() || looks_like_research_prompt(&topic) {
        None
    } else {
        Some(topic)
    }
}

fn strip_list_marker(line: &str) -> Option<&str> {
    let value = line.trim();
    if value.is_empty() {
        return None;
    }
    let first = value.chars().next()?;
    if matches!(first, '-' | '*' | '+') {
        return Some(value[first.len_utf8()..].trim());
    }
    let digit_count = value.chars().take_while(char::is_ascii_digit).count();
    if digit_count > 0 {
        let rest = &value[digit_count..];
        let marker = rest.chars().next()?;
        if matches!(marker, '.' | ')' | ':') {
            return Some(rest[marker.len_utf8()..].trim());
        }
    }
    None
}

fn clean_search_text(value: &str) -> String {
    value
        .trim()
        .trim_matches(|character: char| {
            character.is_whitespace()
                || matches!(
                    character,
                    '`' | '"' | '\'' | ':' | ';' | ',' | '.' | '?' | '!'
                )
        })
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}

fn extract_criteria(prompt: &str) -> Vec<Criterion> {
    let mut criteria = Vec::new();
    for line in prompt.lines() {
        let Some(item) = strip_list_marker(line.trim()) else {
            continue;
        };
        append_criteria_from_text(item, &mut criteria);
    }
    if criteria.is_empty() {
        append_criteria_from_text(prompt, &mut criteria);
    }
    if criteria.is_empty() {
        criteria.extend([
            Criterion::KeyDifferences,
            Criterion::UseCases,
            Criterion::Advantages,
            Criterion::Disadvantages,
        ]);
    }
    criteria
}

/// Add every comparison column the text names. Walks the `research_criterion`
/// meanings in declaration order (which fixes the column order) and adds a
/// criterion when any of its surface words occurs as a raw substring — the same
/// substring contract the legacy code used, so space-guarded stems like 'pro '
/// and ' con ' still avoid matching inside 'process'/'control'. The trigger
/// words live in the seed data; the code names only the language-independent
/// slug that keys each column.
fn append_criteria_from_text(text: &str, criteria: &mut Vec<Criterion>) {
    let normalized = normalize_prompt(text);
    for meaning in seed::lexicon().meanings_with_role(seed::ROLE_RESEARCH_CRITERION) {
        if meaning
            .words()
            .any(|word| normalized.match_indices(word).next().is_some())
            && let Some(criterion) = Criterion::from_slug(&meaning.slug)
        {
            push_unique(criteria, criterion);
        }
    }
}

fn push_unique(criteria: &mut Vec<Criterion>, criterion: Criterion) {
    if !criteria.contains(&criterion) {
        criteria.push(criterion);
    }
}

fn render_comparison_table(topics: &[String], criteria: &[Criterion], language: &str) -> String {
    let procedure = research_table_procedure();
    let table_text = procedure.table_text(language);
    let intro = table_text.map_or("", |text| text.intro.as_str());
    let topic_label = table_text.map_or("topic", |text| text.topic_label.as_str());
    let mut body = format!("{intro}\n\n");
    body.push('|');
    let _ = write!(body, " {topic_label} |");
    for criterion in criteria {
        let label = procedure
            .criterion_text(*criterion)
            .and_then(|text| localized_value(&text.labels, language))
            .unwrap_or_else(|| criterion.slug());
        let _ = write!(body, " {label} |");
    }
    body.push('\n');
    body.push('|');
    body.push_str(" --- |");
    for _ in criteria {
        body.push_str(" --- |");
    }
    body.push('\n');
    for topic in topics {
        let _ = write!(body, "| {} |", table_escape(topic));
        for criterion in criteria {
            let cell = procedure
                .criterion_text(*criterion)
                .and_then(|text| localized_value(&text.instructions, language))
                .unwrap_or_else(|| criterion.slug());
            let _ = write!(body, " {} |", table_escape(cell));
        }
        body.push('\n');
    }
    body.trim_end().to_owned()
}

fn localized_value<'a>(values: &'a [(String, String)], language: &str) -> Option<&'a str> {
    values
        .iter()
        .find(|(candidate, _)| candidate == language)
        .or_else(|| values.iter().find(|(candidate, _)| candidate == "en"))
        .map(|(_, value)| value.as_str())
}

fn research_table_procedure() -> &'static ResearchTableProcedure {
    static CELL: OnceLock<ResearchTableProcedure> = OnceLock::new();
    CELL.get_or_init(|| parse_research_table_procedure(RESEARCH_TABLE_PROCEDURE_LINO))
}

fn parse_research_table_procedure(text: &str) -> ResearchTableProcedure {
    let tree = parse_lino(text);
    let Some(root) = tree
        .children
        .iter()
        .find(|node| node.name == "research_table_procedure")
    else {
        return ResearchTableProcedure::default();
    };
    let statuses = root
        .children
        .iter()
        .filter(|node| node.name == "status")
        .filter_map(|node| {
            let status = ResearchResultStatus::from_slug(&node.id)?;
            Some((status, child_values(node, "marker")))
        })
        .collect();
    let table_texts = root
        .children
        .iter()
        .filter(|node| node.name == "table_text")
        .map(|node| LocalizedTableText {
            language: node.id.clone(),
            intro: node.find_child_value("intro").to_owned(),
            topic_label: node.find_child_value("topic_label").to_owned(),
        })
        .collect();
    let criteria = root
        .children
        .iter()
        .filter(|node| node.name == "criterion")
        .map(|node| CriterionText {
            slug: node.id.clone(),
            labels: localized_children(node, "label"),
            instructions: localized_children(node, "instruction"),
        })
        .collect();
    ResearchTableProcedure {
        statuses,
        table_texts,
        criteria,
    }
}

fn child_values(node: &LinoNode, name: &str) -> Vec<String> {
    node.children
        .iter()
        .filter(|child| child.name == name)
        .map(|child| child.id.to_lowercase())
        .collect()
}

fn localized_children(node: &LinoNode, name: &str) -> Vec<(String, String)> {
    node.children
        .iter()
        .filter(|child| child.name == name)
        .filter_map(|child| {
            let (language, value) = child.id.split_once(char::is_whitespace)?;
            Some((
                language.to_owned(),
                value.trim().trim_matches('"').replace("\"\"", "\""),
            ))
        })
        .collect()
}

fn table_escape(value: &str) -> String {
    value.replace('|', "\\|").replace('\n', " ")
}

fn compact_log_value(value: &str) -> String {
    value.split_whitespace().collect::<Vec<_>>().join(" ")
}
