//! Code-explanation handler (issue #1177, E142 code-task family).
//!
//! Recognizes "explain this code" / "what does this function do" requests
//! (English and Russian; cue phrases live in `data/seed/code-task-cues.lino`),
//! extracts the code under discussion, and explains it line by line: each
//! line is matched against the construct table in
//! `data/seed/meanings-code-structure-explanations.lino` (sentence and
//! grounding URL are seed data), the matched line's placeholders are filled
//! from the actual code, and the overall summary states the property the
//! function NAME promises (the same `function_intent` table the debugging
//! handler uses).
//!
//! The scan is structural: no code is executed, and the answer (a template
//! from `data/seed/multilingual-responses.lino`) says so.

use super::finalize_simple;
use crate::engine::SymbolicAnswer;
use crate::event_log::EventLog;
use crate::seed::parser::parse_lino;

const CUES_PATH: &str = "data/seed/code-task-cues.lino";
const STRUCTURES_PATH: &str = "data/seed/meanings-code-structure-explanations.lino";
const INTENT: &str = "code_explanation";
/// Built-in calls and the construct each one names.
const BUILTINS: &[(&str, &str)] = &[
    ("sum(", "builtin_sum"),
    ("len(", "builtin_len"),
    ("max(", "builtin_max"),
    ("min(", "builtin_min"),
];

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

/// Fill a localized response template's `{placeholder}` slots.
fn template(intent: &str, values: &[(&str, &str)]) -> String {
    let mut out = crate::seed::localized_response(intent, "en").unwrap_or_default();
    for (key, value) in values {
        out = out.replace(&format!("{{{key}}}"), value);
    }
    out
}

/// One `structure` record: a construct, its meaning sentence, its grounding,
/// and the generic matching rows (`head`, `only`, `infix`) it may carry.
struct Structure {
    construct: String,
    meaning: String,
    grounding: String,
    heads: Vec<String>,
    only: String,
    infixes: Vec<String>,
}

/// The ids of every `name` child of a construct body.
fn child_ids(body: &crate::seed::parser::LinoNode, name: &str) -> Vec<String> {
    body.children
        .iter()
        .filter(|child| child.name == name)
        .map(|child| child.id.clone())
        .collect()
}

fn structures() -> Vec<Structure> {
    let Some(text) = seed_text(STRUCTURES_PATH) else {
        return Vec::new();
    };
    let tree = parse_lino(text);
    let mut out = Vec::new();
    for body in tree
        .children
        .iter()
        .filter(|child| child.name == "structure")
        .flat_map(|record| record.children.iter())
        .filter(|child| child.name == "construct")
    {
        // Each construct owns its own fields under the shared structure root.
        out.push(Structure {
            construct: body.id.clone(),
            meaning: body.find_child_value("explanation").to_string(),
            grounding: body.find_child_value("grounding").to_string(),
            heads: child_ids(body, "head"),
            only: body.find_child_value("only").to_string(),
            infixes: child_ids(body, "infix"),
        });
    }
    out
}

/// One `function_intent` record: the property a function name promises.
struct FunctionIntent {
    names: Vec<String>,
    property: String,
    correct_form: String,
    grounding: String,
}

fn function_intents() -> Vec<FunctionIntent> {
    let Some(text) = seed_text(STRUCTURES_PATH) else {
        return Vec::new();
    };
    let tree = parse_lino(text);
    let mut out = Vec::new();
    for record in tree
        .children
        .iter()
        .filter(|child| child.name == "function_intent")
    {
        let name = record.find_child_value("name").to_string();
        if name.is_empty() {
            continue;
        }
        let mut names = vec![name];
        // An intent's fields sit under its `name` child.
        let Some(body) = record.children.first().filter(|child| child.name == "name") else {
            continue;
        };
        for alias in body.children.iter().filter(|child| child.name == "alias") {
            if !alias.id.is_empty() {
                names.push(alias.id.clone());
            }
        }
        out.push(FunctionIntent {
            names,
            property: body.find_child_value("property").to_string(),
            correct_form: body.find_child_value("correct_form").to_string(),
            grounding: body.find_child_value("grounding").to_string(),
        });
    }
    out
}

/// The name of the first function defined in the code.
fn function_name(code: &str) -> Option<String> {
    for line in code.lines() {
        let trimmed = line.trim_start();
        for keyword in ["def ", "fn ", "function ", "async function "] {
            if let Some(rest) = trimmed.strip_prefix(keyword) {
                let name: String = rest
                    .chars()
                    .take_while(|c| c.is_alphanumeric() || *c == '_')
                    .collect();
                if !name.is_empty() {
                    return Some(name);
                }
            }
        }
    }
    None
}

/// The run of identifier characters starting at `text`.
fn identifier_at(text: &str) -> &str {
    let end = text
        .find(|c: char| !c.is_alphanumeric() && c != '_')
        .unwrap_or(text.len());
    &text[..end]
}

/// The balanced-parenthesis span starting at the '(' at `open_at`.
fn inside_parens(line: &str, open_at: usize) -> &str {
    let mut depth = 0usize;
    for (offset, ch) in line[open_at..].char_indices() {
        match ch {
            '(' => depth += 1,
            ')' => {
                depth -= 1;
                if depth == 0 {
                    return &line[open_at + 1..open_at + offset];
                }
            }
            _ => {}
        }
    }
    ""
}

/// One explained code line: the construct, the meaning's placeholder values,
/// and whether the line is a loop.
type LineExplanation = (String, Vec<(&'static str, String)>, bool);

/// The remainder of a keyword-headed line: trailing `:` / `{` / `;` and one pair of
/// wrapping parentheses removed.
fn head_rest(rest: &str) -> String {
    let mut out = rest.trim();
    while let Some(stripped) = out
        .strip_suffix(':')
        .or_else(|| out.strip_suffix('{'))
        .or_else(|| out.strip_suffix(';'))
    {
        out = stripped.trim_end();
    }
    if out.starts_with('(') && out.ends_with(')') && inside_parens(out, 0).len() + 2 == out.len() {
        out = out[1..out.len() - 1].trim();
    }
    out.to_owned()
}

/// Match a line against the seed rows that carry `head` or `only`. The
/// longest head wins; a head ending in a word character must not run into
/// another identifier character.
fn match_seed_head(trimmed: &str, table: &[Structure]) -> Option<LineExplanation> {
    let mut best: Option<LineExplanation> = None;
    let mut best_length = 0usize;
    for structure in table {
        if !structure.only.is_empty() && trimmed.chars().all(|ch| structure.only.contains(ch)) {
            return Some((structure.construct.clone(), Vec::new(), false));
        }
        for head in &structure.heads {
            if head.is_empty() || !trimmed.starts_with(head.as_str()) || head.len() <= best_length {
                continue;
            }
            let last = head.chars().next_back().unwrap_or(' ');
            let next = trimmed[head.len()..].chars().next();
            if last.is_alphanumeric() && next.is_some_and(|ch| ch.is_alphanumeric() || ch == '_') {
                continue;
            }
            best = Some((
                structure.construct.clone(),
                vec![("rest", head_rest(&trimmed[head.len()..]))],
                false,
            ));
            best_length = head.len();
        }
    }
    best
}

/// Match a line against the seed rows that carry `infix`.
fn match_seed_infix(trimmed: &str, table: &[Structure]) -> Option<LineExplanation> {
    for structure in table {
        for infix in &structure.infixes {
            if infix.is_empty() {
                continue;
            }
            let Some(at) = trimmed.find(infix.as_str()) else {
                continue;
            };
            let right = trimmed[at + infix.len()..].trim();
            return Some((
                structure.construct.clone(),
                vec![
                    ("left", trimmed[..at].trim().to_owned()),
                    ("op", infix.trim().to_owned()),
                    ("right", right.trim_end_matches(';').to_owned()),
                ],
                false,
            ));
        }
    }
    None
}

/// A bare call `name(args)` at the start of a line.
fn match_call(trimmed: &str) -> Option<LineExplanation> {
    let name = identifier_at(trimmed);
    if name.is_empty() || !trimmed[name.len()..].starts_with('(') {
        return None;
    }
    Some((
        "function_call".to_owned(),
        vec![
            ("name", name.to_owned()),
            ("args", inside_parens(trimmed, name.len()).to_owned()),
        ],
        false,
    ))
}

/// Match one code line against the construct table, filling the meaning's
/// placeholders from the line. Returns (construct, meaning, `loop_seen`).
fn explain_line(line: &str, table: &[Structure]) -> Option<LineExplanation> {
    let trimmed = line.trim();
    if trimmed.is_empty() {
        return None;
    }
    if let Some(headed) = match_seed_head(trimmed, table) {
        return Some(headed);
    }
    // JavaScript constructs first (their markers are the most specific).
    if let Some(at) = trimmed.find(".then(") {
        let value = trimmed[..at].trim();
        let handler = inside_parens(trimmed, at + 5);
        return Some((
            "promise_then".to_owned(),
            vec![("value", value.to_owned()), ("handler", handler.to_owned())],
            false,
        ));
    }
    if let Some(at) = trimmed.find("=>") {
        let args = trimmed[..at].trim().trim_matches('(').trim_matches(')');
        let expr = trimmed[at + 2..].trim();
        return Some((
            "arrow_function".to_owned(),
            vec![("args", args.to_owned()), ("expr", expr.to_owned())],
            false,
        ));
    }
    if let Some(rest) = trimmed.strip_prefix("const ")
        && let Some((name, value)) = rest.split_once(" = ")
    {
        return Some((
            "const_declaration".to_owned(),
            vec![
                ("name", name.trim().to_owned()),
                ("value", value.trim().to_owned()),
            ],
            false,
        ));
    }
    for keyword in ["def ", "fn ", "function ", "async function "] {
        if let Some(rest) = trimmed.strip_prefix(keyword) {
            let name = identifier_at(rest);
            if name.is_empty() {
                continue;
            }
            let args = rest
                .find('(')
                .map(|open| inside_parens(rest, open).to_owned())
                .unwrap_or_default();
            return Some((
                "function_definition".to_owned(),
                vec![("name", name.to_owned()), ("args", args)],
                false,
            ));
        }
    }
    // Python compound constructs.
    if trimmed.starts_with('[') && trimmed.contains(" for ") {
        let for_at = trimmed.find(" for ")?;
        let expr = trimmed[1..for_at].trim();
        let after_for = &trimmed[for_at + 5..];
        let in_at = after_for.find(" in ")?;
        let item = after_for[..in_at].trim();
        let items = after_for[in_at + 4..].trim().trim_end_matches(']');
        return Some((
            "comprehension".to_owned(),
            vec![
                ("expr", expr.to_owned()),
                ("item", item.to_owned()),
                ("items", items.to_owned()),
            ],
            true,
        ));
    }
    if let Some(rest) = trimmed.strip_prefix("for ")
        && let Some(in_at) = rest.find(" in ")
    {
        let item = rest[..in_at].trim();
        let items = rest[in_at + 4..].trim().trim_end_matches(':');
        return Some((
            "for_loop".to_owned(),
            vec![("item", item.to_owned()), ("items", items.to_owned())],
            true,
        ));
    }
    if let Some(rest) = trimmed.strip_prefix("if ") {
        let condition = rest.trim().trim_end_matches(':');
        return Some((
            "if_statement".to_owned(),
            vec![("condition", condition.to_owned())],
            false,
        ));
    }
    for (marker, construct) in BUILTINS {
        if let Some(open) = trimmed.find(marker) {
            let items = inside_parens(trimmed, open + marker.len() - 1);
            return Some((
                (*construct).to_owned(),
                vec![("items", items.to_owned())],
                false,
            ));
        }
    }
    if let Some(rest) = trimmed.strip_prefix("return ") {
        return Some((
            "return_statement".to_owned(),
            vec![("value", rest.trim().to_owned())],
            false,
        ));
    }
    if let Some(infixed) = match_seed_infix(trimmed, table) {
        return Some(infixed);
    }
    if trimmed.contains(" == ") || trimmed.contains(" != ") {
        return Some(("comparison".to_owned(), Vec::new(), false));
    }
    if let Some((name, value)) = trimmed.split_once(" = ") {
        return Some((
            "assignment".to_owned(),
            vec![
                ("name", name.trim().to_owned()),
                ("value", value.trim().to_owned()),
            ],
            false,
        ));
    }
    if trimmed.find('/').is_some() {
        return Some(("division".to_owned(), Vec::new(), false));
    }
    // indexing: `identifier[...]`
    if let Some(open) = trimmed.find('[') {
        let before = trimmed[..open].trim_end();
        if before
            .chars()
            .last()
            .is_some_and(|c| c.is_alphanumeric() || c == '_')
            && let Some(close) = trimmed[open..].find(']')
        {
            let items = before;
            let index = &trimmed[open + 1..open + close];
            return Some((
                "indexing".to_owned(),
                vec![("items", items.to_owned()), ("index", index.to_owned())],
                false,
            ));
        }
    }
    // method call: `receiver.method(...)`
    if let Some(dot) = trimmed.find('.')
        && let Some(open) = trimmed[dot..].find('(')
    {
        let value = trimmed[..dot].trim();
        let method = identifier_at(&trimmed[dot + 1..]);
        let args = inside_parens(trimmed, dot + open);
        if !method.is_empty() && !value.is_empty() {
            return Some((
                "method_call".to_owned(),
                vec![
                    ("value", value.to_owned()),
                    ("method", method.to_owned()),
                    ("args", args.to_owned()),
                ],
                false,
            ));
        }
    }
    match_call(trimmed)
}

/// True when any cue phrase for the intent occurs in the normalized prompt
/// or the raw prompt lowercased.
fn triggered(prompt: &str, normalized: &str) -> bool {
    let lower = prompt.to_lowercase();
    cue_phrases(INTENT, "")
        .iter()
        .any(|phrase| normalized.contains(phrase.as_str()) || lower.contains(phrase.as_str()))
}

/// Try to recognize a code-explanation request and explain the code line by
/// line against the construct table. Returns `None` when the prompt is not
/// an explanation request or carries no code.
pub fn handle_code_explanation(
    prompt: &str,
    normalized: &str,
    log: &mut EventLog,
) -> Option<SymbolicAnswer> {
    if !triggered(prompt, normalized) {
        return None;
    }
    let code = super::code_debugging::code_block(prompt)?;
    let table = structures();
    log.append(
        "code_explanation:request",
        format!("line={}", code.lines().count()),
    );

    let mut lines_body = String::new();
    let mut matched = 0usize;
    let mut total = 0usize;
    let mut loop_seen = false;
    for line in code.lines().filter(|line| !line.trim().is_empty()) {
        total += 1;
        let explained = explain_line(line, &table).and_then(|(construct, values, is_loop)| {
            table
                .iter()
                .find(|s| s.construct == construct)
                .map(|structure| (structure, values, is_loop))
        });
        let Some((structure, values, is_loop)) = explained else {
            log.append("code_explanation:unmatched", line.trim().to_owned());
            lines_body.push_str(&template(
                "code_explanation_unmatched_line",
                &[("line", line.trim())],
            ));
            continue;
        };
        matched += 1;
        loop_seen |= is_loop;
        log.append("code_explanation:construct", structure.construct.clone());
        let mut meaning = structure.meaning.clone();
        for (key, value) in &values {
            meaning = meaning.replace(&format!("{{{key}}}"), value);
        }
        lines_body.push_str(&template(
            "code_explanation_line",
            &[
                ("line", line.trim()),
                ("meaning", &meaning),
                ("grounding", &structure.grounding),
            ],
        ));
    }

    let (body, confidence) = if matched == 0 {
        log.append("code_explanation:no_construct", "table=0".to_owned());
        (template("code_explanation_no_construct", &[]), 0.4)
    } else {
        let intents = function_intents();
        let name = function_name(&code);
        let overall = name
            .as_deref()
            .and_then(|name| {
                intents
                    .iter()
                    .find(|intent| intent.names.iter().any(|n| name.contains(n.as_str())))
            })
            .map_or_else(
                || template("code_explanation_no_overall", &[]),
                |intent| {
                    template(
                        "code_explanation_overall",
                        &[
                            ("name", name.as_deref().unwrap_or_default()),
                            ("property", &intent.property),
                            ("correct_form", &intent.correct_form),
                            ("grounding", &intent.grounding),
                        ],
                    )
                },
            );
        let cost = if loop_seen {
            template("code_explanation_cost_linear", &[])
        } else {
            template("code_explanation_cost_constant", &[])
        };
        (
            template(
                "code_explanation_lines",
                &[
                    ("lines", &lines_body),
                    ("matched", &matched.to_string()),
                    ("total", &total.to_string()),
                    ("overall", &overall),
                    ("cost", &cost),
                ],
            ),
            0.7,
        )
    };

    Some(finalize_simple(
        prompt,
        log,
        INTENT,
        "response:code_explanation",
        &body,
        confidence,
    ))
}
