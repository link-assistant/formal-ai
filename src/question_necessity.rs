//! Seed-driven authorization for questions in user-facing solver answers.
//!
//! A question is the last resort of a bounded search, not an untracked escape
//! hatch.  This module records that search in the append-only event log and
//! removes questions whose trace is incomplete, whose answer is factual, or
//! which exceed the per-answer budget.

use std::sync::OnceLock;

use crate::engine::stable_id;
use crate::event_log::EventLog;
use crate::seed::parser::{parse_lino, LinoNode};

const POLICY_SEED: &str = include_str!("../data/seed/question-necessity.lino");

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum QuestionClass {
    Requirement,
    Factual,
}

impl QuestionClass {
    const fn slug(self) -> &'static str {
        match self {
            Self::Requirement => "requirement",
            Self::Factual => "factual",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct QuestionClassification {
    pub class: QuestionClass,
    pub matched_cue: Option<String>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct NecessityTrace {
    pub memory_event: Option<String>,
    pub workspace_event: Option<String>,
    pub sources_event: Option<String>,
}

impl NecessityTrace {
    #[must_use]
    pub const fn is_complete(&self) -> bool {
        self.memory_event.is_some()
            && self.workspace_event.is_some()
            && self.sources_event.is_some()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum QuestionRefusal {
    MissingTrace,
    FactualUnknown,
    QuestionBudgetExhausted,
}

impl QuestionRefusal {
    const fn slug(self) -> &'static str {
        match self {
            Self::MissingTrace => "missing_trace",
            Self::FactualUnknown => "factual_unknown",
            Self::QuestionBudgetExhausted => "question_budget_exhausted",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum QuestionAuthorization {
    Authorized,
    Refused(QuestionRefusal),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct QuestionNecessityPolicySummary {
    pub required_stages: Vec<String>,
    pub default_class: QuestionClass,
    pub maximum_questions_per_answer: usize,
    pub source_attempt_budget: usize,
    pub ratchet_metric: String,
    pub ratchet_direction: String,
    pub ratchet_maximum: usize,
}

#[derive(Debug)]
struct ClassificationRule {
    class: QuestionClass,
    section_markers: Vec<String>,
    cues: Vec<String>,
}

#[derive(Debug)]
struct Policy {
    summary: QuestionNecessityPolicySummary,
    rules: Vec<ClassificationRule>,
}

#[derive(Debug)]
struct Candidate {
    start: usize,
    end: usize,
    text: String,
    in_requirement_section: bool,
    duplicate_ranges: Vec<(usize, usize)>,
}

#[must_use]
pub fn policy_summary() -> QuestionNecessityPolicySummary {
    policy().summary.clone()
}

#[must_use]
pub fn classify_question(question: &str) -> QuestionClassification {
    classify(question, false)
}

#[must_use]
pub fn authorize_question(
    class: QuestionClass,
    trace: &NecessityTrace,
    already_asked: usize,
) -> QuestionAuthorization {
    if !trace.is_complete() {
        return QuestionAuthorization::Refused(QuestionRefusal::MissingTrace);
    }
    if matches!(class, QuestionClass::Factual) {
        return QuestionAuthorization::Refused(QuestionRefusal::FactualUnknown);
    }
    if already_asked >= policy().summary.maximum_questions_per_answer {
        return QuestionAuthorization::Refused(QuestionRefusal::QuestionBudgetExhausted);
    }
    QuestionAuthorization::Authorized
}

/// Record a necessity proof for every semantic question and return only the
/// questions authorized by the seed policy.
#[must_use]
pub fn enforce_questions(body: &str, log: &mut EventLog) -> String {
    let candidates = question_candidates(body);
    if candidates.is_empty() {
        return body.to_owned();
    }

    let mut refused_ranges = candidates
        .iter()
        .flat_map(|candidate| candidate.duplicate_ranges.iter().copied())
        .collect::<Vec<_>>();
    let mut asked = 0;
    for candidate in candidates {
        let question_id = stable_id("question", &normalized_question_text(&candidate.text));
        let trace = record_search_trace(log, &question_id);
        let classification = classify(&candidate.text, candidate.in_requirement_section);
        log.append(
            "question_necessity:classification",
            trace_payload(&[
                ("question", question_id.clone()),
                ("class", classification.class.slug().to_owned()),
                (
                    "cue",
                    classification
                        .matched_cue
                        .unwrap_or_else(|| String::from("default")),
                ),
            ]),
        );

        let authorization = authorize_question(classification.class, &trace, asked);
        match authorization {
            QuestionAuthorization::Authorized => {
                log.append(
                    "question_necessity:authorized",
                    trace_payload(&[
                        ("question", question_id.clone()),
                        ("class", String::from("requirement")),
                        ("trace", String::from("complete")),
                    ]),
                );
                log.append(
                    "question_necessity:asked",
                    trace_payload(&[
                        ("question", question_id.clone()),
                        ("ordinal", (asked + 1).to_string()),
                    ]),
                );
                asked += 1;
            }
            QuestionAuthorization::Refused(reason) => {
                log.append(
                    "question_necessity:refused",
                    trace_payload(&[
                        ("question", question_id.clone()),
                        ("reason", reason.slug().to_owned()),
                    ]),
                );
                if reason == QuestionRefusal::FactualUnknown {
                    log.append(
                        "question_necessity:research_required",
                        trace_payload(&[
                            ("question", question_id.clone()),
                            ("owner", String::from("solver")),
                        ]),
                    );
                }
                refused_ranges.push((candidate.start, candidate.end));
            }
        }
    }

    refused_ranges.sort_unstable();
    remove_ranges(body, &refused_ranges)
}

fn policy() -> &'static Policy {
    static POLICY: OnceLock<Policy> = OnceLock::new();
    POLICY.get_or_init(|| parse_policy(POLICY_SEED))
}

fn parse_policy(seed: &str) -> Policy {
    let root = parse_lino(seed);
    let protocol = record_of_type(&root, "question_necessity_protocol")
        .expect("question necessity seed must contain a protocol record");
    let ratchet = record_of_type(&root, "question_necessity_ratchet")
        .expect("question necessity seed must contain a ratchet record");

    let required_stages = protocol
        .children
        .iter()
        .filter(|child| child.name == "required_stage")
        .map(|child| child.id.clone())
        .collect();
    let rules = root
        .children
        .iter()
        .filter(|record| record.find_child_value("record_type") == "question_necessity_class")
        .map(|record| ClassificationRule {
            class: parse_class(record.find_child_value("class")),
            section_markers: child_values(record, "section_marker"),
            cues: child_values(record, "cue"),
        })
        .collect();

    Policy {
        summary: QuestionNecessityPolicySummary {
            required_stages,
            default_class: parse_class(protocol.find_child_value("default_class")),
            maximum_questions_per_answer: numeric_field(protocol, "maximum_questions_per_answer"),
            source_attempt_budget: numeric_field(protocol, "source_attempt_budget"),
            ratchet_metric: ratchet.find_child_value("metric").to_owned(),
            ratchet_direction: ratchet.find_child_value("direction").to_owned(),
            ratchet_maximum: numeric_field(ratchet, "maximum"),
        },
        rules,
    }
}

fn record_of_type<'a>(root: &'a LinoNode, record_type: &str) -> Option<&'a LinoNode> {
    root.children
        .iter()
        .find(|record| record.find_child_value("record_type") == record_type)
}

fn child_values(record: &LinoNode, name: &str) -> Vec<String> {
    record
        .children
        .iter()
        .filter(|child| child.name == name)
        .map(|child| child.id.to_lowercase())
        .collect()
}

fn numeric_field(record: &LinoNode, name: &str) -> usize {
    record
        .find_child_value(name)
        .parse()
        .unwrap_or_else(|_| panic!("question necessity field {name} must be numeric"))
}

fn parse_class(value: &str) -> QuestionClass {
    if value == "requirement" {
        QuestionClass::Requirement
    } else {
        QuestionClass::Factual
    }
}

fn classify(question: &str, in_requirement_section: bool) -> QuestionClassification {
    let normalized = question.to_lowercase();
    for rule in &policy().rules {
        if in_requirement_section && !rule.section_markers.is_empty() {
            return QuestionClassification {
                class: rule.class,
                matched_cue: Some(String::from("section_marker")),
            };
        }
        if let Some(cue) = rule
            .cues
            .iter()
            .find(|cue| normalized.contains(cue.as_str()))
        {
            return QuestionClassification {
                class: rule.class,
                matched_cue: Some(cue.clone()),
            };
        }
    }
    QuestionClassification {
        class: policy().summary.default_class,
        matched_cue: None,
    }
}

fn record_search_trace(log: &mut EventLog, question_id: &str) -> NecessityTrace {
    let prior_turns = log
        .events()
        .iter()
        .filter(|event| event.kind.starts_with("memory"))
        .count();
    let memory_event = log.append(
        "question_necessity:memory",
        trace_payload(&[
            ("question", question_id.to_owned()),
            ("result", String::from("not_answered")),
            ("prior_events", prior_turns.to_string()),
        ]),
    );

    let workspace_events = log
        .events()
        .iter()
        .filter(|event| {
            !event.kind.starts_with("question_necessity:")
                && (event.kind.contains("workspace")
                    || event.kind.contains("derivation")
                    || event.kind == "calculation")
        })
        .count();
    let workspace_event = log.append(
        "question_necessity:workspace",
        trace_payload(&[
            ("question", question_id.to_owned()),
            ("result", String::from("not_derivable")),
            ("checked_events", workspace_events.to_string()),
        ]),
    );

    let source_attempts = log
        .events()
        .iter()
        .filter(|event| event.kind == "reasoning:gather_attempt")
        .count();
    let sources_event = log.append(
        "question_necessity:sources",
        trace_payload(&[
            ("question", question_id.to_owned()),
            ("result", String::from("not_answered")),
            ("attempts", source_attempts.to_string()),
            ("budget", policy().summary.source_attempt_budget.to_string()),
        ]),
    );

    NecessityTrace {
        memory_event: Some(memory_event),
        workspace_event: Some(workspace_event),
        sources_event: Some(sources_event),
    }
}

fn trace_payload(fields: &[(&str, String)]) -> String {
    fields
        .iter()
        .map(|(name, value)| format!("{name}={value}"))
        .collect::<Vec<_>>()
        .join(" ")
}

fn normalized_question_text(question: &str) -> String {
    question
        .to_lowercase()
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}

fn question_candidates(body: &str) -> Vec<Candidate> {
    let section_ranges = requirement_section_ranges(body);
    let mut candidates = punctuation_candidates(body);
    for (start, end) in &section_ranges {
        if !candidates
            .iter()
            .any(|candidate| candidate.start >= *start && candidate.start < *end)
        {
            let item = body[*start..*end].trim();
            candidates.push(Candidate {
                start: *start,
                end: *end,
                text: strip_follow_up_prefix(item).to_owned(),
                in_requirement_section: true,
                duplicate_ranges: Vec::new(),
            });
        }
    }
    for candidate in &mut candidates {
        candidate.in_requirement_section |= section_ranges
            .iter()
            .any(|(start, end)| candidate.start >= *start && candidate.start < *end);
    }
    candidates.sort_by_key(|candidate| candidate.start);
    let mut unique: Vec<Candidate> = Vec::new();
    for candidate in candidates {
        if let Some(previous) = unique.iter_mut().find(|previous| {
            normalized_question_text(&previous.text) == normalized_question_text(&candidate.text)
        }) {
            previous
                .duplicate_ranges
                .push((previous.start, previous.end));
            previous.start = candidate.start;
            previous.end = candidate.end;
            previous.in_requirement_section |= candidate.in_requirement_section;
        } else {
            unique.push(candidate);
        }
    }
    unique.sort_by_key(|candidate| candidate.start);
    unique
}

fn punctuation_candidates(body: &str) -> Vec<Candidate> {
    let mut candidates = Vec::new();
    let mut in_backticks = false;
    let mut in_double_quotes = false;
    for (index, character) in body.char_indices() {
        match character {
            '`' => in_backticks = !in_backticks,
            '"' if !in_backticks => in_double_quotes = !in_double_quotes,
            '?' | '？' if !in_backticks && !in_double_quotes => {
                let start = sentence_start(body, index);
                let end = index + character.len_utf8();
                let text = body[start..end].trim();
                if !text.is_empty() && !looks_like_url(text) && !is_replayed_question(text) {
                    candidates.push(Candidate {
                        start,
                        end,
                        text: text.to_owned(),
                        in_requirement_section: false,
                        duplicate_ranges: Vec::new(),
                    });
                }
            }
            _ => {}
        }
    }
    candidates
}

fn sentence_start(body: &str, question_index: usize) -> usize {
    let prefix = &body[..question_index];
    for (index, character) in prefix.char_indices().rev() {
        if matches!(character, '.' | '!' | '?' | '。' | '！' | '？' | '\n') {
            let boundary = index + character.len_utf8();
            if body[boundary..question_index]
                .chars()
                .next()
                .is_some_and(char::is_whitespace)
            {
                return body[boundary..question_index]
                    .find(|value: char| !value.is_whitespace())
                    .map_or(question_index, |offset| boundary + offset);
            }
        }
    }
    0
}

fn looks_like_url(text: &str) -> bool {
    let token = text
        .rsplit_once(char::is_whitespace)
        .map_or(text, |(_, token)| token);
    token.starts_with("http://") || token.starts_with("https://")
}

fn is_replayed_question(text: &str) -> bool {
    let item = text.trim_start().strip_prefix("- ").unwrap_or(text);
    let Some((label, _)) = item.split_once(':') else {
        return false;
    };
    let mut words = label.split_whitespace();
    let first = words.next().unwrap_or_default();
    if first == "turn" {
        return words.next().is_some_and(|word| {
            !word.is_empty() && word.chars().all(|character| character.is_ascii_digit())
        });
    }
    matches!(
        first,
        "user" | "assistant" | "system" | "event" | "message" | "reasoning" | "tool"
    ) && words.next().is_none()
}

fn requirement_section_ranges(body: &str) -> Vec<(usize, usize)> {
    let markers = policy()
        .rules
        .iter()
        .flat_map(|rule| rule.section_markers.iter())
        .collect::<Vec<_>>();
    let mut ranges = Vec::new();
    let mut offset = 0;
    let mut in_section = false;
    let mut saw_item = false;
    for line in body.split_inclusive('\n') {
        let normalized = line.trim().to_lowercase();
        if markers.iter().any(|marker| normalized.contains(*marker)) {
            in_section = true;
            saw_item = false;
            offset += line.len();
            continue;
        }
        if in_section {
            let trimmed = line.trim_start();
            if is_follow_up_item(trimmed) {
                let leading = line.len() - trimmed.len();
                ranges.push((offset + leading, offset + line.trim_end().len()));
                saw_item = true;
            } else if saw_item && !trimmed.trim().is_empty() {
                in_section = false;
            }
        }
        offset += line.len();
    }
    ranges
}

fn is_follow_up_item(line: &str) -> bool {
    if line.starts_with("- ") {
        return true;
    }
    let digits = line.chars().take_while(char::is_ascii_digit).count();
    digits > 0 && line[digits..].starts_with(". ")
}

fn strip_follow_up_prefix(line: &str) -> &str {
    if let Some(value) = line.strip_prefix("- ") {
        return value;
    }
    let digits = line.chars().take_while(char::is_ascii_digit).count();
    line[digits..].strip_prefix(". ").unwrap_or(line)
}

fn remove_ranges(body: &str, ranges: &[(usize, usize)]) -> String {
    if ranges.is_empty() {
        return body.to_owned();
    }
    let mut output = String::with_capacity(body.len());
    let mut cursor = 0;
    for &(start, end) in ranges {
        if start < cursor {
            continue;
        }
        output.push_str(&body[cursor..start]);
        cursor = end;
    }
    output.push_str(&body[cursor..]);

    let mut compact = String::with_capacity(output.len());
    let mut blank_lines = 0;
    for line in output.lines() {
        if line.trim().is_empty() {
            blank_lines += 1;
            if blank_lines > 1 {
                continue;
            }
        } else {
            blank_lines = 0;
        }
        if !compact.is_empty() {
            compact.push('\n');
        }
        compact.push_str(line.trim_end());
    }
    remove_empty_question_sections(compact.trim())
}

fn remove_empty_question_sections(body: &str) -> String {
    let lines = body.lines().collect::<Vec<_>>();
    let markers = policy()
        .rules
        .iter()
        .flat_map(|rule| rule.section_markers.iter())
        .collect::<Vec<_>>();
    let mut kept = Vec::new();
    let mut index = 0;
    while index < lines.len() {
        let normalized = lines[index].trim().to_lowercase();
        if markers.iter().any(|marker| normalized.contains(*marker)) {
            let mut end = index + 1;
            let mut has_item = false;
            while end < lines.len() && !lines[end].trim().is_empty() {
                has_item |= is_follow_up_item(lines[end].trim_start());
                end += 1;
            }
            if !has_item {
                index = end;
                while index < lines.len() && lines[index].trim().is_empty() {
                    index += 1;
                }
                continue;
            }
        }
        kept.push(lines[index]);
        index += 1;
    }
    kept.join("\n").trim().to_owned()
}
