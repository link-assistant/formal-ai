//! Code-refactoring handler (issue #1177, E142 code-task family).
//!
//! Recognizes "refactor this" / "rewrite the promise chain with async/await"
//! requests (English and Russian; cue phrases live in
//! `data/seed/code-task-cues.lino`), and performs exactly one structural
//! transformation: a JavaScript promise chain (`.then`/`.catch`) becomes an
//! `async function` with `await` — each `.then(handler)` turns into a
//! `const <param> = await <previous>;` in order, the last handler's body
//! becomes a final `await`, and `.catch(handler)` becomes the catch clause.
//! Handler bodies are preserved verbatim.
//!
//! The equivalence check is structural (no code is executed) and the answer
//! (a template from `data/seed/multilingual-responses.lino`) states both
//! facts. Anything that is not a promise chain — or a handler the parser
//! does not fully understand — is refused by name rather than guessed.

use crate::engine::SymbolicAnswer;
use crate::event_log::EventLog;
use crate::seed::parser::parse_lino;
use super::finalize_simple;

const CUES_PATH: &str = "data/seed/code-task-cues.lino";
const INTENT: &str = "code_refactoring";

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
        for phrase in record.children.iter().filter(|child| child.name == "phrase") {
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

/// One chain handler: its parameter and its body expression.
struct Handler {
    param: String,
    body: String,
}

/// The balanced-parenthesis span starting at the '(' at `open_at`, with the
/// index just past the closing ')'.
fn paren_span(text: &str, open_at: usize) -> (&str, usize) {
    let mut depth = 0usize;
    for (offset, ch) in text[open_at..].char_indices() {
        match ch {
            '(' => depth += 1,
            ')' => {
                depth -= 1;
                if depth == 0 {
                    return (&text[open_at + 1..open_at + offset], open_at + offset + 1);
                }
            }
            _ => {}
        }
    }
    ("", open_at)
}

/// Parse one handler expression: `x => expr`, `(x) => expr`, or
/// `function (x) { body }`. Returns `None` for any other shape.
fn parse_handler(handler: &str) -> Option<Handler> {
    let trimmed = handler.trim();
    if let Some(at) = trimmed.find("=>") {
        let param = trimmed[..at]
            .trim()
            .trim_matches('(')
            .trim_matches(')')
            .trim();
        let body = trimmed[at + 2..].trim();
        if param.is_empty() || body.is_empty() {
            return None;
        }
        return Some(Handler {
            param: param.to_owned(),
            body: body.to_owned(),
        });
    }
    if let Some(rest) = trimmed.strip_prefix("function") {
        let open = rest.find('(')?;
        let (param, after) = paren_span(rest, open);
        let brace = after + rest[after..].find('{')?;
        let mut depth = 0usize;
        for (offset, ch) in rest[brace..].char_indices() {
            match ch {
                '{' => depth += 1,
                '}' => {
                    depth -= 1;
                    if depth == 0 {
                        let body = rest[brace + 1..brace + offset].trim();
                        if param.trim().is_empty() || body.is_empty() {
                            return None;
                        }
                        return Some(Handler {
                            param: param.trim().to_owned(),
                            body: body.to_owned(),
                        });
                    }
                }
                _ => {}
            }
        }
    }
    None
}

/// The promise chain: the head expression, the `.then` handlers in order,
/// and the `.catch` handler when present.
struct Chain {
    head: String,
    thens: Vec<Handler>,
    catch: Option<Handler>,
}

/// Parse the chain out of whitespace-flattened code.
fn parse_chain(flat: &str) -> Option<Chain> {
    let first_then = flat.find(".then(")?;
    let head = flat[..first_then].trim().trim_end_matches(';').trim().to_owned();
    if head.is_empty() {
        return None;
    }
    let mut thens = Vec::new();
    let mut catch = None;
    let mut cursor = first_then;
    loop {
        let then_at = flat[cursor..].find(".then(").map(|o| cursor + o);
        let catch_at = flat[cursor..].find(".catch(").map(|o| cursor + o);
        let (at, marker, length) = match (then_at, catch_at) {
            (Some(t), Some(c)) if t < c => (t, "then", 6),
            (_, Some(c)) => (c, "catch", 7),
            (Some(t), None) => (t, "then", 6),
            (None, None) => break,
        };
        let (handler_text, end) = paren_span(flat, at + length - 1);
        let handler = parse_handler(handler_text)?;
        if marker == "then" {
            thens.push(handler);
        } else {
            catch = Some(handler);
        }
        cursor = end;
    }
    if thens.is_empty() {
        return None;
    }
    Some(Chain { head, thens, catch })
}

/// Render the async/await rewrite. Every handler body is copied verbatim.
fn render_async(chain: &Chain) -> String {
    let mut lines: Vec<String> = Vec::new();
    lines.push(["async ", "function run() {"].concat());
    if chain.catch.is_some() {
        lines.push(["  try {"].concat());
    }
    let indent = if chain.catch.is_some() { "    " } else { "  " };
    let mut previous = chain.head.clone();
    for handler in &chain.thens {
        lines.push(
            [
                indent,
                "const ",
                &handler.param,
                " = await ",
                &previous,
                ";",
            ]
            .concat(),
        );
        previous = handler.body.clone();
    }
    lines.push([indent, "await ", &previous, ";"].concat());
    if let Some(catch) = &chain.catch {
        lines.push(["  } catch (", &catch.param, ") {"].concat());
        lines.push(["    ", &catch.body].concat());
        lines.push(["  }"].concat());
    }
    lines.push(["}"].concat());
    lines.join("\n")
}

/// True when any cue phrase for the intent occurs in the normalized prompt
/// or the raw prompt lowercased.
fn triggered(prompt: &str, normalized: &str) -> bool {
    let lower = prompt.to_lowercase();
    cue_phrases(INTENT, "")
        .iter()
        .any(|phrase| normalized.contains(phrase.as_str()) || lower.contains(phrase.as_str()))
}

/// Try to recognize a refactoring request and apply the one implemented
/// structural transformation (promise chain → async/await). Refuses by name
/// for anything else.
pub fn handle_code_refactoring(
    prompt: &str,
    normalized: &str,
    log: &mut EventLog,
) -> Option<SymbolicAnswer> {
    if !triggered(prompt, normalized) {
        return None;
    }
    let code = code_block(prompt);
    log.append("code_refactoring:request", "cued".to_owned());

    let chain = code.as_deref().and_then(|code| {
        let flat = code.split_whitespace().collect::<Vec<_>>().join(" ");
        parse_chain(&flat)
    });

    let (body, confidence) = match chain {
        Some(chain) => {
            log.append(
                "code_refactoring:chain",
                format!("then={}", chain.thens.len()),
            );
            let rewritten = render_async(&chain);
            (template("code_refactoring_async", &[("code", &rewritten)]), 0.7)
        }
        None => {
            log.append("code_refactoring:refusal", "chain=none".to_owned());
            (template("code_refactoring_refusal", &[]), 0.4)
        }
    };

    Some(finalize_simple(
        prompt,
        log,
        INTENT,
        "response:code_refactoring",
        &body,
        confidence,
    ))
}
