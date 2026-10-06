//! Code-debugging handler (issue #1177, E142 code-task family).
//!
//! Recognizes "find the bug in …" / "debug …" requests (English and Russian;
//! cue phrases live in `data/seed/code-task-cues.lino`), extracts the code
//! under discussion, derives the property the function NAME promises from
//! the `function_intent` records in
//! `data/seed/meanings-code-structure-explanations.lino` (each citing a math
//! source), and scans the function's expressions for suspicious constructs.
//!
//! The defect scan is structural: a binary `+`/`-` applied at top level
//! AFTER a division shifts the quotient (e.g. `sum(xs) / len(xs) - 1`
//! computes mean-minus-one, because `/` binds tighter than `-`), which
//! contradicts what a name like `average` promises. No code is executed and
//! the answer (a template from `data/seed/multilingual-responses.lino`)
//! says so.

use super::finalize_simple;
use crate::engine::SymbolicAnswer;
use crate::event_log::EventLog;
use crate::seed::parser::parse_lino;

const CUES_PATH: &str = "data/seed/code-task-cues.lino";
const INTENTS_PATH: &str = "data/seed/meanings-code-structure-explanations.lino";
const INTENT: &str = "code_debugging";

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

/// One `function_intent` record: the property a function name promises.
struct FunctionIntent {
    names: Vec<String>,
    property: String,
    correct_form: String,
    grounding: String,
}

/// Load the `function_intent` table from the meanings seed file.
fn function_intents() -> Vec<FunctionIntent> {
    let Some(text) = seed_text(INTENTS_PATH) else {
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

/// Extract the code under discussion, shared by the code-task family.
///
/// The first fenced block, else a backtick span that looks like code, else the
/// whole prompt when it carries code markers, else the code-shaped text after
/// the request's colon.
pub(super) fn code_block(prompt: &str) -> Option<String> {
    if let Some(start) = prompt.find("```") {
        let rest = &prompt[start + 3..];
        let after_open = rest.find('\n').map_or(rest, |nl| &rest[nl + 1..]);
        if let Some(end) = after_open.find("```") {
            let code = &after_open[..end];
            if !code.trim().is_empty() {
                return Some(code.to_owned());
            }
        }
    }
    if let Some(start) = prompt.find('`')
        && let Some(end) = prompt[start + 1..].find('`')
    {
        let code = &prompt[start + 1..start + 1 + end];
        let code_markers = ["(", "def ", "=>", "return "];
        if code_markers.iter().any(|marker| code.contains(marker)) {
            return Some(code.to_owned());
        }
    }
    let markers = ["def ", "function ", "fn ", "=>", "return "];
    if markers.iter().any(|marker| prompt.contains(marker)) {
        return Some(prompt.to_owned());
    }
    // The colon after the request introduces the code itself
    // ("Explain this code: print(sum(range(10)))") when it carries a call, an
    // assignment or a subscript.
    let payload = prompt.split_once(": ").map(|(_, rest)| rest.trim())?;
    payload
        .contains(['(', '=', '['])
        .then(|| payload.to_owned())
}

/// The name of the first function defined in the code (`def name(`,
/// `fn name(`, or `function name(`).
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

/// Byte offsets within `expr` of (the first top-level `/`, a top-level
/// binary `+`/`-` after it). The addition/subtraction then applies to the
/// whole quotient, not to an operand of the division.
///
/// A `+`/`-` counts as binary only when the previous non-space character can
/// end an operand (identifier, `)`, `]`, quote); comparisons and statements
/// (`=`, `,`, `;`, `<`, `>`, …) end the arithmetic expression and reset the
/// scan, so `a / b == c - 1` is correctly not flagged.
fn quotient_shift(expr: &str) -> Option<(usize, usize)> {
    let mut depth = 0usize;
    let mut seen_division = false;
    let mut prev_ends_operand = false;
    let mut division_offset = None;
    for (offset, ch) in expr.char_indices() {
        match ch {
            '(' | '[' => {
                depth += 1;
                prev_ends_operand = false;
            }
            ')' | ']' => {
                depth = depth.saturating_sub(1);
                prev_ends_operand = true;
            }
            '/' if depth == 0 => {
                if division_offset.is_none() {
                    division_offset = Some(offset);
                }
                seen_division = true;
                prev_ends_operand = false;
            }
            '+' | '-' if depth == 0 && seen_division && prev_ends_operand => {
                return Some((division_offset?, offset));
            }
            '=' | ',' | ';' | '<' | '>' | '!' | '&' | '|' | '?' | ':' if depth == 0 => {
                seen_division = false;
                prev_ends_operand = false;
            }
            // Whitespace keeps the previous non-space character's verdict,
            // as the contract above states (mirrors the JavaScript twin in
            // js/worker/formal_ai_worker_code_tasks.js).
            c if c.is_whitespace() => {}
            c => {
                prev_ends_operand =
                    c.is_alphanumeric() || c == '_' || c == '"' || c == '\'' || c == '.';
            }
        }
    }
    None
}

/// A detected defect: the line, the operator that shifts the quotient
/// (e.g. `- 1`), and the two candidate fixes.
struct Defect {
    line_number: usize,
    line: String,
    shift: String,
    fixed: String,
    parenthesized: String,
}

/// Scan the code for the quotient-shift defect on `return` lines and
/// assignment right-hand sides.
fn scan_for_defect(code: &str) -> Option<Defect> {
    for (index, line) in code.lines().enumerate() {
        let trimmed = line.trim_start();
        let expression = if let Some(rest) = trimmed.strip_prefix("return ") {
            rest.trim().to_owned()
        } else if let Some((_, rhs)) = line.split_once(" = ") {
            rhs.trim().to_owned()
        } else if let Some(at) = trimmed.find("return ") {
            trimmed[at + 7..].trim().to_owned()
        } else {
            continue;
        };
        if !expression.contains('/') {
            continue;
        }
        let Some((div_at, op_at)) = quotient_shift(&expression) else {
            continue;
        };
        let shift = expression[op_at..].trim().to_owned();
        let fixed = expression[..op_at].trim().to_owned();
        let divisor = expression[div_at + 1..op_at].trim().to_owned();
        let parenthesized = format!("{} / ({} {})", expression[..div_at].trim(), divisor, shift);
        return Some(Defect {
            line_number: index + 1,
            line: line.trim().to_owned(),
            shift,
            fixed,
            parenthesized,
        });
    }
    None
}

/// True when any cue phrase for the intent occurs in the normalized prompt
/// or the raw prompt lowercased.
fn triggered(prompt: &str, normalized: &str) -> bool {
    let lower = prompt.to_lowercase();
    cue_phrases(INTENT, "")
        .iter()
        .any(|phrase| normalized.contains(phrase.as_str()) || lower.contains(phrase.as_str()))
}

/// Try to recognize a code-debugging request and locate a defect. Returns
/// `None` when the prompt is not a debugging request or carries no code.
pub fn handle_code_debugging(
    prompt: &str,
    normalized: &str,
    log: &mut EventLog,
) -> Option<SymbolicAnswer> {
    if !triggered(prompt, normalized) {
        return None;
    }
    let code = code_block(prompt)?;
    log.append("code_debugging:request", "code extracted".to_owned());
    let intents = function_intents();
    let name = function_name(&code);
    if let Some(name) = &name {
        log.append("code_debugging:function_name", name.clone());
    }
    let intent: Option<&FunctionIntent> = name.as_ref().and_then(|name| {
        intents
            .iter()
            .find(|intent| intent.names.iter().any(|n| name.contains(n.as_str())))
    });
    if let Some(intent) = intent {
        log.append("code_debugging:intent_property", intent.property.clone());
    }

    let (body, confidence) = if let (Some(intent), Some(defect)) = (intent, scan_for_defect(&code))
    {
        log.append(
            "code_debugging:defect",
            format!("line {}: `{}`", defect.line_number, defect.shift),
        );
        (
            template(
                "code_debugging_defect",
                &[
                    ("name", name.as_deref().unwrap_or_default()),
                    ("property", &intent.property),
                    ("grounding", &intent.grounding),
                    ("line_no", &defect.line_number.to_string()),
                    ("line", &defect.line),
                    ("shift", &defect.shift),
                    ("fixed", &defect.fixed),
                    ("parenthesized", &defect.parenthesized),
                    ("correct_form", &intent.correct_form),
                ],
            ),
            0.8,
        )
    } else {
        log.append(
            "code_debugging:defect",
            "no recognized defect pattern".to_owned(),
        );
        let intent_note = if intent.is_none() {
            template(
                "code_debugging_no_intent",
                &[("name", name.as_deref().unwrap_or_default())],
            )
        } else {
            String::new()
        };
        (
            template("code_debugging_no_defect", &[("intent_note", &intent_note)]),
            0.5,
        )
    };

    Some(finalize_simple(
        prompt,
        log,
        INTENT,
        "response:code_debugging",
        &body,
        confidence,
    ))
}
