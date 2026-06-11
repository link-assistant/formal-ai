//! General text manipulation through composed substitution rules.

use std::collections::BTreeSet;
use std::fmt::Write as _;

use crate::engine::{stable_id, SymbolicAnswer};
use crate::event_log::EventLog;
use crate::solver::{ConversationRole, ConversationTurn};
use crate::solver_handlers::finalize_simple;
use crate::substitution::{
    CrudEvent, SubstitutionGraph, SubstitutionRuleSet, SubstitutionTraceReport,
};

pub fn try_text_manipulation(
    prompt: &str,
    normalized: &str,
    log: &mut EventLog,
) -> Option<SymbolicAnswer> {
    try_text_manipulation_with_history(prompt, normalized, log, &[])
}

pub fn try_text_manipulation_with_history(
    prompt: &str,
    normalized: &str,
    log: &mut EventLog,
    history: &[ConversationTurn],
) -> Option<SymbolicAnswer> {
    let request = TextRequest::parse_with_history(prompt, normalized, history)?;
    let chain = SubstitutionTextChain::build(&request.input, &request.operations)?;

    log.append(
        "text_input",
        format!(
            "bytes={} chars={}",
            request.input.len(),
            request.input.chars().count()
        ),
    );
    for step in &chain.steps {
        log.append("text_operation", step.operation.slug().to_owned());
        log.append("text_rule", step.rule_id.clone());
    }
    log.append(
        "text_rule_chain",
        chain
            .steps
            .iter()
            .map(|step| step.rule_id.as_str())
            .collect::<Vec<_>>()
            .join(">"),
    );
    log.append("text_substitution_rules", chain.rules.links_notation());
    log.append("text_substitution_trace", chain.report.links_notation());
    log.append("text_substitution_graph", chain.graph.links_notation());
    log.append("text_result", chain.result.clone());

    Some(finalize_simple(
        prompt,
        log,
        "text_manipulation",
        "response:text_manipulation",
        &chain.result,
        1.0,
    ))
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct TextRequest {
    input: String,
    operations: Vec<TextOperation>,
}

impl TextRequest {
    fn parse_with_history(
        prompt: &str,
        normalized: &str,
        history: &[ConversationTurn],
    ) -> Option<Self> {
        let vocabulary = crate::seed::operation_vocabulary();
        let quoted = quoted_segments(prompt);
        let fallback_input = || last_assistant_text_artifact(history);

        if vocabulary.matches("replace", normalized) && quoted.len() >= 2 {
            let from = quoted[0].clone();
            let to = quoted[1].clone();
            let input = quoted
                .get(2)
                .cloned()
                .or_else(|| text_after_colon(prompt))
                .or_else(fallback_input)?;
            return Self::build(input, vec![TextOperation::Replace { from, to }]);
        }

        if vocabulary.matches("count_occurrences", normalized) && !quoted.is_empty() {
            let needle = quoted[0].clone();
            let input = quoted
                .get(1)
                .cloned()
                .or_else(|| text_after_colon(prompt))
                .or_else(fallback_input)?;
            return Self::build(input, vec![TextOperation::CountOccurrences { needle }]);
        }

        let input = quoted
            .first()
            .cloned()
            .or_else(|| text_after_colon(prompt))
            .or_else(fallback_input)?;
        let mut operations = Vec::new();
        append_simple_operations(&vocabulary, normalized, &mut operations);
        Self::build(input, operations)
    }

    fn build(input: String, operations: Vec<TextOperation>) -> Option<Self> {
        if operations.is_empty() || input.is_empty() {
            return None;
        }
        Some(Self { input, operations })
    }
}

fn append_simple_operations(
    vocabulary: &crate::seed::OperationVocabulary,
    normalized: &str,
    operations: &mut Vec<TextOperation>,
) {
    if vocabulary.matches("lowercase", normalized) {
        operations.push(TextOperation::Lowercase);
    } else if vocabulary.matches("uppercase", normalized) {
        operations.push(TextOperation::Uppercase);
    }

    if vocabulary.matches("reverse_words", normalized) {
        operations.push(TextOperation::ReverseWords);
    }
    if vocabulary.matches("extract_email", normalized) {
        operations.push(TextOperation::ExtractEmails);
    }
    if vocabulary.matches("deduplicate_lines", normalized) {
        operations.push(TextOperation::DeduplicateLines);
    }
    if vocabulary.matches("sort_lines", normalized) {
        operations.push(TextOperation::SortLines);
    }
    if vocabulary.matches("count_unique_words", normalized) {
        operations.push(TextOperation::CountUniqueWords);
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum TextOperation {
    Uppercase,
    Lowercase,
    Replace { from: String, to: String },
    ReverseWords,
    ExtractEmails,
    CountOccurrences { needle: String },
    CountUniqueWords,
    DeduplicateLines,
    SortLines,
}

impl TextOperation {
    const fn slug(&self) -> &'static str {
        match self {
            Self::Uppercase => "uppercase",
            Self::Lowercase => "lowercase",
            Self::Replace { .. } => "replace_text",
            Self::ReverseWords => "reverse_words",
            Self::ExtractEmails => "extract_email",
            Self::CountOccurrences { .. } => "count_occurrences",
            Self::CountUniqueWords => "count_unique_words",
            Self::DeduplicateLines => "deduplicate_lines",
            Self::SortLines => "sort_lines",
        }
    }

    fn apply(&self, input: &str) -> String {
        match self {
            Self::Uppercase => input.chars().flat_map(char::to_uppercase).collect(),
            Self::Lowercase => input.chars().flat_map(char::to_lowercase).collect(),
            Self::Replace { from, to } => replace_text(input, from, to),
            Self::ReverseWords => input.split_whitespace().rev().collect::<Vec<_>>().join(" "),
            Self::ExtractEmails => extract_email_addresses(input).join("\n"),
            Self::CountOccurrences { needle } => {
                if needle.is_empty() {
                    String::from("0")
                } else {
                    input.matches(needle).count().to_string()
                }
            }
            Self::CountUniqueWords => count_unique_words(input).to_string(),
            Self::DeduplicateLines => deduplicate_lines(input).join("\n"),
            Self::SortLines => {
                let mut lines = input.lines().map(str::to_owned).collect::<Vec<_>>();
                lines.sort();
                lines.join("\n")
            }
        }
    }
}

fn last_assistant_text_artifact(history: &[ConversationTurn]) -> Option<String> {
    history
        .iter()
        .rev()
        .find(|turn| turn.role == ConversationRole::Assistant && !turn.content.trim().is_empty())
        .map(|turn| turn.content.clone())
}

fn replace_text(input: &str, from: &str, to: &str) -> String {
    let direct = input.replace(from, to);
    if from.is_empty() || direct != input {
        return direct;
    }
    replace_word_sequence(input, from, to).unwrap_or(direct)
}

fn replace_word_sequence(input: &str, from: &str, to: &str) -> Option<String> {
    let needle = normalized_word_spans(from)
        .into_iter()
        .map(|span| span.word)
        .collect::<Vec<_>>();
    if needle.is_empty() {
        return None;
    }

    let haystack = normalized_word_spans(input);
    if haystack.len() < needle.len() {
        return None;
    }

    let mut out = String::new();
    let mut cursor = 0usize;
    let mut index = 0usize;
    let mut replaced = false;
    while index + needle.len() <= haystack.len() {
        let matches = needle
            .iter()
            .enumerate()
            .all(|(offset, word)| haystack[index + offset].word == *word);
        if matches {
            let start = haystack[index].start;
            let end = haystack[index + needle.len() - 1].end;
            out.push_str(&input[cursor..start]);
            out.push_str(to);
            cursor = end;
            index += needle.len();
            replaced = true;
        } else {
            index += 1;
        }
    }
    if !replaced {
        return None;
    }
    out.push_str(&input[cursor..]);
    Some(out)
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct WordSpan {
    word: String,
    start: usize,
    end: usize,
}

fn normalized_word_spans(text: &str) -> Vec<WordSpan> {
    let mut spans = Vec::new();
    let mut current_start = None;
    let mut current = String::new();
    let mut current_end = 0usize;

    for (index, character) in text.char_indices() {
        if character.is_alphanumeric() {
            if current_start.is_none() {
                current_start = Some(index);
            }
            current.extend(character.to_lowercase());
            current_end = index + character.len_utf8();
        } else if let Some(start) = current_start.take() {
            spans.push(WordSpan {
                word: std::mem::take(&mut current),
                start,
                end: current_end,
            });
        }
    }

    if let Some(start) = current_start {
        spans.push(WordSpan {
            word: current,
            start,
            end: current_end,
        });
    }
    spans
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct TextStep {
    operation: TextOperation,
    rule_id: String,
    before: String,
    after: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct SubstitutionTextChain {
    result: String,
    steps: Vec<TextStep>,
    rules: SubstitutionRuleSet,
    report: SubstitutionTraceReport,
    graph: SubstitutionGraph,
}

impl SubstitutionTextChain {
    fn build(input: &str, operations: &[TextOperation]) -> Option<Self> {
        if operations.is_empty() {
            return None;
        }

        let mut current = input.to_owned();
        let mut steps = Vec::new();
        let mut used_rule_ids = BTreeSet::new();
        for operation in operations {
            let after = operation.apply(&current);
            let rule_id = unique_rule_id(operation.slug(), &mut used_rule_ids);
            steps.push(TextStep {
                operation: operation.clone(),
                rule_id,
                before: current,
                after: after.clone(),
            });
            current = after;
        }

        let rules_text = build_rules_links_notation(input, &steps);
        let rules = SubstitutionRuleSet::from_links_notation(&rules_text).ok()?;
        let mut graph = SubstitutionGraph::new().with_link("stage:0", &text_node(input));
        let report = graph.apply_rules_with_limit(&rules, CrudEvent::Manual, steps.len());
        if report.applied_count() != steps.len() {
            return None;
        }

        Some(Self {
            result: current,
            steps,
            rules,
            report,
            graph,
        })
    }
}

fn unique_rule_id(slug: &str, used: &mut BTreeSet<String>) -> String {
    let base = format!("rule_{slug}");
    if used.insert(base.clone()) {
        return base;
    }
    for index in 2.. {
        let candidate = format!("{base}_{index}");
        if used.insert(candidate.clone()) {
            return candidate;
        }
    }
    unreachable!("unbounded suffix search must return")
}

fn build_rules_links_notation(input: &str, steps: &[TextStep]) -> String {
    let mut out = String::new();
    push_lino_node(&mut out, 0, "substitution_rules", None);
    push_lino_node(
        &mut out,
        2,
        "id",
        Some(&stable_id(
            "text_substitution_rules",
            &format!("{input}:{}", steps.len()),
        )),
    );
    for (index, step) in steps.iter().enumerate() {
        push_lino_node(&mut out, 2, "rule", Some(&step.rule_id));
        push_lino_node(&mut out, 4, "order", Some(&(index + 1).to_string()));
        push_lino_node(&mut out, 4, "event", Some("manual"));
        push_lino_node(
            &mut out,
            4,
            "replace",
            Some(&format!("stage:{index} -> {}", text_node(&step.before))),
        );
        push_lino_node(
            &mut out,
            6,
            "with",
            Some(&format!(
                "stage:{} -> {}",
                index + 1,
                text_node(&step.after)
            )),
        );
        push_lino_node(
            &mut out,
            6,
            "with",
            Some(&format!(
                "stage:{} -> operation:{}",
                index + 1,
                step.operation.slug()
            )),
        );
    }
    out.trim_end().to_owned()
}

fn text_node(text: &str) -> String {
    format!("text:hex:{}", hex_encode(text.as_bytes()))
}

fn hex_encode(bytes: &[u8]) -> String {
    let mut out = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        let _ = write!(out, "{byte:02x}");
    }
    out
}

fn quoted_segments(text: &str) -> Vec<String> {
    let mut segments = Vec::new();
    let mut cursor = 0usize;
    while cursor < text.len() {
        let Some((relative_start, open, close)) =
            text[cursor..]
                .char_indices()
                .find_map(|(index, character)| {
                    quote_close_for(character).map(|close| (index, character, close))
                })
        else {
            break;
        };
        let content_start = cursor + relative_start + open.len_utf8();
        let Some(relative_end) = text[content_start..].find(close) else {
            break;
        };
        let content_end = content_start + relative_end;
        segments.push(text[content_start..content_end].to_owned());
        cursor = content_end + close.len_utf8();
    }
    segments
}

const fn quote_close_for(open: char) -> Option<char> {
    match open {
        '\'' => Some('\''),
        '"' => Some('"'),
        '`' => Some('`'),
        '«' => Some('»'),
        _ => None,
    }
}

fn text_after_colon(prompt: &str) -> Option<String> {
    let (_, tail) = prompt.rsplit_once(':')?;
    let text = tail
        .trim()
        .trim_matches(|character| matches!(character, '"' | '\'' | '`'))
        .trim();
    (!text.is_empty()).then(|| text.to_owned())
}

fn extract_email_addresses(input: &str) -> Vec<String> {
    input
        .split_whitespace()
        .map(clean_email_candidate)
        .filter(|candidate| looks_like_email(candidate))
        .map(ToOwned::to_owned)
        .collect()
}

fn clean_email_candidate(candidate: &str) -> &str {
    candidate
        .trim_matches(|character: char| {
            !(character.is_ascii_alphanumeric() || matches!(character, '@' | '.' | '_' | '-' | '+'))
        })
        .trim_matches('.')
}

fn looks_like_email(candidate: &str) -> bool {
    if candidate
        .chars()
        .filter(|character| *character == '@')
        .count()
        != 1
    {
        return false;
    }
    let Some((local, domain)) = candidate.split_once('@') else {
        return false;
    };
    !local.is_empty()
        && domain.contains('.')
        && domain
            .split('.')
            .all(|segment| !segment.is_empty() && segment.chars().all(is_email_domain_char))
}

const fn is_email_domain_char(character: char) -> bool {
    character.is_ascii_alphanumeric() || character == '-'
}

fn count_unique_words(input: &str) -> usize {
    input
        .split_whitespace()
        .map(clean_word)
        .filter(|word| !word.is_empty())
        .collect::<BTreeSet<_>>()
        .len()
}

fn clean_word(word: &str) -> String {
    word.trim_matches(|character: char| !character.is_alphanumeric())
        .to_owned()
}

fn deduplicate_lines(input: &str) -> Vec<String> {
    let mut seen = BTreeSet::new();
    let mut lines = Vec::new();
    for line in input.lines() {
        if seen.insert(line.to_owned()) {
            lines.push(line.to_owned());
        }
    }
    lines
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
