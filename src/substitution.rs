//! Data-driven link-pattern substitution rules.
//!
//! This module implements the issue #301 `replace x y` primitive over
//! doublet-shaped link patterns. Rules are stored as Links Notation, sorted by
//! explicit order and id, attached to CRUD events, and each applied rule
//! produces an inspectable trace link.

use std::collections::{BTreeMap, BTreeSet};
use std::error::Error;
use std::fmt;
use std::fmt::Write as _;

use lino_objects_codec::format::parse_indented;

use crate::engine::{stable_id, KNOWLEDGE_SCHEMA_VERSION};
use crate::link_store::{DoubletLink, LinkRecord};
use crate::seed::parser::{parse_lino, LinoNode};

const DEFAULT_MAX_APPLICATIONS: usize = 64;

/// CRUD event that can trigger substitution rules.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum CrudEvent {
    Manual,
    Create,
    Read,
    Update,
    Delete,
}

impl CrudEvent {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Manual => "manual",
            Self::Create => "create",
            Self::Read => "read",
            Self::Update => "update",
            Self::Delete => "delete",
        }
    }

    fn parse(value: &str) -> Result<Self, SubstitutionRuleError> {
        match value.trim().to_ascii_lowercase().as_str() {
            "manual" | "apply" | "learned" => Ok(Self::Manual),
            "create" | "created" => Ok(Self::Create),
            "read" | "select" | "query" => Ok(Self::Read),
            "update" | "updated" => Ok(Self::Update),
            "delete" | "deleted" => Ok(Self::Delete),
            other => Err(SubstitutionRuleError::InvalidEvent(other.to_owned())),
        }
    }
}

impl fmt::Display for CrudEvent {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.as_str())
    }
}

/// A concrete link in the substitution graph.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct SubstitutionLink {
    pub from: String,
    pub to: String,
}

impl SubstitutionLink {
    #[must_use]
    pub fn new(from: impl Into<String>, to: impl Into<String>) -> Self {
        Self {
            from: from.into(),
            to: to.into(),
        }
    }

    #[must_use]
    pub fn pattern_text(&self) -> String {
        format!("{} -> {}", self.from, self.to)
    }
}

/// A parsed link pattern. Nodes can be literals (`kind:cat`), whole-node
/// variables (`$node`), or prefix variables (`assignee:$person`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LinkPattern {
    from: PatternNode,
    to: PatternNode,
}

impl LinkPattern {
    pub fn parse(text: &str) -> Result<Self, SubstitutionRuleError> {
        let Some((from, to)) = text.split_once("->") else {
            return Err(SubstitutionRuleError::InvalidPattern(format!(
                "expected `from -> to`, got `{text}`"
            )));
        };
        let from = PatternNode::parse(from.trim())?;
        let to = PatternNode::parse(to.trim())?;
        Ok(Self { from, to })
    }

    #[must_use]
    pub fn literal_pair(&self) -> Option<(&str, &str)> {
        match (&self.from, &self.to) {
            (PatternNode::Literal(from), PatternNode::Literal(to)) => Some((from, to)),
            _ => None,
        }
    }

    fn instantiate(&self, bindings: &BTreeMap<String, String>) -> Option<SubstitutionLink> {
        Some(SubstitutionLink {
            from: self.from.instantiate(bindings)?,
            to: self.to.instantiate(bindings)?,
        })
    }
}

impl fmt::Display for LinkPattern {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{} -> {}", self.from, self.to)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum PatternNode {
    Literal(String),
    Variable(String),
    PrefixVariable { prefix: String, variable: String },
}

impl PatternNode {
    fn parse(text: &str) -> Result<Self, SubstitutionRuleError> {
        if text.is_empty() {
            return Err(SubstitutionRuleError::InvalidPattern(String::from(
                "pattern node is empty",
            )));
        }
        if let Some(variable) = text.strip_prefix('$') {
            validate_variable(variable, text)?;
            return Ok(Self::Variable(variable.to_owned()));
        }
        if let Some((prefix, variable)) = text.split_once('$') {
            validate_variable(variable, text)?;
            return Ok(Self::PrefixVariable {
                prefix: prefix.to_owned(),
                variable: variable.to_owned(),
            });
        }
        Ok(Self::Literal(text.to_owned()))
    }

    fn instantiate(&self, bindings: &BTreeMap<String, String>) -> Option<String> {
        match self {
            Self::Literal(value) => Some(value.clone()),
            Self::Variable(variable) => bindings.get(variable).cloned(),
            Self::PrefixVariable { prefix, variable } => bindings
                .get(variable)
                .map(|value| format!("{prefix}{value}")),
        }
    }
}

impl fmt::Display for PatternNode {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Literal(value) => formatter.write_str(value),
            Self::Variable(variable) => write!(formatter, "${variable}"),
            Self::PrefixVariable { prefix, variable } => write!(formatter, "{prefix}${variable}"),
        }
    }
}

/// One `replace x y` operation inside a substitution rule.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SubstitutionAction {
    pub remove: LinkPattern,
    pub add: Vec<LinkPattern>,
}

/// A data-defined substitution rule.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SubstitutionRule {
    pub id: String,
    pub order: i64,
    pub events: Vec<CrudEvent>,
    pub conditions: Vec<LinkPattern>,
    pub actions: Vec<SubstitutionAction>,
}

impl SubstitutionRule {
    #[must_use]
    pub fn matches_event(&self, event: CrudEvent) -> bool {
        self.events.contains(&event)
    }
}

/// Ordered collection of substitution rules imported from `.lino` data.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SubstitutionRuleSet {
    pub id: String,
    pub rules: Vec<SubstitutionRule>,
}

impl SubstitutionRuleSet {
    pub fn from_links_notation(text: &str) -> Result<Self, SubstitutionRuleError> {
        let trimmed = text.trim();
        if trimmed.is_empty() {
            return Err(SubstitutionRuleError::EmptyDocument);
        }
        parse_indented(trimmed)
            .map_err(|error| SubstitutionRuleError::IllFormedLinksNotation(format!("{error:?}")))?;
        let tree = parse_lino(trimmed);
        let root = tree
            .children
            .first()
            .ok_or(SubstitutionRuleError::EmptyDocument)?;
        if root.name != "substitution_rules" {
            return Err(SubstitutionRuleError::NotSubstitutionRules(
                root.name.clone(),
            ));
        }
        let mut rules = root
            .children
            .iter()
            .filter(|child| child.name == "rule")
            .map(parse_rule)
            .collect::<Result<Vec<_>, _>>()?;
        rules.sort_by(|left, right| left.order.cmp(&right.order).then(left.id.cmp(&right.id)));
        let child_id = root.find_child_value("id");
        let id = if child_id.is_empty() {
            root.id.clone()
        } else {
            child_id.to_owned()
        };
        Ok(Self { id, rules })
    }

    #[must_use]
    pub fn links_notation(&self) -> String {
        let mut out = String::new();
        push_lino_node(&mut out, 0, "substitution_rules", None);
        push_lino_node(&mut out, 2, "id", Some(&self.id));
        for rule in &self.rules {
            push_lino_node(&mut out, 2, "rule", Some(&rule.id));
            push_lino_node(&mut out, 4, "order", Some(&rule.order.to_string()));
            for event in &rule.events {
                push_lino_node(&mut out, 4, "event", Some(event.as_str()));
            }
            for condition in &rule.conditions {
                push_lino_node(&mut out, 4, "when", Some(&condition.to_string()));
            }
            for action in &rule.actions {
                push_lino_node(&mut out, 4, "replace", Some(&action.remove.to_string()));
                for add in &action.add {
                    push_lino_node(&mut out, 6, "with", Some(&add.to_string()));
                }
            }
        }
        out.trim_end().to_owned()
    }
}

/// In-memory links network that applies substitution rules over concrete links.
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct SubstitutionGraph {
    links: BTreeSet<SubstitutionLink>,
}

impl SubstitutionGraph {
    #[must_use]
    pub const fn new() -> Self {
        Self {
            links: BTreeSet::new(),
        }
    }

    #[must_use]
    pub fn with_link(mut self, from: &str, to: &str) -> Self {
        self.insert_link(from, to);
        self
    }

    pub fn insert_link(&mut self, from: &str, to: &str) -> bool {
        self.links.insert(SubstitutionLink::new(from, to))
    }

    pub fn remove_link(&mut self, from: &str, to: &str) -> bool {
        self.links.remove(&SubstitutionLink::new(from, to))
    }

    #[must_use]
    pub fn contains_link(&self, from: &str, to: &str) -> bool {
        self.links.contains(&SubstitutionLink::new(from, to))
    }

    #[must_use]
    pub fn links(&self) -> Vec<SubstitutionLink> {
        self.links.iter().cloned().collect()
    }

    #[must_use]
    pub fn links_notation(&self) -> String {
        let mut out = String::new();
        push_lino_node(&mut out, 0, "substitution_graph", None);
        push_lino_node(
            &mut out,
            2,
            "id",
            Some(&stable_id("substitution_graph", &self.canonical_links())),
        );
        for link in &self.links {
            push_lino_node(&mut out, 2, "link", Some(&link.pattern_text()));
        }
        out.trim_end().to_owned()
    }

    #[must_use]
    pub fn apply_rules(
        &mut self,
        rules: &SubstitutionRuleSet,
        event: CrudEvent,
    ) -> SubstitutionTraceReport {
        self.apply_rules_with_limit(rules, event, DEFAULT_MAX_APPLICATIONS)
    }

    #[must_use]
    pub fn apply_rules_with_limit(
        &mut self,
        rules: &SubstitutionRuleSet,
        event: CrudEvent,
        max_applications: usize,
    ) -> SubstitutionTraceReport {
        let mut report = SubstitutionTraceReport::new(event);
        while report.traces.len() < max_applications {
            let Some(trace) = self.apply_first_rule(rules, event, report.traces.len()) else {
                return report;
            };
            report.traces.push(trace);
        }
        let mut probe = self.clone();
        report.terminated_by_guard = probe
            .apply_first_rule(rules, event, report.traces.len())
            .is_some();
        report
    }

    #[must_use]
    pub fn create_link(
        &mut self,
        from: &str,
        to: &str,
        rules: &SubstitutionRuleSet,
    ) -> SubstitutionTraceReport {
        self.insert_link(from, to);
        self.apply_rules(rules, CrudEvent::Create)
    }

    #[must_use]
    pub fn read_link(
        &mut self,
        from: &str,
        to: &str,
        rules: &SubstitutionRuleSet,
    ) -> (bool, SubstitutionTraceReport) {
        let exists = self.contains_link(from, to);
        let report = self.apply_rules(rules, CrudEvent::Read);
        (exists, report)
    }

    #[must_use]
    pub fn update_link(
        &mut self,
        old_from: &str,
        old_to: &str,
        new_from: &str,
        new_to: &str,
        rules: &SubstitutionRuleSet,
    ) -> SubstitutionTraceReport {
        self.remove_link(old_from, old_to);
        self.insert_link(new_from, new_to);
        self.apply_rules(rules, CrudEvent::Update)
    }

    #[must_use]
    pub fn delete_link(
        &mut self,
        from: &str,
        to: &str,
        rules: &SubstitutionRuleSet,
    ) -> SubstitutionTraceReport {
        self.remove_link(from, to);
        self.apply_rules(rules, CrudEvent::Delete)
    }

    fn apply_first_rule(
        &mut self,
        rules: &SubstitutionRuleSet,
        event: CrudEvent,
        sequence: usize,
    ) -> Option<SubstitutionTrace> {
        rules
            .rules
            .iter()
            .filter(|rule| rule.matches_event(event))
            .find_map(|rule| self.apply_rule(rule, event, sequence))
    }

    fn apply_rule(
        &mut self,
        rule: &SubstitutionRule,
        event: CrudEvent,
        sequence: usize,
    ) -> Option<SubstitutionTrace> {
        let mut required_patterns = rule.conditions.clone();
        for action in &rule.actions {
            required_patterns.push(action.remove.clone());
        }
        let bindings = self.find_bindings(&required_patterns)?;
        let before = self.links.clone();
        let mut removed = Vec::new();
        let mut added = Vec::new();
        for action in &rule.actions {
            let remove = action.remove.instantiate(&bindings)?;
            if self.links.remove(&remove) {
                removed.push(remove);
            }
            for add in &action.add {
                let link = add.instantiate(&bindings)?;
                if self.links.insert(link.clone()) {
                    added.push(link);
                }
            }
        }
        if self.links == before {
            return None;
        }
        Some(SubstitutionTrace::new(
            sequence, &rule.id, event, bindings, removed, added,
        ))
    }

    fn find_bindings(&self, patterns: &[LinkPattern]) -> Option<BTreeMap<String, String>> {
        self.find_bindings_from(patterns, BTreeMap::new())
    }

    fn find_bindings_from(
        &self,
        patterns: &[LinkPattern],
        bindings: BTreeMap<String, String>,
    ) -> Option<BTreeMap<String, String>> {
        let Some((pattern, remaining)) = patterns.split_first() else {
            return Some(bindings);
        };
        for link in &self.links {
            let mut candidate = bindings.clone();
            if pattern_matches_link(pattern, link, &mut candidate) {
                if let Some(found) = self.find_bindings_from(remaining, candidate) {
                    return Some(found);
                }
            }
        }
        None
    }

    fn canonical_links(&self) -> String {
        let mut out = String::new();
        for link in &self.links {
            let _ = write!(out, "{}->{};", link.from, link.to);
        }
        out
    }
}

/// One recorded substitution application.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SubstitutionTrace {
    pub id: String,
    pub sequence: usize,
    pub rule_id: String,
    pub event: CrudEvent,
    pub bindings: BTreeMap<String, String>,
    pub removed: Vec<SubstitutionLink>,
    pub added: Vec<SubstitutionLink>,
}

impl SubstitutionTrace {
    fn new(
        sequence: usize,
        rule_id: &str,
        event: CrudEvent,
        bindings: BTreeMap<String, String>,
        removed: Vec<SubstitutionLink>,
        added: Vec<SubstitutionLink>,
    ) -> Self {
        let id = stable_id(
            "substitution_trace",
            &canonical_trace(sequence, rule_id, event, &bindings, &removed, &added),
        );
        Self {
            id,
            sequence,
            rule_id: rule_id.to_owned(),
            event,
            bindings,
            removed,
            added,
        }
    }
}

/// Trace report for one substitution rule evaluation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SubstitutionTraceReport {
    pub event: CrudEvent,
    pub traces: Vec<SubstitutionTrace>,
    pub terminated_by_guard: bool,
}

impl SubstitutionTraceReport {
    #[must_use]
    pub const fn new(event: CrudEvent) -> Self {
        Self {
            event,
            traces: Vec::new(),
            terminated_by_guard: false,
        }
    }

    #[must_use]
    pub const fn applied_count(&self) -> usize {
        self.traces.len()
    }

    #[must_use]
    pub fn links_notation(&self) -> String {
        let mut out = String::new();
        push_lino_node(&mut out, 0, "substitution_trace_report", None);
        push_lino_node(
            &mut out,
            2,
            "id",
            Some(&stable_id("substitution_trace_report", &self.canonical())),
        );
        push_lino_node(&mut out, 2, "event", Some(self.event.as_str()));
        push_lino_node(
            &mut out,
            2,
            "terminated_by_guard",
            Some(if self.terminated_by_guard {
                "true"
            } else {
                "false"
            }),
        );
        for trace in &self.traces {
            push_lino_node(&mut out, 2, "trace", Some(&trace.id));
            push_lino_node(&mut out, 4, "sequence", Some(&trace.sequence.to_string()));
            push_lino_node(&mut out, 4, "rule_id", Some(&trace.rule_id));
            push_lino_node(&mut out, 4, "event", Some(trace.event.as_str()));
            for (name, value) in &trace.bindings {
                push_lino_node(&mut out, 4, "binding", Some(&format!("{name}={value}")));
            }
            for link in &trace.removed {
                push_lino_node(&mut out, 4, "removed", Some(&link.pattern_text()));
            }
            for link in &trace.added {
                push_lino_node(&mut out, 4, "added", Some(&link.pattern_text()));
            }
        }
        out.trim_end().to_owned()
    }

    #[must_use]
    pub fn trace_link_records(&self) -> Vec<LinkRecord> {
        self.traces.iter().map(trace_link_record).collect()
    }

    fn canonical(&self) -> String {
        let mut out = format!(
            "event={};terminated={};",
            self.event.as_str(),
            self.terminated_by_guard
        );
        for trace in &self.traces {
            out.push_str(&trace.id);
            out.push(';');
        }
        out
    }
}

/// Errors returned while importing substitution rule data.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SubstitutionRuleError {
    EmptyDocument,
    IllFormedLinksNotation(String),
    NotSubstitutionRules(String),
    MissingReplacement(String),
    InvalidPattern(String),
    InvalidEvent(String),
}

impl fmt::Display for SubstitutionRuleError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EmptyDocument => formatter.write_str("substitution rule document is empty"),
            Self::IllFormedLinksNotation(message) => {
                write!(
                    formatter,
                    "ill-formed substitution rule Links Notation: {message}"
                )
            }
            Self::NotSubstitutionRules(root) => {
                write!(formatter, "{root} is not a substitution_rules document")
            }
            Self::MissingReplacement(rule_id) => {
                write!(formatter, "rule {rule_id} has replace without with")
            }
            Self::InvalidPattern(message) => write!(formatter, "invalid link pattern: {message}"),
            Self::InvalidEvent(event) => write!(formatter, "invalid CRUD event: {event}"),
        }
    }
}

impl Error for SubstitutionRuleError {}

fn parse_rule(node: &LinoNode) -> Result<SubstitutionRule, SubstitutionRuleError> {
    let mut order = 0;
    let mut events = Vec::new();
    let mut conditions = Vec::new();
    let mut actions = Vec::new();

    for child in &node.children {
        match child.name.as_str() {
            "order" => {
                order = child.id.trim().parse::<i64>().unwrap_or(0);
            }
            "event" => {
                events.push(CrudEvent::parse(&child.id)?);
            }
            "when" => {
                conditions.push(LinkPattern::parse(&child.id)?);
            }
            "replace" => {
                let add = child
                    .children
                    .iter()
                    .filter(|grandchild| grandchild.name == "with")
                    .map(|grandchild| LinkPattern::parse(&grandchild.id))
                    .collect::<Result<Vec<_>, _>>()?;
                if add.is_empty() {
                    return Err(SubstitutionRuleError::MissingReplacement(node.id.clone()));
                }
                actions.push(SubstitutionAction {
                    remove: LinkPattern::parse(&child.id)?,
                    add,
                });
            }
            _ => {}
        }
    }
    if events.is_empty() {
        events.push(CrudEvent::Manual);
    }
    Ok(SubstitutionRule {
        id: node.id.clone(),
        order,
        events,
        conditions,
        actions,
    })
}

fn validate_variable(variable: &str, raw: &str) -> Result<(), SubstitutionRuleError> {
    if variable.is_empty()
        || !variable
            .chars()
            .all(|ch| ch.is_ascii_alphanumeric() || ch == '_')
    {
        return Err(SubstitutionRuleError::InvalidPattern(format!(
            "invalid variable in `{raw}`"
        )));
    }
    Ok(())
}

fn pattern_matches_link(
    pattern: &LinkPattern,
    link: &SubstitutionLink,
    bindings: &mut BTreeMap<String, String>,
) -> bool {
    node_matches(&pattern.from, &link.from, bindings)
        && node_matches(&pattern.to, &link.to, bindings)
}

fn node_matches(
    pattern: &PatternNode,
    value: &str,
    bindings: &mut BTreeMap<String, String>,
) -> bool {
    match pattern {
        PatternNode::Literal(literal) => literal == value,
        PatternNode::Variable(variable) => bind_variable(bindings, variable, value),
        PatternNode::PrefixVariable { prefix, variable } => {
            let Some(captured) = value.strip_prefix(prefix) else {
                return false;
            };
            bind_variable(bindings, variable, captured)
        }
    }
}

fn bind_variable(bindings: &mut BTreeMap<String, String>, variable: &str, value: &str) -> bool {
    if let Some(existing) = bindings.get(variable) {
        return existing == value;
    }
    bindings.insert(variable.to_owned(), value.to_owned());
    true
}

fn canonical_trace(
    sequence: usize,
    rule_id: &str,
    event: CrudEvent,
    bindings: &BTreeMap<String, String>,
    removed: &[SubstitutionLink],
    added: &[SubstitutionLink],
) -> String {
    let mut out = format!("{sequence}:{}:{rule_id};", event.as_str());
    for (key, value) in bindings {
        let _ = write!(out, "binding:{key}={value};");
    }
    for link in removed {
        let _ = write!(out, "removed:{};", link.pattern_text());
    }
    for link in added {
        let _ = write!(out, "added:{};", link.pattern_text());
    }
    out
}

fn trace_link_record(trace: &SubstitutionTrace) -> LinkRecord {
    let mut links = Vec::new();
    push_doublet(&mut links, &trace.id, "Type");
    push_doublet(&mut links, "Type", "SubstitutionTraceLink");
    push_doublet(&mut links, "SubstitutionTraceLink", "SubType");
    push_doublet(&mut links, "SubType", trace.event.as_str());
    push_doublet(&mut links, trace.event.as_str(), "Value");
    push_doublet(&mut links, &trace.id, &trace.rule_id);
    push_doublet(
        &mut links,
        &trace.id,
        &format!("schema_version:{KNOWLEDGE_SCHEMA_VERSION}"),
    );
    push_field(&mut links, &trace.id, "rule_id", &trace.rule_id);
    push_field(&mut links, &trace.id, "event", trace.event.as_str());
    push_field(
        &mut links,
        &trace.id,
        "sequence",
        &trace.sequence.to_string(),
    );
    for (name, value) in &trace.bindings {
        push_field(&mut links, &trace.id, "binding", &format!("{name}={value}"));
    }
    for link in &trace.removed {
        push_field(&mut links, &trace.id, "removed", &link.pattern_text());
    }
    for link in &trace.added {
        push_field(&mut links, &trace.id, "added", &link.pattern_text());
    }
    LinkRecord {
        stable_id: trace.id.clone(),
        schema_version: String::from(KNOWLEDGE_SCHEMA_VERSION),
        record_type: String::from("SubstitutionTraceLink"),
        source_id: trace.rule_id.clone(),
        links,
    }
}

fn push_lino_node(out: &mut String, indent: usize, name: &str, value: Option<&str>) {
    for _ in 0..indent {
        out.push(' ');
    }
    out.push_str(name);
    if let Some(value) = value {
        out.push_str(" \"");
        out.push_str(&escape_lino_value(value));
        out.push('"');
    }
    out.push('\n');
}

fn escape_lino_value(value: &str) -> String {
    value
        .replace('\\', "\\\\")
        .replace('"', "\\\"")
        .replace('\n', "\\n")
}

fn push_field(links: &mut Vec<DoubletLink>, record_id: &str, key: &str, value: &str) {
    let field = format!("field:{key}");
    let field_value = format!("value:{value}");
    push_doublet(links, record_id, &field);
    push_doublet(links, &field, &field_value);
}

fn push_doublet(links: &mut Vec<DoubletLink>, from: &str, to: &str) {
    links.push(DoubletLink {
        index: stable_id("doublet", &format!("{from}->{to}")),
        from: from.to_owned(),
        to: to.to_owned(),
    });
}
