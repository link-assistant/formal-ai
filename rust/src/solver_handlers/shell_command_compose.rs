//! Shell-command composition handler (issue #1177, E142 code-task family).
//!
//! Recognizes natural-language shell requests ("find .log files larger than
//! 10 MB under /var", "show the last 20 lines of app.log", English and
//! Russian; action-verb cues live in `data/seed/code-task-cues.lino`), and
//! composes a command from the manual-page flag table in
//! `data/seed/manual-pages.lino` — so every emitted flag is explained and
//! cites the upstream GNU manual. Covers `find` (-name/-size/-mtime/-type),
//! `ls` (-l/-a/-R), `head`/`tail` (-n) and `grep` (-r/-i/-l).
//!
//! Nothing is executed: the answer (a template from
//! `data/seed/multilingual-responses.lino`) is a composed suggestion and
//! says so. Requests outside the flag table are refused by name.
//!
//! Like every handler in this family, all response prose comes from seed
//! templates; the Rust here only recognizes cues, composes code, and
//! renders request→emission mapping rows (each row's shape is itself a
//! seed template, `mapping_line`).

use super::finalize_simple;
use crate::engine::SymbolicAnswer;
use crate::event_log::EventLog;
use crate::seed::parser::{LinoNode, parse_lino};

const CUES_PATH: &str = "data/seed/code-task-cues.lino";
const MANUAL_PATH: &str = "data/seed/manual-pages.lino";
const INTENT: &str = "shell_command_compose";
/// Other code-task intents: when one of their cues matches, this handler
/// steps aside (they are the more specific request).
const OTHER_INTENTS: &[&str] = &[
    "code_debugging",
    "regex_synthesis",
    "sql_synthesis",
    "code_explanation",
    "code_review",
    "test_generation",
    "code_refactoring",
    "format_conversion",
];
/// Size-test qualifiers: they turn a bare "10 MB" mention into a
/// strictly-greater find test.
const SIZE_QUALIFIERS: &[&str] = &["larger", "bigger", "over", "more", "least", "больше"];

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

/// True when any cue phrase of the intent (any role) matches.
fn any_cue_matches(intent: &str, prompt: &str, normalized: &str) -> bool {
    let lower = prompt.to_lowercase();
    let Some(text) = seed_text(CUES_PATH) else {
        return false;
    };
    let tree = parse_lino(text);
    for record in tree.children.iter().filter(|child| child.name == "cues") {
        if record.find_child_value("intent") != intent {
            continue;
        }
        for phrase in record
            .children
            .iter()
            .flat_map(|child| child.children.iter())
            .filter(|phrase| phrase.name == "phrase")
        {
            if normalized.contains(phrase.id.as_str()) || lower.contains(phrase.id.as_str()) {
                return true;
            }
        }
    }
    false
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

/// Render the request→emission mapping rows through the shared seed
/// template (the row shape — quote, arrow, indentation — is data too).
fn mapping_rows(rows: &[(String, String)]) -> String {
    rows.iter()
        .map(|(request, emission)| {
            template(
                "mapping_line",
                &[("request", request), ("emission", emission)],
            )
        })
        .collect()
}

/// One flag row of a manual-page record.
struct ManualFlag {
    spelling: String,
    meaning: String,
}

/// One `command` record from `data/seed/manual-pages.lino`.
struct ManualCommand {
    name: String,
    package: String,
    url: String,
    flags: Vec<ManualFlag>,
}

fn manual_pages() -> Vec<ManualCommand> {
    let Some(text) = seed_text(MANUAL_PATH) else {
        return Vec::new();
    };
    let tree = parse_lino(text);
    let mut out = Vec::new();
    for record in tree.children.iter().filter(|child| child.name == "command") {
        let name = record.find_child_value("name").to_string();
        if name.is_empty() {
            continue;
        }
        // A command's fields sit under its `name` child.
        let Some(body) = record.children.first().filter(|child| child.name == "name") else {
            continue;
        };
        let mut flags = Vec::new();
        for flag in body.children.iter().filter(|child| child.name == "flag") {
            flags.push(ManualFlag {
                spelling: flag.find_child_value("spelling").to_string(),
                meaning: flag.find_child_value("meaning").to_string(),
            });
        }
        out.push(ManualCommand {
            name,
            package: body.find_child_value("package").to_string(),
            url: body.find_child_value("url").to_string(),
            flags,
        });
    }
    out
}

/// A composed command plus the mapping rows and manual rows explaining each
/// emitted flag.
struct Composed {
    command: String,
    package: String,
    url: String,
    /// (request echo, emitted code) rows.
    rows: Vec<(String, String)>,
    /// (actual flag text, manual meaning) per emitted flag.
    explained: Vec<(String, String)>,
}

/// The manual meaning rows for the flags a composition actually used.
fn explained_flags(manual: &ManualCommand, used: &[String]) -> Vec<(String, String)> {
    manual
        .flags
        .iter()
        .filter(|flag| used.iter().any(|spelling| flag.spelling == *spelling))
        .map(|flag| (flag.spelling.clone(), flag.meaning.clone()))
        .collect()
}

fn tokens(normalized: &str) -> Vec<&str> {
    normalized.split(' ').filter(|t| !t.is_empty()).collect()
}

/// A contiguous echo of the request's own tokens (the mapping rows quote
/// the request, never a Rust-authored sentence).
fn echo(tokens: &[&str], from: usize, to: usize) -> String {
    tokens
        .get(from..=to)
        .map(|slice| slice.join(" "))
        .unwrap_or_default()
}

/// The search root: the first path-like word of the raw prompt ("/var",
/// "/var/log", "~"), defaulting to ".".
fn root_path(prompt: &str) -> String {
    for word in prompt.split_whitespace() {
        let trimmed = word.trim_matches(|c: char| !c.is_alphanumeric() && c != '/' && c != '~');
        if trimmed.starts_with('/') || trimmed.strip_prefix("~/").is_some() {
            return trimmed.to_owned();
        }
    }
    ".".to_owned()
}

/// A file argument: a word with an inner dot ("app.log"), not a path.
fn file_argument(prompt: &str) -> Option<String> {
    for word in prompt.split_whitespace() {
        let trimmed =
            word.trim_matches(|c: char| !c.is_alphanumeric() && c != '.' && c != '-' && c != '_');
        if trimmed.find('.').is_some_and(|pos| pos > 0) {
            return Some(trimmed.to_owned());
        }
    }
    None
}

/// The numeric value of a word: a digit string, or a word in the `number`
/// word map.
fn number_value(word: &str, numbers: &[LinoNode]) -> Option<u32> {
    if let Ok(value) = word.parse::<u32>() {
        return Some(value);
    }
    numbers
        .iter()
        .find(|entry| entry.find_child_value("word") == word)
        .and_then(|entry| entry.find_child_value("value").parse::<u32>().ok())
}

/// The `-name` pattern: a raw ".ext" mention, or the word before "files"
/// ("log files", "python files") looked up in the extension word map.
fn name_pattern(prompt: &str, normalized_tokens: &[&str]) -> Option<String> {
    for word in prompt.split_whitespace() {
        let trimmed = word.trim_matches(|c: char| !c.is_alphanumeric() && c != '.');
        if let Some(ext) = trimmed.strip_prefix('.') {
            if !ext.is_empty() && ext.chars().all(|c| c.is_alphanumeric()) {
                return Some(format!("*.{ext}"));
            }
        }
    }
    let extensions = word_entries("extension");
    for (index, token) in normalized_tokens.iter().enumerate() {
        if matches!(*token, "files" | "файлы" | "файлов") && index > 0 {
            if let Some(entry) = extensions
                .iter()
                .find(|entry| entry.find_child_value("word") == normalized_tokens[index - 1])
            {
                return Some(entry.find_child_value("value").to_owned());
            }
        }
    }
    None
}

/// The size test: a number with a unit word ("10 mb", "10mb") qualified by
/// a larger-than word. Returns (number, unit letter, request token span).
fn size_test(normalized_tokens: &[&str]) -> Option<(u32, String, usize, usize)> {
    let qualifier_at = normalized_tokens
        .iter()
        .position(|token| SIZE_QUALIFIERS.contains(token))?;
    let units = word_entries("size_unit");
    let numbers = word_entries("number");
    for (index, token) in normalized_tokens.iter().enumerate() {
        if let Some(value) = number_value(token, &numbers) {
            if let Some(next) = normalized_tokens.get(index + 1) {
                if let Some(unit) = units
                    .iter()
                    .find(|entry| entry.find_child_value("word") == *next)
                {
                    return Some((
                        value,
                        unit.find_child_value("value").to_owned(),
                        qualifier_at,
                        index + 1,
                    ));
                }
            }
        }
        // joined form "10mb"
        let digits: String = token.chars().take_while(|c| c.is_ascii_digit()).collect();
        if !digits.is_empty() {
            let rest = &token[digits.len()..];
            if let Some(unit) = units
                .iter()
                .find(|entry| entry.find_child_value("word") == rest)
            {
                if let Ok(value) = digits.parse::<u32>() {
                    return Some((
                        value,
                        unit.find_child_value("value").to_owned(),
                        qualifier_at,
                        index,
                    ));
                }
            }
        }
    }
    None
}

/// The mtime test: a number of days ("older than 5 days" → +5, "modified
/// in the last 3 days" → -3).
fn mtime_test(normalized_tokens: &[&str]) -> Option<String> {
    let numbers = word_entries("number");
    let days_at = normalized_tokens
        .iter()
        .position(|token| matches!(*token, "days" | "day" | "дней" | "дня"))?;
    let count = number_value(normalized_tokens[days_at.checked_sub(1)?], &numbers)?;
    let older = normalized_tokens
        .iter()
        .any(|token| matches!(*token, "older" | "старше"));
    let sign = if older { "+" } else { "-" };
    Some(format!("{}{}", sign, count))
}

/// "last/first N lines of FILE" → head/tail.
fn line_slice(normalized_tokens: &[&str], prompt: &str) -> Option<Composed> {
    let numbers = word_entries("number");
    let last = normalized_tokens
        .iter()
        .any(|token| matches!(*token, "last" | "последние" | "последних"));
    let first = normalized_tokens
        .iter()
        .any(|token| matches!(*token, "first" | "первые" | "первых"));
    if !last && !first {
        return None;
    }
    let lines_at = normalized_tokens
        .iter()
        .position(|token| matches!(*token, "lines" | "line" | "строк" | "строки"))?;
    let count = number_value(normalized_tokens[lines_at.checked_sub(1)?], &numbers)?;
    let direction_at = if last {
        normalized_tokens
            .iter()
            .position(|token| matches!(*token, "last" | "последние" | "последних"))?
    } else {
        normalized_tokens
            .iter()
            .position(|token| matches!(*token, "first" | "первые" | "первых"))?
    };
    let tool = if last { "tail" } else { "head" };
    let file = file_argument(prompt)?;
    let manual = manual_pages()
        .into_iter()
        .find(|command| command.name == tool)?;
    let meaning = manual
        .flags
        .iter()
        .find(|flag| flag.spelling == "-n N")
        .map(|flag| flag.meaning.clone())
        .unwrap_or_default();
    Some(Composed {
        command: format!("{} -n {} {}", tool, count, file),
        package: manual.package,
        url: manual.url,
        rows: vec![
            (
                echo(normalized_tokens, direction_at, lines_at),
                format!("{} -n {}", tool, count),
            ),
            (file.clone(), file),
        ],
        explained: vec![("-n N".to_owned(), meaning)],
    })
}

/// "search for X" / "files containing X" → grep.
fn grep_search(normalized_tokens: &[&str], prompt: &str) -> Option<Composed> {
    let containment_at = normalized_tokens.iter().position(|token| {
        matches!(
            *token,
            "contain" | "contains" | "containing" | "содержат" | "содержит"
        )
    });
    let search_at = normalized_tokens
        .iter()
        .position(|token| matches!(*token, "search" | "поищи"))
        .filter(|_| normalized_tokens.iter().any(|token| *token == "for"));
    let cue_at = containment_at.or(search_at)?;
    // The pattern: the raw word after "for " / "containing " (case kept).
    let lower = prompt.to_lowercase();
    let pattern = ["for ", "containing ", "contains ", "contain "]
        .iter()
        .find_map(|lead| {
            lower.find(lead).map(|at| {
                prompt[at + lead.len()..]
                    .split_whitespace()
                    .next()
                    .unwrap_or_default()
                    .trim_matches(|c: char| !c.is_alphanumeric() && c != '_' && c != '-')
                    .to_owned()
            })
        })
        .filter(|pattern| !pattern.is_empty())?;
    let path = root_path(prompt);
    let mut flags = vec!["-r".to_owned()];
    if normalized_tokens
        .iter()
        .any(|token| matches!(*token, "ignore" | "insensitive" | "регистр"))
        || normalized_tokens
            .iter()
            .any(|token| *token == "case" || *token == "регистру")
    {
        flags.push("-i".to_owned());
    }
    if normalized_tokens
        .iter()
        .any(|token| matches!(*token, "which" | "names" | "имена"))
    {
        flags.push("-l".to_owned());
    }
    let manual = manual_pages()
        .into_iter()
        .find(|command| command.name == "grep")?;
    let flag_list = flags.join(" ");
    let explained = explained_flags(&manual, &flags);
    let rows = vec![
        (
            echo(normalized_tokens, cue_at, cue_at),
            "grep(1)".to_owned(),
        ),
        (pattern.clone(), format!("'{pattern}'")),
        (path.clone(), path.clone()),
    ];
    Some(Composed {
        command: format!("grep {} '{}' {}", flag_list, pattern, path),
        package: manual.package,
        url: manual.url,
        rows,
        explained,
    })
}

/// The find composition: path + -name + -size + -mtime + -type.
fn find_files(normalized_tokens: &[&str], prompt: &str) -> Option<Composed> {
    let pattern = name_pattern(prompt, normalized_tokens);
    let size = size_test(normalized_tokens);
    let mtime = mtime_test(normalized_tokens);
    let directories_at = normalized_tokens.iter().position(|token| {
        matches!(
            *token,
            "directories" | "directory" | "dirs" | "директории" | "папки"
        )
    });
    if pattern.is_none() && size.is_none() && mtime.is_none() && directories_at.is_none() {
        return None;
    }
    let path = root_path(prompt);
    let mut parts = vec![format!("find {}", path)];
    let mut rows: Vec<(String, String)> = vec![(path.clone(), format!("find {}", path))];
    let mut used_flags: Vec<String> = Vec::new();
    if let Some(pattern) = &pattern {
        parts.push(format!("-name '{}'", pattern));
        rows.push((pattern.clone(), format!("-name '{}'", pattern)));
        used_flags.push("-name pattern".to_owned());
    }
    if let Some((count, unit, span_from, span_to)) = &size {
        parts.push(format!("-size +{}{}", count, unit));
        rows.push((
            echo(normalized_tokens, *span_from, *span_to),
            format!("-size +{}{}", count, unit),
        ));
        used_flags.push("-size +N[kMG]".to_owned());
    }
    if let Some(mtime) = &mtime {
        parts.push(format!("-mtime {}", mtime));
        rows.push((mtime.clone(), format!("-mtime {}", mtime)));
        used_flags.push("-mtime N".to_owned());
    }
    if let Some(dir_at) = directories_at {
        parts.push("-type d".to_owned());
        rows.push((
            echo(normalized_tokens, dir_at, dir_at),
            "-type d".to_owned(),
        ));
        used_flags.push("-type f".to_owned());
    }
    let manual = manual_pages()
        .into_iter()
        .find(|command| command.name == "find")?;
    let explained = explained_flags(&manual, &used_flags);
    Some(Composed {
        command: parts.join(" "),
        package: manual.package,
        url: manual.url,
        rows,
        explained,
    })
}

/// The ls composition: flags for long/hidden/recursive listings.
fn ls_listing(normalized_tokens: &[&str], prompt: &str) -> Option<Composed> {
    let mut flags: Vec<String> = Vec::new();
    let mut rows: Vec<(String, String)> = Vec::new();
    if let Some(at) = normalized_tokens
        .iter()
        .position(|token| matches!(*token, "long" | "detailed" | "подробно"))
    {
        flags.push("-l".to_owned());
        rows.push((echo(normalized_tokens, at, at), "-l".to_owned()));
    }
    if let Some(at) = normalized_tokens
        .iter()
        .position(|token| matches!(*token, "hidden" | "скрыт" | "скрытые"))
        .or_else(|| {
            normalized_tokens
                .iter()
                .position(|token| *token == "all" || *token == "все")
        })
    {
        flags.push("-a".to_owned());
        rows.push((echo(normalized_tokens, at, at), "-a".to_owned()));
    }
    if let Some(at) = normalized_tokens
        .iter()
        .position(|token| matches!(*token, "recursive" | "рекурсив" | "поддиректории"))
    {
        flags.push("-R".to_owned());
        rows.push((echo(normalized_tokens, at, at), "-R".to_owned()));
    }
    let path = root_path(prompt);
    let manual = manual_pages()
        .into_iter()
        .find(|command| command.name == "ls")?;
    let flag_list = if flags.is_empty() {
        String::new()
    } else {
        format!("{} ", flags.join(" "))
    };
    rows.push((path.clone(), path.clone()));
    let explained = explained_flags(&manual, &flags);
    Some(Composed {
        command: format!("ls {}{}", flag_list, path),
        package: manual.package,
        url: manual.url,
        rows,
        explained,
    })
}

/// Try to recognize a natural-language shell request and compose a command
/// from the manual-page flag table. Refuses by name when nothing in the
/// table covers the request.
pub fn handle_shell_command_compose(
    prompt: &str,
    normalized: &str,
    log: &mut EventLog,
) -> Option<SymbolicAnswer> {
    let normalized_tokens = tokens(normalized);
    let actions = cue_phrases(INTENT, "action");
    let has_action = normalized_tokens
        .iter()
        .any(|token| actions.iter().any(|action| action == token));
    if !has_action {
        return None;
    }
    let file_context: Vec<String> = word_entries("file_context")
        .iter()
        .map(|entry| entry.find_child_value("word").to_owned())
        .collect();
    let has_context = normalized_tokens
        .iter()
        .any(|token| file_context.iter().any(|word| word == token));
    if !has_context {
        return None;
    }
    // A more specific code-task intent cued? Step aside.
    if OTHER_INTENTS
        .iter()
        .any(|intent| any_cue_matches(intent, prompt, normalized))
    {
        return None;
    }
    log.append(
        "shell_command_compose:request",
        "action + file context".to_owned(),
    );

    let composition = line_slice(&normalized_tokens, prompt)
        .or_else(|| grep_search(&normalized_tokens, prompt))
        .or_else(|| find_files(&normalized_tokens, prompt))
        .or_else(|| ls_listing(&normalized_tokens, prompt));

    let (body, confidence) = match composition {
        Some(composed) => {
            log.append("shell_command_compose:command", composed.command.clone());
            let mut flags = String::new();
            for (flag, meaning) in &composed.explained {
                flags.push_str(&template(
                    "flag_line",
                    &[("flag", flag), ("meaning", meaning)],
                ));
            }
            (
                template(
                    "shell_command_composed",
                    &[
                        ("command", &composed.command),
                        ("package", &composed.package),
                        ("flags", &flags),
                        ("manual", &composed.url),
                        ("derivation", &mapping_rows(&composed.rows)),
                    ],
                ),
                0.7,
            )
        }
        None => {
            log.append(
                "shell_command_compose:refusal",
                "no manual-page entry covers the request".to_owned(),
            );
            (template("shell_compose_refusal", &[]), 0.4)
        }
    };

    Some(finalize_simple(
        prompt,
        log,
        INTENT,
        "response:shell_command_compose",
        &body,
        confidence,
    ))
}
