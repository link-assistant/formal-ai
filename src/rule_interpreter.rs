//! Generic interpreter for seed handler rules (issue #1085 D1.3).
//!
//! A specialized handler used to be a Rust function that recognised a prompt
//! and rendered a seeded reply. `data/seed/handler-rules.lino` states the same
//! thing as links: a `handler` block per precedence name, each `rule` with a
//! `when` tree of conditions over seed roles and prompt shape, optional
//! captured `value`s, `log` steps, and a `respond` line naming a multilingual
//! response. This module walks those links; the dispatch table resolves a
//! precedence name to a block here when no native function claims it, so a
//! handler migrates by moving its recognition and wording into data and
//! deleting the Rust.

use std::collections::BTreeMap;
use std::sync::OnceLock;

use crate::engine::{SymbolicAnswer, normalize_prompt, unknown_answer};
use crate::event_log::EventLog;
use crate::language::detect as detect_language;
use crate::seed::{self, HANDLER_RULES_LINO, Slot};
use crate::seed_links::SeedLinkNetwork;
use crate::solver_handlers::finalize_simple;

mod parser;

use parser::{parse_rule, parse_tree};

/// Every handler rule set declared in `data/seed/handler-rules.lino`.
#[derive(Debug)]
pub struct HandlerRules {
    handlers: Vec<HandlerRuleSet>,
}

/// The rules behind one precedence name, tried in document order.
#[derive(Debug)]
pub struct HandlerRuleSet {
    name: String,
    rules: Vec<Rule>,
}

#[derive(Debug)]
struct Rule {
    name: String,
    when: Condition,
    values: Vec<(String, ValueSource)>,
    steps: Vec<Step>,
    response: Response,
    intent: Option<String>,
    link: Option<String>,
    confidence: f32,
}

#[derive(Debug, Clone, Copy)]
enum Subject {
    Normalized,
    Cleaned,
    Lowercase,
    Prompt,
    Trimmed,
    /// The normalized prompt with one leading and one trailing space, so a
    /// surface whose seeded form carries a word boundary matches at the edges
    /// of the input as well as inside it (issue #1138 B9, plan 09 leaf 9).
    ///
    /// `role_padded` built this shape privately for one condition; as a subject
    /// every condition can ask for it, which is what lets the Russian
    /// preposition test in
    /// `src/intent_formalization/prompt_relevants.rs` become
    /// `role temporal_preposition of padded` — seed data in every language
    /// rather than one Russian preposition compiled into Rust.
    Padded,
}

/// A structural property of the input that carries no natural language
/// (issue #1138 B9, plan 09 leaf 9).
///
/// The promotion predicates being migrated to data are not all lexical: some
/// ask whether the prompt contains a number or a time separator at all. Those
/// questions have no seeded surface to match, and writing them as a `substring`
/// of a punctuation mark would hide a structural test inside a lexical one. A
/// `shape` says what it is.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Shape {
    /// Any Unicode decimal digit.
    Digit,
    /// A colon, ASCII or fullwidth: what separates an hour from a minute in
    /// every locale this system answers in. The literal replacement for
    /// `normalized.contains(':')`.
    TimeSeparator,
    /// An absolute web address, or a bare `www.` host.
    Url,
    /// A whitespace-delimited token that is a filesystem path: it carries a
    /// separator and is not a URL.
    Path,
    /// A matched pair of quotation marks, in any of the scripts the seed
    /// declares languages for.
    Quoted,
}

#[derive(Debug, Clone, Copy)]
enum RoleMode {
    Spelled,
    Raw,
    Languages,
    Forms,
    /// A two-word role surface whose object occurs between its two words.
    Separated,
    /// The whole subject equals one of the role's surfaces (issue #1095).
    Whole,
}

#[derive(Debug)]
enum Condition {
    All(Vec<Self>),
    Any(Vec<Self>),
    None(Vec<Self>),
    Role(String, RoleMode, Subject),
    RoleLead(String, Subject),
    RolePrefix(String, Subject),
    RolePadded(String, Subject),
    Word(String, Subject),
    Substring(String, Subject),
    Prefix(String, Subject),
    OnlyCharacters(String),
    UnbalancedParentheses,
    RouteExact(String),
    HistoryRole(String),
    Shape(Shape, Subject),
}

/// One lexical surface read by the condition evaluator. Both backends expose
/// this language-neutral shape; matching stays in one evaluator below.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConditionSurface {
    pub text: String,
    pub language: String,
    pub slot: Slot,
}

/// Source of the lexical and exact-route facts used by conditions.
///
/// Evaluation is singular: every condition reads facts through this trait, and
/// since plan 09 leaf 40 the one implementation in `src/` is the projected
/// link store ([`LinkStoreSource`]). Tests may inject their own backends.
pub trait ConditionSource {
    fn role_surfaces(&self, role: &str) -> Vec<ConditionSurface>;
    fn exact_route_surfaces(&self, slug: &str) -> Vec<String>;
}

/// Stage-2 backend built by querying the boot-time link projection.
///
/// Since plan 09 leaf 40 this is the only condition backend in `src/`: the
/// production read path is the link store, and the parsed seed tables remain
/// only the projection's *input*, never a competing condition source.
#[derive(Debug, Clone)]
pub struct LinkStoreSource {
    roles: BTreeMap<String, Vec<ConditionSurface>>,
    routes: BTreeMap<String, Vec<String>>,
}

#[derive(Debug)]
enum ValueSource {
    Backticks,
    AgentInfo { key: String, default: String },
    Literal(String),
    TrimmedPrompt,
}

#[derive(Debug)]
enum ValueRef {
    Prompt,
    Trimmed,
    Capture(String),
    Literal(String),
}

#[derive(Debug)]
enum Step {
    Log { kind: &'static str, value: ValueRef },
}

#[derive(Debug)]
enum Fallback {
    Localized,
    Exact,
    Language(String),
}

#[derive(Debug)]
enum Response {
    Seed {
        intent: String,
        fallback: Fallback,
        texts: Vec<(String, String)>,
    },
    Unknown,
}

struct Node {
    line: usize,
    name: String,
    args: Vec<String>,
    children: Vec<Self>,
}

struct Context<'a> {
    source: &'a dyn ConditionSource,
    prompt: &'a str,
    normalized: &'a str,
    cleaned: String,
    lowercase: String,
    /// The normalized prompt with one leading and one trailing space; see
    /// [`Subject::Padded`].
    padded: String,
    language: String,
}

impl LinkStoreSource {
    /// The process-wide store backend over the boot-time projection.
    ///
    /// The projection is built once per process, so the condition vocabulary
    /// read from it is cached once with it. Every production caller shares this
    /// instance; `from_store` stays public for injected stores in tests.
    #[must_use]
    pub fn shared() -> &'static Self {
        static CELL: OnceLock<LinkStoreSource> = OnceLock::new();
        CELL.get_or_init(|| Self::from_store(crate::seed_links::network()))
    }

    /// Read the condition vocabulary from a projected store. The resulting
    /// index owns only query results; it never consults `seed::lexicon()`.
    #[must_use]
    pub fn from_store(store: &SeedLinkNetwork) -> Self {
        let mut roles: BTreeMap<String, Vec<ConditionSurface>> = BTreeMap::new();
        for path in store.document_paths() {
            let Some(document) = store.document(path) else {
                continue;
            };
            for root in queried_nodes(store, document)
                .into_iter()
                .filter(|node| node.to == "meanings")
            {
                for meaning in queried_nodes(store, &root.index) {
                    let children = queried_nodes(store, &meaning.index);
                    let meaning_roles = children
                        .iter()
                        .filter(|child| child.to == "role")
                        .filter_map(|child| store.value_of(&child.index))
                        .map(str::to_owned)
                        .collect::<Vec<_>>();
                    if meaning_roles.is_empty() {
                        continue;
                    }
                    let mut surfaces = Vec::new();
                    for child in &children {
                        if child.to == "lexeme" {
                            let language = store
                                .field(&child.index, "language")
                                .or_else(|| store.value_of(&child.index))
                                .unwrap_or_default();
                            surfaces.extend(
                                queried_nodes(store, &child.index)
                                    .into_iter()
                                    .filter(|surface| {
                                        matches!(surface.to.as_str(), "surface" | "word")
                                    })
                                    .filter_map(|surface| {
                                        stored_surface(store, &surface.index, language)
                                    }),
                            );
                        } else if child.to == "surface" {
                            let language =
                                store.field(&child.index, "language").unwrap_or_default();
                            if let Some(surface) = stored_surface(store, &child.index, language) {
                                surfaces.push(surface);
                            }
                        }
                    }
                    for role in meaning_roles {
                        let entries = roles.entry(role).or_default();
                        for surface in &surfaces {
                            if !entries.contains(surface) {
                                entries.push(surface.clone());
                            }
                        }
                    }
                }
            }
        }

        let mut routes = BTreeMap::new();
        if let Some(document) = store.document(seed::INTENT_ROUTING_PATH) {
            for root in queried_nodes(store, document)
                .into_iter()
                .filter(|node| node.to == "intent_routing")
            {
                for intent in queried_nodes(store, &root.index)
                    .into_iter()
                    .filter(|node| node.to == "intent")
                {
                    let Some(slug) = store.field(&intent.index, "slug") else {
                        continue;
                    };
                    let mut surfaces = store.field_values(&intent.index, "keyword");
                    surfaces.extend(store.field_values(&intent.index, "phrase"));
                    // A family that retired its rows onto a declared role
                    // (`role_surface`, issue #1138 plan 10 leaf 20) keeps its
                    // exact-match semantics: the role's surfaces are the
                    // route's surfaces, so `route_exact` conditions keep
                    // deciding on the same whole prompts.
                    for role in store.field_values(&intent.index, "role_surface") {
                        if let Some(entries) = roles.get(&role) {
                            for surface in entries {
                                if !surfaces.contains(&surface.text) {
                                    surfaces.push(surface.text.clone());
                                }
                            }
                        }
                    }
                    routes.insert(slug.to_owned(), surfaces);
                }
            }
        }
        Self { roles, routes }
    }
}

impl ConditionSource for LinkStoreSource {
    fn role_surfaces(&self, role: &str) -> Vec<ConditionSurface> {
        self.roles.get(role).cloned().unwrap_or_default()
    }

    fn exact_route_surfaces(&self, slug: &str) -> Vec<String> {
        self.routes.get(slug).cloned().unwrap_or_default()
    }
}

fn queried_nodes(store: &SeedLinkNetwork, parent: &str) -> Vec<crate::link_store::DoubletLink> {
    store
        .query(&SeedLinkNetwork::children_pattern(parent))
        .into_iter()
        .filter(|link| !link.index.ends_with('='))
        .collect()
}

fn stored_surface(store: &SeedLinkNetwork, node: &str, language: &str) -> Option<ConditionSurface> {
    let text = match store.field(node, "text") {
        Some(text) if !text.is_empty() => text.to_owned(),
        _ => match store.field(node, "codepoints") {
            Some(codepoints) if !codepoints.is_empty() => {
                crate::seed::parser::decode_codepoints(codepoints)
            }
            _ => store.value_of(node).unwrap_or_default().to_owned(),
        },
    };
    (!text.is_empty()).then(|| ConditionSurface {
        slot: slot_of(&text),
        text,
        language: language.to_owned(),
    })
}

fn slot_of(text: &str) -> Slot {
    match text.split_once('…') {
        None => Slot::Bare,
        Some((before, after)) => match (!before.is_empty(), !after.is_empty()) {
            (true, true) => Slot::Circumfix,
            (true, false) => Slot::Prefix,
            (false, true) => Slot::Suffix,
            (false, false) => Slot::Bare,
        },
    }
}

/// The embedded rule document, parsed once.
///
/// # Panics
///
/// Panics when the embedded document is malformed; `tests/unit` parses it
/// explicitly so the panic can only follow a seed edit that skipped the tests.
#[must_use]
pub fn rules() -> &'static HandlerRules {
    static CACHE: OnceLock<HandlerRules> = OnceLock::new();
    CACHE.get_or_init(|| {
        HandlerRules::parse(HANDLER_RULES_LINO).unwrap_or_else(|error| panic!("{error}"))
    })
}

/// Precedence names the embedded rule document implements.
#[must_use]
pub fn handler_names() -> Vec<&'static str> {
    rules().handler_names().collect()
}

/// Run the rule set behind `name` the way a native handler would run.
#[must_use]
pub fn run_handler(
    name: &str,
    prompt: &str,
    normalized: &str,
    log: &mut EventLog,
) -> Option<SymbolicAnswer> {
    rules()
        .handler(name)?
        .run_with_source(LinkStoreSource::shared(), prompt, normalized, log)
}

/// Whether any rule behind `name` accepts `prompt`, without producing an answer.
#[must_use]
pub fn handler_matches(name: &str, prompt: &str) -> bool {
    rules().handler(name).is_some_and(|set| {
        set.matches_with_source(LinkStoreSource::shared(), prompt, &prompt.to_lowercase())
    })
}

/// One rule-condition result, before value capture or response rendering.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConditionVerdict {
    pub owner: String,
    pub matched: bool,
}

impl HandlerRules {
    /// Parse a rule document.
    ///
    /// # Errors
    ///
    /// Returns a `handler_rules:<line>:<reason>` code for the first malformed
    /// line.
    pub fn parse(text: &str) -> Result<Self, String> {
        let nodes = parse_tree(text);
        let root = nodes
            .iter()
            .find(|node| node.name == "handler_rules")
            .ok_or_else(|| String::from("handler_rules:0:missing_root"))?;
        let mut handlers = Vec::new();
        for node in root.children.iter().filter(|node| node.name == "handler") {
            let name = node.first_arg()?;
            let mut rules = Vec::new();
            for rule_node in node.children.iter().filter(|child| child.name == "rule") {
                rules.push(parse_rule(rule_node)?);
            }
            if rules.is_empty() {
                return Err(node.error("handler_without_rules"));
            }
            handlers.push(HandlerRuleSet { name, rules });
        }
        Ok(Self { handlers })
    }

    /// The rule set behind a precedence name.
    #[must_use]
    pub fn handler(&self, name: &str) -> Option<&HandlerRuleSet> {
        self.handlers.iter().find(|set| set.name == name)
    }

    /// Precedence names in document order.
    pub fn handler_names(&self) -> impl Iterator<Item = &str> {
        self.handlers.iter().map(|set| set.name.as_str())
    }

    /// Number of rules across every handler.
    #[must_use]
    pub fn rule_count(&self) -> usize {
        self.handlers.iter().map(|set| set.rules.len()).sum()
    }

    /// Evaluate every declared rule condition through one source. No
    /// short-circuit by handler occurs, so the parity gate covers all rows.
    #[must_use]
    pub fn condition_verdicts(
        &self,
        source: &dyn ConditionSource,
        prompt: &str,
        normalized: &str,
        log: &EventLog,
    ) -> Vec<ConditionVerdict> {
        let language = detect_language(prompt);
        let context = Context::new(source, prompt, normalized, language.slug());
        let mut verdicts = Vec::with_capacity(self.rule_count());
        for handler in &self.handlers {
            for rule in &handler.rules {
                verdicts.push(ConditionVerdict {
                    owner: format!("rule:{}:{}", handler.name, rule.name),
                    matched: rule.when.holds(&context, log),
                });
            }
        }
        verdicts
    }
}

impl HandlerRuleSet {
    /// The precedence name this set implements.
    #[must_use]
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Rule names in the order they are tried.
    pub fn rule_names(&self) -> impl Iterator<Item = &str> {
        self.rules.iter().map(|rule| rule.name.as_str())
    }

    /// Try each rule using an explicit condition source.
    #[must_use]
    pub fn run_with_source(
        &self,
        source: &dyn ConditionSource,
        prompt: &str,
        normalized: &str,
        log: &mut EventLog,
    ) -> Option<SymbolicAnswer> {
        let language = detect_language(prompt);
        let context = Context::new(source, prompt, normalized, language.slug());
        self.rules.iter().find_map(|rule| rule.run(&context, log))
    }

    /// Whether any rule accepts a prompt through an explicit source.
    #[must_use]
    pub fn matches_with_source(
        &self,
        source: &dyn ConditionSource,
        prompt: &str,
        normalized: &str,
    ) -> bool {
        let language = detect_language(prompt);
        let context = Context::new(source, prompt, normalized, language.slug());
        let scratch = EventLog::default();
        self.rules.iter().any(|rule| {
            rule.when.holds(&context, &scratch) && rule.resolve_values(&context).is_some()
        })
    }
}

impl<'a> Context<'a> {
    fn new(
        source: &'a dyn ConditionSource,
        prompt: &'a str,
        normalized: &'a str,
        language: &str,
    ) -> Self {
        Self {
            source,
            prompt,
            normalized,
            cleaned: normalize_prompt(normalized),
            lowercase: prompt.to_lowercase(),
            padded: {
                let mut padded = String::with_capacity(normalized.len() + 2);
                padded.push(' ');
                padded.push_str(normalized);
                padded.push(' ');
                padded
            },
            language: language.to_owned(),
        }
    }

    fn text(&self, subject: Subject) -> &str {
        match subject {
            Subject::Normalized => self.normalized,
            Subject::Cleaned => &self.cleaned,
            Subject::Lowercase => &self.lowercase,
            Subject::Prompt => self.prompt,
            Subject::Trimmed => self.normalized.trim(),
            Subject::Padded => &self.padded,
        }
    }

    fn languages(&self) -> Vec<&str> {
        if self.language == "en" {
            vec!["en"]
        } else {
            vec![self.language.as_str(), "en"]
        }
    }
}

impl Rule {
    fn run(&self, context: &Context<'_>, log: &mut EventLog) -> Option<SymbolicAnswer> {
        if !self.when.holds(context, log) {
            return None;
        }
        let values = self.resolve_values(context)?;
        log.append("rule_interpreter:rule", self.name.clone());
        for step in &self.steps {
            match step {
                Step::Log { kind, value } => {
                    let payload = value.resolve(context, &values)?;
                    log.append(kind, payload);
                }
            }
        }
        let body = self.response.render(context, &values)?;
        let intent = self.intent.as_deref().unwrap_or(&self.name);
        let default_link = format!("response:{intent}");
        let link = self.link.as_deref().unwrap_or(&default_link);
        Some(finalize_simple(
            context.prompt,
            log,
            intent,
            link,
            &body,
            self.confidence,
        ))
    }

    fn resolve_values(&self, context: &Context<'_>) -> Option<Vec<(String, String)>> {
        let mut resolved = Vec::with_capacity(self.values.len());
        for (name, source) in &self.values {
            let value = match source {
                ValueSource::Backticks => context
                    .prompt
                    .split('`')
                    .nth(1)
                    .map(str::trim)
                    .filter(|term| !term.is_empty())?
                    .to_owned(),
                ValueSource::AgentInfo { key, default } => seed::agent_info()
                    .get(key)
                    .cloned()
                    .unwrap_or_else(|| default.clone()),
                ValueSource::Literal(text) => text.clone(),
                ValueSource::TrimmedPrompt => context.prompt.trim().to_owned(),
            };
            resolved.push((name.clone(), value));
        }
        Some(resolved)
    }
}

impl ValueRef {
    fn resolve(&self, context: &Context<'_>, values: &[(String, String)]) -> Option<String> {
        match self {
            Self::Prompt => Some(context.prompt.to_owned()),
            Self::Trimmed => Some(context.prompt.trim().to_owned()),
            Self::Capture(name) => values
                .iter()
                .find(|(candidate, _)| candidate == name)
                .map(|(_, value)| value.clone()),
            Self::Literal(text) => Some(text.clone()),
        }
    }
}

impl Response {
    fn render(&self, context: &Context<'_>, values: &[(String, String)]) -> Option<String> {
        match self {
            Self::Unknown => Some(unknown_answer().to_owned()),
            Self::Seed {
                intent,
                fallback,
                texts,
            } => {
                let pairs: Vec<(&str, &str)> = values
                    .iter()
                    .map(|(name, value)| (name.as_str(), value.as_str()))
                    .collect();
                let inline = |language: &str| {
                    texts
                        .iter()
                        .find(|(candidate, _)| candidate == language)
                        .map(|(_, text)| substitute(text, &pairs))
                };
                inline(&context.language)
                    .or_else(|| match fallback {
                        Fallback::Exact => seed::render_response(intent, &context.language, &pairs),
                        Fallback::Localized => seed::localized_response(intent, &context.language)
                            .map(|text| substitute(&text, &pairs)),
                        Fallback::Language(language) => {
                            seed::render_response(intent, &context.language, &pairs)
                                .or_else(|| seed::render_response(intent, language, &pairs))
                        }
                    })
                    .or_else(|| inline("en"))
            }
        }
    }
}

fn substitute(text: &str, pairs: &[(&str, &str)]) -> String {
    pairs
        .iter()
        .fold(text.to_owned(), |rendered, (name, value)| {
            rendered.replace(&format!("{{{name}}}"), value)
        })
}

impl Condition {
    fn holds(&self, context: &Context<'_>, log: &EventLog) -> bool {
        match self {
            Self::All(children) => children.iter().all(|child| child.holds(context, log)),
            Self::Any(children) => children.iter().any(|child| child.holds(context, log)),
            Self::None(children) => !children.iter().any(|child| child.holds(context, log)),
            Self::Role(role, mode, subject) => {
                let text = context.text(*subject);
                let surfaces = context.source.role_surfaces(role);
                match mode {
                    RoleMode::Spelled => surfaces
                        .iter()
                        .any(|surface| spelled_surface_present(text, surface)),
                    RoleMode::Raw => surfaces
                        .iter()
                        .any(|surface| !surface.text.is_empty() && text.contains(&surface.text)),
                    RoleMode::Languages => {
                        let languages = context.languages();
                        surfaces.iter().any(|surface| {
                            languages.contains(&surface.language.as_str())
                                && !surface.text.is_empty()
                                && text.contains(&surface.text)
                        })
                    }
                    RoleMode::Forms => surfaces
                        .iter()
                        .any(|surface| !surface.text.is_empty() && text.contains(&surface.text)),
                    RoleMode::Separated => surfaces
                        .iter()
                        .any(|surface| separated_surface_present(text, &surface.text)),
                    // Equality, not containment: a turn that *is* a surface
                    // ("continue") carries the role; a request that contains
                    // the word ("continue the migration in src/queue.rs")
                    // keeps its own meaning (issue #1095).
                    RoleMode::Whole => {
                        !text.is_empty() && surfaces.iter().any(|surface| surface.text == text)
                    }
                }
            }
            Self::RoleLead(role, subject) => {
                let text = context.text(*subject);
                context
                    .source
                    .role_surfaces(role)
                    .into_iter()
                    .any(|surface| match surface.slot {
                        Slot::Prefix => text.starts_with(before_slot(&surface.text)),
                        _ => text.contains(surface.text.as_str()),
                    })
            }
            Self::RolePrefix(role, subject) => {
                let text = context.text(*subject);
                context
                    .source
                    .role_surfaces(role)
                    .into_iter()
                    .filter(|surface| surface.slot == Slot::Prefix)
                    .any(|surface| text.starts_with(before_slot(&surface.text)))
            }
            Self::RolePadded(role, subject) => {
                let text = context.text(*subject);
                let padded = format!(" {text} ");
                context
                    .source
                    .role_surfaces(role)
                    .into_iter()
                    .any(|surface| {
                        let marker = surface.text.as_str();
                        if marker.starts_with(' ') || marker.ends_with(' ') {
                            padded.contains(marker)
                        } else {
                            text.contains(marker)
                        }
                    })
            }
            Self::Word(word, subject) => context
                .text(*subject)
                .split_whitespace()
                .any(|token| token == word),
            Self::Substring(needle, subject) => context.text(*subject).contains(needle.as_str()),
            Self::Prefix(needle, subject) => context.text(*subject).starts_with(needle.as_str()),
            Self::OnlyCharacters(set) => {
                let trimmed = context.prompt.trim();
                !trimmed.is_empty() && trimmed.chars().all(|ch| set.contains(ch))
            }
            Self::UnbalancedParentheses => {
                let opens = context.prompt.chars().filter(|ch| *ch == '(').count();
                let closes = context.prompt.chars().filter(|ch| *ch == ')').count();
                opens != closes
            }
            Self::RouteExact(slug) => context
                .source
                .exact_route_surfaces(slug)
                .iter()
                .any(|surface| surface == &context.cleaned),
            Self::HistoryRole(role) => {
                log.events()
                    .iter()
                    .filter(|event| {
                        event.kind == "prior_turn:user" || event.kind == "prior_turn:assistant"
                    })
                    .any(|event| {
                        let payload = event.payload.to_lowercase();
                        context.source.role_surfaces(role).iter().any(|surface| {
                            !surface.text.is_empty() && payload.contains(&surface.text)
                        })
                    })
            }
            Self::Shape(shape, subject) => shape.holds(context.text(*subject)),
        }
    }
}

/// Whether a seeded role surface is present, including the open slot carried by
/// a semantic form such as `how many … remain`.
///
/// The rule interpreter used to compare the literal ellipsis character for
/// [`Slot`] forms. That made slot-backed meanings usable by their dedicated
/// recognizers but invisible to data-authored handler promotions. Interpreting
/// the same slot metadata here keeps the lexicon as the single recognition
/// authority: a newly seeded paraphrase can affect dispatch without a Rust
/// phrase branch.
///
/// A prefix surface's lead half ("prove …") is a phrase the request is expected
/// to *start some rendering of*, so it must be present as complete words. A raw
/// substring would let an unrelated word of another language that merely embeds
/// the phrase — Spanish "proveedores" embeds "prove" — claim the surface and
/// fire the promotion that reads it (issue #1138, plan 03 wave F).
fn spelled_surface_present(text: &str, surface: &ConditionSurface) -> bool {
    let expected = surface.text.as_str();
    match surface.slot {
        Slot::Bare => bounded_surface_present(text, expected),
        Slot::Prefix => expected
            .split_once('…')
            .is_some_and(|(before, _)| phrase_present_as_words(text, before.trim())),
        Slot::Suffix => expected
            .split_once('…')
            .is_some_and(|(_, after)| text.contains(after.trim())),
        Slot::Circumfix => expected.split_once('…').is_some_and(|(before, after)| {
            let before = before.trim();
            let after = after.trim();
            if before.is_empty() || after.is_empty() {
                return false;
            }
            text.find(before).is_some_and(|start| {
                let tail = &text[start + before.len()..];
                tail.find(after)
                    .is_some_and(|offset| !tail[..offset].trim().is_empty())
            })
        }),
    }
}

fn bounded_surface_present(text: &str, expected: &str) -> bool {
    if expected.is_empty() {
        return false;
    }
    if crate::coding::contains_cjk(expected) {
        return text.contains(expected);
    }
    text == expected
        || text.starts_with(&format!("{expected} "))
        || text.ends_with(&format!(" {expected}"))
        || text.contains(&format!(" {expected} "))
}

/// Whether `phrase` occurs in `text` as complete words: every character border
/// of the match is a word boundary, where a word boundary is the text edge or a
/// non-alphanumeric character. Punctuation counts as a boundary, so "¿Cuántos"
/// still begins the phrase "cuántos …", while the "prove" inside
/// "proveedores" does not — an embedded word is not the phrase.
fn phrase_present_as_words(text: &str, phrase: &str) -> bool {
    if phrase.is_empty() {
        return false;
    }
    if crate::coding::contains_cjk(phrase) {
        return text.contains(phrase);
    }
    let mut search = 0;
    while let Some(start) = text[search..].find(phrase) {
        let start = search + start;
        let end = start + phrase.len();
        let opens_word = text[..start]
            .chars()
            .next_back()
            .is_none_or(|before| !before.is_alphanumeric());
        let closes_word = text[end..]
            .chars()
            .next()
            .is_none_or(|after| !after.is_alphanumeric());
        if opens_word && closes_word {
            return true;
        }
        search = end;
    }
    false
}

/// The furthest a phrasal verb's object may separate its verb and particle.
///
/// This mirrors the semantic lexicon's bound, but operates on
/// [`ConditionSource`] surfaces so every condition backend evaluates the same
/// data-owned condition.
const SEPARATED_ROLE_OBJECT_LIMIT: usize = 6;

fn separated_surface_present(text: &str, surface: &str) -> bool {
    let mut parts = surface.split_whitespace();
    let (Some(verb), Some(particle), None) = (parts.next(), parts.next(), parts.next()) else {
        return false;
    };
    let words: Vec<&str> = text.split_whitespace().collect();
    words.iter().enumerate().any(|(index, word)| {
        *word == verb
            && words
                .iter()
                .skip(index + 2)
                .take(SEPARATED_ROLE_OBJECT_LIMIT)
                .any(|later| *later == particle)
    })
}

fn before_slot(text: &str) -> &str {
    text.split_once('…').map_or(text, |(before, _)| before)
}

/// Quotation marks that open and close a span, paired.
///
/// Seed data, not prose: these are the delimiter pairs the five registered
/// languages write quotations with. A `shape quoted` asks whether a span is
/// delimited, never what it says.
const QUOTE_PAIRS: [(char, char); 6] = [
    ('"', '"'),
    ('\'', '\''),
    ('\u{ab}', '\u{bb}'),
    ('\u{201c}', '\u{201d}'),
    ('\u{300c}', '\u{300d}'),
    ('\u{2018}', '\u{2019}'),
];

impl Shape {
    /// Whether the subject text has this structural property.
    fn holds(self, text: &str) -> bool {
        match self {
            Self::Digit => text.chars().any(char::is_numeric),
            Self::TimeSeparator => text.contains(':') || text.contains('\u{ff1a}'),
            Self::Url => text
                .split_whitespace()
                .any(|token| is_url(&token.to_lowercase())),
            Self::Path => text.split_whitespace().any(|token| {
                let lowered = token.to_lowercase();
                !is_url(&lowered) && (token.contains('/') || token.contains('\\'))
            }),
            Self::Quoted => QUOTE_PAIRS.iter().any(|(open, close)| {
                if open == close {
                    text.matches(*open).count() >= 2
                } else {
                    text.find(*open)
                        .is_some_and(|start| text[start..].chars().skip(1).any(|ch| ch == *close))
                }
            }),
        }
    }
}

/// How an absolute web address begins. Structural tokens, not prose: a scheme
/// and the conventional bare host, neither of which is written in any language.
const URL_PREFIXES: [&str; 3] = ["http://", "https://", "www."];

/// Whether one whitespace-delimited, already lowercased token is a web address.
fn is_url(token: &str) -> bool {
    URL_PREFIXES.iter().any(|prefix| token.starts_with(*prefix))
}
