//! Shell command rewrites that produce command text without executing it.
//!
//! The shell syntax the rewrite reads and writes (the loop and detached-session
//! templates, the command joiners, the prompt markers) is the `shell_syntax`
//! map of `data/seed/code-task-cues.lino`, and the command heads and prose
//! leads the detector weighs are that file's `shell_command_transform` cue
//! records (issue #918), the same records the browser twin reads.

use super::shell_command_compose::{cue_phrases, word_entries};
use crate::engine::SymbolicAnswer;
use crate::event_log::EventLog;
use crate::solver::{ConversationRole, ConversationTurn};
use crate::solver_handlers::finalize_simple;
use crate::solver_helpers::extract_backticked;

const CUE_INTENT: &str = "shell_command_transform";

/// Every value of `word` in the `shell_syntax` map, in seed order.
fn syntax(word: &str) -> Vec<String> {
    word_entries("shell_syntax")
        .iter()
        .filter(|entry| entry.find_child_value("word") == word)
        .map(|entry| entry.find_child_value("value").to_owned())
        .collect()
}

/// The first value of `word` in the `shell_syntax` map, or empty.
fn syntax_template(word: &str) -> String {
    syntax(word).into_iter().next().unwrap_or_default()
}

/// Whether `line` opens with the session program: the first token of the
/// seeded screen template.
fn opens_session(line: &str) -> bool {
    let template = syntax_template("screen_template");
    let program = template.split_whitespace().next();
    program.is_some() && line.split_whitespace().next() == program
}

pub fn try_shell_command_transform(
    prompt: &str,
    normalized: &str,
    log: &mut EventLog,
) -> Option<SymbolicAnswer> {
    try_shell_command_transform_with_history(prompt, normalized, log, &[])
}

pub fn try_shell_command_transform_with_history(
    prompt: &str,
    normalized: &str,
    log: &mut EventLog,
    history: &[ConversationTurn],
) -> Option<SymbolicAnswer> {
    if let Some(command) = build_screen_command(prompt, normalized, history, log) {
        return Some(finalize_shell_transform(
            prompt,
            log,
            "screen_session",
            &command,
        ));
    }

    if wants_infinite_loop(normalized) {
        let command = extract_shell_command(prompt)?;
        let loop_command = wrap_in_infinite_loop(&command);
        log.append("shell_transform", "infinite_loop".to_owned());
        log.append("shell_command:input", command);
        log.append("shell_command:output", loop_command.clone());
        return Some(finalize_shell_transform(
            prompt,
            log,
            "infinite_loop",
            &loop_command,
        ));
    }

    None
}

fn build_screen_command(
    prompt: &str,
    normalized: &str,
    history: &[ConversationTurn],
    log: &mut EventLog,
) -> Option<String> {
    let template = syntax_template("screen_template");
    let program = template.split_whitespace().next()?;
    if !normalized.contains(program) {
        return None;
    }
    if !wants_screen_execution(normalized) {
        return None;
    }

    let session = extract_screen_session(prompt)?;
    let loop_command = extract_loop_command(prompt)
        .or_else(|| last_loop_command_from_history(history))
        .or_else(|| extract_shell_command(prompt).map(|command| wrap_in_infinite_loop(&command)))?;
    let command = template
        .replace("{session}", &session)
        .replace("{command}", &shell_single_quote(&loop_command));
    log.append("shell_transform", "screen_session".to_owned());
    log.append("screen_session", session);
    log.append("shell_command:input", loop_command);
    log.append("shell_command:output", command.clone());
    Some(command)
}

fn finalize_shell_transform(
    prompt: &str,
    log: &mut EventLog,
    operation: &str,
    command: &str,
) -> SymbolicAnswer {
    log.append("shell_transform:operation", operation.to_owned());
    finalize_simple(
        prompt,
        log,
        "shell_command_transform",
        "response:shell_command_transform",
        command,
        0.92,
    )
}

/// Whether the prompt asks for the command to repeat forever. The cue words
/// (English, Russian, Hindi, Chinese) live in the `shell_infinite_loop` set of
/// `data/meta/cue-lexicon.lino`, matched as substrings of the normalized prompt.
fn wants_infinite_loop(normalized: &str) -> bool {
    crate::cue_lexicon::matches("shell_infinite_loop", normalized)
}

/// Whether the prompt asks for the command to run inside a `screen` session
/// (on one line, executed inside it). The cue words live in the
/// `shell_screen_execution` set of `data/meta/cue-lexicon.lino`.
fn wants_screen_execution(normalized: &str) -> bool {
    crate::cue_lexicon::matches("shell_screen_execution", normalized)
}

fn extract_shell_command(prompt: &str) -> Option<String> {
    for line in prompt.lines() {
        let trimmed = strip_code_fence(line.trim());
        if let Some(command) = command_after_shell_prompt(trimmed) {
            return Some(command);
        }
    }

    if let Some(backticked) = extract_backticked(prompt) {
        let command = strip_code_fence(backticked.trim());
        if looks_like_shell_command(command) && !opens_session(command) {
            return Some(command.to_owned());
        }
    }

    prompt
        .lines()
        .map(str::trim)
        .map(strip_code_fence)
        .map(command_span)
        .find(|line| looks_like_shell_command(line) && !opens_session(line))
        .map(str::to_owned)
}

/// Whether the request carries the command or loop it rewrites.
///
/// The `shell_command_operand` claim evidence of issue #1175 R3.
#[must_use]
pub fn names_shell_command(prompt: &str) -> bool {
    extract_shell_command(prompt).is_some() || extract_loop_command(prompt).is_some()
}

/// The command span of a line. A line such as *"Make this a single line loop:
/// sleep 5m && cleanup -f"* carries a prose lead that ends at a colon; the
/// lead is the request, not part of the command, so the command is the span
/// after the colon. The lead counts as prose only when it is plain words
/// (letters, digits, spaces, apostrophes, hyphens), so a colon inside a real
/// command (a URL, a quoted string, a `host:path`) never splits it.
fn command_span(line: &str) -> &str {
    let Some((lead, rest)) = line.split_once(": ") else {
        return line;
    };
    let rest = strip_code_fence(rest.trim());
    let prose_lead = !lead.trim().is_empty()
        && lead
            .chars()
            .all(|ch| ch.is_alphanumeric() || ch.is_whitespace() || ch == '\'' || ch == '-');
    if prose_lead && looks_like_shell_command(rest) {
        rest
    } else {
        line
    }
}

fn command_after_shell_prompt(line: &str) -> Option<String> {
    for marker in syntax("prompt_marker") {
        if let Some(index) = line.rfind(marker.as_str()) {
            let command = line[index + marker.len()..].trim();
            if looks_like_shell_command(command) {
                return Some(command.to_owned());
            }
        }
    }
    None
}

fn strip_code_fence(line: &str) -> &str {
    line.trim_matches('`').trim()
}

fn looks_like_shell_command(candidate: &str) -> bool {
    let candidate = candidate.trim();
    if candidate.is_empty()
        || candidate.contains('\n')
        || candidate.ends_with('?')
        || cue_phrases(CUE_INTENT, "prose_lead")
            .iter()
            .any(|lead| candidate.starts_with(lead.as_str()))
    {
        return false;
    }
    if syntax("joiner")
        .iter()
        .any(|joiner| candidate.contains(joiner.as_str()))
    {
        return true;
    }

    let first = candidate.split_whitespace().next().unwrap_or_default();
    cue_phrases(CUE_INTENT, "command_head")
        .iter()
        .any(|head| head == first)
}

fn wrap_in_infinite_loop(command: &str) -> String {
    if extract_loop_command(command).is_some() {
        return command.trim().to_owned();
    }
    syntax_template("loop_template").replace("{command}", command.trim())
}

fn extract_screen_session(prompt: &str) -> Option<String> {
    let command = extract_backticked(prompt)
        .filter(|text| opens_session(text))
        .or_else(|| {
            prompt
                .lines()
                .map(str::trim)
                .find(|line| opens_session(line))
                .map(str::to_owned)
        })?;

    let mut session = None;
    for token in command.split_whitespace().skip(1) {
        if !token.starts_with('-') {
            session = Some(token);
        }
    }
    let session = session?;
    if session.is_empty() {
        None
    } else {
        Some(session.to_owned())
    }
}

fn extract_loop_command(text: &str) -> Option<String> {
    let trimmed = strip_code_fence(text.trim());
    if is_loop_command(trimmed) {
        return Some(trimmed.to_owned());
    }
    text.lines()
        .map(str::trim)
        .map(strip_code_fence)
        .find(|line| is_loop_command(line))
        .map(str::to_owned)
}

fn last_loop_command_from_history(history: &[ConversationTurn]) -> Option<String> {
    history
        .iter()
        .rev()
        .filter(|turn| turn.role == ConversationRole::Assistant)
        .find_map(|turn| extract_loop_command(&turn.content))
}

/// Whether `candidate` is already in the seeded loop form: the loop template
/// brackets it.
fn is_loop_command(candidate: &str) -> bool {
    syntax_template("loop_template")
        .split_once("{command}")
        .is_some_and(|(open, close)| {
            !open.is_empty() && candidate.starts_with(open) && candidate.ends_with(close)
        })
}

fn shell_single_quote(command: &str) -> String {
    format!("'{}'", command.replace('\'', r"'\''"))
}
