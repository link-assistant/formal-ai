//! Terminal-command intent detection (issue #513, visible fix for #511).
//!
//! When a prompt asks to run a shell/terminal command, the symbolic solver
//! used to fall through to the `unknown` fallback. This module recognizes the
//! shape of a terminal request (fenced/backtick command, a "run ... in
//! terminal" phrasing, an explicit leading shell token followed by
//! argument-shaped words, or a seed-backed semantic shell intent) and returns
//! an `agent_suggestion` intent that (a) names the detected command, (b)
//! explains agent mode, and (c) offers to switch agent mode on and grant the
//! `shell` capability.
//!
//! The detection rules are intentionally mirrored in the JavaScript worker
//! (`js/worker/formal_ai_worker.js`, `tryTerminalCommand`) so both engines stay
//! at parity. The trigger vocabulary itself (terminal/shell phrases, run verbs,
//! Chinese run verbs, leading shell tokens) is **not** hardcoded here: it lives
//! in `data/seed/terminal-commands.lino` and is parsed by
//! [`seed::terminal_command_vocabulary`], so detection is data-driven and the
//! project rule against hardcoded natural language in the solver is upheld.

use crate::engine::SymbolicAnswer;
use crate::event_log::EventLog;
use crate::language::Language;
use crate::seed::{self, TerminalCommandVocabulary};
use crate::solver_handlers::finalize_simple;

/// Extract the first backtick-delimited span, if any (single or fenced).
fn extract_backtick_command(prompt: &str) -> Option<String> {
    let bytes: Vec<char> = prompt.chars().collect();
    let first = bytes.iter().position(|&c| c == '`')?;
    // Skip any run of backticks (handles ``` fenced blocks).
    let mut start = first;
    while start < bytes.len() && bytes[start] == '`' {
        start += 1;
    }
    let mut end = start;
    while end < bytes.len() && bytes[end] != '`' {
        end += 1;
    }
    if end <= start {
        return None;
    }
    let command: String = bytes[start..end].iter().collect();
    let command = command.trim();
    if command.is_empty() {
        None
    } else {
        Some(command.to_owned())
    }
}

/// Tokenize a lowercase prompt into alphanumeric/underscore word tokens.
fn word_tokens(lower: &str) -> Vec<String> {
    lower
        .split(|c: char| !(c.is_alphanumeric() || c == '_'))
        .filter(|t| !t.is_empty())
        .map(ToOwned::to_owned)
        .collect()
}

/// English function words that never appear as bare tokens in the argument
/// positions of the seed's shell commands (`ls`, `git`, `find`, `make`, …) but
/// do appear in natural language that merely *starts* with one of those words:
/// "Find **the** bug: …", "Make **a** 3-day itinerary **for** a first visit
/// **to** Rome." A leading seed token followed by any of these is a sentence
/// about work, not a command line. Issue #1175.
const NATURAL_LANGUAGE_MARKER_WORDS: &[&str] = &[
    "the", "a", "an", "me", "my", "you", "your", "for", "with", "to", "of", "in", "on", "at",
    "and", "or", "that", "this", "it", "is", "are", "please", "help",
];

/// Replace every quoted span in `rest` with spaces, so the natural-language
/// checks that read the result see only the words the shell would also treat
/// as words: `git commit -m "fix the bug"` keeps working because the quoted
/// prose is blanked. A quote opens a span only at a token boundary (start or
/// after whitespace) and closes at the next identical quote, so the apostrophe
/// inside `don't` never opens a span.
fn mask_quoted_spans(rest: &str) -> String {
    let chars: Vec<char> = rest.chars().collect();
    let mut out = String::with_capacity(rest.len());
    let mut index = 0;
    while index < chars.len() {
        let character = chars[index];
        if matches!(character, '\'' | '"' | '`') && (index == 0 || chars[index - 1].is_whitespace())
        {
            // Opening quote: blank through the matching close quote
            // (inclusive); an unterminated quote blanks to the end.
            out.push(' ');
            index += 1;
            while index < chars.len() && chars[index] != character {
                out.push(' ');
                index += 1;
            }
            if index < chars.len() {
                out.push(' ');
                index += 1;
            }
        } else {
            out.push(character);
            index += 1;
        }
    }
    out
}

/// Whether the words after a leading shell token parse as that command's
/// arguments rather than as a natural-language sentence about the work.
///
/// The leading-token path of [`detect_terminal_command`] used to fire whenever
/// a prompt's first word was a seed shell token, and many of those tokens are
/// ordinary English words (`find`, `make`, `file`, `which`, `head`, `tail`,
/// `touch`, `kill`, `export`, `cat`). Issue #1175 reported the fallout: "Find
/// the bug: def average(xs): …" and "Make a 3-day itinerary for a first visit
/// to Rome." were answered with "It looks like you want to run a terminal
/// command". The remainder qualifies as arguments only when:
///
///  - it does not end with a question mark (ASCII or full-width) — commands
///    are not questions;
///  - no bare token ends with sentence punctuation `.` `?` `!` `,` `;` —
///    except a token that is entirely dots, because `.` and `..` are paths
///    (`find . -name '*.log'` must keep working);
///  - it contains no `": "` outside quotes — the "Find the bug: …" shape
///    introduces the task after the colon;
///  - no bare token is a [`NATURAL_LANGUAGE_MARKER_WORDS`] word.
///
/// Quoted spans are data, not prose, so every check reads a
/// [`mask_quoted_spans`] copy. An empty remainder ("git", "ls") vacuously
/// qualifies.
fn parses_as_command_arguments(rest: &str) -> bool {
    let masked = mask_quoted_spans(rest);
    let trimmed = masked.trim();
    if trimmed.ends_with('?') || trimmed.ends_with('？') {
        return false;
    }
    if trimmed
        .char_indices()
        .any(|(index, character)| character == ':' && trimmed[index + 1..].starts_with(' '))
    {
        return false;
    }
    for token in trimmed.split_whitespace() {
        if NATURAL_LANGUAGE_MARKER_WORDS.contains(&token) {
            return false;
        }
        let all_dots = token.chars().all(|character| character == '.');
        let ends_with_sentence_punctuation = token.ends_with('.')
            || token.ends_with('?')
            || token.ends_with('!')
            || token.ends_with(',')
            || token.ends_with(';');
        if ends_with_sentence_punctuation && !all_dots {
            return false;
        }
    }
    true
}

/// Return the leading shell command (the prompt itself) when it starts with a
/// recognized shell token, e.g. `ls ~` or `git status`. The token set comes
/// from `data/seed/terminal-commands.lino`.
///
/// A seed token as the first word is necessary but not sufficient: because
/// many shell tokens double as ordinary English words, the rest of the prompt
/// must also [`parse as command arguments`](parses_as_command_arguments) —
/// "git status" and "find . -name '*.log'" are commands, "Find the bug: …"
/// and "Make a 3-day itinerary …" are sentences that merely start like one
/// (issue #1175). And a request that
/// [`formalizes into work obligations`](crate::intent_formalization::request_carries_work_obligations)
/// — a quoted output literal plus an authoring clause, a file-naming clause —
/// is a sentence about building something even when its language's marker
/// words the argument check above does not know, so the obligation graph, not
/// the leading word, decides (issue #1166).
fn leading_shell_command(prompt: &str, vocab: &TerminalCommandVocabulary) -> Option<String> {
    let trimmed = prompt.trim().trim_matches('`').trim();
    let first = trimmed.split_whitespace().next()?;
    let normalized: String = first
        .chars()
        .take_while(|c| c.is_alphanumeric() || *c == '_' || *c == '-')
        .collect::<String>()
        .to_lowercase();
    if !vocab.shell_tokens.iter().any(|t| t == &normalized) {
        return None;
    }
    let rest = &trimmed[first.len()..];
    if !parses_as_command_arguments(rest) {
        return None;
    }
    if crate::intent_formalization::request_carries_work_obligations(trimmed) {
        return None;
    }
    Some(trimmed.to_owned())
}

/// Detect a terminal-command request and, if found, return the command text.
/// All trigger vocabulary is read from the seed-backed `vocab`.
fn detect_terminal_command(prompt: &str, vocab: &TerminalCommandVocabulary) -> Option<String> {
    let lower = prompt.to_lowercase();
    let has_phrase = vocab.terminal_phrases.iter().any(|p| lower.contains(p));
    let tokens = word_tokens(&lower);
    let has_verb = vocab
        .run_verbs
        .iter()
        .any(|v| tokens.iter().any(|t| t == v))
        || vocab.cjk_run_verbs.iter().any(|v| lower.contains(v));
    let backtick = extract_backtick_command(prompt);
    let leading = leading_shell_command(prompt, vocab);

    // Classify as a terminal request when:
    //  - a backtick command is paired with a run verb or a terminal phrase, or
    //  - a run verb is paired with an explicit terminal phrase, or
    //  - the prompt itself starts with a known shell token and its remainder
    //    parses as command arguments rather than prose (issue #1175).
    if backtick.is_some() && (has_verb || has_phrase) {
        return backtick;
    }
    if has_phrase && has_verb {
        return backtick.or(leading);
    }
    if let Some(cmd) = leading {
        return Some(cmd);
    }
    None
}

/// Build the localized response body for a detected terminal command.
///
/// The natural-language prose lives in `data/seed/multilingual-responses.lino`
/// under the `agent_suggestion` intent (with a `{command}` placeholder), so this
/// function only looks the template up via [`seed::response_for`] and fills in
/// the detected command — no per-language wording is hardcoded here.
#[allow(clippy::literal_string_with_formatting_args)]
fn terminal_body(command: &str, language: Language) -> String {
    let template =
        seed::localized_response("agent_suggestion", language.slug()).unwrap_or_default();
    template.replace("{command}", command)
}

/// Try to recognize a terminal-command request. Returns `Some` with an
/// `agent_suggestion` answer when the prompt looks like a shell command.
pub fn try_terminal_command(
    prompt: &str,
    language: Language,
    log: &mut EventLog,
) -> Option<SymbolicAnswer> {
    let vocab = seed::terminal_command_vocabulary();
    let command = detect_terminal_command(prompt, &vocab)
        .or_else(|| crate::agentic_coding::semantic_shell_command_for_task(prompt))?;
    log.append("terminal:command", command.clone());
    log.append("terminal:agent_suggestion", "shell".to_owned());
    let body = terminal_body(&command, language);
    Some(finalize_simple(
        prompt,
        log,
        "agent_suggestion",
        "response:agent_suggestion",
        &body,
        0.6,
    ))
}
