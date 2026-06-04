//! Links-Notation intent formalization and cache.
//!
//! The P/Q formalization layer identifies meaning anchors inside a prompt.
//! This module wraps those anchors in the routing-facing intent record: what
//! kind of impulse arrived, what is already known, and which handlers or rules
//! are relevant.

use std::collections::BTreeMap;
use std::fmt::Write as _;
use std::sync::OnceLock;

use crate::engine::{
    normalize_prompt, program_language_by_alias, program_spec, stable_id, SelectedRule,
    WRITE_PROGRAM_INTENT,
};
use crate::event_log::EventLog;
use crate::link_store::{LinkStore, LinkStoreError};
use crate::memory::MemoryEvent;
use crate::probability::ProbabilityStore;
use crate::seed;
use crate::solver::{ConversationTurn, UniversalSolver};
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
    // CJK scripts have no inter-word spaces, so match those aliases by substring
    // (see `coding::catalog::contains_cjk`). Latin/Cyrillic keep strict
    // whitespace boundaries so short tokens never match inside larger words.
    if crate::coding::contains_cjk(expected) {
        return normalized.contains(expected);
    }
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

/// Outcome of trying to complete a follow-up `write_program` request from the
/// conversation so far (issue #324).
pub(crate) struct WriteProgramRecovery {
    /// The rule after recovery — upgraded to [`SelectedRule::WriteProgram`] when
    /// enough context was found, otherwise the original unsupported rule with any
    /// parameters we managed to fill in.
    pub rule: SelectedRule,
    /// A short trace describing what was carried over, for the event log. `None`
    /// when nothing was recovered.
    pub trace: Option<String>,
    /// The program-modification plan as Links Notation, surfaced when a modifier
    /// rewrote the task via the substitution pipeline (issue #324 R4/R6). `None`
    /// when no modification rule fired.
    pub plan: Option<String>,
}

/// Outcome of recognizing a bare imperative as a follow-up that refers to the
/// active program artifact rather than a standalone algorithm/text request.
pub(crate) struct ProgramCoreferenceRewrite {
    pub rule: SelectedRule,
    pub trace: String,
}

/// Issue #357: after a program has been generated, users often ask bare
/// imperative follow-ups such as "Sort the results in reverse order" or
/// "Сделай сортировку результатов..." without repeating "program". Those turns
/// may route to another handler (or to unknown) even though "results" refers to
/// the active program artifact. Reclassify only that narrow shape as an
/// unsupported `write_program` request, then let the existing context recovery
/// bind the concrete task and language.
#[must_use]
pub(crate) fn rewrite_bare_program_coreference_rule(
    rule: &SelectedRule,
    follow_up: &str,
    history: &[ConversationTurn],
) -> Option<ProgramCoreferenceRewrite> {
    if matches!(
        rule,
        SelectedRule::WriteProgram(_) | SelectedRule::UnsupportedWriteProgram { .. }
    ) {
        return None;
    }

    let normalized = normalize_prompt(follow_up);
    if !crate::program_coreference::looks_like_bare_program_artifact_follow_up(&normalized) {
        return None;
    }

    let context = active_program_context(history)?;
    let trace = format!(
        "referent=active_program_artifact task={} language={}",
        context.task, context.language
    );
    Some(ProgramCoreferenceRewrite {
        rule: SelectedRule::UnsupportedWriteProgram {
            task: Some(context.task),
            language: Some(context.language),
        },
        trace,
    })
}

pub(crate) struct ActiveProgramContext {
    pub(crate) task: String,
    pub(crate) language: String,
}

pub(crate) fn active_program_context(history: &[ConversationTurn]) -> Option<ActiveProgramContext> {
    let mut task = None;
    let mut language = None;
    for turn in history.iter().rev() {
        let normalized = normalize_prompt(&turn.content);
        let Some(parameters) = write_program_parameters(&normalized) else {
            continue;
        };
        if task.is_none() {
            task = parameters.get("task").cloned();
        }
        if language.is_none() {
            language = parameters.get("language").cloned();
        }
        if task.is_some() && language.is_some() {
            break;
        }
    }
    Some(ActiveProgramContext {
        task: task?,
        language: language?,
    })
}

/// Issue #324: a follow-up such as "Сделай так, чтобы программа принимала путь
/// как аргумент" ("make the program accept a path as an argument") routes to
/// `write_program` because it pairs a program noun with an imperative verb, yet
/// it names neither a concrete task nor a language — both came from the previous
/// turn. Without conversation context this surfaced the user-reported error
/// ("I do not have a template for language `missing` and task `missing`").
///
/// When the selected rule is [`SelectedRule::UnsupportedWriteProgram`] we recover
/// the missing task and language from the most recent prior turn that named them
/// and apply any data-defined modification modifier present in the follow-up.
/// If the recovered `(task, language)` pair has a template we upgrade the rule
/// to a concrete program; otherwise we return the rule with whatever we could
/// fill in so the unsupported message is still as specific as possible.
#[must_use]
pub(crate) fn recover_write_program_rule(
    rule: SelectedRule,
    follow_up: &str,
    history: &[ConversationTurn],
) -> WriteProgramRecovery {
    let SelectedRule::UnsupportedWriteProgram { task, language } = &rule else {
        return WriteProgramRecovery {
            rule,
            trace: None,
            plan: None,
        };
    };

    let mut recovered_task = task.clone();
    let mut recovered_language = language.clone();

    if recovered_task.is_none() || recovered_language.is_none() {
        for turn in history.iter().rev() {
            let normalized = normalize_prompt(&turn.content);
            let Some(parameters) = write_program_parameters(&normalized) else {
                continue;
            };
            if recovered_task.is_none() {
                recovered_task = parameters.get("task").cloned();
            }
            if recovered_language.is_none() {
                recovered_language = parameters.get("language").cloned();
            }
            if recovered_task.is_some() && recovered_language.is_some() {
                break;
            }
        }
    }

    // A modification follow-up lowers the recovered base task through the Links
    // Notation substitution pipeline, which rewrites e.g. `list_files ->
    // list_files_arg` or `list_files_arg -> list_files_arg_reverse_sort`. The
    // plan is captured as Links Notation for transparent tracing.
    let normalized_follow_up = normalize_prompt(follow_up);
    let modifiers = detected_program_modifiers(&normalized_follow_up);
    let mut plan = None;
    if !modifiers.is_empty() {
        if let Some(base) = recovered_task.as_deref() {
            let lowered = crate::program_plan::lower(base, &modifiers);
            if lowered.was_modified() {
                plan = Some(lowered.links_notation());
            }
            recovered_task = Some(lowered.resolved_task);
        }
    }

    if let (Some(task_slug), Some(language_slug)) =
        (recovered_task.as_deref(), recovered_language.as_deref())
    {
        if let Some(spec) = program_spec(task_slug, language_slug) {
            let trace = format!("write_program task={task_slug} language={language_slug}");
            return WriteProgramRecovery {
                rule: SelectedRule::WriteProgram(spec),
                trace: Some(trace),
                plan,
            };
        }
    }

    WriteProgramRecovery {
        rule: SelectedRule::UnsupportedWriteProgram {
            task: recovered_task,
            language: recovered_language,
        },
        trace: None,
        plan,
    }
}

fn operation_vocabulary() -> &'static seed::OperationVocabulary {
    static VOCABULARY: OnceLock<seed::OperationVocabulary> = OnceLock::new();
    VOCABULARY.get_or_init(seed::operation_vocabulary)
}

/// Detect the modification modifiers present in a (normalized) request, returned
/// as the slugs the substitution pipeline keys on.
///
/// Recognition is data-driven in two stages: `operation-vocabulary.lino` owns
/// natural-language trigger phrases, and `program-plan-rules.lino` decides which
/// operation slugs are valid program modifiers by declaring
/// `request:modifier -> <slug>` conditions.
pub(crate) fn detected_program_modifiers(normalized: &str) -> Vec<String> {
    let program_modifiers = crate::program_plan::modifier_slugs();
    operation_vocabulary()
        .detect(normalized)
        .into_iter()
        .filter(|slug| program_modifiers.contains(slug.as_str()))
        .collect()
}

fn write_program_parameters(normalized: &str) -> Option<BTreeMap<String, String>> {
    let task = crate::coding::program_task_by_alias(normalized);
    let language = requested_program_language(normalized);
    // Issue #386: "write a <program>" is recognised by *meaning*, not a hardcoded
    // per-language word list. The prompt asks for a program when it evidences a
    // `program_kind` meaning (the artefact: program / script / code / function)
    // *and* a `program_request` meaning (the verb: write / create / show /
    // generate / make / build). The surface words for every language live once,
    // in `data/seed/meanings.lino`; this code understands the concepts.
    let lexicon = crate::seed::lexicon();
    let asks_for_program = lexicon.mentions_role(crate::seed::ROLE_PROGRAM_KIND, normalized)
        && lexicon.mentions_role(crate::seed::ROLE_PROGRAM_REQUEST, normalized);
    if task.is_none() && !asks_for_program {
        return None;
    }
    let mut parameters = BTreeMap::new();
    if let Some(task) = task {
        // Issue #358: modification phrases in the same turn lower the base task
        // through the data-backed substitution pipeline so composed requests can
        // resolve directly.
        let modifiers = detected_program_modifiers(normalized);
        let task_slug = crate::program_plan::resolve_task(task.slug, &modifiers);
        parameters.insert(String::from("task"), task_slug);
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
    // Issue #386: the function words that introduce an *unknown* implementation
    // language ("write a program in <name>", "на языке <name>") are seed data,
    // not literals baked into the parser. Source the head-initial English/Russian
    // surfaces of the target-preposition and "language" noun roles from the
    // lexicon so this positional extractor reasons over the ontology instead of a
    // hardcoded `matches!` list. The catalog-driven `program_language_by_alias`
    // above already resolves every *known* language across all four supported
    // languages; this fallback only reads the bare name trailing the marker, so
    // it consults the two head-initial languages whose name follows the marker
    // (the head-final Hindi/Chinese forms are carried in the seed for coverage
    // but place the name before the marker, which this scan does not chase).
    let lexicon = crate::seed::lexicon();
    let preposition_surfaces = lexicon.words_for_role_in_languages(
        crate::seed::ROLE_IMPLEMENTATION_LANGUAGE_PREPOSITION,
        &["en", "ru"],
    );
    let language_noun_surfaces = lexicon.words_for_role_in_languages(
        crate::seed::ROLE_IMPLEMENTATION_LANGUAGE_NOUN,
        &["en", "ru"],
    );
    let tokens = normalized.split_whitespace().collect::<Vec<_>>();
    for (index, token) in tokens.iter().enumerate() {
        if !preposition_surfaces
            .iter()
            .any(|surface| surface.as_str() == *token)
        {
            continue;
        }
        let Some(next) = tokens.get(index + 1) else {
            continue;
        };
        if language_noun_surfaces
            .iter()
            .any(|surface| surface.as_str() == *next)
        {
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
    let operation_view = seed::operation_vocabulary().canonicalized_prompt(normalized);
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
            "handler:program_synthesis",
            looks_like_program_synthesis(&operation_view),
        ),
        (
            "handler:text_manipulation",
            looks_like_text_manipulation(&operation_view),
        ),
        (
            "handler:software_project",
            has_any_token(normalized, &["build", "create", "implement", "develop"]),
        ),
        (
            "handler:meta_explanation",
            seed::lexicon().mentions_role_raw(seed::ROLE_ASSISTANT_MECHANISM_INQUIRY, normalized),
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

fn looks_like_program_synthesis(normalized: &str) -> bool {
    // Routing mirror of `crate::solver_handlers::program_synthesis`'s gate, over
    // the canonicalized view: a function *subject*, a *domain* signal (Python or
    // a data kind) or the similar-elements task signal, and a request *action*
    // verb. Every surface word comes from the meaning lexicon, not from literals.
    let lexicon = crate::seed::lexicon();
    let similar_elements = lexicon
        .meaning("signal_similar_elements")
        .is_some_and(|signal| signal.evidenced_in(normalized));
    lexicon.mentions_role(crate::seed::ROLE_PROGRAM_SYNTHESIS_SUBJECT, normalized)
        && (lexicon.mentions_role(crate::seed::ROLE_PROGRAM_SYNTHESIS_DOMAIN, normalized)
            || similar_elements)
        && lexicon.mentions_role(crate::seed::ROLE_PROGRAM_SYNTHESIS_ACTION, normalized)
}

fn looks_like_text_manipulation(normalized: &str) -> bool {
    [
        "uppercase",
        "lowercase",
        "replace",
        "extract email",
        "count occurrences",
        "count unique words",
        "deduplicate lines",
        "sort lines",
        "reverse words",
    ]
    .iter()
    .any(|operation| normalized.contains(operation))
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
            | "text_manipulation"
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
    // The interrogative openers (the wh-words) are carried by the
    // `interrogative_opener` meaning in the seed, not hardcoded here. English and
    // Russian are head-initial, so the opener fronts the prompt and a prefix match
    // — the bare word followed immediately by a space — detects it; the head-final
    // Hindi and Chinese surfaces are carried for coverage but place the question
    // word later, which this front scan does not chase.
    crate::seed::lexicon()
        .words_for_role_in_languages(crate::seed::ROLE_INTERROGATIVE_OPENER, &["en", "ru"])
        .iter()
        .any(|word| {
            normalized
                .strip_prefix(word.as_str())
                .is_some_and(|rest| rest.starts_with(' '))
        })
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
