//! Follow-up comparison-table handler for research workflows.
//!
//! Agent mode may split "search for X; then create a comparison table" into a
//! search step followed by a bare table-construction step. This handler keeps
//! that second step bound to the prior research prompt instead of letting it
//! fall through to the unknown opener.

use std::fmt::Write as _;

use crate::engine::{normalize_prompt, SymbolicAnswer};
use crate::event_log::EventLog;
use crate::seed;
use crate::solver_helpers::{last_assistant_turn, last_user_turn};

use super::finalize_simple;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Criterion {
    KeyDifferences,
    UseCases,
    Advantages,
    Disadvantages,
}

impl Criterion {
    const fn label(self) -> &'static str {
        match self {
            Self::KeyDifferences => "Key differences",
            Self::UseCases => "Use cases",
            Self::Advantages => "Advantages",
            Self::Disadvantages => "Disadvantages",
        }
    }

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

    let body = render_comparison_table(&topics, &criteria);
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
    if normalized.contains("no cors enabled web search results")
        || normalized.contains("no cors readable web search results")
        || normalized.contains("no usable cors search results")
        || normalized.contains("не получены результаты веб поиска")
        || normalized.contains("未获取到")
        || normalized.contains("कोई खोज परिणाम नहीं")
    {
        return ResearchResultStatus::NoResults;
    }
    if normalized.contains("all cors readable search providers are disabled")
        || normalized.contains("all cors enabled search providers are disabled")
        || normalized.contains("все cors совместимые поисковые провайдеры отключены")
        || normalized.contains("所有支持 cors 的搜索提供方都已禁用")
        || normalized.contains("सभी cors समर्थित खोज प्रदाता अक्षम हैं")
    {
        return ResearchResultStatus::AllProvidersDisabled;
    }
    if normalized.contains("web search requested")
        || normalized.contains("open research question detected")
        || normalized.contains("search providers that can be queried")
        || normalized.contains("verify claims against")
    {
        return ResearchResultStatus::SearchPlanOnly;
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
    if topics.is_empty() {
        if let Some(after_colon) = prompt.split_once(':').map(|(_, tail)| tail) {
            let topic = clean_search_text(after_colon);
            if !topic.is_empty() {
                topics.push(topic);
            }
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
        if meaning.words().any(|word| normalized.contains(word)) {
            if let Some(criterion) = Criterion::from_slug(&meaning.slug) {
                push_unique(criteria, criterion);
            }
        }
    }
}

fn push_unique(criteria: &mut Vec<Criterion>, criterion: Criterion) {
    if !criteria.contains(&criterion) {
        criteria.push(criterion);
    }
}

fn render_comparison_table(topics: &[String], criteria: &[Criterion]) -> String {
    let mut body = String::from(
        "Research comparison table (draft; verify claims against the Step 1 source links).\n\n",
    );
    body.push('|');
    body.push_str(" Topic |");
    for criterion in criteria {
        let _ = write!(body, " {} |", criterion.label());
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
            let _ = write!(body, " {} |", table_escape(&cell_for(topic, *criterion)));
        }
        body.push('\n');
    }
    body.trim_end().to_owned()
}

fn cell_for(topic: &str, criterion: Criterion) -> String {
    let normalized = normalize_prompt(topic);
    if normalized.contains("machine learning algorithm") {
        return match criterion {
            Criterion::KeyDifferences => {
                "Broad family of data-driven methods; includes supervised, unsupervised, and reinforcement approaches.".to_owned()
            }
            Criterion::UseCases => {
                "Classification, regression, clustering, recommendation, anomaly detection, and forecasting.".to_owned()
            }
            Criterion::Advantages => {
                "Flexible toolkit; often efficient on structured data; many models are easier to inspect than deep nets.".to_owned()
            }
            Criterion::Disadvantages => {
                "Model choice, preprocessing, and feature design can dominate results; overfitting remains a risk.".to_owned()
            }
        };
    }
    if normalized.contains("deep learning") && normalized.contains("traditional ml") {
        return match criterion {
            Criterion::KeyDifferences => {
                "Deep learning learns layered representations; traditional ML often relies more on explicit feature engineering.".to_owned()
            }
            Criterion::UseCases => {
                "Deep learning fits images, speech, and language at scale; traditional ML fits many tabular and smaller-data tasks.".to_owned()
            }
            Criterion::Advantages => {
                "Deep learning scales with data and reduces manual features; traditional ML is usually faster and more interpretable.".to_owned()
            }
            Criterion::Disadvantages => {
                "Deep learning needs more data/compute and is harder to explain; traditional ML may underfit unstructured signals.".to_owned()
            }
        };
    }
    if normalized.contains("neural network") {
        return match criterion {
            Criterion::KeyDifferences => {
                "Built from weighted layers, activations, losses, and optimization; provides the base mechanism for deep learning.".to_owned()
            }
            Criterion::UseCases => {
                "Pattern recognition, embeddings, sequence modeling, vision, speech, and nonlinear function approximation.".to_owned()
            }
            Criterion::Advantages => {
                "Captures nonlinear relationships and can be trained end-to-end for complex perception tasks.".to_owned()
            }
            Criterion::Disadvantages => {
                "Requires tuning and regularization; decisions can be opaque; training can be unstable on poor data.".to_owned()
            }
        };
    }

    match criterion {
        Criterion::KeyDifferences => {
            "Use the prior search sources to identify what distinguishes this topic from the others.".to_owned()
        }
        Criterion::UseCases => {
            "Summarize the practical settings where the Step 1 sources apply this topic.".to_owned()
        }
        Criterion::Advantages => {
            "Extract strengths reported by the prior search sources before treating them as verified.".to_owned()
        }
        Criterion::Disadvantages => {
            "Extract limitations reported by the prior search sources before treating them as verified.".to_owned()
        }
    }
}

fn table_escape(value: &str) -> String {
    value.replace('|', "\\|").replace('\n', " ")
}

fn compact_log_value(value: &str) -> String {
    value.split_whitespace().collect::<Vec<_>>().join(" ")
}
