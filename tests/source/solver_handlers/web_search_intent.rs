//! Natural-language web-search intent recognition.
//!
//! Every surface cue this recogniser reasons about — the explicit command
//! prefixes, the action verbs, the source/signal nouns, the topic connectives,
//! the query noise, the follow-up instruction verbs and clause boundaries, and
//! the research/enumeration vocabulary — is sourced from the language-independent
//! meaning lexicon (`data/seed/meanings-web-search*.lino`,
//! `meanings-web-research.lino`, `meanings-web-followup.lino`). The handler
//! references those meanings by their semantic *role* (e.g.
//! [`ROLE_WEB_SEARCH_EXPLICIT_PREFIX`], [`ROLE_FOLLOWUP_INSTRUCTION_VERB`]) and
//! by the *slot* each word form occupies (prefix / suffix / bare), never by raw
//! words baked into the code. Adding a language or a synonym is therefore a pure
//! data edit: drop a `word`/`description` into the relevant meaning and this
//! handler reasons about it automatically. The follow-up truncation in
//! particular is a universal boundary algorithm — a follow-up clause is detected
//! structurally (an instruction verb immediately preceded by sentence
//! punctuation or a chained clause-continuation marker), not by memorising the
//! handful of `". compare"`-style fragments the prompts happen to use.

use std::sync::OnceLock;

use crate::coding::contains_cjk;
use crate::engine::normalize_prompt;
use crate::seed::{
    self, Slot, WordForm, ROLE_CLAUSE_CONTINUATION_MARKER, ROLE_ENUMERATION_CONSTRAINT,
    ROLE_ENUMERATION_REQUEST_OPENER, ROLE_FOLLOWUP_INSTRUCTION_VERB,
    ROLE_RESEARCH_EVALUATION_DOMAIN, ROLE_RESEARCH_EVIDENCE_DOMAIN, ROLE_RESEARCH_QUESTION_OPENER,
    ROLE_RESEARCH_SUPERLATIVE_MODIFIER, ROLE_WEB_SEARCH_ACTION, ROLE_WEB_SEARCH_EXPLICIT_PREFIX,
    ROLE_WEB_SEARCH_IMPERATIVE_LEAD, ROLE_WEB_SEARCH_NEWS_RECENCY, ROLE_WEB_SEARCH_NEWS_SUBJECT,
    ROLE_WEB_SEARCH_QUERY_LEADING_NOISE, ROLE_WEB_SEARCH_QUERY_TRAILING_NOISE,
    ROLE_WEB_SEARCH_RECORDS_SUBJECT, ROLE_WEB_SEARCH_SIGNAL, ROLE_WEB_SEARCH_SOURCE_ONLY,
    ROLE_WEB_SEARCH_STRONG_ACTION, ROLE_WEB_SEARCH_TOPIC_MARKER,
};

use super::web_requests::normalize_url_candidate;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum WebSearchQueryKind {
    ExplicitPrefix,
    SemanticAction,
    LatestNews,
    RecordsInformationRequest,
    ImplicitResearchQuestion,
    EnumerationResearchRequest,
    UnresolvedBareTerm,
}

impl WebSearchQueryKind {
    pub(crate) const fn as_str(self) -> &'static str {
        match self {
            Self::ExplicitPrefix => "explicit_prefix",
            Self::SemanticAction => "semantic_action",
            Self::LatestNews => "latest_news",
            Self::RecordsInformationRequest => "records_information_request",
            Self::ImplicitResearchQuestion => "implicit_research_question",
            Self::EnumerationResearchRequest => "enumeration_research_request",
            Self::UnresolvedBareTerm => "unresolved_bare_term",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct WebSearchRequest {
    pub(super) query: String,
    pub(super) kind: WebSearchQueryKind,
}

pub(super) fn extract_web_search_request(
    prompt: &str,
    normalized: &str,
) -> Option<WebSearchRequest> {
    let normalized_words = normalize_prompt(prompt);
    if normalized_words.starts_with("search conversations ")
        || normalized_words.starts_with("search my conversations ")
        || normalized_words.starts_with("search my chats ")
        || is_personal_fact_filter_request(&normalized_words)
    {
        return None;
    }
    // Try the punctuation-preserving `normalized` first so the follow-up
    // truncation downstream can see sentence boundaries (`normalize_prompt`
    // strips punctuation, which would hide the period in
    // "… Thomas Edison. Compare …"); fall back to the punctuation-stripped,
    // whitespace-collapsed `normalized_words` for prompts whose leading layout
    // only `normalize_prompt` cleans up.
    for &prefix in &markers().explicit_prefixes {
        if let Some(query) = normalized.strip_prefix(prefix) {
            if let Some(query) = valid_search_query(query) {
                return Some(WebSearchRequest {
                    query,
                    kind: WebSearchQueryKind::ExplicitPrefix,
                });
            }
        }
        if let Some(query) = normalized_words.strip_prefix(prefix) {
            if let Some(query) = valid_search_query(query) {
                return Some(WebSearchRequest {
                    query,
                    kind: WebSearchQueryKind::ExplicitPrefix,
                });
            }
        }
    }
    if is_text_extraction_request(&normalized_words) {
        return None;
    }
    if let Some(query) = extract_semantic_web_search_query(&normalized_words) {
        return Some(WebSearchRequest {
            query,
            kind: WebSearchQueryKind::SemanticAction,
        });
    }
    if let Some(query) = extract_latest_news_search_request(&normalized_words) {
        return Some(WebSearchRequest {
            query,
            kind: WebSearchQueryKind::LatestNews,
        });
    }
    if let Some(query) = extract_records_information_request(&normalized_words) {
        return Some(WebSearchRequest {
            query,
            kind: WebSearchQueryKind::RecordsInformationRequest,
        });
    }
    if let Some(query) = extract_enumeration_research_request(&normalized_words) {
        return Some(WebSearchRequest {
            query,
            kind: WebSearchQueryKind::EnumerationResearchRequest,
        });
    }
    extract_implicit_research_question(&normalized_words).map(|query| WebSearchRequest {
        query,
        kind: WebSearchQueryKind::ImplicitResearchQuestion,
    })
}

fn is_personal_fact_filter_request(normalized: &str) -> bool {
    normalized.contains("facts i have contributed")
        || normalized.contains("facts ive contributed")
        || normalized.contains("facts i contributed")
        || normalized.contains("my facts")
}

fn clean_search_query(value: &str) -> String {
    value
        .trim()
        .trim_matches(is_url_wrapper_punctuation)
        .trim_end_matches(is_url_trailing_punctuation)
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}

const fn is_url_wrapper_punctuation(character: char) -> bool {
    matches!(
        character,
        '<' | '>' | '(' | ')' | '[' | ']' | '{' | '}' | '"' | '\'' | '`' | '«' | '»'
    )
}

const fn is_url_trailing_punctuation(character: char) -> bool {
    matches!(character, '.' | ',' | '!' | '?' | ';' | ':' | '…')
}

/// Sentence-ending punctuation that can introduce a follow-up instruction
/// clause. Universal across the supported languages — the ASCII marks plus the
/// fullwidth/ideographic forms a CJK prompt would use.
const fn is_sentence_boundary(character: char) -> bool {
    matches!(
        character,
        '.' | '?' | '!' | ';' | ':' | '。' | '？' | '！' | '；' | '：'
    )
}

/// Every surface cue the web-search recogniser reasons about, projected out of
/// the meaning lexicon by role and slot. Built once and cached: because
/// [`seed::lexicon`] returns a `'static` reference, the projected literals are
/// themselves `'static` and need no allocation beyond the backing vectors.
struct WebSearchMarkers {
    /// Lead-ins of an explicit "search X for …" command (prefix slot).
    explicit_prefixes: Vec<&'static str>,
    /// Bare search verbs that signal an action is requested.
    action_markers: Vec<&'static str>,
    /// The subset of action verbs strong enough to stand without a source noun.
    strong_action_markers: Vec<&'static str>,
    /// Source/topic nouns that corroborate a weak action verb.
    signal_markers: Vec<&'static str>,
    /// Topic connectives whose object follows them ("about …", "о …").
    topic_after_markers: Vec<&'static str>,
    /// Topic connectives whose object precedes them ("… के बारे में").
    topic_before_markers: Vec<&'static str>,
    /// Imperative search leads whose query follows them ("search for …").
    imperative_lead_markers: Vec<&'static str>,
    /// Politeness / determiner noise stripped from the front of a query.
    leading_noise: Vec<&'static str>,
    /// Source/medium noise stripped from the end of a query.
    trailing_noise: Vec<&'static str>,
    /// Bare source words that are not, on their own, a valid query.
    source_only: Vec<String>,
    /// News/headline subject markers for bare latest-news requests.
    news_subject_markers: Vec<&'static str>,
    /// Freshness markers that pair with news/headline subjects.
    news_recency_markers: Vec<&'static str>,
    /// Records/documents subject nouns for verbless "records about X" requests.
    records_subject_markers: Vec<&'static str>,
    /// Verbs that open a follow-up instruction clause ("compare", "summarize").
    followup_verbs: Vec<&'static str>,
    /// Conjunctions/adverbs that, like punctuation, mark a clause boundary.
    continuation_markers: Vec<&'static str>,
    /// Question openers of an implicit research request ("what is the …").
    research_question_prefixes: Vec<&'static str>,
    /// Superlative/recency modifiers that make a question researchable.
    research_modifiers: Vec<&'static str>,
    /// Evidence nouns (dataset, benchmark, paper …) of a research question.
    research_evidence_domains: Vec<&'static str>,
    /// Evaluation nouns (validation, quality, comparison …) of a question.
    research_evaluation_domains: Vec<&'static str>,
    /// Openers of an enumeration research request ("list all …").
    enumeration_prefixes: Vec<&'static str>,
    /// Constraint connectives that make an enumeration researchable.
    enumeration_constraint_markers: Vec<&'static str>,
}

/// Build (once) the marker projection from the meaning lexicon.
fn markers() -> &'static WebSearchMarkers {
    static CACHE: OnceLock<WebSearchMarkers> = OnceLock::new();
    CACHE.get_or_init(|| WebSearchMarkers {
        explicit_prefixes: prefix_literals(ROLE_WEB_SEARCH_EXPLICIT_PREFIX),
        action_markers: bare_literals(ROLE_WEB_SEARCH_ACTION),
        strong_action_markers: bare_literals(ROLE_WEB_SEARCH_STRONG_ACTION),
        signal_markers: bare_literals(ROLE_WEB_SEARCH_SIGNAL),
        topic_after_markers: prefix_literals(ROLE_WEB_SEARCH_TOPIC_MARKER),
        topic_before_markers: suffix_literals(ROLE_WEB_SEARCH_TOPIC_MARKER),
        imperative_lead_markers: prefix_literals(ROLE_WEB_SEARCH_IMPERATIVE_LEAD),
        leading_noise: prefix_literals(ROLE_WEB_SEARCH_QUERY_LEADING_NOISE),
        trailing_noise: suffix_literals(ROLE_WEB_SEARCH_QUERY_TRAILING_NOISE),
        source_only: source_literals(ROLE_WEB_SEARCH_SOURCE_ONLY),
        news_subject_markers: bare_literals(ROLE_WEB_SEARCH_NEWS_SUBJECT),
        news_recency_markers: bare_literals(ROLE_WEB_SEARCH_NEWS_RECENCY),
        records_subject_markers: bare_literals(ROLE_WEB_SEARCH_RECORDS_SUBJECT),
        followup_verbs: bare_literals(ROLE_FOLLOWUP_INSTRUCTION_VERB),
        continuation_markers: bare_literals(ROLE_CLAUSE_CONTINUATION_MARKER),
        research_question_prefixes: prefix_literals(ROLE_RESEARCH_QUESTION_OPENER),
        research_modifiers: bare_literals(ROLE_RESEARCH_SUPERLATIVE_MODIFIER),
        research_evidence_domains: bare_literals(ROLE_RESEARCH_EVIDENCE_DOMAIN),
        research_evaluation_domains: bare_literals(ROLE_RESEARCH_EVALUATION_DOMAIN),
        enumeration_prefixes: prefix_literals(ROLE_ENUMERATION_REQUEST_OPENER),
        enumeration_constraint_markers: bare_literals(ROLE_ENUMERATION_CONSTRAINT),
    })
}

/// The literal lead-in (text before the `…` slot) of every prefix-slot form of
/// a role, in lexicon declaration order.
fn prefix_literals(role: &str) -> Vec<&'static str> {
    seed::lexicon()
        .role_word_forms(role)
        .into_iter()
        .filter(|form| form.slot() == Slot::Prefix)
        .map(WordForm::before_slot)
        .collect()
}

/// The literal tail (text after the `…` slot) of every suffix-slot form of a
/// role, in lexicon declaration order.
fn suffix_literals(role: &str) -> Vec<&'static str> {
    seed::lexicon()
        .role_word_forms(role)
        .into_iter()
        .filter(|form| form.slot() == Slot::Suffix)
        .map(WordForm::after_slot)
        .collect()
}

/// The surface text of every bare-slot form of a role, in lexicon declaration
/// order. A meaning's roles apply to all its forms, so we keep only the bare
/// detection tokens and drop any prefix/suffix surfaces the meaning also owns.
fn bare_literals(role: &str) -> Vec<&'static str> {
    seed::lexicon()
        .role_word_forms(role)
        .into_iter()
        .filter(|form| form.slot() == Slot::Bare)
        .map(|form| form.text.as_str())
        .collect()
}

/// The distinct surface words of a role, normalised to a trimmed lowercase key
/// for equality comparison against a cleaned query.
fn source_literals(role: &str) -> Vec<String> {
    seed::lexicon()
        .words_for_role(role)
        .iter()
        .map(|word| word.trim().to_lowercase())
        .collect()
}

fn extract_semantic_web_search_query(normalized: &str) -> Option<String> {
    let markers = markers();
    let has_action = contains_any_search_marker(normalized, &markers.action_markers);
    if !has_action {
        return None;
    }
    let has_strong_action = contains_any_search_marker(normalized, &markers.strong_action_markers);
    if !has_strong_action && !contains_any_search_marker(normalized, &markers.signal_markers) {
        return None;
    }
    for &marker in &markers.topic_after_markers {
        if let Some(index) = normalized.find(marker) {
            let start = index + marker.len();
            if let Some(query) = valid_search_query(&normalized[start..]) {
                return Some(query);
            }
        }
    }
    for &marker in &markers.topic_before_markers {
        if let Some(index) = normalized.find(marker) {
            if let Some(query) = valid_search_query(&normalized[..index]) {
                return Some(query);
            }
        }
    }
    for &marker in &markers.imperative_lead_markers {
        if let Some(index) = normalized.find(marker) {
            let start = index + marker.len();
            if let Some(query) = valid_search_query(&normalized[start..]) {
                return Some(query);
            }
        }
    }
    None
}

fn is_text_extraction_request(normalized: &str) -> bool {
    let vocabulary = seed::operation_vocabulary();
    vocabulary.matches("extract_url", normalized)
        || vocabulary.matches("extract_email", normalized)
        || vocabulary.matches("extract_number", normalized)
}

fn extract_latest_news_search_request(normalized: &str) -> Option<String> {
    let markers = markers();
    if !contains_any_search_marker(normalized, &markers.news_subject_markers)
        || !contains_any_search_marker(normalized, &markers.news_recency_markers)
    {
        return None;
    }
    valid_news_search_query(normalized)
}

/// A verbless "records about a subject" request — "financial records for boeing",
/// "statistics on icas", "записи о boeing", "boeing के रिकॉर्ड".
///
/// It fires only when the prompt names a retrievable record subject
/// ([`ROLE_WEB_SEARCH_RECORDS_SUBJECT`]: records / filings / statements /
/// financials / statistics / dossier and their translations) *and* ties it to a
/// subject with a topic connective ([`ROLE_WEB_SEARCH_TOPIC_MARKER`]: for /
/// about / on / of, о, के बारे में, 关于 …). Requiring both keeps it from
/// stealing bare fact-lookups ("what is a financial record") while routing the
/// "<records> <connective> <subject>" shape to web search without an imperative
/// search verb. The whole prompt is the query, cleaned like a news request.
fn extract_records_information_request(normalized: &str) -> Option<String> {
    let markers = markers();
    if !contains_any_search_marker(normalized, &markers.records_subject_markers) {
        return None;
    }
    let has_topic_marker = markers
        .topic_after_markers
        .iter()
        .chain(markers.topic_before_markers.iter())
        .any(|marker| contains_search_marker(normalized, marker));
    if !has_topic_marker {
        return None;
    }
    valid_news_search_query(normalized)
}

fn extract_implicit_research_question(normalized: &str) -> Option<String> {
    let markers = markers();
    if !starts_with_any(normalized, &markers.research_question_prefixes) {
        return None;
    }
    let padded = format!(" {normalized} ");
    let has_modifier = markers
        .research_modifiers
        .iter()
        .any(|marker| padded.contains(marker));
    let has_evidence_domain = markers
        .research_evidence_domains
        .iter()
        .any(|marker| padded.contains(marker));
    let has_evaluation_domain = markers
        .research_evaluation_domains
        .iter()
        .any(|marker| padded.contains(marker));
    if !(has_modifier || has_evidence_domain && has_evaluation_domain) {
        return None;
    }
    let query = strip_implicit_research_prefix(normalized);
    valid_search_query(query)
}

fn extract_enumeration_research_request(normalized: &str) -> Option<String> {
    let query = strip_enumeration_research_prefix(normalized)?;
    if !looks_like_enumeration_research_query(query) {
        return None;
    }
    valid_search_query(query)
}

fn starts_with_any(value: &str, prefixes: &[&str]) -> bool {
    prefixes.iter().any(|prefix| value.starts_with(prefix))
}

fn strip_implicit_research_prefix(value: &str) -> &str {
    for &prefix in &markers().research_question_prefixes {
        if let Some(stripped) = value.strip_prefix(prefix) {
            return stripped;
        }
    }
    value
}

fn strip_enumeration_research_prefix(value: &str) -> Option<&str> {
    for &prefix in &markers().enumeration_prefixes {
        if let Some(stripped) = value.strip_prefix(prefix) {
            return Some(stripped);
        }
    }
    None
}

fn looks_like_enumeration_research_query(query: &str) -> bool {
    if query.split_whitespace().count() < 3 {
        return false;
    }
    contains_any_search_marker(query, &markers().enumeration_constraint_markers)
}

fn contains_any_search_marker(normalized: &str, markers: &[&str]) -> bool {
    markers
        .iter()
        .any(|marker| contains_search_marker(normalized, marker))
}

fn contains_search_marker(normalized: &str, marker: &str) -> bool {
    if marker.starts_with(' ') || marker.ends_with(' ') {
        let padded = format!(" {normalized} ");
        padded.contains(marker)
    } else {
        normalized.contains(marker)
    }
}

fn valid_search_query(value: &str) -> Option<String> {
    let query = clean_semantic_search_query(value);
    valid_clean_search_query(query)
}

fn valid_news_search_query(value: &str) -> Option<String> {
    let query = clean_search_query(truncate_search_instruction_tail(value));
    valid_clean_search_query(query)
}

fn valid_clean_search_query(query: String) -> Option<String> {
    let query_key = query.to_lowercase();
    if query.is_empty()
        || markers().source_only.iter().any(|word| word == &query_key)
        || normalize_url_candidate(&query).is_some()
    {
        return None;
    }
    Some(query)
}

/// Drop a trailing follow-up instruction clause ("… and summarize who won",
/// "… . Compare their patents") from a query.
///
/// This is a universal boundary algorithm, not a list of memorised fragments: a
/// follow-up clause is one of the lexicon's [`ROLE_FOLLOWUP_INSTRUCTION_VERB`]
/// surfaces sitting immediately after a *boundary* — either sentence
/// punctuation ([`is_sentence_boundary`]) or a run of
/// [`ROLE_CLAUSE_CONTINUATION_MARKER`] words (and / then / and then, walked back
/// so the compound needs no stored surface). The query is cut at the start of
/// the earliest such boundary. A bare verb with no boundary before it is part of
/// the topic and left untouched.
fn truncate_search_instruction_tail(value: &str) -> &str {
    let markers = markers();
    // ASCII-lowercase keeps byte offsets identical to `value` (it only folds
    // A–Z), so indices computed here slice `value` safely; the non-ASCII verbs
    // are already lowercase in the lexicon and unaffected by the fold.
    let lower = value.to_ascii_lowercase();
    let mut cut = value.len();
    for &verb in &markers.followup_verbs {
        let cjk = contains_cjk(verb);
        let mut from = 0;
        while let Some(relative) = lower[from..].find(verb) {
            let start = from + relative;
            let end = start + verb.len();
            from = end;
            // Space-delimited scripts require a whole-token match; CJK verbs have
            // no word boundaries and match as bare substrings.
            if !cjk && (!is_token_start(&lower, start) || !is_token_end(&lower, end)) {
                continue;
            }
            if let Some(boundary) = boundary_before(&lower, start, markers) {
                cut = cut.min(boundary);
            }
        }
    }
    value[..cut].trim()
}

/// Whether `index` begins a whitespace/punctuation-delimited token in `text`
/// (the preceding char is non-alphanumeric, or there is none).
fn is_token_start(text: &str, index: usize) -> bool {
    !text[..index]
        .chars()
        .next_back()
        .is_some_and(char::is_alphanumeric)
}

/// Whether `index` ends a whitespace/punctuation-delimited token in `text` (the
/// following char is non-alphanumeric, or there is none).
fn is_token_end(text: &str, index: usize) -> bool {
    !text[index..]
        .chars()
        .next()
        .is_some_and(char::is_alphanumeric)
}

/// If the text immediately before `verb_start` is a follow-up boundary, return
/// the byte offset at which to cut (the start of the boundary run); otherwise
/// `None`.
fn boundary_before(text: &str, verb_start: usize, markers: &WebSearchMarkers) -> Option<usize> {
    let head = text[..verb_start].trim_end();
    if head.is_empty() {
        // The verb opens the value — there is no preceding clause to split off.
        return None;
    }
    if head.ends_with(is_sentence_boundary) {
        return Some(head.len());
    }
    // Walk back over a run of clause-continuation markers ("and", "then",
    // "and then"); the cut falls at the start of the run.
    let mut cursor = head;
    let mut matched = false;
    loop {
        let trimmed = cursor.trim_end();
        let shortened = markers
            .continuation_markers
            .iter()
            .find(|&&marker| ends_with_token(trimmed, marker))
            .map(|&marker| &trimmed[..trimmed.len() - marker.len()]);
        match shortened {
            Some(rest) => {
                cursor = rest;
                matched = true;
            }
            None => break,
        }
    }
    matched.then(|| cursor.trim_end().len())
}

/// Whether `haystack` ends with `marker` as a whole token. CJK markers match as
/// bare substrings; space-delimited markers require a preceding whitespace (or
/// for the whole string to be exactly the marker).
fn ends_with_token(haystack: &str, marker: &str) -> bool {
    if contains_cjk(marker) {
        haystack.ends_with(marker)
    } else {
        haystack == marker
            || haystack
                .strip_suffix(marker)
                .is_some_and(|head| head.ends_with(char::is_whitespace))
    }
}

fn clean_semantic_search_query(value: &str) -> String {
    let markers = markers();
    let mut query = clean_search_query(truncate_search_instruction_tail(value));
    loop {
        let before = query.clone();
        for &prefix in &markers.leading_noise {
            if let Some(stripped) = query.strip_prefix(prefix) {
                query = clean_search_query(stripped);
            }
        }
        for &suffix in &markers.trailing_noise {
            if let Some(stripped) = query.strip_suffix(suffix) {
                query = clean_search_query(stripped);
            }
        }
        if query == before {
            return query;
        }
    }
}
