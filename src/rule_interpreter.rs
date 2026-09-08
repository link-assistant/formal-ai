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

use std::sync::OnceLock;

use crate::engine::{SymbolicAnswer, normalize_prompt, unknown_answer};
use crate::event_log::EventLog;
use crate::language::detect as detect_language;
use crate::seed::{self, HANDLER_RULES_LINO, Lexicon, Slot};
use crate::solver_handlers::finalize_simple;

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
}

#[derive(Debug, Clone, Copy)]
enum RoleMode {
    Spelled,
    Raw,
    Languages,
    Forms,
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
    lexicon: &'a Lexicon,
    prompt: &'a str,
    normalized: &'a str,
    cleaned: String,
    lowercase: String,
    language: String,
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
        .run_with(seed::lexicon(), prompt, normalized, log)
}

/// Whether any rule behind `name` accepts `prompt`, without producing an answer.
#[must_use]
pub fn handler_matches(name: &str, prompt: &str) -> bool {
    rules()
        .handler(name)
        .is_some_and(|set| set.matches_with(seed::lexicon(), prompt, &prompt.to_lowercase()))
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

    /// Try each rule in order against `lexicon`; the first accepted rule answers.
    #[must_use]
    pub fn run_with(
        &self,
        lexicon: &Lexicon,
        prompt: &str,
        normalized: &str,
        log: &mut EventLog,
    ) -> Option<SymbolicAnswer> {
        let language = detect_language(prompt);
        let context = Context::new(lexicon, prompt, normalized, language.slug());
        self.rules.iter().find_map(|rule| rule.run(&context, log))
    }

    /// Whether any rule accepts the prompt; nothing is logged or rendered.
    #[must_use]
    pub fn matches_with(&self, lexicon: &Lexicon, prompt: &str, normalized: &str) -> bool {
        let language = detect_language(prompt);
        let context = Context::new(lexicon, prompt, normalized, language.slug());
        let scratch = EventLog::default();
        self.rules.iter().any(|rule| {
            rule.when.holds(&context, &scratch) && rule.resolve_values(&context).is_some()
        })
    }
}

impl<'a> Context<'a> {
    fn new(lexicon: &'a Lexicon, prompt: &'a str, normalized: &'a str, language: &str) -> Self {
        Self {
            lexicon,
            prompt,
            normalized,
            cleaned: normalize_prompt(normalized),
            lowercase: prompt.to_lowercase(),
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
                match mode {
                    RoleMode::Spelled => context.lexicon.mentions_role(role, text),
                    RoleMode::Raw => context.lexicon.mentions_role_raw(role, text),
                    RoleMode::Languages => context.lexicon.mentions_role_in_languages_raw(
                        role,
                        text,
                        &context.languages(),
                    ),
                    RoleMode::Forms => context
                        .lexicon
                        .role_word_forms(role)
                        .into_iter()
                        .any(|form| !form.text.is_empty() && text.contains(&form.text)),
                }
            }
            Self::RoleLead(role, subject) => {
                let text = context.text(*subject);
                context
                    .lexicon
                    .role_word_forms(role)
                    .into_iter()
                    .any(|form| match form.slot() {
                        Slot::Prefix => text.starts_with(form.before_slot()),
                        _ => text.contains(form.text.as_str()),
                    })
            }
            Self::RolePrefix(role, subject) => {
                let text = context.text(*subject);
                context
                    .lexicon
                    .role_word_forms(role)
                    .into_iter()
                    .filter(|form| form.slot() == Slot::Prefix)
                    .any(|form| text.starts_with(form.before_slot()))
            }
            Self::RolePadded(role, subject) => {
                let text = context.text(*subject);
                let padded = format!(" {text} ");
                context
                    .lexicon
                    .role_word_forms(role)
                    .into_iter()
                    .any(|form| {
                        let marker = form.text.as_str();
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
            Self::RouteExact(slug) => seed::intent_routing()
                .intents
                .iter()
                .find(|route| route.slug == *slug)
                .is_some_and(|route| {
                    route.keywords.contains(&context.cleaned)
                        || route.phrases.contains(&context.cleaned)
                }),
            Self::HistoryRole(role) => log
                .events()
                .iter()
                .filter(|event| {
                    event.kind == "prior_turn:user" || event.kind == "prior_turn:assistant"
                })
                .any(|event| {
                    context
                        .lexicon
                        .mentions_role_raw(role, &event.payload.to_lowercase())
                }),
        }
    }
}

fn parse_rule(node: &Node) -> Result<Rule, String> {
    let name = node.first_arg()?;
    let mut when = None;
    let mut values = Vec::new();
    let mut steps = Vec::new();
    let mut response = None;
    let mut intent = None;
    let mut link = None;
    let mut confidence = 1.0;
    for child in &node.children {
        match child.name.as_str() {
            "when" => when = Some(Condition::All(parse_conditions(&child.children)?)),
            "value" => values.push(parse_value(child)?),
            "log" => {
                let kind = child.first_arg()?;
                let value = child.args.get(1).map_or_else(
                    || ValueRef::Literal(String::new()),
                    |raw| parse_value_ref(raw),
                );
                steps.push(Step::Log {
                    kind: Box::leak(kind.into_boxed_str()),
                    value,
                });
            }
            "respond" => {
                let intent_name = child.first_arg()?;
                let mut fallback = Fallback::Localized;
                let mut texts = Vec::new();
                for option in &child.children {
                    match option.name.as_str() {
                        "fallback" => {
                            fallback = match option.first_arg()?.as_str() {
                                "localized" => Fallback::Localized,
                                "exact" => Fallback::Exact,
                                language => Fallback::Language(language.to_owned()),
                            };
                        }
                        "text" => {
                            let language = option.first_arg()?;
                            let text = option
                                .args
                                .get(1)
                                .cloned()
                                .ok_or_else(|| option.error("text_without_body"))?;
                            texts.push((language, text));
                        }
                        _ => return Err(option.error("unknown_respond_option")),
                    }
                }
                response = Some(Response::Seed {
                    intent: intent_name,
                    fallback,
                    texts,
                });
            }
            "respond_unknown" => response = Some(Response::Unknown),
            "intent" => intent = Some(child.first_arg()?),
            "link" => link = Some(child.first_arg()?),
            "confidence" => {
                confidence = child
                    .first_arg()?
                    .parse::<f32>()
                    .map_err(|_| child.error("confidence_not_a_number"))?;
            }
            _ => return Err(child.error("unknown_rule_field")),
        }
    }
    Ok(Rule {
        name,
        when: when.ok_or_else(|| node.error("rule_without_when"))?,
        values,
        steps,
        response: response.ok_or_else(|| node.error("rule_without_respond"))?,
        intent,
        link,
        confidence,
    })
}

fn parse_conditions(nodes: &[Node]) -> Result<Vec<Condition>, String> {
    nodes.iter().map(parse_condition).collect()
}

fn parse_condition(node: &Node) -> Result<Condition, String> {
    let (options, subject) = split_subject(&node.args[node.args.len().min(1)..]);
    let subject = subject.map_or(Ok(Subject::Normalized), |name| parse_subject(node, name))?;
    Ok(match node.name.as_str() {
        "all" => Condition::All(parse_conditions(&node.children)?),
        "any" => Condition::Any(parse_conditions(&node.children)?),
        "none" => Condition::None(parse_conditions(&node.children)?),
        "role" => {
            let mode = match options.first().map(String::as_str) {
                None | Some("spelled") => RoleMode::Spelled,
                Some("raw") => RoleMode::Raw,
                Some("languages") => RoleMode::Languages,
                Some("forms") => RoleMode::Forms,
                Some(_) => return Err(node.error("unknown_role_mode")),
            };
            Condition::Role(node.first_arg()?, mode, subject)
        }
        "role_lead" => Condition::RoleLead(node.first_arg()?, subject),
        "role_prefix" => Condition::RolePrefix(node.first_arg()?, subject),
        "role_padded" => Condition::RolePadded(node.first_arg()?, subject),
        "word" => Condition::Word(node.first_arg()?, subject),
        "substring" => Condition::Substring(node.first_arg()?, subject),
        "prefix" => Condition::Prefix(node.first_arg()?, subject),
        "only_characters" => Condition::OnlyCharacters(node.first_arg()?),
        "unbalanced_parentheses" => Condition::UnbalancedParentheses,
        "route_exact" => Condition::RouteExact(node.first_arg()?),
        "history_role" => Condition::HistoryRole(node.first_arg()?),
        _ => return Err(node.error("unknown_condition")),
    })
}

fn split_subject(args: &[String]) -> (Vec<String>, Option<&str>) {
    let mut options = Vec::new();
    let mut subject = None;
    let mut iter = args.iter();
    while let Some(arg) = iter.next() {
        if arg == "of" {
            subject = iter.next().map(String::as_str);
        } else {
            options.push(arg.clone());
        }
    }
    (options, subject)
}

fn parse_subject(node: &Node, name: &str) -> Result<Subject, String> {
    Ok(match name {
        "normalized" => Subject::Normalized,
        "cleaned" => Subject::Cleaned,
        "lowercase" => Subject::Lowercase,
        "prompt" => Subject::Prompt,
        "trimmed" => Subject::Trimmed,
        _ => return Err(node.error("unknown_subject")),
    })
}

fn parse_value(node: &Node) -> Result<(String, ValueSource), String> {
    let name = node.first_arg()?;
    let kind = node
        .args
        .get(1)
        .ok_or_else(|| node.error("value_without_source"))?;
    let source = match kind.as_str() {
        "backticks" => ValueSource::Backticks,
        "trimmed_prompt" => ValueSource::TrimmedPrompt,
        "literal" => ValueSource::Literal(
            node.args
                .get(2)
                .cloned()
                .ok_or_else(|| node.error("literal_without_text"))?,
        ),
        "agent_info" => {
            let key = node
                .args
                .get(2)
                .cloned()
                .ok_or_else(|| node.error("agent_info_without_key"))?;
            let default = match node.args.get(3).map(String::as_str) {
                Some("default") => node.args[4..].join(" "),
                _ => String::new(),
            };
            ValueSource::AgentInfo { key, default }
        }
        _ => return Err(node.error("unknown_value_source")),
    };
    Ok((name, source))
}

fn parse_value_ref(raw: &str) -> ValueRef {
    match raw {
        "$prompt" => ValueRef::Prompt,
        "$trimmed" => ValueRef::Trimmed,
        _ => raw.strip_prefix('$').map_or_else(
            || ValueRef::Literal(raw.to_owned()),
            |name| ValueRef::Capture(name.to_owned()),
        ),
    }
}

impl Node {
    fn first_arg(&self) -> Result<String, String> {
        self.args
            .first()
            .cloned()
            .ok_or_else(|| self.error("missing_argument"))
    }

    fn error(&self, reason: &str) -> String {
        let line = self.line;
        let name = &self.name;
        format!("handler_rules:{line}:{reason}:{name}")
    }
}

/// Parse an indented Links Notation document into a tree of `name args…` nodes.
///
/// Values may be wrapped in double or single quotes to keep spaces; there are no
/// escape sequences.
fn parse_tree(text: &str) -> Vec<Node> {
    let mut roots: Vec<Node> = Vec::new();
    let mut stack: Vec<(usize, Node)> = Vec::new();
    for (index, raw) in text.lines().enumerate() {
        let trimmed = raw.trim_start();
        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }
        let indent = raw.len() - trimmed.len();
        let mut tokens = tokenize(trimmed.trim_end());
        if tokens.is_empty() {
            continue;
        }
        let node = Node {
            line: index + 1,
            name: tokens.remove(0),
            args: tokens,
            children: Vec::new(),
        };
        while stack.last().is_some_and(|(depth, _)| *depth >= indent) {
            let (_, finished) = stack.pop().expect("stack_checked");
            attach(&mut roots, &mut stack, finished);
        }
        stack.push((indent, node));
    }
    while let Some((_, finished)) = stack.pop() {
        attach(&mut roots, &mut stack, finished);
    }
    roots
}

fn attach(roots: &mut Vec<Node>, stack: &mut [(usize, Node)], node: Node) {
    match stack.last_mut() {
        Some((_, parent)) => parent.children.push(node),
        None => roots.push(node),
    }
}

fn tokenize(line: &str) -> Vec<String> {
    let mut tokens = Vec::new();
    let mut current = String::new();
    let mut quote: Option<char> = None;
    for ch in line.chars() {
        match quote {
            Some(open) if ch == open => {
                quote = None;
                tokens.push(std::mem::take(&mut current));
            }
            None if ch == '"' || ch == '\'' => quote = Some(ch),
            None if ch.is_whitespace() => {
                if !current.is_empty() {
                    tokens.push(std::mem::take(&mut current));
                }
            }
            _ => current.push(ch),
        }
    }
    if !current.is_empty() {
        tokens.push(current);
    }
    tokens
}
