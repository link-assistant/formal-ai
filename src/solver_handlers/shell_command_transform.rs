//! Shell command rewrites that produce command text without executing it.

use crate::engine::SymbolicAnswer;
use crate::event_log::EventLog;
use crate::solver::{ConversationRole, ConversationTurn};
use crate::solver_handlers::finalize_simple;
use crate::solver_helpers::extract_backticked;

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
    if !normalized.contains("screen") {
        return None;
    }
    if !(normalized.contains("single line")
        || normalized.contains("one line")
        || normalized.contains("execute")
        || normalized.contains("run")
        || normalized.contains("inside"))
    {
        return None;
    }

    let session = extract_screen_session(prompt)?;
    let loop_command = extract_loop_command(prompt)
        .or_else(|| last_loop_command_from_history(history))
        .or_else(|| extract_shell_command(prompt).map(|command| wrap_in_infinite_loop(&command)))?;
    let command = format!(
        "screen -dmS {session} bash -c {}",
        shell_single_quote(&loop_command)
    );
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

fn wants_infinite_loop(normalized: &str) -> bool {
    normalized.contains("loop")
        || normalized.contains("infinite")
        || normalized.contains("forever")
        || normalized.contains("repeatedly")
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
        if looks_like_shell_command(command) && !command.starts_with("screen ") {
            return Some(command.to_owned());
        }
    }

    prompt
        .lines()
        .map(str::trim)
        .map(strip_code_fence)
        .find(|line| looks_like_shell_command(line) && !line.starts_with("screen "))
        .map(str::to_owned)
}

fn command_after_shell_prompt(line: &str) -> Option<String> {
    for marker in ["$ ", "# ", "% "] {
        if let Some(index) = line.rfind(marker) {
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
        || candidate.starts_with("make a ")
        || candidate.starts_with("can we ")
    {
        return false;
    }
    if candidate.contains(" && ")
        || candidate.contains(" || ")
        || candidate.contains(" | ")
        || candidate.contains("; ")
    {
        return true;
    }

    let first = candidate.split_whitespace().next().unwrap_or_default();
    matches!(
        first,
        "awk"
            | "bash"
            | "cargo"
            | "curl"
            | "docker"
            | "find"
            | "git"
            | "grep"
            | "hive-cleanup"
            | "jq"
            | "make"
            | "node"
            | "npm"
            | "python"
            | "python3"
            | "sed"
            | "sh"
            | "sleep"
            | "tar"
            | "while"
    )
}

fn wrap_in_infinite_loop(command: &str) -> String {
    if extract_loop_command(command).is_some() {
        return command.trim().to_owned();
    }
    format!("while true; do {}; done", command.trim())
}

fn extract_screen_session(prompt: &str) -> Option<String> {
    let command = extract_backticked(prompt)
        .filter(|text| text.split_whitespace().next() == Some("screen"))
        .or_else(|| {
            prompt
                .lines()
                .map(str::trim)
                .find(|line| line.starts_with("screen "))
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

fn is_loop_command(candidate: &str) -> bool {
    candidate.starts_with("while true; do ") && candidate.ends_with("; done")
}

fn shell_single_quote(command: &str) -> String {
    format!("'{}'", command.replace('\'', r"'\''"))
}
