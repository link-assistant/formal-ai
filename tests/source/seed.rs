//! Universal Links Notation seed shared by every formal-ai interface.
//!
//! `data/seed/*.lino` is the canonical source of truth for the agent's
//! multilingual responses, concept knowledge base, tool registry, language
//! detection rules, prompt-question patterns, and metadata. The browser
//! worker, the Rust library, the CLI, the HTTP server, and the Telegram bot
//! all read from the same files.
//!
//! In the browser the files are fetched at runtime by `seed_loader.js`. In
//! Rust they are compiled into the binary with [`include_str!`] so even
//! offline builds expose the same data. The two implementations stay
//! consistent through `scripts/sync-seed.sh`, which mirrors `data/seed/` into
//! `src/web/seed/` for GitHub Pages deployment.
//!
//! See `VISION.md` and `REQUIREMENTS.md` (R97-R104) for the universal
//! data-driven configuration goal.
//!
//! # Stability
//!
//! The parser is intentionally tiny — Links Notation files in this repo are
//! shallow trees of `name "value"` lines with two-space indentation. The
//! schema for each category is documented in the corresponding `.lino` file.

mod brainstorm;
mod coreference;
mod embedded;
mod facts;
mod meanings;
mod operation_vocabulary;
pub(crate) mod parser;
mod personas;
mod projects;
mod roles;
mod summary_topics;

use std::collections::BTreeMap;

use parser::{
    escape_value, find_closing_quote, parse_codepoint, parse_lino, split_pipe_list, unescape_value,
    LinoNode,
};

pub use brainstorm::{brainstorm_seeds, BrainstormCategory, BrainstormSeeds};
pub use coreference::{coreference_seeds, Antecedent, CoreferenceSeeds, Pronoun};
pub use embedded::{
    seed_files, AGENT_INFO_LINO, BRAINSTORM_SEEDS_LINO, CONCEPTS_LINO, CONCEPT_CONTEXTS_LINO,
    COREFERENCE_LINO, DEMO_DIALOGS_LINO, ENVIRONMENTS_LINO, FACTS_LINO, GREETINGS_LINO,
    HELLO_WORLD_PROGRAMS_LINO, IDENTITY_LINO, INTENT_ROUTING_LINO, LANGUAGE_DETECTION_LINO,
    MEANINGS_CALENDAR_LINO, MEANINGS_FACTS_LINO, MEANINGS_LINO, MEANINGS_SOFTWARE_PROJECT_LINO,
    MEANINGS_UNITS_LINO, MEANING_FILES, MULTILINGUAL_RESPONSES_LINO, OPERATION_VOCABULARY_LINO,
    PERSONAS_LINO, PROGRAM_PLAN_RULES_LINO, PROJECTS_LINO, PROMPT_PATTERNS_LINO,
    SELF_IMPROVEMENT_LOOP_LINO, SUMMARY_TOPICS_LINO, TOOLS_LINO,
};
pub use facts::{facts, FactRecord, LocalizedFact};
pub use meanings::{lexicon, Lexeme, Lexicon, Meaning, Slot, WordForm};
pub use operation_vocabulary::{
    operation_vocabulary, OperationLanguageForms, OperationTrigger, OperationVocabulary,
};
pub use personas::{persona_seeds, Persona, PersonaSeeds, PersonaTopic};
pub use projects::{
    projects_registry, LocalizedProject, ProjectRecord, ProjectStatement, ProjectsRegistry,
};
pub use roles::{
    ROLE_ANSWER_RATIONALE_LEAD, ROLE_ARCHITECTURE_CONCEPT, ROLE_ARITHMETIC_OPERATOR_WORD,
    ROLE_ASSISTANT_MECHANISM_INQUIRY, ROLE_ASSISTANT_SELF_REFERENCE,
    ROLE_BEHAVIOR_RULE_EDIT_DIRECTIVE, ROLE_BINARY_RELATION_PROPERTY,
    ROLE_CALCULATION_BASIS_REFERENCE, ROLE_CALCULATION_DOMAIN_TERM, ROLE_CALCULATION_REQUEST_CUE,
    ROLE_CALCULATION_RESULT_QUERY_CUE, ROLE_CALCULATOR_TOOL_NAME, ROLE_CALENDAR_DAY_REFERENCE,
    ROLE_CALENDAR_DIRECTION_NEXT, ROLE_CALENDAR_DIRECTION_PREVIOUS, ROLE_CALENDAR_EVENT,
    ROLE_CALENDAR_QUESTION, ROLE_CALENDAR_SCHEDULE_ACTION, ROLE_CALENDAR_TIME,
    ROLE_CALENDAR_TIMEZONE_ALIAS, ROLE_CALENDAR_TODAY, ROLE_CALENDAR_WEEKDAY, ROLE_CAPABILITY_QUERY, ROLE_CAPABILITY_QUERY_MORE,
    ROLE_CARDINAL_NUMBER_WORD, ROLE_CAUSAL_INTERROGATIVE, ROLE_CIRCULAR_JOKE_PHRASE,
    ROLE_CLARIFICATION_REQUEST, ROLE_CLAUSE_CONTINUATION_MARKER, ROLE_CODE_METHOD_NOUN,
    ROLE_COMMON_TYPO, ROLE_COMPARISON_DIFFERENCE_CUE, ROLE_COMPARISON_TABLE_NOUN,
    ROLE_COMPARISON_TABLE_TRIGGER, ROLE_COMPOSITIONAL_GENITIVE_HEAD, ROLE_COMPOSITIONAL_LEMMA,
    ROLE_COMPOSITIONAL_PHRASE, ROLE_COMPOUNDING_ACTION_CUE, ROLE_COMPOUNDING_FREQUENCY_CUE,
    ROLE_CONVERSATION_REFERENCE, ROLE_CONVERSATION_SUMMARY_COURTESY,
    ROLE_CONVERSATION_SUMMARY_DIRECTIVE, ROLE_CONVERSATION_SUMMARY_PHRASE,
    ROLE_CONVERSATION_TOPIC_OPENER, ROLE_CONVERSION_ACTION_CUE, ROLE_CURRENCY_EUR_REFERENCE,
    ROLE_CURRENCY_RUB_REFERENCE, ROLE_CURRENCY_USD_REFERENCE, ROLE_DEFINITION_ARTIFACT_REQUEST,
    ROLE_DEFINITION_COMMAND, ROLE_DEFINITION_MERGE_ACTION, ROLE_DEFINITION_MERGE_MARKER,
    ROLE_DEFINITION_MERGE_TAIL_BOUNDARY, ROLE_DETAIL_MODIFIER, ROLE_ENUMERATION_CONSTRAINT,
    ROLE_ENUMERATION_REQUEST_OPENER, ROLE_EXCHANGE_RATE_REFERENCE, ROLE_EXPLANATION_REQUEST_LEAD,
    ROLE_FACT_RELATION, ROLE_FEATURE_ACTION_ARITHMETIC, ROLE_FEATURE_ACTION_PLANNING,
    ROLE_FEATURE_CAPABILITY_ALIAS, ROLE_FEATURE_CAPABILITY_QUESTION, ROLE_FINAL_AMOUNT_REFERENCE,
    ROLE_FOLLOWUP_INSTRUCTION_VERB, ROLE_GAME_TRACKER_DOMAIN, ROLE_GAME_TRACKER_MECHANIC,
    ROLE_HELLO_WORLD_REFERENCE, ROLE_HTTP_FETCH, ROLE_IMPLEMENTATION_LANGUAGE_NOUN,
    ROLE_IMPLEMENTATION_LANGUAGE_PREPOSITION, ROLE_INTEREST_CUE, ROLE_INTERROGATIVE_OPENER,
    ROLE_INVESTMENT_CUE, ROLE_KNOWLEDGE_INVENTORY_INTERROGATIVE, ROLE_KNOWLEDGE_INVENTORY_NOUN,
    ROLE_KNOWLEDGE_INVENTORY_PHRASE, ROLE_KNOWLEDGE_POSSESSION, ROLE_LINKS_NOTATION_FORMAT,
    ROLE_LIVE_RATE_FRESHNESS_CUE, ROLE_LOCAL_SHELL_REQUEST_CUE, ROLE_MATH_FUNCTION_NAME,
    ROLE_MEASUREMENT_UNIT, ROLE_MECHANISM_INQUIRY, ROLE_MECHANISM_PREDICATE,
    ROLE_NETWORK_CAPABILITY_CUE, ROLE_NONDETERMINISTIC_MARKER, ROLE_NON_REFERENTIAL_SUBJECT,
    ROLE_OPERATING_PRINCIPLE, ROLE_OUTPUT_DISPLAY_REQUEST, ROLE_PHYSICAL_ACTION_TRIGGER,
    ROLE_PHYSICAL_DIMENSION, ROLE_PLAYWRIGHT_SCRIPT_CUE, ROLE_PLAYWRIGHT_TOOL_NAME,
    ROLE_POLITENESS_CUE, ROLE_PRIOR_ANSWER_REFERENCE, ROLE_PROCEDURAL_REQUEST,
    ROLE_PROCEDURAL_TASK_MODIFIER, ROLE_PROGRAM_ARTIFACT, ROLE_PROGRAM_GENUS, ROLE_PROGRAM_KIND,
    ROLE_PROGRAM_LANGUAGE_ALIAS, ROLE_PROGRAM_MODIFICATION, ROLE_PROGRAM_REQUEST,
    ROLE_PROGRAM_SYNTHESIS_ACTION, ROLE_PROGRAM_SYNTHESIS_DOMAIN, ROLE_PROGRAM_SYNTHESIS_SIGNAL,
    ROLE_PROGRAM_SYNTHESIS_SUBJECT, ROLE_PROGRAM_SYNTHESIS_TASK, ROLE_PROGRAM_TASK_ALIAS,
    ROLE_PROOF_CLAIM_SCAFFOLD, ROLE_PROOF_CONCEPT_DETERMINISM, ROLE_PROOF_CONCEPT_GODEL,
    ROLE_PROOF_DIRECTIVE, ROLE_PROOF_MARKER, ROLE_PROOF_REQUEST_LEAD, ROLE_QUANTITY_CONVERSION_CUE,
    ROLE_RESEARCH_CRITERION, ROLE_RESEARCH_EVALUATION_DOMAIN, ROLE_RESEARCH_EVIDENCE_DOMAIN,
    ROLE_RESEARCH_PROMPT_SIGNAL, ROLE_RESEARCH_QUESTION_OPENER, ROLE_RESEARCH_SUPERLATIVE_MODIFIER,
    ROLE_RULE_LISTING_PHRASE, ROLE_RULE_LISTING_REQUEST, ROLE_RULE_LISTING_SCOPE,
    ROLE_RULE_LISTING_SUBJECT, ROLE_SCRIPT_AUTHORING_VERB, ROLE_SCRIPT_OR_CODE_ARTIFACT,
    ROLE_SELF_FACT_QUERY, ROLE_SELF_INTRODUCTION_REQUEST, ROLE_SHELL_CAPABILITY_CUE,
    ROLE_SKILL_TEACHING_RESPONSE_VERB, ROLE_SKILL_TEACHING_TRIGGER_LEAD, ROLE_SKILL_WHEN_THEN_PAIR,
    ROLE_SOFTWARE_APPROVAL_TRIGGER, ROLE_SOFTWARE_ARTIFACT_KIND, ROLE_SOFTWARE_AUTHORING_ACTION,
    ROLE_SOFTWARE_BASH_COMMAND, ROLE_SOFTWARE_DELIVERY_MODE, ROLE_SOFTWARE_FEATURE,
    ROLE_SOFTWARE_FOLLOWUP_DEMONSTRATION, ROLE_SOFTWARE_FOLLOWUP_EXECUTION,
    ROLE_SOFTWARE_FOLLOWUP_VERIFICATION, ROLE_SOFTWARE_IMPLEMENTATION_LANGUAGE,
    ROLE_SOFTWARE_REQUIREMENT_CATEGORY, ROLE_SOFTWARE_STEP_GRANULARITY,
    ROLE_SUMMARY_CLASSIFICATION_CUE, ROLE_TOOL_ARGUMENT_MARKER, ROLE_TOOL_INVOCATION_CUE,
    ROLE_TOPIC_SCAN_STOP_WORD, ROLE_TRANSLATION_ACTION, ROLE_TRANSLATION_INTO_MARKER,
    ROLE_TRANSLATION_OBJECT_MARKER, ROLE_TRANSLATION_PROPERTY, ROLE_TRANSLATION_SOURCE_MARKER,
    ROLE_TRANSLATION_TARGET_DIRECTION, ROLE_TRANSLATION_TARGET_MARKER,
    ROLE_TRANSLATION_UNQUOTED_FRAME, ROLE_URL_NAVIGATE, ROLE_VULGAR_CONTENT_MARKER,
    ROLE_WEB_MEDIUM, ROLE_WEB_SEARCH_ACTION, ROLE_WEB_SEARCH_EXPLICIT_PREFIX,
    ROLE_WEB_SEARCH_HISTORY_SIGNAL, ROLE_WEB_SEARCH_IMPERATIVE_LEAD, ROLE_WEB_SEARCH_NEWS_RECENCY,
    ROLE_WEB_SEARCH_NEWS_SUBJECT, ROLE_WEB_SEARCH_QUERY_LEADING_NOISE,
    ROLE_WEB_SEARCH_QUERY_TRAILING_NOISE, ROLE_WEB_SEARCH_SIGNAL, ROLE_WEB_SEARCH_SOURCE_ONLY,
    ROLE_WEB_SEARCH_STRONG_ACTION, ROLE_WEB_SEARCH_TOOL_NAME, ROLE_WEB_SEARCH_TOPIC_MARKER,
    ROLE_WHO_QUESTION_LEAD, ROLE_WHO_QUESTION_TAIL, ROLE_WIKIDATA_ENTITY_ANCHOR,
    ROLE_YEAR_UNIT_CUE,
};
pub use summary_topics::{summary_topic_seeds, SummaryTopic, SummaryTopicSeeds};

/// Merge every embedded seed file into a single Links Notation document.
///
/// The output uses the `formal_ai_seed_bundle` header and is exactly what the
/// browser `Download bundle` action produces minus the user-specific event
/// log: it represents the AI's static knowledge surface, fully portable in
/// one file.
#[must_use]
pub fn merged_bundle() -> String {
    bundle_from_files(&seed_files())
}

/// Render an arbitrary list of `(file_name, contents)` pairs as a bundle.
///
/// The output uses the `formal_ai_seed_bundle` header. Used by
/// [`merged_bundle`] for the compile-time seed and by tooling that needs to
/// bundle a custom seed (for example a user-edited overlay).
#[must_use]
pub fn bundle_from_files(files: &[(&str, &str)]) -> String {
    let mut out = String::new();
    out.push_str("formal_ai_seed_bundle\n");
    for (name, contents) in files {
        out.push_str("  file \"");
        out.push_str(&escape_value(name));
        out.push_str("\"\n");
        for line in contents.lines() {
            if line.is_empty() {
                continue;
            }
            out.push_str("    ");
            out.push_str(line);
            out.push('\n');
        }
    }
    out
}

/// Parse a bundle produced by [`merged_bundle`] back into split file pairs.
///
/// The result is a list of `(file_name, contents)` pairs. The inverse of
/// [`bundle_from_files`] — callers can round-trip the universal seed through
/// a single `.lino` document for import/export, while still recovering the
/// per-category split files that drive the rest of the loader.
///
/// The parser accepts both bundle dialects:
///
/// - flat `formal_ai_seed_bundle` — `file "name"` directly at indent 2,
/// - nested `formal_ai_bundle` (the format the browser demo writes and the
///   one [`memory::export_bundle`](crate::memory::export_bundle) produces)
///   where `seed_files` wraps the file list, so each `file "name"` sits at
///   indent 4 and the body at indent 6.
///
/// Sections with no body produce an empty contents string. Indentation
/// inside a section is reproduced verbatim (with the leading bundle prefix
/// stripped) so the round-trip preserves shape.
#[must_use]
pub fn parse_bundle(text: &str) -> Vec<(String, String)> {
    let mut sections: Vec<(String, String)> = Vec::new();
    let mut current_name: Option<String> = None;
    let mut current_body = String::new();
    let mut file_indent: usize = 2;
    let mut body_indent: usize = 4;
    let mut inside_seed_files = false;
    for line in text.lines() {
        if line.is_empty() {
            if current_name.is_some() {
                current_body.push('\n');
            }
            continue;
        }
        let indent = line.chars().take_while(|c| *c == ' ').count();
        let trimmed = &line[indent..];
        // Top-level header (e.g. `formal_ai_seed_bundle` or
        // `formal_ai_bundle`). Start of document.
        if indent == 0 {
            if let Some(name) = current_name.take() {
                sections.push((name, std::mem::take(&mut current_body)));
            }
            inside_seed_files = false;
            file_indent = 2;
            body_indent = 4;
            continue;
        }
        // Wrapper section for the nested dialect: `  seed_files`.
        if indent == 2 && trimmed == "seed_files" {
            if let Some(name) = current_name.take() {
                sections.push((name, std::mem::take(&mut current_body)));
            }
            inside_seed_files = true;
            file_indent = 4;
            body_indent = 6;
            continue;
        }
        // Sibling section at the same indent as `seed_files` (e.g.
        // `demo_memory`) ends the seed list in the nested dialect.
        if inside_seed_files && indent == 2 {
            if let Some(name) = current_name.take() {
                sections.push((name, std::mem::take(&mut current_body)));
            }
            inside_seed_files = false;
            continue;
        }
        // Section header: `file "name"` at the dialect's file_indent.
        if indent == file_indent && trimmed.starts_with("file ") {
            if let Some(name) = current_name.take() {
                sections.push((name, std::mem::take(&mut current_body)));
            }
            if let Some(rest) = trimmed.strip_prefix("file ") {
                let rest = rest.trim();
                if let Some(stripped) = rest.strip_prefix('"') {
                    if let Some(close) = find_closing_quote(stripped) {
                        current_name = Some(unescape_value(&stripped[..close]));
                    }
                }
            }
            continue;
        }
        // Section body: strip the body_indent prefix.
        if current_name.is_some() {
            let prefix: String = " ".repeat(body_indent);
            let stripped = line
                .strip_prefix(prefix.as_str())
                .unwrap_or_else(|| line.trim_start());
            current_body.push_str(stripped);
            current_body.push('\n');
        }
    }
    if let Some(name) = current_name.take() {
        sections.push((name, current_body));
    }
    sections
}

/// A single response variant for an intent in a particular language.
#[derive(Debug, Clone)]
pub struct ResponseRecord {
    pub id: String,
    pub intent: String,
    pub language: String,
    pub text: String,
}

/// Parse `multilingual-responses.lino` into structured records.
#[must_use]
pub fn multilingual_responses() -> Vec<ResponseRecord> {
    let tree = parse_lino(MULTILINGUAL_RESPONSES_LINO);
    let mut out = Vec::new();
    if let Some(root) = tree.children.first() {
        for entry in root.children.iter().filter(|c| c.name == "response") {
            let intent = entry.find_child_value("intent").to_string();
            let language = entry.find_child_value("language").to_string();
            let text = entry.find_child_value("text").to_string();
            if intent.is_empty() || language.is_empty() {
                continue;
            }
            out.push(ResponseRecord {
                id: entry.id.clone(),
                intent,
                language,
                text,
            });
        }
    }
    out
}

/// Look up a localized response by intent and language, returning `None` if
/// the seed has no matching record.
#[must_use]
pub fn response_for(intent: &str, language: &str) -> Option<String> {
    for record in multilingual_responses() {
        if record.intent == intent && record.language == language {
            return Some(record.text);
        }
    }
    None
}

/// Generic key/value config from `agent-info.lino`.
#[must_use]
pub fn agent_info() -> BTreeMap<String, String> {
    let tree = parse_lino(AGENT_INFO_LINO);
    let mut out = BTreeMap::new();
    if let Some(root) = tree.children.first() {
        for entry in root.children.iter().filter(|c| c.name == "field") {
            let key = entry.id.clone();
            let value = entry.find_child_value("value").to_string();
            if !key.is_empty() {
                out.insert(key, value);
            }
        }
    }
    out
}

/// A Unicode-range based language detection rule.
#[derive(Debug, Clone)]
pub struct LanguageRule {
    pub id: String,
    pub language: String,
    pub label: String,
    pub start: u32,
    pub end: u32,
}

#[must_use]
pub fn language_rules() -> Vec<LanguageRule> {
    let tree = parse_lino(LANGUAGE_DETECTION_LINO);
    let mut out = Vec::new();
    if let Some(root) = tree.children.first() {
        for entry in root.children.iter().filter(|c| c.name == "rule") {
            let language = entry.find_child_value("language").to_string();
            if language.is_empty() {
                continue;
            }
            out.push(LanguageRule {
                id: entry.id.clone(),
                language,
                label: entry.find_child_value("label").to_string(),
                start: parse_codepoint(entry.find_child_value("start")),
                end: parse_codepoint(entry.find_child_value("end")),
            });
        }
    }
    out
}

/// A multilingual question pattern for routing intents.
#[derive(Debug, Clone)]
pub struct PromptPattern {
    pub id: String,
    pub intent: String,
    pub language: String,
    pub kind: String,
    pub text: String,
}

/// A language-specific variant of a concept (term, aliases, summary, source).
///
/// Used to deliver a localized definition to the user when their prevailing
/// language matches one of the records nested under `localized "<lang>"` in
/// `data/seed/concepts.lino`. Empty fields fall back to the parent concept.
#[derive(Debug, Clone, Default)]
pub struct LocalizedConcept {
    pub language: String,
    pub term: String,
    pub aliases: Vec<String>,
    pub summary: String,
    pub source: String,
    pub source_kind: String,
}

/// A concept record from the offline knowledge base.
///
/// `contexts` is optional and lists `|`-separated context labels in any of the
/// supported languages (e.g. "ml|machine learning|машинное обучение|机器学习").
/// When a concept can be disambiguated by an in-question context delimiter
/// (e.g. "what is IIR in ML"), the lookup ranker prefers the record whose
/// `contexts` list contains the parsed context over context-less records.
///
/// `wikidata` (optional) anchors the concept to a Wikidata Q-ID so cross-
/// language fall-back goes via the structured knowledge graph the same way
/// the human-language / meta-expression repositories already model it.
///
/// `context_links` (optional) lists the slugs of `concept_contexts.lino`
/// records that disambiguate this concept; the response handler can resolve
/// the localized context label from there.
///
/// `localized` (optional) carries per-language overrides of `term`,
/// `aliases`, `summary`, `source`, and `source_kind`. The solver picks the
/// override matching the user's prevailing language and falls back to the
/// outer (English) values when no override exists.
#[derive(Debug, Clone)]
pub struct ConceptRecord {
    pub slug: String,
    pub term: String,
    pub category: String,
    pub aliases: Vec<String>,
    pub contexts: Vec<String>,
    pub context_links: Vec<String>,
    pub wikidata: String,
    pub summary: String,
    pub source: String,
    pub source_kind: String,
    pub localized: Vec<LocalizedConcept>,
}

impl ConceptRecord {
    /// Pick the localized variant matching `language`, falling back to the
    /// English variant or to `None` if no overrides exist for this concept.
    #[must_use]
    pub fn localized_for(&self, language: &str) -> Option<&LocalizedConcept> {
        self.localized
            .iter()
            .find(|loc| loc.language == language)
            .or_else(|| self.localized.iter().find(|loc| loc.language == "en"))
    }
}

#[must_use]
pub fn concepts() -> Vec<ConceptRecord> {
    let tree = parse_lino(CONCEPTS_LINO);
    let mut out = Vec::new();
    let entries: &[LinoNode] = if tree.name.is_empty() {
        tree.children.as_slice()
    } else {
        std::slice::from_ref(&tree)
    };
    for entry in entries {
        if !entry.name.starts_with("concept_") {
            continue;
        }
        let aliases = split_pipe_list(entry.find_child_value("aliases"));
        let contexts = split_pipe_list(entry.find_child_value("contexts"));
        let context_links = split_pipe_list(entry.find_child_value("context_links"));
        let summary = entry.find_child_value("summary").to_string();
        let term = entry.find_child_value("term").to_string();
        if term.is_empty() || summary.is_empty() {
            continue;
        }
        let mut localized = Vec::new();
        for child in entry.children.iter().filter(|c| c.name == "localized") {
            let lang = child.id.clone();
            if lang.is_empty() {
                continue;
            }
            localized.push(LocalizedConcept {
                language: lang,
                term: child.find_child_value("term").to_string(),
                aliases: split_pipe_list(child.find_child_value("aliases")),
                summary: child.find_child_value("summary").to_string(),
                source: child.find_child_value("source").to_string(),
                source_kind: child.find_child_value("source_kind").to_string(),
            });
        }
        out.push(ConceptRecord {
            slug: entry.name.clone(),
            term,
            category: entry.find_child_value("category").to_string(),
            aliases,
            contexts,
            context_links,
            wikidata: entry.find_child_value("wikidata").to_string(),
            summary,
            source: entry.find_child_value("source").to_string(),
            source_kind: entry.find_child_value("source_kind").to_string(),
            localized,
        });
    }
    out
}

/// A localized label for a disambiguating concept context.
#[derive(Debug, Clone, Default)]
pub struct LocalizedContextLabel {
    pub language: String,
    pub text: String,
}

/// A disambiguating concept context (e.g. "machine learning") with a Wikidata
/// Q-ID anchor and per-language localized labels. Loaded from
/// `data/seed/concept-contexts.lino`.
#[derive(Debug, Clone, Default)]
pub struct ContextRecord {
    pub slug: String,
    pub wikidata: String,
    pub aliases: Vec<String>,
    pub labels: Vec<LocalizedContextLabel>,
}

impl ContextRecord {
    /// Pick the localized label matching `language`, falling back to the
    /// English label or the slug.
    #[must_use]
    pub fn label_for(&self, language: &str) -> &str {
        if let Some(label) = self.labels.iter().find(|l| l.language == language) {
            return &label.text;
        }
        if let Some(label) = self.labels.iter().find(|l| l.language == "en") {
            return &label.text;
        }
        &self.slug
    }

    /// Returns true when `value` (normalized lowercase) matches one of this
    /// record's aliases or localized labels.
    #[must_use]
    pub fn matches(&self, value: &str) -> bool {
        let needle = value.trim().to_lowercase();
        if needle.is_empty() {
            return false;
        }
        if self
            .aliases
            .iter()
            .any(|alias| alias.trim().to_lowercase() == needle)
        {
            return true;
        }
        self.labels
            .iter()
            .any(|label| label.text.trim().to_lowercase() == needle)
    }
}

#[must_use]
pub fn concept_contexts() -> Vec<ContextRecord> {
    let tree = parse_lino(CONCEPT_CONTEXTS_LINO);
    let mut out = Vec::new();
    if let Some(root) = tree.children.first() {
        for entry in root.children.iter().filter(|c| c.name == "context") {
            let slug = entry.id.clone();
            if slug.is_empty() {
                continue;
            }
            let aliases = split_pipe_list(entry.find_child_value("aliases"));
            let mut labels = Vec::new();
            for child in entry.children.iter().filter(|c| c.name == "label") {
                let lang = child.id.clone();
                if lang.is_empty() {
                    continue;
                }
                labels.push(LocalizedContextLabel {
                    language: lang,
                    text: child.find_child_value("text").to_string(),
                });
            }
            out.push(ContextRecord {
                slug,
                wikidata: entry.find_child_value("wikidata").to_string(),
                aliases,
                labels,
            });
        }
    }
    out
}

/// Intent routing record from `data/seed/intent-routing.lino`.
///
/// Match semantics (mirrored in `src/web/formal_ai_worker.js`):
/// - `keywords`: exact match of the entire normalized prompt
/// - `phrases`: exact match of the entire normalized prompt (kept as a
///   separate label so multi-word entries are easy to spot in `.lino`)
/// - `tokens`: any single whitespace-separated token equals the value
/// - `combos`: every token in the combo appears as a whitespace-separated
///   token in the prompt (in any order)
#[derive(Debug, Clone, Default)]
pub struct IntentRoute {
    pub id: String,
    pub slug: String,
    pub response_link: String,
    pub keywords: Vec<String>,
    pub phrases: Vec<String>,
    pub tokens: Vec<String>,
    pub combos: Vec<Vec<String>>,
}

#[derive(Debug, Clone, Default)]
pub struct IntentRouting {
    pub intents: Vec<IntentRoute>,
    pub article_prefixes: Vec<String>,
    pub trace_prefixes: Vec<String>,
}

#[must_use]
pub fn intent_routing() -> IntentRouting {
    let tree = parse_lino(INTENT_ROUTING_LINO);
    let mut routing = IntentRouting::default();
    if let Some(root) = tree.children.first() {
        for child in &root.children {
            match child.name.as_str() {
                "intent" => {
                    let mut keywords = Vec::new();
                    let mut phrases = Vec::new();
                    let mut tokens = Vec::new();
                    let mut combos = Vec::new();
                    for entry in &child.children {
                        match entry.name.as_str() {
                            "keyword" => keywords.push(entry.id.clone()),
                            "phrase" => phrases.push(entry.id.clone()),
                            "token" => tokens.push(entry.id.clone()),
                            "combo" => combos.push(
                                entry
                                    .id
                                    .split('+')
                                    .map(str::trim)
                                    .filter(|s| !s.is_empty())
                                    .map(ToOwned::to_owned)
                                    .collect(),
                            ),
                            _ => {}
                        }
                    }
                    routing.intents.push(IntentRoute {
                        id: child.id.clone(),
                        slug: child.find_child_value("slug").to_string(),
                        response_link: child.find_child_value("response_link").to_string(),
                        keywords,
                        phrases,
                        tokens,
                        combos,
                    });
                }
                "article" => routing.article_prefixes.push(child.id.clone()),
                "trace_prefix" => routing.trace_prefixes.push(child.id.clone()),
                _ => {}
            }
        }
    }
    routing
}

#[must_use]
pub fn prompt_patterns() -> Vec<PromptPattern> {
    let tree = parse_lino(PROMPT_PATTERNS_LINO);
    let mut out = Vec::new();
    if let Some(root) = tree.children.first() {
        for entry in root.children.iter().filter(|c| c.name == "pattern") {
            let text = entry.find_child_value("text").to_string();
            if text.is_empty() {
                continue;
            }
            out.push(PromptPattern {
                id: entry.id.clone(),
                intent: entry.find_child_value("intent").to_string(),
                language: entry.find_child_value("language").to_string(),
                kind: entry.find_child_value("kind").to_string(),
                text,
            });
        }
    }
    out
}

/// One self-describing entry from `environments.lino`.
///
/// The seed declares every supported surface (browser demo, Rust library,
/// CLI, HTTP server, desktop shell, Telegram bot, Docker microservice) and how memory
/// migrates between them. The AI itself can therefore answer "where can I
/// run?" and "how do I move my memory from CLI to web?" from data rather
/// than from hardcoded strings.
#[derive(Debug, Clone, Default)]
pub struct EnvironmentRecord {
    pub id: String,
    pub label: String,
    pub runtime: String,
    pub seed_path: String,
    pub memory_store: String,
    pub memory_export_command: String,
    pub bundle_export_command: String,
    pub bundle_import_command: String,
    pub start_command: String,
    pub package_command: String,
    pub tools: Vec<String>,
}

#[derive(Debug, Clone, Default)]
pub struct MigrationFlow {
    pub id: String,
    pub description: String,
    pub file_format: String,
}

#[derive(Debug, Clone, Default)]
pub struct EnvironmentDirectory {
    pub environments: Vec<EnvironmentRecord>,
    pub migration_description: String,
    pub flows: Vec<MigrationFlow>,
}

#[must_use]
pub fn environment_directory() -> EnvironmentDirectory {
    let tree = parse_lino(ENVIRONMENTS_LINO);
    let mut directory = EnvironmentDirectory::default();
    for root in &tree.children {
        match root.name.as_str() {
            "environments" => {
                for entry in root.children.iter().filter(|c| c.name == "environment") {
                    let tools_raw = entry.find_child_value("tools").to_string();
                    let tools = if tools_raw.is_empty() {
                        Vec::new()
                    } else {
                        tools_raw
                            .split('|')
                            .map(str::trim)
                            .filter(|s| !s.is_empty())
                            .map(ToOwned::to_owned)
                            .collect()
                    };
                    directory.environments.push(EnvironmentRecord {
                        id: entry.id.clone(),
                        label: entry.find_child_value("label").to_string(),
                        runtime: entry.find_child_value("runtime").to_string(),
                        seed_path: entry.find_child_value("seed_path").to_string(),
                        memory_store: entry.find_child_value("memory_store").to_string(),
                        memory_export_command: entry
                            .find_child_value("memory_export_command")
                            .to_string(),
                        bundle_export_command: entry
                            .find_child_value("bundle_export_command")
                            .to_string(),
                        bundle_import_command: entry
                            .find_child_value("bundle_import_command")
                            .to_string(),
                        start_command: entry.find_child_value("start_command").to_string(),
                        package_command: entry.find_child_value("package_command").to_string(),
                        tools,
                    });
                }
            }
            "migration" => {
                directory.migration_description = root.find_child_value("description").to_string();
                for entry in root.children.iter().filter(|c| c.name == "flow") {
                    directory.flows.push(MigrationFlow {
                        id: entry.id.clone(),
                        description: entry.find_child_value("description").to_string(),
                        file_format: entry.find_child_value("file_format").to_string(),
                    });
                }
            }
            _ => {}
        }
    }
    directory
}

/// Convenience accessor returning just the environment records (without the
/// migration flow descriptions). Used by the CLI/HTTP `bundle` printers and
/// by tests that pin self-awareness coverage.
#[must_use]
pub fn environment_records() -> Vec<EnvironmentRecord> {
    environment_directory().environments
}

#[path = "source_tests/seed/tests.rs"]
mod tests;
