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

mod agentic_tool_capabilities;
mod brainstorm;
mod client_completion;
mod client_integrations;
mod coreference;
mod draft_strategies;
mod embedded;
mod entity_names;
mod facts;
mod grounding_overrides;
mod handler_precedence;
mod market_price_references;
mod meanings;
mod model_aliases;
mod operation_vocabulary;
pub(crate) mod parser;
mod personas;
mod projects;
mod roles;
mod shell_intents;
mod summary_topics;
mod terminal_commands;

use std::collections::BTreeMap;

use parser::{
    escape_value, find_closing_quote, parse_codepoint, parse_lino, split_pipe_list, unescape_value,
    LinoNode,
};

pub use agentic_tool_capabilities::{agentic_tool_capabilities, AgenticToolCapability};
pub use brainstorm::{brainstorm_seeds, BrainstormCategory, BrainstormSeeds};
pub use client_completion::{software_authoring_completion_contract, ClientCompletionContract};
pub use client_integrations::{
    client_integrations, ClientIntegration, ClientIntegrationGlobalConfig,
    ClientIntegrationInvocation, ClientVerification, ConfigFormat, ModeArgPosition,
    ModelArgPosition, TemplateEnv,
};
pub use coreference::{coreference_seeds, Antecedent, CoreferenceSeeds, Pronoun};
pub use draft_strategies::{draft_strategies, draft_strategies_from};
pub use embedded::{
    seed_files, AGENTIC_TOOL_CAPABILITIES_LINO, AGENT_INFO_LINO, BRAINSTORM_SEEDS_LINO,
    CLIENT_COMPLETION_CONTRACTS_LINO, CLIENT_INTEGRATIONS_LINO, CODING_IDIOMS_LINO,
    COMPUTER_USE_TASKS_LINO, CONCEPTS_LINO, CONCEPT_CONTEXTS_LINO, COREFERENCE_LINO,
    DEMO_DIALOGS_LINO, DRAFT_STRATEGIES_LINO, ENTITY_NAMES_LINO, ENVIRONMENTS_LINO, FACTS_LINO,
    GREETINGS_LINO, HANDLER_PRECEDENCE_LINO, HELLO_WORLD_PROGRAMS_LINO, IDENTITY_LINO,
    INTENT_ROUTING_LINO, LANGUAGES_LINO, LANGUAGE_DETECTION_LINO, LEARNING_SOURCES_LINO,
    MARKET_PRICE_REFERENCES_LINO, MEANINGS_CALENDAR_LINO, MEANINGS_CODING_TASKS_LINO,
    MEANINGS_FACTS_LINO, MEANINGS_LINKS_ROOT_LINO, MEANINGS_LINO, MEANINGS_NUMBER_CONSTRAINTS_LINO,
    MEANINGS_SEMANTIC_META_LINO, MEANINGS_SOFTWARE_PROJECT_LINO, MEANINGS_UNITS_LINO,
    MEANING_FILES, MODEL_ALIASES_LINO, MULTILINGUAL_RESPONSES_DECOMPOSITION_LINO,
    MULTILINGUAL_RESPONSES_ENTITIES_LINO, MULTILINGUAL_RESPONSES_LANGUAGE_PROTOCOL_LINO,
    MULTILINGUAL_RESPONSES_LINO, MULTILINGUAL_RESPONSES_MEMORY_PROGRAM_LINO,
    MULTILINGUAL_RESPONSES_PATTERN_LINO, MULTILINGUAL_RESPONSES_PROCEDURE_LINO,
    NUMERIC_LIST_OPERATIONS_LINO, OPERATION_VOCABULARY_LINO, PERSONAS_LINO,
    PROGRAM_CST_GRAMMARS_LINO, PROGRAM_PLAN_RULES_LINO, PROJECTS_LINO, PROMPT_PATTERNS_LINO,
    RESPONSE_FILES, SELF_IMPROVEMENT_LOOP_LINO, SHELL_INTENTS_LINO, SUMMARY_TOPICS_LINO,
    TERMINAL_COMMANDS_LINO, TOOLS_LINO,
};
pub use entity_names::{entity_names, EntityName};
pub use facts::{facts, FactRecord, LocalizedFact};
pub use grounding_overrides::{
    cache_contains, override_facts, override_reason, parse_record, resolve, OverrideFact,
};
pub use handler_precedence::{handler_precedence, handler_precedence_from};
pub use market_price_references::{market_price_assets, MarketPriceAsset, MarketPricePeriod};
pub use meanings::{
    lexicon, parse_lexicon_text, ArithmeticOperator, Lexeme, Lexicon, Meaning, SemanticFacet, Slot,
    WordForm,
};
pub use model_aliases::{
    canonical_model_id, model_aliases, resolve_model_id, try_resolve_model_id, ModelAliasRegistry,
};
pub use operation_vocabulary::{
    operation_vocabulary, OperationLanguageForms, OperationTrigger, OperationVocabulary,
};
pub use personas::{persona_seeds, Persona, PersonaSeeds, PersonaTopic};
pub use projects::{
    projects_registry, LocalizedProject, ProjectRecord, ProjectStatement, ProjectsRegistry,
};
// `roles` re-exports its own submodules with globs; mirror that here so the
// per-role constant list does not have to be restated (and keeps this file
// under the 1000-line limit as new roles land).
pub use roles::*;
pub use shell_intents::{
    shell_intent_vocabulary, LocalPathSearchKind, LocalPathSearchScope, ShellIntent,
    ShellIntentArgument, ShellIntentVocabulary,
};
pub use summary_topics::{summary_topic_seeds, SummaryTopic, SummaryTopicSeeds};
pub use terminal_commands::{terminal_command_vocabulary, TerminalCommandVocabulary};

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
    let mut out = Vec::new();
    for source in RESPONSE_FILES {
        let tree = parse_lino(source);
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

/// Look up one response and substitute its named template fields.
#[must_use]
pub fn render_response(intent: &str, language: &str, values: &[(&str, &str)]) -> Option<String> {
    response_for(intent, language).map(|mut rendered| {
        for (name, value) in values {
            rendered = rendered.replace(&format!("{{{name}}}"), value);
        }
        rendered
    })
}

/// Look up a localized response, applying the registry's `explicit_gap`
/// fallback policy (`data/seed/languages.lino`).
///
/// Issue #706: a language that the detection registry knows but a given intent
/// has no localized text for is a *gap*, not an English prompt. Handlers that
/// used to write `response_for(intent, language).or_else(|| response_for(intent,
/// "en"))` silently answered a Spanish speaker in English; going through this
/// helper instead prefers the seed's `language unknown` variant — the record
/// that says out loud that the language is unsupported — before falling back to
/// English as a last resort.
#[must_use]
pub fn localized_response(intent: &str, language: &str) -> Option<String> {
    if let Some(text) = response_for(intent, language) {
        return Some(text);
    }
    if crate::language::from_slug(language).is_some() {
        if let Some(text) = response_for(intent, "unknown") {
            return Some(text);
        }
    }
    response_for(intent, "en")
}

/// Look up a localized response whose `text` is a native reference list.
///
/// Scalar text is returned as a one-element vector, which keeps this helper
/// useful for seed records that may migrate between scalar and list forms.
#[must_use]
pub fn response_values_for(intent: &str, language: &str) -> Option<Vec<String>> {
    response_for(intent, language).map(|text| split_pipe_list(&text))
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

/// The languages the agent answers in, declared by `agent-info.lino`.
///
/// Stored as a reference list (`supported_languages ("en" "ru" "hi" "zh")`) so
/// the multi-value is a sequence of separate references rather than a single
/// `|`-packed string. This resolves it to the individual language ids in
/// declaration order.
#[must_use]
pub fn supported_languages() -> Vec<String> {
    agent_info()
        .get("supported_languages")
        .map(|value| split_pipe_list(value))
        .unwrap_or_default()
}

/// A Unicode-range based language detection rule.
#[derive(Debug, Clone)]
pub struct LanguageRule {
    pub id: String,
    pub language: String,
    pub label: String,
    pub start: u32,
    pub end: u32,
    /// URL fragment identifying a source host that publishes in this language,
    /// e.g. `://ru.wikipedia.org/`. Empty when the language declares no host.
    ///
    /// Issue #699 batch 2: the definition merger used to branch on Wikipedia
    /// hosts in Rust. Host-to-language is language identity data, so it lives
    /// beside the script ranges that already answer the same question.
    pub source_host: String,
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
                source_host: entry.find_child_value("source-host").to_string(),
            });
        }
    }
    out
}

/// The language of a source URL, decided by the `source-host` fragments
/// declared in `data/seed/language-detection.lino`.
///
/// Falls back to `en`, matching the detection rules' own default.
#[must_use]
pub fn language_of_source(source: &str) -> String {
    language_rules()
        .into_iter()
        .find(|rule| !rule.source_host.is_empty() && source.contains(&rule.source_host))
        .map_or_else(|| String::from("en"), |rule| rule.language)
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

/// One learnable data source declared by `learning-sources.lino` (issue #499).
///
/// The seed names each external data source the engine can *learn from* when a
/// user points it there — a host, the natural-language keywords that name it in
/// any supported language, and the `capability` slug that says which learning
/// loop ingests it. Routing reads this data rather than branching on a specific
/// URL, so a new source is a data edit, never a code change.
#[derive(Debug, Clone, Default)]
pub struct LearningSource {
    pub id: String,
    pub capability: String,
    pub host: String,
    pub keywords: Vec<String>,
}

/// The learnable-source registry plus the shared, language-agnostic directive
/// cues that mark a "learn from this source" request (issue #499).
#[derive(Debug, Clone, Default)]
pub struct LearningSources {
    pub sources: Vec<LearningSource>,
    pub directive_cues: Vec<String>,
}

impl LearningSources {
    /// Match a lowercased prompt against the registry and return the source the
    /// user is teaching the engine to learn from, if any.
    ///
    /// A directive is only recognized when the prompt carries **both** a
    /// language-agnostic learning cue (e.g. "learn from", "узнаешь",
    /// "यहाँ से सीख", "在这里了解") **and** a reference to a declared source — its
    /// host or one of its native-language keywords. This is the single source of
    /// truth shared by the chat handler (`crate::solver_handlers::try_learn_from_source`)
    /// and the Agent CLI planner
    /// ([`crate::agentic_coding::google_trends_learning::is_google_trends_learning_task`]),
    /// so the *same* natural-language teaching directive drives both the chat
    /// acknowledgement and the artifact-writing recipe. Callers pass an
    /// already-lowercased prompt so the seed's lowercased cues/keywords match
    /// directly (issue #499).
    #[must_use]
    pub fn match_directive(&self, lowercased: &str) -> Option<&LearningSource> {
        let has_cue = self
            .directive_cues
            .iter()
            .any(|cue| lowercased.contains(cue.as_str()));
        if !has_cue {
            return None;
        }
        self.sources.iter().find(|source| {
            (!source.host.is_empty() && lowercased.contains(source.host.as_str()))
                || source
                    .keywords
                    .iter()
                    .any(|keyword| lowercased.contains(keyword.as_str()))
        })
    }
}

/// Parse the learnable-source registry from `learning-sources.lino`.
///
/// The keywords and directive cues are stored lowercased in the seed so a
/// lowercased prompt matches them directly (Rust's `to_lowercase` folds the
/// Cyrillic, Devanagari, and Han text too).
#[must_use]
pub fn learning_sources() -> LearningSources {
    let tree = parse_lino(LEARNING_SOURCES_LINO);
    let mut registry = LearningSources::default();
    if let Some(root) = tree.children.first() {
        for child in &root.children {
            match child.name.as_str() {
                "source" => {
                    let keywords = child
                        .children
                        .iter()
                        .filter(|entry| entry.name == "keyword")
                        .map(|entry| entry.id.clone())
                        .collect();
                    registry.sources.push(LearningSource {
                        id: child.id.clone(),
                        capability: child.find_child_value("capability").to_string(),
                        host: child.find_child_value("host").to_string(),
                        keywords,
                    });
                }
                "directive" => {
                    for entry in child.children.iter().filter(|entry| entry.name == "cue") {
                        registry.directive_cues.push(entry.id.clone());
                    }
                }
                _ => {}
            }
        }
    }
    registry
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
                    let tools = split_pipe_list(entry.find_child_value("tools"));
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
                directory.migration_description =
                    child_value_alias(root, "note", "description").to_string();
                for entry in root.children.iter().filter(|c| c.name == "flow") {
                    directory.flows.push(MigrationFlow {
                        id: entry.id.clone(),
                        description: child_value_alias(entry, "note", "description").to_string(),
                        file_format: entry.find_child_value("file_format").to_string(),
                    });
                }
            }
            _ => {}
        }
    }
    directory
}

fn child_value_alias<'a>(node: &'a LinoNode, primary: &str, fallback: &str) -> &'a str {
    let value = node.find_child_value(primary);
    if value.is_empty() {
        node.find_child_value(fallback)
    } else {
        value
    }
}

/// Convenience accessor returning just the environment records (without the
/// migration flow descriptions). Used by the CLI/HTTP `bundle` printers and
/// by tests that pin self-awareness coverage.
#[must_use]
pub fn environment_records() -> Vec<EnvironmentRecord> {
    environment_directory().environments
}
