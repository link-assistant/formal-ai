//! Links-Notation intent formalization and cache.
//!
//! The P/Q formalization layer identifies meaning anchors inside a prompt.
//! This module wraps those anchors in the routing-facing intent record: what
//! kind of impulse arrived, what is already known, and which handlers or rules
//! are relevant.

use std::collections::BTreeMap;
use std::fmt::Write as _;

use crate::engine::{
    normalize_prompt, program_language_by_alias, program_spec, stable_id, SelectedRule,
    WRITE_PROGRAM_INTENT,
};
use crate::event_log::EventLog;
use crate::link_store::{LinkStore, LinkStoreError};
use crate::memory::MemoryEvent;
use crate::probability::ProbabilityStore;
use crate::seed;
use crate::solver::UniversalSolver;
use crate::translation::{FormalizationAnchorKind, FormalizationCandidate, FormalizationRole};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IntentKind {
    Task,
    Question,
    Requirement,
    Statement,
    Courtesy,
    Unknown,
}

impl IntentKind {
    #[must_use]
    pub const fn slug(self) -> &'static str {
        match self {
            Self::Task => "task",
            Self::Question => "question",
            Self::Requirement => "requirement",
            Self::Statement => "statement",
            Self::Courtesy => "courtesy",
            Self::Unknown => "unknown",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IntentFormalization {
    pub impulse_id: String,
    pub source_text: String,
    pub normalized_text: String,
    pub language: String,
    pub kind: IntentKind,
    pub knowns: Vec<String>,
    pub relevants: Vec<String>,
    pub parameters: BTreeMap<String, String>,
    pub route: Option<String>,
    pub response_link: Option<String>,
}

impl IntentFormalization {
    #[must_use]
    pub fn to_links_notation(&self) -> String {
        let mut out = format!(
            "intent_formalization \"{}\"\n",
            escape_lino_value(&self.impulse_id)
        );
        let _ = writeln!(
            out,
            "  impulse_id \"{}\"",
            escape_lino_value(&self.impulse_id)
        );
        let _ = writeln!(
            out,
            "  source_text \"{}\"",
            escape_lino_value(&self.source_text)
        );
        let _ = writeln!(
            out,
            "  normalized_text \"{}\"",
            escape_lino_value(&self.normalized_text)
        );
        let _ = writeln!(out, "  language \"{}\"", escape_lino_value(&self.language));
        let _ = writeln!(out, "  kind \"{}\"", self.kind.slug());
        if let Some(route) = &self.route {
            let _ = writeln!(out, "  route \"{}\"", escape_lino_value(route));
        }
        if let Some(response_link) = &self.response_link {
            let _ = writeln!(
                out,
                "  response_link \"{}\"",
                escape_lino_value(response_link)
            );
        }
        for (name, value) in &self.parameters {
            let _ = writeln!(
                out,
                "  parameter \"{}={}\"",
                escape_lino_value(name),
                escape_lino_value(value)
            );
        }
        for known in &self.knowns {
            let _ = writeln!(out, "  known \"{}\"", escape_lino_value(known));
        }
        for relevant in &self.relevants {
            let _ = writeln!(out, "  relevant \"{}\"", escape_lino_value(relevant));
        }
        out
    }

    #[must_use]
    pub fn has_relevant_handler(&self, name: &str) -> bool {
        self.relevants
            .iter()
            .any(|relevant| relevant == &format!("handler:{name}"))
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IntentFormalizationCacheEntry {
    pub formalization: IntentFormalization,
    pub cache_hit: bool,
}

#[derive(Debug, Default, Clone)]
pub struct IntentFormalizationCache {
    records: BTreeMap<String, IntentFormalization>,
}

impl IntentFormalizationCache {
    #[must_use]
    pub const fn new() -> Self {
        Self {
            records: BTreeMap::new(),
        }
    }

    #[must_use]
    pub fn len(&self) -> usize {
        self.records.len()
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.records.is_empty()
    }

    #[must_use]
    pub fn get(&self, prompt: &str) -> Option<&IntentFormalization> {
        let id = impulse_id_for(prompt);
        self.records.get(&id)
    }

    pub fn formalize_or_insert(
        &mut self,
        prompt: &str,
        language: &str,
        candidate: Option<&FormalizationCandidate>,
    ) -> IntentFormalizationCacheEntry {
        let id = impulse_id_for(prompt);
        if let Some(formalization) = self.records.get(&id) {
            return IntentFormalizationCacheEntry {
                formalization: formalization.clone(),
                cache_hit: true,
            };
        }

        let formalization = formalize_intent(prompt, language, candidate);
        self.records.insert(id, formalization.clone());
        IntentFormalizationCacheEntry {
            formalization,
            cache_hit: false,
        }
    }

    pub fn append_to_link_store<S: LinkStore>(
        &self,
        store: &mut S,
    ) -> Result<usize, LinkStoreError> {
        let mut inserted = 0;
        for formalization in self.records.values() {
            store.append_memory_event(MemoryEvent {
                id: formalization.impulse_id.clone(),
                kind: Some(String::from("intent_formalization")),
                content: Some(formalization.to_links_notation()),
                evidence: vec![format!("intent_formalization:{}", formalization.impulse_id)],
                ..MemoryEvent::default()
            })?;
            inserted += 1;
        }
        Ok(inserted)
    }
}

impl UniversalSolver {
    #[must_use]
    pub fn solve_with_intent_cache(
        &self,
        prompt: &str,
        intent_cache: &mut IntentFormalizationCache,
    ) -> crate::engine::SymbolicAnswer {
        self.solve_with_history_probability_store_and_intent_cache(
            prompt,
            &[],
            &ProbabilityStore::new(),
            intent_cache,
        )
    }
}

#[must_use]
pub fn impulse_id_for(prompt: &str) -> String {
    stable_id("impulse", &normalize_prompt(prompt))
}

#[must_use]
pub fn formalize_intent(
    prompt: &str,
    language: &str,
    candidate: Option<&FormalizationCandidate>,
) -> IntentFormalization {
    let normalized = normalize_prompt(prompt);
    let route = route_for_prompt(&normalized);
    let parameters = write_program_parameters(&normalized).unwrap_or_default();
    let mut knowns = vec![
        format!("impulse:{}", impulse_id_for(prompt)),
        format!("language:{language}"),
    ];
    let mut relevants = Vec::new();

    if let Some(candidate) = candidate {
        append_candidate_knowns(candidate, &mut knowns, &mut relevants);
    }
    for (name, value) in &parameters {
        push_unique(&mut knowns, format!("parameter:{name}:{value}"));
    }
    append_prompt_relevants(prompt, &normalized, &mut relevants);

    let route_slug = route
        .as_ref()
        .map(|matched| matched.slug.clone())
        .or_else(|| route_from_relevants(&relevants));
    if let Some(route_slug) = &route_slug {
        push_unique(&mut relevants, format!("route:{route_slug}"));
        if specialized_handler_name(route_slug).is_some() {
            push_unique(&mut relevants, format!("handler:{route_slug}"));
        }
    }

    IntentFormalization {
        impulse_id: impulse_id_for(prompt),
        source_text: prompt.to_owned(),
        normalized_text: normalized.clone(),
        language: language.to_owned(),
        kind: infer_kind(prompt, &normalized, route_slug.as_deref(), candidate),
        knowns,
        relevants,
        parameters,
        response_link: route.map(|matched| matched.response_link),
        route: route_slug,
    }
}

pub(crate) fn record_intent_formalization(
    log: &mut EventLog,
    entry: &IntentFormalizationCacheEntry,
) {
    let formalization = &entry.formalization;
    let cache_state = if entry.cache_hit { "hit" } else { "miss" };
    log.append(
        "intent_formalization_cache",
        format!("{cache_state} {}", formalization.impulse_id),
    );
    if entry.cache_hit {
        log.append(
            "cache_hit",
            format!("intent_formalization:{}", formalization.impulse_id),
        );
    } else {
        log.append("intent_formalization", formalization.to_links_notation());
    }
    log.append(
        "intent_formalization:kind",
        formalization.kind.slug().to_owned(),
    );
    if let Some(route) = &formalization.route {
        log.append("intent_formalization:route", route.clone());
    }
    for relevant in &formalization.relevants {
        log.append("intent_formalization:relevant", relevant.clone());
    }
}

#[must_use]
pub(crate) fn select_rule_for_intent(intent: &IntentFormalization) -> SelectedRule {
    match intent.route.as_deref() {
        Some("greeting") => SelectedRule::Greeting,
        Some("farewell") => SelectedRule::Farewell,
        Some("test_status") => SelectedRule::TestStatus,
        Some("courtesy_response") => SelectedRule::CourtesyResponse,
        Some("assistant_name") => SelectedRule::AssistantName,
        Some("identity") => SelectedRule::Identity,
        Some(WRITE_PROGRAM_INTENT) => write_program_rule_for_intent(intent),
        _ => SelectedRule::Unknown,
    }
}

#[must_use]
pub(crate) fn ordered_handler_names<'a>(
    intent: &IntentFormalization,
    names: impl Iterator<Item = &'a str>,
) -> Vec<&'a str> {
    let names = names.collect::<Vec<_>>();
    let mut ordered = Vec::new();
    for relevant in &intent.relevants {
        let Some(name) = relevant.strip_prefix("handler:") else {
            continue;
        };
        if let Some(matched) = names.iter().copied().find(|candidate| *candidate == name) {
            if !ordered.contains(&matched) {
                ordered.push(matched);
            }
        }
    }
    for name in names {
        if !ordered.contains(&name) {
            ordered.push(name);
        }
    }
    ordered
}

#[derive(Debug, Clone)]
struct MatchedRoute {
    slug: String,
    response_link: String,
}

fn route_for_prompt(normalized: &str) -> Option<MatchedRoute> {
    if write_program_parameters(normalized).is_some() {
        return Some(MatchedRoute {
            slug: String::from(WRITE_PROGRAM_INTENT),
            response_link: String::from("response:write_program"),
        });
    }
    seed::intent_routing()
        .intents
        .into_iter()
        .find(|route| matches_route(normalized, route))
        .map(|route| MatchedRoute {
            slug: route.slug,
            response_link: route.response_link,
        })
}

fn matches_route(normalized: &str, route: &seed::IntentRoute) -> bool {
    route.keywords.iter().any(|keyword| normalized == keyword)
        || route.phrases.iter().any(|phrase| normalized == phrase)
        || route
            .tokens
            .iter()
            .any(|token| contains_token(normalized, token))
        || route.combos.iter().any(|combo| {
            !combo.is_empty() && combo.iter().all(|token| contains_token(normalized, token))
        })
}

fn contains_token(normalized: &str, expected: &str) -> bool {
    normalized.split_whitespace().any(|token| token == expected)
}

fn write_program_rule_for_intent(intent: &IntentFormalization) -> SelectedRule {
    let task = intent.parameters.get("task").cloned();
    let language = intent.parameters.get("language").cloned();
    if let (Some(task_slug), Some(language_slug)) = (task.as_deref(), language.as_deref()) {
        if let Some(spec) = program_spec(task_slug, language_slug) {
            return SelectedRule::WriteProgram(spec);
        }
    }
    SelectedRule::UnsupportedWriteProgram { task, language }
}

fn write_program_parameters(normalized: &str) -> Option<BTreeMap<String, String>> {
    let task = crate::engine_hello_world::program_task_by_alias(normalized);
    let language = requested_program_language(normalized);
    let asks_for_program = contains_token(normalized, "program")
        && (contains_token(normalized, "write")
            || contains_token(normalized, "create")
            || contains_token(normalized, "show"));
    if task.is_none() && !asks_for_program {
        return None;
    }
    let mut parameters = BTreeMap::new();
    if let Some(task) = task {
        parameters.insert(String::from("task"), String::from(task.slug));
    }
    if let Some(language) = language {
        parameters.insert(String::from("language"), language);
    }
    Some(parameters)
}

fn requested_program_language(normalized: &str) -> Option<String> {
    if let Some(language) = program_language_by_alias(normalized) {
        return Some(String::from(language.slug));
    }
    let tokens = normalized.split_whitespace().collect::<Vec<_>>();
    for (index, token) in tokens.iter().enumerate() {
        if !matches!(*token, "in" | "на") {
            continue;
        }
        let Some(next) = tokens.get(index + 1) else {
            continue;
        };
        if matches!(*next, "language" | "языке") {
            if let Some(after_language_word) = tokens.get(index + 2) {
                return Some((*after_language_word).to_owned());
            }
            continue;
        }
        return Some((*next).to_owned());
    }
    None
}

fn append_candidate_knowns(
    candidate: &FormalizationCandidate,
    knowns: &mut Vec<String>,
    relevants: &mut Vec<String>,
) {
    for slot in &candidate.slots {
        push_unique(
            knowns,
            slot_known_link(slot.role, slot.anchor.kind, &slot.anchor.id),
        );
        if slot.role == FormalizationRole::Predicate && slot.anchor.id == "wikidata:P5972" {
            push_unique(relevants, String::from("handler:translation"));
            push_unique(relevants, String::from("route:translation"));
        }
    }
    for term in &candidate.unresolved_terms {
        push_unique(knowns, format!("formalization_unresolved:{term}"));
    }
}

fn slot_known_link(role: FormalizationRole, kind: FormalizationAnchorKind, id: &str) -> String {
    match (role, kind) {
        (FormalizationRole::Subject, FormalizationAnchorKind::WikidataItem) => {
            format!("formalization:subject_q:{id}")
        }
        (FormalizationRole::Predicate, FormalizationAnchorKind::WikidataProperty) => {
            format!("formalization:predicate_p:{id}")
        }
        (FormalizationRole::Object, FormalizationAnchorKind::WikidataItem) => {
            format!("formalization:object_q:{id}")
        }
        (_, FormalizationAnchorKind::WikidataItem) => {
            format!("formalization:item_q:{id}")
        }
        (_, FormalizationAnchorKind::WikidataProperty) => {
            format!("formalization:property_p:{id}")
        }
        (
            _,
            FormalizationAnchorKind::WikipediaArticle | FormalizationAnchorKind::WiktionaryEntry,
        ) => {
            format!("formalization:fallback:{id}")
        }
        (_, FormalizationAnchorKind::RawText) => format!("formalization:raw:{id}"),
    }
}

fn append_prompt_relevants(prompt: &str, normalized: &str, relevants: &mut Vec<String>) {
    let lower_prompt = prompt.to_ascii_lowercase();
    let handlers = [
        (
            "handler:execution_failure",
            lower_prompt.contains("undefined_function")
                || normalized.contains("undefined function"),
        ),
        ("handler:arithmetic", looks_arithmetic(prompt, normalized)),
        (
            "handler:web_search",
            has_any_token(normalized, &["search", "google", "find"]),
        ),
        (
            "handler:procedural_how_to",
            normalized.starts_with("how to "),
        ),
        (
            "handler:proof_request",
            has_any_token(normalized, &["prove", "proof"]),
        ),
        (
            "handler:write_script",
            has_any_token(normalized, &["script", "code"]),
        ),
        (
            "handler:write_program",
            write_program_parameters(normalized).is_some(),
        ),
        (
            "handler:software_project",
            has_any_token(normalized, &["build", "create", "implement", "develop"]),
        ),
        (
            "handler:meta_explanation",
            normalized.contains("how you work") || normalized.contains("как ты работаешь"),
        ),
        (
            "handler:concept_lookup",
            normalized.starts_with("what is ") || normalized.starts_with("define "),
        ),
    ];
    for (handler, matches) in handlers {
        if matches {
            push_unique(relevants, String::from(handler));
        }
    }
}

fn looks_arithmetic(prompt: &str, normalized: &str) -> bool {
    let raw = prompt.to_ascii_lowercase();
    raw.chars().any(|c| c.is_ascii_digit())
        && ["+", "-", "*", "/", "plus", "minus", "times", "divided"]
            .iter()
            .any(|operator| raw.contains(operator) || normalized.contains(operator))
}

fn has_any_token(normalized: &str, tokens: &[&str]) -> bool {
    tokens.iter().any(|token| contains_token(normalized, token))
}

fn route_from_relevants(relevants: &[String]) -> Option<String> {
    relevants.iter().find_map(|relevant| {
        relevant
            .strip_prefix("route:")
            .or_else(|| relevant.strip_prefix("handler:"))
            .and_then(specialized_handler_name)
            .map(str::to_owned)
    })
}

fn specialized_handler_name(slug: &str) -> Option<&str> {
    match slug {
        "translation" => Some("translation"),
        "algorithm" => Some("algorithm"),
        "software_project_plan" | "software_project_implementation" => Some("software_project"),
        "meta_explanation" => Some("meta_explanation"),
        "concept_lookup" | "concept_lookup_in_context" => Some("concept_lookup"),
        "arithmetic" => Some("arithmetic"),
        "web_search" => Some("web_search"),
        "procedural_how_to" => Some("procedural_how_to"),
        "proof_request" => Some("proof_request"),
        "write_script" => Some("write_script"),
        other if !other.is_empty() => Some(other),
        _ => None,
    }
}

fn infer_kind(
    prompt: &str,
    normalized: &str,
    route: Option<&str>,
    candidate: Option<&FormalizationCandidate>,
) -> IntentKind {
    match route {
        Some("greeting" | "farewell" | "courtesy_response") => IntentKind::Courtesy,
        Some("assistant_name" | "identity") => IntentKind::Question,
        Some(
            "translation"
            | "algorithm"
            | "write_program"
            | "software_project_plan"
            | "software_project_implementation",
        ) => IntentKind::Task,
        _ if prompt.contains('?') || starts_with_question_word(normalized) => IntentKind::Question,
        _ if has_any_token(normalized, &["must", "should", "require", "requires"]) => {
            IntentKind::Requirement
        }
        _ if has_any_token(
            normalized,
            &[
                "translate",
                "write",
                "calculate",
                "search",
                "find",
                "prove",
                "define",
            ],
        ) =>
        {
            IntentKind::Task
        }
        _ if candidate.is_some_and(|candidate| !candidate.slots.is_empty()) => {
            IntentKind::Statement
        }
        _ => IntentKind::Unknown,
    }
}

fn starts_with_question_word(normalized: &str) -> bool {
    [
        "what ",
        "who ",
        "why ",
        "where ",
        "when ",
        "how ",
        "which ",
        "что ",
        "кто ",
        "как ",
        "где ",
        "когда ",
        "почему ",
    ]
    .iter()
    .any(|prefix| normalized.starts_with(prefix))
}

fn push_unique(values: &mut Vec<String>, value: String) {
    if !values.contains(&value) {
        values.push(value);
    }
}

fn escape_lino_value(value: &str) -> String {
    value
        .replace('\\', "\\\\")
        .replace('"', "\\\"")
        .replace('\n', "\\n")
        .replace('\r', "\\r")
        .replace('\t', "\\t")
}
