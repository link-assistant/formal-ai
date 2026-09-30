//! Code-review handler (issue #1177, E142 code-task family).
//!
//! Recognizes "review this code" / "any problems with this code" requests
//! (English and Russian; cue phrases live in `data/seed/code-task-cues.lino`),
//! detects the language structurally, and checks the code against the rule
//! table in `data/seed/code-review-rules.lino`: every rule's detect/avoid
//! strings, scope, severity, advice and upstream source URL are seed data —
//! the Rust here only scopes lines (any line, function-definition line, loop
//! body) and applies the table.
//!
//! No code is executed and no linter is run; the answer (a template from
//! `data/seed/multilingual-responses.lino`) states both facts and names the
//! rules' upstream documents.

use crate::engine::SymbolicAnswer;
use crate::event_log::EventLog;
use crate::seed::parser::parse_lino;
use super::finalize_simple;

const CUES_PATH: &str = "data/seed/code-task-cues.lino";
const RULES_PATH: &str = "data/seed/code-review-rules.lino";
const INTENT: &str = "code_review";
/// Markers that make the code read as JavaScript rather than Python.
const JS_MARKERS: &[&str] = &["=>", "const ", ".then(", "let ", "=== "];
/// Function-definition keywords for def-line scoping.
const DEF_KEYWORDS: [&str; 4] = ["def ", "fn ", "function ", "async function "];

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
        if record.find_child_value("role") != role {
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

/// Fill a localized response template's `{placeholder}` slots.
fn template(intent: &str, values: &[(&str, &str)]) -> String {
    let mut out = crate::seed::localized_response(intent, "en").unwrap_or_default();
    for (key, value) in values {
        out = out.replace(&format!("{{{key}}}"), value);
    }
    out
}

/// One review rule from the seed table.
struct Rule {
    id: String,
    language: String,
    scope: String,
    severity: String,
    detects: Vec<String>,
    avoids: Vec<String>,
    title: String,
    advice: String,
    source: String,
}

fn rules() -> Vec<Rule> {
    let Some(text) = seed_text(RULES_PATH) else {
        return Vec::new();
    };
    let tree = parse_lino(text);
    let mut out = Vec::new();
    for record in tree.children.iter().filter(|child| child.name == "rule") {
        // A rule's fields sit under its `id` child; the `rule` head only
        // names the record kind, and the id child carries the rule id.
        let Some(body) = record.children.first().filter(|child| child.name == "id") else {
            continue;
        };
        let collect = |name: &str| -> Vec<String> {
            body.children
                .iter()
                .filter(|child| child.name == name)
                .map(|child| child.id.clone())
                .filter(|value| !value.is_empty())
                .collect()
        };
        out.push(Rule {
            id: record.find_child_value("id").to_string(),
            language: body.find_child_value("language").to_string(),
            scope: body.find_child_value("scope").to_string(),
            severity: body.find_child_value("severity").to_string(),
            detects: collect("detect"),
            avoids: collect("avoid"),
            title: body.find_child_value("title").to_string(),
            advice: body.find_child_value("advice").to_string(),
            source: body.find_child_value("source").to_string(),
        });
    }
    out
}

/// Extract the code under discussion: the first fenced block, else a
/// backtick span that looks like code, else the whole prompt when it carries
/// code markers.
fn code_block(prompt: &str) -> Option<String> {
    if let Some(start) = prompt.find("```") {
        let rest = &prompt[start + 3..];
        let after_open = match rest.find('\n') {
            Some(nl) => &rest[nl + 1..],
            None => rest,
        };
        if let Some(end) = after_open.find("```") {
            let code = &after_open[..end];
            if !code.trim().is_empty() {
                return Some(code.to_owned());
            }
        }
    }
    if let Some(start) = prompt.find('`') {
        if let Some(end) = prompt[start + 1..].find('`') {
            let code = &prompt[start + 1..start + 1 + end];
            let code_markers = ["(", "def ", "=>", "return "];
            if code_markers.iter().any(|marker| code.contains(marker)) {
                return Some(code.to_owned());
            }
        }
    }
    let markers = ["def ", "function ", "fn ", "=>", "return "];
    if markers.iter().any(|marker| prompt.contains(marker)) {
        return Some(prompt.to_owned());
    }
    None
}

/// The leading-indent width of a code line (spaces; tabs count as one).
fn indent_of(line: &str) -> usize {
    line.chars().take_while(|c| *c == ' ' || *c == '\t').count()
}

/// True when the line opens a loop whose indented body follows.
fn is_loop_header(line: &str) -> bool {
    let trimmed = line.trim_start();
    trimmed.strip_prefix("for ").is_some() || trimmed.strip_prefix("while ").is_some()
}

/// True when the line defines a function (def-line scope).
fn is_def_line(line: &str) -> bool {
    let trimmed = line.trim_start();
    DEF_KEYWORDS
        .iter()
        .any(|keyword| trimmed.strip_prefix(keyword).is_some())
}

/// The line indices inside loop bodies: after each loop header, every line
/// indented deeper than the header until indentation returns.
fn loop_body_lines(code: &str) -> Vec<usize> {
    let lines: Vec<&str> = code.lines().collect();
    let mut out = Vec::new();
    let mut index = 0usize;
    while index < lines.len() {
        if is_loop_header(lines[index]) {
            let header_indent = indent_of(lines[index]);
            let mut body = index + 1;
            while body < lines.len() {
                let line = lines[body];
                if !line.trim().is_empty() && indent_of(line) <= header_indent {
                    break;
                }
                out.push(body);
                body += 1;
            }
            index = body;
        } else {
            index += 1;
        }
    }
    out
}

/// One reported finding: the rule and the line it fired on.
struct Finding<'a> {
    rule: &'a Rule,
    line_number: usize,
    line: String,
}

/// Apply every rule whose language matches and whose scope contains the
/// line, to the code.
fn review<'a>(code: &str, language: &str, table: &'a [Rule]) -> Vec<Finding<'a>> {
    let loop_bodies = loop_body_lines(code);
    let mut out = Vec::new();
    for rule in table {
        if rule.language != language {
            continue;
        }
        for (index, line) in code.lines().enumerate() {
            let trimmed = line.trim();
            if trimmed.is_empty() {
                continue;
            }
            let in_scope = match rule.scope.as_str() {
                "def_line" => is_def_line(line),
                "loop_body" => loop_bodies.contains(&index),
                _ => true,
            };
            if !in_scope {
                continue;
            }
            let detected = rule
                .detects
                .iter()
                .any(|detect| trimmed.contains(detect.as_str()));
            let suppressed = rule
                .avoids
                .iter()
                .any(|avoid| trimmed.contains(avoid.as_str()));
            if detected && !suppressed {
                out.push(Finding {
                    rule,
                    line_number: index + 1,
                    line: trimmed.to_owned(),
                });
            }
        }
    }
    out
}

/// True when any cue phrase for the intent occurs in the normalized prompt
/// or the raw prompt lowercased.
fn triggered(prompt: &str, normalized: &str) -> bool {
    let lower = prompt.to_lowercase();
    cue_phrases(INTENT, "")
        .iter()
        .any(|phrase| normalized.contains(phrase.as_str()) || lower.contains(phrase.as_str()))
}

/// Try to recognize a code-review request and check the code against the
/// seed rule table. Returns `None` when the prompt is not a review request
/// or carries no code.
pub fn handle_code_review(
    prompt: &str,
    normalized: &str,
    log: &mut EventLog,
) -> Option<SymbolicAnswer> {
    if !triggered(prompt, normalized) {
        return None;
    }
    let code = code_block(prompt)?;
    let language = if JS_MARKERS.iter().any(|marker| code.contains(marker)) {
        "javascript"
    } else {
        "python"
    };
    log.append(
        "code_review:request",
        format!("lang={}", language),
    );
    let table = rules();
    let findings = review(&code, language, &table);
    log.append(
        "code_review:findings",
        format!("n={}", findings.len()),
    );

    let (body, confidence) = if findings.is_empty() {
        (template("code_review_no_match", &[]), 0.4)
    } else {
        let mut rendered = String::new();
        for finding in &findings {
            log.append("code_review:finding", finding.rule.id.clone());
            rendered.push_str(&template(
                "code_review_finding",
                &[
                    ("severity", &finding.rule.severity),
                    ("title", &finding.rule.title),
                    ("line_no", &finding.line_number.to_string()),
                    ("line", &finding.line),
                    ("advice", &finding.rule.advice),
                    ("source", &finding.rule.source),
                ],
            ));
        }
        (template("code_review_findings", &[("findings", &rendered)]), 0.7)
    };

    Some(finalize_simple(
        prompt,
        log,
        INTENT,
        "response:code_review",
        &body,
        confidence,
    ))
}
