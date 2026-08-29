//! Natural-language web-search intent recognition.
//!
//! Every surface cue this recogniser reasons about — explicit prefixes, action
//! verbs, source nouns, query noise, follow-up verbs, and research vocabulary —
//! comes from the language-independent meaning lexicon:
//! `data/seed/meanings-web-search*.lino`, `meanings-web-research.lino`, and
//! `meanings-web-followup.lino`. The handler references those meanings by their
//! semantic *role* (for example [`ROLE_WEB_SEARCH_EXPLICIT_PREFIX`] and
//! [`ROLE_FOLLOWUP_INSTRUCTION_VERB`]) and by each word form's *slot* (prefix,
//! suffix, or bare), never by raw words. Adding a synonym remains a data edit:
//! add a `word`/`description` and the handler reasons about it automatically.
//! Follow-up truncation is a universal boundary algorithm. It detects a clause
//! structurally from an instruction verb immediately preceded by sentence
//! punctuation or a chained clause-continuation marker, rather than memorising
//! the handful of `". compare"`-style fragments the prompts happen to use.

use crate::coding::contains_cjk;
use crate::concepts::{extract_concept_query, lookup_concept_query};
use crate::engine::normalize_prompt;
use crate::seed::{
    self, Slot, ROLE_ASSISTANT_SELF_REFERENCE, ROLE_CAPABILITY_QUERY, ROLE_CAPABILITY_QUERY_MORE,
    ROLE_NON_REFERENTIAL_SUBJECT, ROLE_SELF_INTRODUCTION_REQUEST,
};

use super::web_requests::normalize_url_candidate;
use crate::web_search_markers::{markers, WebSearchMarkers};

/// Keep a schemeless local filename from becoming a synthetic HTTPS host.
///
/// Explicit `http(s)://` URLs remain authoritative even when their host uses an
/// unusual suffix. This boundary only applies to the ambiguous bare-token form.
pub(super) fn probable_local_file_name(candidate: &str) -> bool {
    let Some((_, extension)) = candidate.rsplit_once('.') else {
        return false;
    };
    matches!(
        extension.to_ascii_lowercase().as_str(),
        "txt"
            | "md"
            | "json"
            | "yaml"
            | "yml"
            | "toml"
            | "rs"
            | "py"
            | "js"
            | "ts"
            | "tsx"
            | "jsx"
            | "css"
            | "html"
            | "xml"
            | "csv"
            | "lino"
            | "log"
            | "sh"
    )
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WebSearchQueryKind {
    ExplicitPrefix,
    SemanticAction,
    LatestNews,
    RecordsInformationRequest,
    ImplicitResearchQuestion,
    EnumerationResearchRequest,
    UnresolvedBareTerm,
    UnknownReasoningFallback,
    DocumentOriginalityCheck,
}

impl WebSearchQueryKind {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::ExplicitPrefix => "explicit_prefix",
            Self::SemanticAction => "semantic_action",
            Self::LatestNews => "latest_news",
            Self::RecordsInformationRequest => "records_information_request",
            Self::ImplicitResearchQuestion => "implicit_research_question",
            Self::EnumerationResearchRequest => "enumeration_research_request",
            Self::UnresolvedBareTerm => "unresolved_bare_term",
            Self::UnknownReasoningFallback => "unknown_reasoning_fallback",
            Self::DocumentOriginalityCheck => "document_originality_check",
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
    if let Some(query) = extract_explicit_web_search_query(normalized)
        .or_else(|| extract_explicit_web_search_query(&normalized_words))
    {
        return Some(WebSearchRequest {
            query,
            kind: WebSearchQueryKind::ExplicitPrefix,
        });
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
    if let Some(query) = extract_source_grounded_question(prompt, &normalized_words) {
        return Some(WebSearchRequest {
            query,
            kind: WebSearchQueryKind::ImplicitResearchQuestion,
        });
    }
    if let Some(query) = extract_current_source_information_request(&normalized_words) {
        return Some(WebSearchRequest {
            query,
            kind: WebSearchQueryKind::ImplicitResearchQuestion,
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
    if let Some(query) = extract_current_public_event_question(&normalized_words) {
        return Some(WebSearchRequest {
            query,
            kind: WebSearchQueryKind::ImplicitResearchQuestion,
        });
    }
    if let Some(query) = extract_term_information_request(prompt, &normalized_words) {
        return Some(WebSearchRequest {
            query,
            kind: WebSearchQueryKind::ImplicitResearchQuestion,
        });
    }
    if let Some(query) = extract_implicit_research_question(&normalized_words) {
        return Some(WebSearchRequest {
            query,
            kind: WebSearchQueryKind::ImplicitResearchQuestion,
        });
    }
    extract_externally_verifiable_question(prompt, &normalized_words).map(|query| {
        WebSearchRequest {
            query,
            kind: WebSearchQueryKind::ImplicitResearchQuestion,
        }
    })
}

/// Capability-intent probe (issue #680): the search query a web-search-intent
/// prompt implies — for *any* phrasing, in any supported language — or [`None`]
/// when the prompt carries no web-search intent. This is the same intent cascade
/// the prose handler ([`super::web_requests::try_web_search`]) runs, exposed so
/// the deterministic agentic planner (`crate::agentic_coding::planner`) can route
/// a search request to the advertised search tool instead of answering in prose.
/// The planner and the prose path therefore share one lexicon-driven detector and
/// never drift. `normalized` mirrors what the specialized-handler dispatch passes
/// every handler — the lowercased prompt (see `meta_method_dispatch`).
#[must_use]
pub fn web_search_query_for(prompt: &str) -> Option<String> {
    extract_web_search_request(prompt, &prompt.to_lowercase()).map(|request| request.query)
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

fn extract_semantic_web_search_query(normalized: &str) -> Option<String> {
    let markers = markers();
    let imperative_candidate =
        imperative_lead_candidate(normalized, &markers.imperative_lead_markers, markers);
    let has_imperative_lead = imperative_candidate.is_some();
    let has_action =
        has_imperative_lead || contains_any_search_marker(normalized, &markers.action_markers);
    if !has_action {
        return None;
    }
    let has_strong_action =
        imperative_lead_candidate(normalized, &markers.strong_imperative_lead_markers, markers)
            .is_some()
            || contains_any_search_marker(normalized, &markers.strong_action_markers);
    if !has_strong_action && !contains_any_search_marker(normalized, &markers.signal_markers) {
        return None;
    }
    for &marker in &markers.topic_after_markers {
        if let Some(index) = normalized.find(marker) {
            let start = index + marker.len();
            let topic = &normalized[start..];
            if states_when_to_search(topic) {
                continue;
            }
            if let Some(query) = valid_search_query(topic) {
                return Some(query);
            }
        }
    }
    for &marker in &markers.topic_before_markers {
        if let Some(index) = normalized.find(marker)
            && let Some(query) = valid_search_query(&normalized[..index]) {
                return Some(query);
            }
    }
    if let Some(query) = imperative_candidate
        .filter(|candidate| !states_when_to_search(candidate))
        .and_then(valid_search_query)
    {
        return Some(query);
    }
    None
}

/// Return the typed argument of an imperative search lead.
///
/// The lead may open the prompt directly, or follow a seeded question opener
/// ("Where can I find …") or an external-source phrase ("On Wikipedia, search
/// …"). Arbitrary mid-sentence occurrences do not qualify, so prose such as
/// "learn from popular Google searches" cannot be mistaken for a command.
fn imperative_lead_candidate<'a>(
    normalized: &'a str,
    leads: &[&str],
    markers: &WebSearchMarkers,
) -> Option<&'a str> {
    for &lead in leads {
        if let Some(candidate) = normalized.strip_prefix(lead) {
            return Some(candidate);
        }
        let Some(index) = normalized.find(lead) else {
            continue;
        };
        let introducer = &normalized[..index];
        let question_led = starts_with_any(normalized, &markers.research_question_prefixes);
        let source_led = contains_any_search_marker(introducer, &markers.source_markers);
        if question_led || source_led {
            return Some(&normalized[index + lead.len()..]);
        }
    }
    None
}

/// Extract the topic from an interrogative grounded in an explicitly named web
/// source. The frame is question shape + external source + topic connective, so
/// unseen wording shares the same route without a sentence template.
fn extract_source_grounded_question(prompt: &str, normalized: &str) -> Option<String> {
    let markers = markers();
    if !question_is_interrogative(prompt, normalized)
        || !contains_any_search_marker(normalized, &markers.source_medium_markers)
    {
        return None;
    }
    extract_topic_subject(normalized)
}

/// Extract a request for fresh information from a named external source. The
/// independent evidence is source + recency + topic shape; no request-specific
/// sentence template or English action verb is needed.
fn extract_current_source_information_request(normalized: &str) -> Option<String> {
    let markers = markers();
    if !contains_any_search_marker(normalized, &markers.source_medium_markers)
        || !contains_any_search_marker(normalized, &markers.news_recency_markers)
        || !contains_any_search_marker(normalized, &markers.information_markers)
    {
        return None;
    }
    extract_topic_subject(normalized)
}

/// Recover the subject selected by a seed-defined topic connective, supporting
/// both prepositions (prefix slot: "about X") and postpositions (suffix slot:
/// "X के बारे में") through the same shape parser.
fn extract_topic_subject(normalized: &str) -> Option<String> {
    let markers = markers();
    for &marker in &markers.topic_after_markers {
        if let Some(index) = normalized.find(marker) {
            let topic = &normalized[index + marker.len()..];
            if states_when_to_search(topic) {
                continue;
            }
            return valid_search_query(topic);
        }
    }
    for &marker in &markers.topic_before_markers {
        if let Some(index) = normalized.find(marker) {
            return valid_search_query(&normalized[..index]);
        }
    }
    None
}

fn extract_explicit_web_search_query(normalized: &str) -> Option<String> {
    let markers = markers();
    for &prefix in &markers.explicit_prefixes {
        if let Some(query) = normalized.strip_prefix(prefix)
            && let Some(query) = valid_search_query(query) {
                return Some(query);
            }
    }
    for &(prefix, suffix) in &markers.explicit_circumfixes {
        if let Some(candidate) = normalized.strip_prefix(prefix).and_then(|rest| {
            rest.strip_suffix(suffix).or_else(|| {
                rest.trim_end_matches(is_url_trailing_punctuation)
                    .strip_suffix(suffix)
            })
        })
            && let Some(query) = valid_search_query(candidate) {
                return Some(query);
            }
    }
    for &suffix in &markers.explicit_suffixes {
        if let Some(query) = normalized.strip_suffix(suffix).or_else(|| {
            normalized
                .trim_end_matches(is_url_trailing_punctuation)
                .strip_suffix(suffix)
        })
            && let Some(query) = valid_search_query(query) {
                return Some(query);
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

/// A question asking which public events are currently active — "which current
/// hackathons", "Какие хакатоны сейчас проходят?", "哪些黑客松现在举行".
///
/// It fires only when a research-question opener combines with a public-event
/// subject ([`ROLE_WEB_SEARCH_PUBLIC_EVENT_SUBJECT`]) and a freshness marker
/// ([`ROLE_WEB_SEARCH_NEWS_RECENCY`]). The subject term is then cleaned with the
/// same query-noise rules as semantic web searches.
fn extract_current_public_event_question(normalized: &str) -> Option<String> {
    let markers = markers();
    if !starts_with_any(normalized, &markers.research_question_prefixes) {
        return None;
    }
    if !contains_any_search_marker(normalized, &markers.public_event_subject_markers)
        || !contains_any_search_marker(normalized, &markers.news_recency_markers)
    {
        return None;
    }
    valid_search_query(strip_implicit_research_prefix(normalized))
}

/// A request to be told about a public term, in every slot form the lexicon
/// declares for [`ROLE_TERM_INFORMATION_REQUEST_OPENER`].
///
/// Word order is a property of the language, not of the intent: English and
/// Russian put the opener first ("tell me about …"), Hindi puts it last
/// ("… के बारे में बताओ"), and Chinese can wrap the term ("给出 … 的 … 背景").
/// Consulting only [`Slot::Prefix`] forms therefore made the recognizer
/// structurally unable to route verb-final and circumfix phrasings, however many
/// surfaces the seed learned (issue #701). All four slot forms are read here, so
/// a learned surface changes behaviour purely as data.
fn extract_term_information_request(prompt: &str, normalized: &str) -> Option<String> {
    if concept_lookup_resolves(prompt) || term_information_prompt_is_local_context(normalized) {
        return None;
    }
    let markers = markers();
    let candidates = markers
        .term_information_prefixes
        .iter()
        .filter_map(|prefix| normalized.strip_prefix(prefix))
        .chain(
            markers
                .term_information_suffixes
                .iter()
                .filter_map(|suffix| normalized.strip_suffix(suffix)),
        )
        .chain(
            markers
                .term_information_circumfixes
                .iter()
                .filter_map(|(before, after)| {
                    normalized
                        .strip_prefix(before)
                        .and_then(|rest| rest.strip_suffix(after))
                }),
        );
    for candidate in candidates {
        if term_information_query_is_local_context(candidate) {
            return None;
        }
        if let Some(query) = valid_search_query(candidate) {
            return Some(query);
        }
    }
    None
}

fn concept_lookup_resolves(prompt: &str) -> bool {
    extract_concept_query(prompt)
        .as_ref()
        .is_some_and(|query| lookup_concept_query(query).is_some())
}

fn term_information_prompt_is_local_context(normalized: &str) -> bool {
    let lexicon = seed::lexicon();
    lexicon.mentions_role(ROLE_SELF_INTRODUCTION_REQUEST, normalized)
        || lexicon.mentions_role(ROLE_CAPABILITY_QUERY, normalized)
        || lexicon.mentions_role(ROLE_CAPABILITY_QUERY_MORE, normalized)
}

fn term_information_query_is_local_context(query: &str) -> bool {
    let query = clean_search_query(query).to_lowercase();
    let lexicon = seed::lexicon();
    lexicon.mentions_role(ROLE_NON_REFERENTIAL_SUBJECT, &query)
        || lexicon.mentions_role(ROLE_ASSISTANT_SELF_REFERENCE, &query)
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

/// Reasoning-driven fallback for the *class* of externally verifiable questions.
///
/// The seed-vocabulary path above ([`extract_implicit_research_question`]) only
/// fires when a question happens to combine a research opener with a memorised
/// research modifier or evidence/evaluation domain. That still leans on a stored
/// word list, so it cannot, on its own, cover the open-ended class the maintainer
/// asked for: *any* question about a real-world product, service, or organisation
/// whose current facts (pricing, availability, features, history) live on the
/// public web rather than in the solver's seed memory.
///
/// This recogniser closes that gap by reasoning about the *referent* instead of
/// the topic vocabulary. It routes a prompt to external research when three
/// structural conditions all hold:
///
/// 1. the prompt is *interrogative* — it opens with a seeded question opener
///    (any language) or ends with a question mark (`?` / fullwidth `？`);
/// 2. it names a *referential external entity* — a Latin token written with
///    interior capitalisation ([`prompt_names_engineered_brand`]), the
///    orthographic signature of an engineered brand/product name (`ChatGPT`,
///    `OpenAI`, `GitHub`, `iPhone`, `TypeScript`); and
/// 3. the solver cannot answer it from local memory — it is neither a seeded
///    concept lookup nor a self-introduction / capability / non-referential
///    subject question.
///
/// Because interior capitalisation is a property of the Latin brand token itself,
/// the rule fires identically whether that token is embedded in English,
/// Cyrillic, Devanagari, or CJK context — so the entire multilingual class is
/// covered by one language-independent structural test, with no per-product or
/// per-language vocabulary to maintain.
fn extract_externally_verifiable_question(prompt: &str, normalized: &str) -> Option<String> {
    if !question_is_interrogative(prompt, normalized) {
        return None;
    }
    if !prompt_names_engineered_brand(prompt) {
        return None;
    }
    // Never poach a prompt the solver can already resolve locally: a seeded
    // concept or a self-introduction / capability question stays with its own
    // handler.
    if concept_lookup_resolves(prompt) || term_information_prompt_is_local_context(normalized) {
        return None;
    }
    // The residual subject, once the question opener is removed, must be a real
    // external topic — not the assistant itself or a non-referential pronoun
    // ("does it …", "do you …").
    let query = strip_implicit_research_prefix(normalized);
    if term_information_query_is_local_context(query) {
        return None;
    }
    valid_search_query(query)
}

/// A prompt is interrogative when it opens with any seeded research question
/// opener (matched on the punctuation-stripped `normalized` form) or the raw
/// prompt ends with a question mark — the ASCII `?` or the fullwidth `？` a CJK
/// prompt uses.
fn question_is_interrogative(prompt: &str, normalized: &str) -> bool {
    starts_with_any(normalized, &markers().research_question_prefixes)
        || prompt.trim_end().ends_with(['?', '？'])
}

/// True when the prompt contains a Latin token written with *interior*
/// capitalisation: a lower-case Latin letter immediately followed by an
/// upper-case Latin letter within a single token. That adjacency is the
/// orthographic signature of an engineered brand/product name — `ChatGPT`,
/// `OpenAI`, `GitHub`, `iPhone`, `TypeScript`, `macOS`.
///
/// The test is deliberately narrow. All-caps acronyms (`BSD`, `ML`, `IIR`,
/// `USD`), plain capitalised proper nouns (`Claude`, `Tesla`, `Wikipedia`), and
/// Title-Cased word sequences (`Hive Mind`) do *not* match — matching them would
/// poach concept-lookup, coding, and unknown-reasoning prompts. Those relevant to
/// external research are already reached by the seed commercial vocabulary or by
/// their own local handlers.
fn prompt_names_engineered_brand(prompt: &str) -> bool {
    let mut prev_is_lower_latin = false;
    for character in prompt.chars() {
        if prev_is_lower_latin && character.is_ascii_uppercase() {
            return true;
        }
        prev_is_lower_latin = character.is_ascii_lowercase();
    }
    false
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

/// Whether the text a topic marker introduces says *when* to search rather than
/// *what* to search for.
///
/// A topic marker is unanchored on purpose — a subject can be named anywhere in
/// a prompt — so the text after it is whatever the caller wrote next, and that
/// is not always a subject. Hive Mind's harness ends every objective with
/// standing policy, and *"Use web research when it materially improves factual
/// accuracy."* pairs a search action with the condition under which to take it.
/// Reading the condition as the subject searched the open web for *"when it
/// materially improves factual accuracy …"* and spent the agent's turn on it,
/// while the objective above went unread (issue #1066).
///
/// The tell is the seed-declared policy lead opening the topic, the same one
/// that tells a rule about running commands from an order to run one
/// (issue #907). A condition is never a subject, so a topic that opens with a
/// lead names nothing to look up and this marker is passed over — a later marker,
/// or a later extractor, may still find the real subject.
fn states_when_to_search(topic: &str) -> bool {
    let Some(condition) = seed::caller_context_vocabulary().policy_lead_clause(topic) else {
        return false;
    };
    opens_with_non_referential_subject(condition)
}

/// Whether `clause` opens with a subject that refers back to the conversation
/// instead of naming something.
///
/// This is what tells the rule from the request when both open with the same
/// word. *"Use web research when **it** materially improves factual accuracy"*
/// and *"Look up when **the next release** ships"* are both `when` clauses; only
/// the first has a subject that names nothing, because *it* is the act of
/// researching — the thing the caller is legislating about. The second names a
/// release, so the clause is the object of the lookup and the search is real.
///
/// Only whole-word ([`Slot::Bare`]) surfaces count, so a topic that merely
/// *begins* with such a word ("this american war, explained") still searches.
fn opens_with_non_referential_subject(clause: &str) -> bool {
    let Some(subject) = clause.split_whitespace().next() else {
        return false;
    };
    seed::lexicon()
        .role_word_forms(ROLE_NON_REFERENTIAL_SUBJECT)
        .iter()
        .any(|form| form.slot() == Slot::Bare && subject == form.text)
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
