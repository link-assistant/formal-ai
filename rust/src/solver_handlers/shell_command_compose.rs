//! Shell-command composition handler (issue #1177, E142 code-task family).
//!
//! Recognizes natural-language shell requests ("find .log files larger than
//! 10 MB under /var", "show the last 20 lines of app.log", English and
//! Russian; action-verb cues live in `data/seed/code-task-cues.lino`), and
//! composes a command from the manual-page flag table in
//! `data/seed/manual-pages.lino` — so every emitted flag is explained and
//! cites the upstream GNU manual. Covers `find` (-name/-size/-mtime/-type),
//! `ls` (-l/-a/-R), `head`/`tail` (-n), `grep` (-r/-i/-l) and a `sed`
//! substitution ("replace foo with bar in config.txt"); a counting request
//! pipes a find into `wc -l`. Every request word the composer reads
//! (line-slice directions, containment, directory, count, "ending in" ...)
//! comes from the `shell_cue` word map, keyed by role.
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
/// The word map holding every shell-request word with the role the
/// composer reads it in (`last`, `containment`, `count`, ...).
const SHELL_CUE_MAP: &str = "shell_cue";

/// Look up embedded seed content by its registered path.
fn seed_text(path: &str) -> Option<&'static str> {
    crate::seed::seed_files()
        .into_iter()
        .find(|(registered, _)| *registered == path)
        .map(|(_, text)| text)
}

/// Trigger phrases for one intent (optionally one named role) from
/// `data/seed/code-task-cues.lino`.
#[must_use]
pub fn cue_phrases(intent: &str, role: &str) -> Vec<String> {
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
pub(super) fn any_cue_matches(intent: &str, prompt: &str, normalized: &str) -> bool {
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
pub(super) fn word_entries(map: &str) -> Vec<LinoNode> {
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

/// The words the `shell_cue` map reads in one role, in seed order.
fn role_words(role: &str) -> Vec<String> {
    word_entries(SHELL_CUE_MAP)
        .iter()
        .filter(|entry| entry.find_child_value("value") == role)
        .map(|entry| entry.find_child_value("word").to_owned())
        .collect()
}

/// Index of the first request token that is one of the role's words.
fn role_position(normalized_tokens: &[&str], role: &str) -> Option<usize> {
    let words = role_words(role);
    normalized_tokens
        .iter()
        .position(|token| words.iter().any(|word| word == token))
}

/// The first raw word after the first seeded lead phrase of a role (case
/// kept); `Some("")` when the lead closes the prompt.
fn word_after_lead(prompt: &str, role: &str) -> Option<String> {
    let lower = prompt.to_lowercase();
    role_words(role).iter().find_map(|lead| {
        let spaced = format!("{lead} ");
        lower.find(spaced.as_str()).map(|at| {
            prompt[at + spaced.len()..]
                .split_whitespace()
                .next()
                .unwrap_or_default()
                .to_owned()
        })
    })
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
    /// The pipe stage a counting request appends (`| wc -l`).
    pipe: Option<PipeStage>,
}

/// One pipe stage: the tool, its manual record, and its explained flags.
struct PipeStage {
    tool: String,
    manual: ManualCommand,
    explained: Vec<(String, String)>,
}

/// A counting request ("count .lino files") pipes the composition into
/// `wc -l`, explained from wc's manual record.
fn counted(normalized_tokens: &[&str], composed: Option<Composed>) -> Option<Composed> {
    let mut composed = composed?;
    let Some(count_at) = role_position(normalized_tokens, "count") else {
        return Some(composed);
    };
    let Some(manual) = manual_pages()
        .into_iter()
        .find(|command| command.name == "wc")
    else {
        return Some(composed);
    };
    composed.command.push_str(" | wc -l");
    composed.rows.push((
        echo(normalized_tokens, count_at, count_at),
        "| wc -l".to_owned(),
    ));
    let explained = explained_flags(&manual, &["-l".to_owned()]);
    composed.pipe = Some(PipeStage {
        tool: "wc".to_owned(),
        manual,
        explained,
    });
    Some(composed)
}

/// Render explained flag rows through the shared `flag_line` template.
fn flag_lines(explained: &[(String, String)]) -> String {
    explained
        .iter()
        .map(|(flag, meaning)| template("flag_line", &[("flag", flag), ("meaning", meaning)]))
        .collect()
}

/// The manual meaning rows for the flags a composition actually used.
fn explained_flags(manual: &ManualCommand, used: &[String]) -> Vec<(String, String)> {
    manual
        .flags
        .iter()
        .filter(|flag| used.contains(&flag.spelling))
        .map(|flag| (flag.spelling.clone(), flag.meaning.clone()))
        .collect()
}

fn tokens(normalized: &str) -> Vec<&str> {
    normalized.split(' ').filter(|t| !t.is_empty()).collect()
}

/// Whether the request acts on files or directories, or is a whole substitution.
///
/// The `filesystem_object` claim evidence of issue #1175 R3.
#[must_use]
pub fn names_filesystem_object(prompt: &str, normalized: &str) -> bool {
    let file_context: Vec<String> = word_entries("file_context")
        .iter()
        .map(|entry| entry.find_child_value("word").to_owned())
        .collect();
    sed_substitution(prompt).is_some()
        || tokens(normalized)
            .iter()
            .any(|token| file_context.iter().any(|word| word == token))
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
        // A relative path names its segments around an inner slash ("data/meta").
        let mut segments = trimmed.split('/');
        if trimmed.split('/').count() > 1 && segments.all(|segment| !segment.is_empty()) {
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
    // "ending in .lino" / "ending with _test.go": the suffix itself.
    let suffix = word_after_lead(prompt, "suffix_lead").unwrap_or_default();
    let suffix =
        suffix.trim_matches(|c: char| !c.is_alphanumeric() && c != '.' && c != '_' && c != '-');
    if !suffix.is_empty() {
        return Some(format!("*{suffix}"));
    }
    for word in prompt.split_whitespace() {
        let trimmed = word.trim_matches(|c: char| !c.is_alphanumeric() && c != '.');
        if let Some(ext) = trimmed.strip_prefix('.')
            && !ext.is_empty()
            && ext.chars().all(char::is_alphanumeric)
        {
            return Some(format!("*.{ext}"));
        }
    }
    let extensions = word_entries("extension");
    let files_nouns = role_words("files_noun");
    for (index, token) in normalized_tokens.iter().enumerate() {
        if files_nouns.iter().any(|noun| noun == token)
            && index > 0
            && let Some(entry) = extensions
                .iter()
                .find(|entry| entry.find_child_value("word") == normalized_tokens[index - 1])
        {
            return Some(entry.find_child_value("value").to_owned());
        }
    }
    None
}

/// The size test: a number with a unit word ("10 mb", "10mb") qualified by
/// a larger-than word. Returns (number, unit letter, request token span).
fn size_test(normalized_tokens: &[&str]) -> Option<(u32, String, usize, usize)> {
    let qualifier_at = role_position(normalized_tokens, "size_qualifier")?;
    let units = word_entries("size_unit");
    let numbers = word_entries("number");
    for (index, token) in normalized_tokens.iter().enumerate() {
        if let Some(value) = number_value(token, &numbers)
            && let Some(next) = normalized_tokens.get(index + 1)
            && let Some(unit) = units
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
        // joined form "10mb"
        let digits: String = token.chars().take_while(char::is_ascii_digit).collect();
        if !digits.is_empty() {
            let rest = &token[digits.len()..];
            if let Some(unit) = units
                .iter()
                .find(|entry| entry.find_child_value("word") == rest)
                && let Ok(value) = digits.parse::<u32>()
            {
                return Some((
                    value,
                    unit.find_child_value("value").to_owned(),
                    qualifier_at,
                    index,
                ));
            }
        }
    }
    None
}

/// The mtime test: a number of days ("older than 5 days" → +5, "modified
/// in the last 3 days" → -3).
fn mtime_test(normalized_tokens: &[&str]) -> Option<String> {
    let numbers = word_entries("number");
    let days_at = role_position(normalized_tokens, "day")?;
    let count = number_value(normalized_tokens[days_at.checked_sub(1)?], &numbers)?;
    let older = role_position(normalized_tokens, "older").is_some();
    let sign = if older { "+" } else { "-" };
    Some(format!("{sign}{count}"))
}

/// "last/first N lines of FILE" → head/tail.
fn line_slice(normalized_tokens: &[&str], prompt: &str) -> Option<Composed> {
    let numbers = word_entries("number");
    let last_at = role_position(normalized_tokens, "last");
    let first_at = role_position(normalized_tokens, "first");
    let last = last_at.is_some();
    if !last && first_at.is_none() {
        return None;
    }
    let lines_at = role_position(normalized_tokens, "line")?;
    let count = number_value(normalized_tokens[lines_at.checked_sub(1)?], &numbers)?;
    let direction_at = last_at.or(first_at)?;
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
        command: format!("{tool} -n {count} {file}"),
        package: manual.package,
        url: manual.url,
        rows: vec![
            (
                echo(normalized_tokens, direction_at, lines_at),
                format!("{tool} -n {count}"),
            ),
            (file.clone(), file),
        ],
        explained: vec![("-n N".to_owned(), meaning)],
        pipe: None,
    })
}

/// "search for X" / "files containing X" → grep.
fn grep_search(normalized_tokens: &[&str], prompt: &str) -> Option<Composed> {
    let containment_at = role_position(normalized_tokens, "containment");
    let search_at = role_position(normalized_tokens, "search")
        .filter(|_| role_position(normalized_tokens, "search_object").is_some());
    let cue_at = containment_at.or(search_at)?;
    // The pattern: the raw word after the first seeded lead (case kept).
    let pattern = word_after_lead(prompt, "pattern_lead")?
        .trim_matches(|c: char| !c.is_alphanumeric() && c != '_' && c != '-')
        .to_owned();
    if pattern.is_empty() {
        return None;
    }
    let path = root_path(prompt);
    let mut flags = vec!["-r".to_owned()];
    if role_position(normalized_tokens, "ignore_case").is_some() {
        flags.push("-i".to_owned());
    }
    if role_position(normalized_tokens, "names_only").is_some() {
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
        command: format!("grep {flag_list} '{pattern}' {path}"),
        package: manual.package,
        url: manual.url,
        rows,
        explained,
        pipe: None,
    })
}

/// Escape the characters a basic regular expression (or, with `&`, a sed
/// replacement) reads specially, so the words are matched literally.
fn sed_literal(word: &str, special: &[char]) -> String {
    let mut out = String::new();
    for c in word.chars() {
        if special.contains(&c) {
            out.push('\\');
        }
        out.push(c);
    }
    out
}

/// "replace foo with bar in config.txt" → `sed -i 's/foo/bar/g' config.txt`:
/// a seeded `substitute` word, the word before the next `substitute_with`
/// word is the pattern, the word after it the replacement, and the first
/// dotted word after that the file. The words are escaped to match
/// literally; a word holding the `/` delimiter or a single quote is not
/// composed.
fn sed_substitution(prompt: &str) -> Option<Composed> {
    let raw: Vec<&str> = prompt.split_whitespace().collect();
    let bare = |word: &str| {
        word.trim_matches(|c: char| matches!(c, '\'' | '"' | '`' | ',' | '?' | '!'))
            .to_owned()
    };
    let lower: Vec<String> = raw.iter().map(|word| bare(word).to_lowercase()).collect();
    let substitute = role_words("substitute");
    let with = role_words("substitute_with");
    let cue_at = lower.iter().position(|word| substitute.contains(word))?;
    let with_at = cue_at
        + 2
        + lower
            .iter()
            .skip(cue_at + 2)
            .position(|word| with.contains(word))?;
    let pattern = bare(raw[with_at - 1]);
    let replacement = bare(raw.get(with_at + 1)?);
    let file = file_argument(&raw.get(with_at + 2..)?.join(" "))?;
    if pattern.is_empty()
        || replacement.is_empty()
        || (pattern.clone() + &replacement).contains(['/', '\''])
    {
        return None;
    }
    let manual = manual_pages()
        .into_iter()
        .find(|command| command.name == "sed")?;
    let expression = [
        "s/",
        &sed_literal(&pattern, &['.', '*', '[', ']', '^', '$', '\\']),
        "/",
        &sed_literal(&replacement, &['&', '\\']),
        "/g",
    ]
    .concat();
    let used = ["-i".to_owned(), "s/regexp/replacement/g".to_owned()];
    let explained = explained_flags(&manual, &used);
    Some(Composed {
        command: ["sed -i '", &expression, "' ", &file].concat(),
        package: manual.package,
        url: manual.url,
        rows: vec![
            (raw[cue_at..=with_at + 1].join(" "), expression),
            (file.clone(), file),
        ],
        explained,
        pipe: None,
    })
}

/// The find composition: path + -name + -size + -mtime + -type.
fn find_files(normalized_tokens: &[&str], prompt: &str) -> Option<Composed> {
    let pattern = name_pattern(prompt, normalized_tokens);
    let size = size_test(normalized_tokens);
    let mtime = mtime_test(normalized_tokens);
    let directories_at = role_position(normalized_tokens, "directory");
    if pattern.is_none() && size.is_none() && mtime.is_none() && directories_at.is_none() {
        return None;
    }
    let path = root_path(prompt);
    let mut parts = vec![format!("find {}", path)];
    let mut rows: Vec<(String, String)> = vec![(path.clone(), format!("find {path}"))];
    let mut used_flags: Vec<String> = Vec::new();
    if let Some(pattern) = &pattern {
        parts.push(format!("-name '{pattern}'"));
        rows.push((pattern.clone(), format!("-name '{pattern}'")));
        used_flags.push("-name pattern".to_owned());
    }
    if let Some((count, unit, span_from, span_to)) = &size {
        parts.push(format!("-size +{count}{unit}"));
        rows.push((
            echo(normalized_tokens, *span_from, *span_to),
            format!("-size +{count}{unit}"),
        ));
        used_flags.push("-size +N[kMG]".to_owned());
    }
    if let Some(mtime) = &mtime {
        parts.push(format!("-mtime {mtime}"));
        rows.push((mtime.clone(), format!("-mtime {mtime}")));
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
        pipe: None,
    })
}

/// The ls composition: flags for long/hidden/recursive listings.
fn ls_listing(normalized_tokens: &[&str], prompt: &str) -> Option<Composed> {
    let mut flags: Vec<String> = Vec::new();
    let mut rows: Vec<(String, String)> = Vec::new();
    if let Some(at) = role_position(normalized_tokens, "long") {
        flags.push("-l".to_owned());
        rows.push((echo(normalized_tokens, at, at), "-l".to_owned()));
    }
    if let Some(at) = role_position(normalized_tokens, "hidden")
        .or_else(|| role_position(normalized_tokens, "all"))
    {
        flags.push("-a".to_owned());
        rows.push((echo(normalized_tokens, at, at), "-a".to_owned()));
    }
    if let Some(at) = role_position(normalized_tokens, "recursive") {
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
        command: format!("ls {flag_list}{path}"),
        package: manual.package,
        url: manual.url,
        rows,
        explained,
        pipe: None,
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
    // An agent opt-in is the agent flow's to serve: it runs the command in
    // the sandbox, where this composer only prints one.
    if crate::solver_helpers::is_agent_opt_in(&prompt.to_lowercase()) {
        return None;
    }
    let normalized_tokens = tokens(normalized);
    // A complete substitution request is its own cue: it composes without
    // the action and file-context words, and nothing else is read then.
    let substitution = sed_substitution(prompt);
    let actions = cue_phrases(INTENT, "action");
    let has_action = normalized_tokens
        .iter()
        .any(|token| actions.iter().any(|action| action == token));
    if !has_action && substitution.is_none() {
        return None;
    }
    let file_context: Vec<String> = word_entries("file_context")
        .iter()
        .map(|entry| entry.find_child_value("word").to_owned())
        .collect();
    let has_context = normalized_tokens
        .iter()
        .any(|token| file_context.iter().any(|word| word == token));
    if !has_context && substitution.is_none() {
        return None;
    }
    // A requested function/program is program synthesis, not a shell command.
    if role_position(&normalized_tokens, "program_artifact").is_some() {
        return None;
    }
    // A more specific code-task intent cued? Step aside.
    if OTHER_INTENTS
        .iter()
        .any(|intent| any_cue_matches(intent, prompt, normalized))
    {
        return None;
    }
    // An inline text payload under a seeded text operation (data/seed/
    // operation-vocabulary.lino) is the text handler's, not a filesystem
    // command. The raw prompt keeps the quotes the payload is read from.
    if super::names_text_operation(&prompt.to_lowercase()) {
        return None;
    }
    log.append(
        "shell_command_compose:request",
        "action + file context".to_owned(),
    );

    let composition = substitution
        .or_else(|| line_slice(&normalized_tokens, prompt))
        .or_else(|| grep_search(&normalized_tokens, prompt))
        .or_else(|| counted(&normalized_tokens, find_files(&normalized_tokens, prompt)))
        .or_else(|| ls_listing(&normalized_tokens, prompt));

    let (body, confidence) = if let Some(composed) = composition {
        log.append("shell_command_compose:command", composed.command.clone());
        let flags = flag_lines(&composed.explained);
        let (intent, pipe_tool, pipe_package, pipe_flags, pipe_manual) =
            composed.pipe.as_ref().map_or_else(
                || ("shell_command_composed", "", "", String::new(), ""),
                |pipe| {
                    (
                        "shell_command_piped",
                        pipe.tool.as_str(),
                        pipe.manual.package.as_str(),
                        flag_lines(&pipe.explained),
                        pipe.manual.url.as_str(),
                    )
                },
            );
        (
            template(
                intent,
                &[
                    ("command", &composed.command),
                    ("package", &composed.package),
                    ("flags", &flags),
                    ("manual", &composed.url),
                    ("pipe_tool", pipe_tool),
                    ("pipe_package", pipe_package),
                    ("pipe_flags", &pipe_flags),
                    ("pipe_manual", pipe_manual),
                    ("derivation", &mapping_rows(&composed.rows)),
                ],
            ),
            0.7,
        )
    } else {
        log.append(
            "shell_command_compose:refusal",
            "no manual-page entry covers the request".to_owned(),
        );
        (template("shell_compose_refusal", &[]), 0.4)
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
