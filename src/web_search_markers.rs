//! The web-search cue projection: every surface marker the recogniser reads,
//! resolved out of the meaning lexicon by semantic role and slot.
//!
//! [`crate::solver_handlers::web_search_intent`] decides *whether a prompt asks
//! for a web search and what it asks about*; this module answers the narrower
//! question that decision rests on — *which surfaces does the seed declare for
//! each role?* The two are separable: the projection has no opinion about a
//! request, and the recogniser has no opinion about the lexicon's shape. Keeping
//! the projection here means adding a role reads as a data-facing edit rather
//! than as growth of a request handler (issue #918's core boundary).
//!
//! Built once and cached: because [`seed::lexicon`] returns a `'static`
//! reference, the projected literals are themselves `'static` and need no
//! allocation beyond the backing vectors.

use std::sync::OnceLock;

use crate::seed::{
    self, ROLE_CLAUSE_CONTINUATION_MARKER, ROLE_ENUMERATION_CONSTRAINT,
    ROLE_ENUMERATION_REQUEST_OPENER, ROLE_FOLLOWUP_INSTRUCTION_VERB,
    ROLE_RESEARCH_EVALUATION_DOMAIN, ROLE_RESEARCH_EVIDENCE_DOMAIN, ROLE_RESEARCH_QUESTION_OPENER,
    ROLE_RESEARCH_SUPERLATIVE_MODIFIER, ROLE_TERM_INFORMATION_REQUEST_OPENER, ROLE_WEB_MEDIUM,
    ROLE_WEB_SEARCH_ACTION, ROLE_WEB_SEARCH_EXPLICIT_PREFIX, ROLE_WEB_SEARCH_IMPERATIVE_LEAD,
    ROLE_WEB_SEARCH_NEWS_RECENCY, ROLE_WEB_SEARCH_NEWS_SUBJECT,
    ROLE_WEB_SEARCH_PUBLIC_EVENT_SUBJECT, ROLE_WEB_SEARCH_QUERY_LEADING_NOISE,
    ROLE_WEB_SEARCH_QUERY_TRAILING_NOISE, ROLE_WEB_SEARCH_RECORDS_SUBJECT, ROLE_WEB_SEARCH_SIGNAL,
    ROLE_WEB_SEARCH_SOURCE_ONLY, ROLE_WEB_SEARCH_STRONG_ACTION, ROLE_WEB_SEARCH_TOPIC_MARKER, Slot,
    WordForm,
};

/// Every surface cue the web-search recogniser reasons about.
///
/// The cues are projected out of the meaning lexicon by role and slot, then
/// built once and cached: because [`seed::lexicon`] returns a `'static`
/// reference, the projected literals are themselves `'static` and need no
/// allocation beyond the backing vectors.
pub struct WebSearchMarkers {
    /// Lead-ins of an explicit "search X for …" command (prefix slot).
    pub explicit_prefixes: Vec<&'static str>,
    /// Tails of an explicit topic-interest/search template (suffix slot).
    pub explicit_suffixes: Vec<&'static str>,
    /// Bracketing explicit topic-interest/search templates (circumfix slot).
    pub explicit_circumfixes: Vec<(&'static str, &'static str)>,
    /// Bare search verbs that signal an action is requested.
    pub action_markers: Vec<&'static str>,
    /// The subset of action verbs strong enough to stand without a source noun.
    pub strong_action_markers: Vec<&'static str>,
    /// Strong action verbs whose typed argument follows in a prefix slot.
    pub strong_imperative_lead_markers: Vec<&'static str>,
    /// Source/topic nouns that corroborate a weak action verb.
    pub signal_markers: Vec<&'static str>,
    /// Topic connectives whose object follows them ("about …", "о …").
    pub topic_after_markers: Vec<&'static str>,
    /// Topic connectives whose object precedes them ("… के बारे में").
    pub topic_before_markers: Vec<&'static str>,
    /// Imperative search leads whose query follows them ("search for …").
    pub imperative_lead_markers: Vec<&'static str>,
    /// Politeness / determiner noise stripped from the front of a query.
    pub leading_noise: Vec<&'static str>,
    /// Source/medium noise stripped from the end of a query.
    pub trailing_noise: Vec<&'static str>,
    /// Bare source words that are not, on their own, a valid query.
    pub source_only: Vec<String>,
    /// External-source markers that may introduce a typed search action.
    pub source_markers: Vec<&'static str>,
    /// Boundary-preserving web-medium markers used as evidence in semantic frames.
    pub source_medium_markers: Vec<&'static str>,
    /// Information-object markers, excluding names of external sources.
    pub information_markers: Vec<&'static str>,
    /// News/headline subject markers for bare latest-news requests.
    pub news_subject_markers: Vec<&'static str>,
    /// Freshness markers that pair with news/headline subjects.
    pub news_recency_markers: Vec<&'static str>,
    /// Records/documents subject nouns for verbless "records about X" requests.
    pub records_subject_markers: Vec<&'static str>,
    /// Public event category nouns for current-event research questions.
    pub public_event_subject_markers: Vec<&'static str>,
    /// Verbs that open a follow-up instruction clause ("compare", "summarize").
    pub followup_verbs: Vec<&'static str>,
    /// Conjunctions/adverbs that, like punctuation, mark a clause boundary.
    pub continuation_markers: Vec<&'static str>,
    /// Tell-me-about openers whose object is a public term.
    pub term_information_prefixes: Vec<&'static str>,
    /// Tell-me-about closers of verb-final languages ("… के बारे में बताओ").
    pub term_information_suffixes: Vec<&'static str>,
    /// Tell-me-about frames that wrap the term on both sides.
    pub term_information_circumfixes: Vec<(&'static str, &'static str)>,
    /// Question openers of an implicit research request ("what is …", "is there …").
    pub research_question_prefixes: Vec<&'static str>,
    /// Superlative/recency modifiers that make a question researchable.
    pub research_modifiers: Vec<&'static str>,
    /// Evidence nouns (dataset, paper, subscription, pricing …) of a research question.
    pub research_evidence_domains: Vec<&'static str>,
    /// Evaluation nouns (validation, comparison, discount, price …) of a question.
    pub research_evaluation_domains: Vec<&'static str>,
    /// Openers of an enumeration research request ("list all …").
    pub enumeration_prefixes: Vec<&'static str>,
    /// Constraint connectives that make an enumeration researchable.
    pub enumeration_constraint_markers: Vec<&'static str>,
}

/// Build (once) the marker projection from the meaning lexicon.
pub fn markers() -> &'static WebSearchMarkers {
    static CACHE: OnceLock<WebSearchMarkers> = OnceLock::new();
    CACHE.get_or_init(|| WebSearchMarkers {
        explicit_prefixes: prefix_literals(ROLE_WEB_SEARCH_EXPLICIT_PREFIX),
        explicit_suffixes: suffix_literals(ROLE_WEB_SEARCH_EXPLICIT_PREFIX),
        explicit_circumfixes: circumfix_literals(ROLE_WEB_SEARCH_EXPLICIT_PREFIX),
        action_markers: bare_literals(ROLE_WEB_SEARCH_ACTION),
        strong_action_markers: bare_literals(ROLE_WEB_SEARCH_STRONG_ACTION),
        strong_imperative_lead_markers: prefix_literals(ROLE_WEB_SEARCH_STRONG_ACTION),
        signal_markers: bare_literals(ROLE_WEB_SEARCH_SIGNAL),
        topic_after_markers: prefix_literals(ROLE_WEB_SEARCH_TOPIC_MARKER),
        topic_before_markers: suffix_literals(ROLE_WEB_SEARCH_TOPIC_MARKER),
        imperative_lead_markers: prefix_literals(ROLE_WEB_SEARCH_IMPERATIVE_LEAD),
        leading_noise: prefix_literals(ROLE_WEB_SEARCH_QUERY_LEADING_NOISE),
        trailing_noise: suffix_literals(ROLE_WEB_SEARCH_QUERY_TRAILING_NOISE),
        source_only: source_literals(ROLE_WEB_SEARCH_SOURCE_ONLY),
        source_markers: bare_literals(ROLE_WEB_SEARCH_SOURCE_ONLY),
        source_medium_markers: bare_literals(ROLE_WEB_MEDIUM),
        information_markers: information_literals(),
        news_subject_markers: bare_literals(ROLE_WEB_SEARCH_NEWS_SUBJECT),
        news_recency_markers: bare_literals(ROLE_WEB_SEARCH_NEWS_RECENCY),
        records_subject_markers: bare_literals(ROLE_WEB_SEARCH_RECORDS_SUBJECT),
        public_event_subject_markers: bare_literals(ROLE_WEB_SEARCH_PUBLIC_EVENT_SUBJECT),
        followup_verbs: bare_literals(ROLE_FOLLOWUP_INSTRUCTION_VERB),
        continuation_markers: bare_literals(ROLE_CLAUSE_CONTINUATION_MARKER),
        term_information_prefixes: prefix_literals(ROLE_TERM_INFORMATION_REQUEST_OPENER),
        term_information_suffixes: suffix_literals(ROLE_TERM_INFORMATION_REQUEST_OPENER),
        term_information_circumfixes: circumfix_literals(ROLE_TERM_INFORMATION_REQUEST_OPENER),
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

/// The literal pair around every circumfix-slot form of a role, in lexicon
/// declaration order.
fn circumfix_literals(role: &str) -> Vec<(&'static str, &'static str)> {
    seed::lexicon()
        .role_word_forms(role)
        .into_iter()
        .filter(|form| form.slot() == Slot::Circumfix)
        .map(|form| (form.before_slot(), form.after_slot()))
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

fn information_literals() -> Vec<&'static str> {
    let sources = bare_literals(ROLE_WEB_SEARCH_SOURCE_ONLY);
    bare_literals(ROLE_WEB_SEARCH_SIGNAL)
        .into_iter()
        .filter(|marker| !sources.iter().any(|source| source.trim() == marker.trim()))
        .collect()
}
