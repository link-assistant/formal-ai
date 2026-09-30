//! Regular-expression synthesis handler (issue #1177, E142 code-task family).
//!
//! Recognizes "write a regular expression that matches …" / "regex for …"
//! requests (English and Russian; cue phrases live in
//! `data/seed/code-task-cues.lino`), extracts character-class constraints
//! (digit/letter counts, optional groups, separators, at-least repetition —
//! all word maps in the same seed file), and composes an anchored pattern
//! from them.
//!
//! The `regex` crate is a dev-only dependency of this project, so the
//! composed pattern is verified **structurally** (balanced groups,
//! well-formed repetitions and classes) and never executed — the answer (a
//! template from `data/seed/multilingual-responses.lino`) states both facts.
//! When the constraints do not compose, the handler refuses by name rather
//! than guessing.

use super::finalize_simple;
use crate::engine::SymbolicAnswer;
use crate::event_log::EventLog;
use crate::seed::parser::{LinoNode, parse_lino};

const CUES_PATH: &str = "data/seed/code-task-cues.lino";
const INTENT: &str = "regex_synthesis";

/// Look up embedded seed content by its registered path.
fn seed_text(path: &str) -> Option<&'static str> {
    crate::seed::seed_files()
        .into_iter()
        .find(|(registered, _)| *registered == path)
        .map(|(_, text)| text)
}

/// Trigger phrases for one intent (optionally one named role) from
/// `data/seed/code-task-cues.lino`.
fn cue_phrases(intent: &str, role: &str) -> Vec<String> {
    let Some(text) = seed_text(CUES_PATH) else {
        return Vec::new();
    };
    let tree = parse_lino(text);
    let mut out = Vec::new();
    for record in tree.children.iter().filter(|child| child.name == "cues") {
        if record.find_child_value("intent") != intent {
            continue;
        }
        if record
            .children
            .iter()
            .find(|child| child.name == "intent")
            .map_or("", |child| child.find_child_value("role"))
            != role
        {
            continue;
        }
        for phrase in record
            .children
            .iter()
            .flat_map(|child| child.children.iter())
            .filter(|phrase| phrase.name == "phrase")
        {
            if !phrase.id.is_empty() {
                out.push(phrase.id.clone());
            }
        }
    }
    out
}

/// The `entry` records of one word map from `data/seed/code-task-cues.lino`.
fn word_entries(map: &str) -> Vec<LinoNode> {
    let Some(text) = seed_text(CUES_PATH) else {
        return Vec::new();
    };
    let tree = parse_lino(text);
    let mut out = Vec::new();
    for record in tree.children.iter().filter(|child| child.name == "map") {
        if record.find_child_value("name") != map {
            continue;
        }
        out.extend(
            record
                .children
                .iter()
                .flat_map(|child| child.children.iter())
                .filter(|entry| entry.name == "entry")
                .cloned(),
        );
    }
    out
}

/// Fill a localized response template's `{placeholder}` slots.
fn template(intent: &str, values: &[(&str, &str)]) -> String {
    let mut out = crate::seed::localized_response(intent, "en").unwrap_or_default();
    for (key, value) in values {
        out = out.replace(&format!("{{{key}}}"), value);
    }
    out
}

/// A class-count mention in the prompt, in order of appearance.
struct ClassMention {
    position: usize,
    class: String,
    class_label: String,
    count: u32,
    at_least: bool,
    optional: bool,
}

/// A separator mention, in order of appearance.
struct SeparatorMention {
    position: usize,
    literal: String,
}

/// The numeric value of a word: a digit string, or a word in the `number`
/// word map ("five" → 5).
fn number_value(word: &str, numbers: &[LinoNode]) -> Option<u32> {
    if let Ok(value) = word.parse::<u32>() {
        return Some(value);
    }
    numbers
        .iter()
        .find(|entry| entry.find_child_value("word") == word)
        .and_then(|entry| entry.find_child_value("value").parse::<u32>().ok())
}

/// Scan the lowercased prompt (word positions kept) for class counts and
/// separators. An optional marker ("optionally") applies to the next class
/// mention seen.
fn scan_mentions(lower: &str) -> (Vec<ClassMention>, Vec<SeparatorMention>) {
    let numbers = word_entries("number");
    let separators_by_word = word_entries("separator");
    let classes_by_word = word_entries("class");
    let optional_markers: Vec<String> = word_entries("optional_marker")
        .iter()
        .map(|entry| entry.find_child_value("word").to_owned())
        .collect();
    let words: Vec<(usize, &str)> = lower
        .split_whitespace()
        .scan(0usize, |position, word| {
            let start = *position;
            *position += word.len() + 1;
            Some((start, word.trim_matches(|c: char| !c.is_alphanumeric())))
        })
        .collect();
    let mut classes = Vec::new();
    let mut separators = Vec::new();
    let mut pending_optional = false;
    for (index, (position, word)) in words.iter().enumerate() {
        if optional_markers.iter().any(|marker| marker == word) {
            pending_optional = true;
            continue;
        }
        if let Some(entry) = separators_by_word
            .iter()
            .find(|entry| entry.find_child_value("word") == *word)
        {
            separators.push(SeparatorMention {
                position: *position,
                literal: entry.find_child_value("value").to_owned(),
            });
            continue;
        }
        let Some(count) = number_value(word, &numbers) else {
            continue;
        };
        // Skip "or more" filler between the count and the class word
        // ("3 or more digits"); the filler itself also means at-least.
        let mut at_least = false;
        let mut cursor = index + 1;
        while matches!(
            words.get(cursor).map(|(_, word)| *word),
            Some("or") | Some("more")
        ) {
            at_least = true;
            cursor += 1;
        }
        let Some((_, next)) = words.get(cursor) else {
            continue;
        };
        let Some(entry) = classes_by_word
            .iter()
            .find(|entry| entry.find_child_value("word") == *next)
        else {
            continue;
        };
        // "at least 3 digits": the word right before the count is "least".
        if index > 0 && words[index - 1].1 == "least" {
            at_least = true;
        }
        let optional = pending_optional;
        pending_optional = false;
        classes.push(ClassMention {
            position: *position,
            class: entry.find_child_value("value").to_owned(),
            class_label: format!("{count} {}", entry.find_child_value("label")),
            count,
            at_least,
            optional,
        });
    }
    (classes, separators)
}

fn render_class(mention: &ClassMention) -> String {
    if mention.at_least {
        format!("{}{{{},}}", mention.class, mention.count)
    } else {
        format!("{}{{{}}}", mention.class, mention.count)
    }
}

/// The separator (if any) that sits between two class mentions in the text.
fn separator_between(
    separators: &[SeparatorMention],
    from: Option<usize>,
    to: usize,
) -> Option<String> {
    let from = from.unwrap_or(usize::MAX);
    separators
        .iter()
        .find(|separator| separator.position > from && separator.position < to)
        .map(|separator| separator.literal.clone())
}

/// Compose the pattern body (without anchors) from the mentions.
fn compose(classes: &[ClassMention], separators: &[SeparatorMention]) -> Option<String> {
    let first = classes.first()?;
    let mut pattern = render_class(first);
    for mention in classes.iter().skip(1) {
        let previous_position = classes
            .iter()
            .filter(|other| other.position < mention.position)
            .next_back()
            .map(|other| other.position);
        let separator = separator_between(separators, previous_position, mention.position);
        let body = format!("{}{}", separator.unwrap_or_default(), render_class(mention));
        if mention.optional {
            pattern.push_str(&format!("({body})?"));
        } else {
            pattern.push_str(&body);
        }
    }
    Some(pattern)
}

/// Structural verification of the composed pattern: balanced groups, every
/// `{n}`/`{n,}` repetition well-formed, character classes closed, every
/// backslash followed by something. Returns the list of problem codes
/// (empty = verified); each code's prose lives in a seed template.
fn verify_structurally(pattern: &str) -> Vec<&'static str> {
    let mut problems = Vec::new();
    let mut group_depth = 0usize;
    let mut class_depth = 0usize;
    let chars: Vec<char> = pattern.chars().collect();
    let mut i = 0usize;
    while i < chars.len() {
        match chars[i] {
            '\\' => {
                if i + 1 >= chars.len() {
                    problems.push("dangling_escape");
                }
                i += 2;
                continue;
            }
            '(' => group_depth += 1,
            ')' => group_depth = group_depth.saturating_sub(1),
            '[' => class_depth += 1,
            ']' => class_depth = class_depth.saturating_sub(1),
            '{' => {
                let mut j = i + 1;
                while j < chars.len() && chars[j].is_ascii_digit() {
                    j += 1;
                }
                if j == i + 1 {
                    problems.push("bad_repetition");
                }
                if j < chars.len() && chars[j] == ',' {
                    j += 1;
                }
                if j >= chars.len() || chars[j] != '}' {
                    problems.push("unclosed_repetition");
                }
                i = j;
            }
            _ => {}
        }
        i += 1;
    }
    if group_depth != 0 {
        problems.push("unbalanced_parens");
    }
    if class_depth != 0 {
        problems.push("unbalanced_class");
    }
    problems
}

/// The prose for one structural problem code (a seed template per code).
fn problem_text(code: &str) -> String {
    let name: String = ["regex_problem_", code].concat();
    crate::seed::localized_response(&name, "en").unwrap_or_default()
}

/// True when any cue phrase for the intent occurs in the normalized prompt
/// or the raw prompt lowercased.
fn triggered(prompt: &str, normalized: &str) -> bool {
    let lower = prompt.to_lowercase();
    cue_phrases(INTENT, "")
        .iter()
        .any(|phrase| normalized.contains(phrase.as_str()) || lower.contains(phrase.as_str()))
}

/// Try to recognize a regex-synthesis request and compose a pattern from the
/// stated constraints. Refuses by name when they do not compose.
pub fn handle_regex_synthesis(
    prompt: &str,
    normalized: &str,
    log: &mut EventLog,
) -> Option<SymbolicAnswer> {
    if !triggered(prompt, normalized) {
        return None;
    }
    let lower = prompt.to_lowercase();
    let (classes, separators) = scan_mentions(&lower);
    log.append(
        "regex_synthesis:request",
        format!("class={}", classes.len()),
    );
    let anchor_exceptions = word_entries("anchor_exception");
    let anchored = !anchor_exceptions
        .iter()
        .any(|entry| normalized.contains(entry.find_child_value("word")));

    let (body, confidence) = match compose(&classes, &separators) {
        Some(body) if verify_structurally(&body).is_empty() => {
            let pattern = if anchored { format!("^{body}$") } else { body };
            log.append("regex_synthesis:pattern", pattern.clone());
            let mut mapping = String::new();
            for (index, mention) in classes.iter().enumerate() {
                let role = if index == 0 {
                    template("regex_role_main", &[])
                } else if mention.optional {
                    template("regex_role_optional", &[])
                } else {
                    template("regex_role_required", &[])
                };
                let separator = if index == 0 {
                    String::new()
                } else {
                    separator_between(
                        &separators,
                        Some(classes[index - 1].position),
                        mention.position,
                    )
                    .map(|literal| template("regex_note_separator", &[("literal", &literal)]))
                    .unwrap_or_default()
                };
                mapping.push_str(&template(
                    "regex_mapping_line",
                    &[
                        ("label", &mention.class_label),
                        ("render", &render_class(mention)),
                        ("role", &role),
                        ("separator", &separator),
                    ],
                ));
            }
            let anchor_note = if anchored {
                template("regex_anchor_note_anchored", &[])
            } else {
                template("regex_anchor_note_unanchored", &[])
            };
            (
                template(
                    "regex_synthesis_pattern",
                    &[
                        ("pattern", &pattern),
                        ("mapping", &mapping),
                        ("anchor_note", &anchor_note),
                    ],
                ),
                0.7,
            )
        }
        Some(broken) => {
            let problems = verify_structurally(&broken)
                .iter()
                .map(|code| problem_text(code))
                .collect::<Vec<_>>()
                .join("; ");
            log.append("regex_synthesis:refusal", format!("problems: {}", problems));
            (
                template(
                    "regex_synthesis_broken",
                    &[("broken", &broken), ("problems", &problems)],
                ),
                0.4,
            )
        }
        None => {
            log.append(
                "regex_synthesis:refusal",
                "no composable constraints found".to_owned(),
            );
            (template("regex_synthesis_refusal", &[]), 0.4)
        }
    };

    Some(finalize_simple(
        prompt,
        log,
        INTENT,
        "response:regex_synthesis",
        &body,
        confidence,
    ))
}
