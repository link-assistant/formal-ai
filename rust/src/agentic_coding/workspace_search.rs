//! Workspace content search (PR #1188 T90, gap G12).
//!
//! "Find all usages of add", "Where is add used in this project?" and "Grep
//! for add" ask for the places a name or a literal appears inside the
//! workspace's files. The seeded `workspace_content_search_form` slot forms
//! (every registered language) read the searched pattern out of the request;
//! the arm greps it -- the client's grep tool when advertised, else `grep -rn`
//! through the shell -- and answers with the file:line hits, or a seeded
//! not-found naming the pattern and the scope. It runs ahead of the file-name
//! locate arm and of web search. Twin of `js/agentic/workspace_search.mjs`.

use std::sync::OnceLock;

use serde_json::json;

use super::planner::{AgenticPlan, Capability, plan_one, tool_for};
use crate::protocol::{ChatMessage, client_working_directory};
use crate::seed;
use crate::seed::SHELL_INTENTS_LINO;
use crate::seed::parser::parse_lino;

const ROLE_SEARCH_FORM: &str = "workspace_content_search_form";
const ROLE_SEARCH_NOISE: &str = "workspace_content_search_noise";
const ROLE_WEB_MEDIUM: &str = "web_medium";
const EDIT_ROLES: [&str; 3] = [
    "file_edit_action_cue",
    "coding_text_remove_action",
    "coding_identifier_rename_action",
];
const CURRENT_SCOPE: &str = ".";
const MODE_SLOT: &str = concat!("{", "mode", "}");
const PATTERN_SLOT: &str = concat!("{", "pattern", "}");
const SCOPE_SLOT: &str = concat!("{", "scope", "}");
const NOISE_SKIP_LIMIT: usize = 3;
const PATTERN_CHAR_LIMIT: usize = 128;
const NO_FILES_FOUND: &str = "No files found";
const QUOTE_PAIRS: [(char, char); 6] = [
    ('"', '"'),
    ('\'', '\''),
    ('`', '`'),
    ('«', '»'),
    ('“', '”'),
    ('‘', '’'),
];
const WRAPPERS: &[char] = &[
    '.', ',', ';', ':', '!', '?', '¿', '¡', '(', ')', '[', ']', '{', '}', '"', '\'', '`', '«', '»',
    '“', '”', '‘', '’', '。', '，', '？', '！', '：', '；',
];

/// A workspace content search read out of a request.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ContentSearch {
    /// The searched identifier or literal.
    pub pattern: String,
    /// The folder or file searched, `.` when the request names none.
    pub scope: String,
    /// Whether the pattern is one identifier (searched as a whole word).
    pub identifier: bool,
}

/// One token read next to a slot: its text, its far edge and whether it was quoted.
struct Token {
    text: String,
    edge: usize,
    quoted: bool,
}

fn is_cjk(character: char) -> bool {
    crate::coding::contains_cjk(character.encode_utf8(&mut [0; 4]))
}

fn is_wrapper(character: char) -> bool {
    WRAPPERS.contains(&character)
}

/// One char per char: a lowercase that keeps every offset.
pub(super) fn lower_chars(chars: &[char]) -> Vec<char> {
    chars
        .iter()
        .map(|character| {
            let mut lower = character.to_lowercase();
            match (lower.next(), lower.next()) {
                (Some(single), None) => single,
                _ => *character,
            }
        })
        .collect()
}

/// Does `needle` start at `at` in `haystack`, on word boundaries?
fn starts_at(haystack: &[char], needle: &[char], at: usize) -> bool {
    let end = at + needle.len();
    if end > haystack.len() || haystack[at..end] != *needle {
        return false;
    }
    let (Some(&first), Some(&last)) = (needle.first(), needle.last()) else {
        return false;
    };
    let left_open = is_cjk(first) || at == 0 || !haystack[at - 1].is_alphanumeric();
    let right_open = is_cjk(last) || end == haystack.len() || !haystack[end].is_alphanumeric();
    left_open && right_open
}

/// The token that starts at `at` (after spaces); its edge is where it ends.
///
/// A token stops at whitespace and at CJK text; a quoted literal runs to its
/// closer.
fn token_after(chars: &[char], at: usize) -> Option<Token> {
    let mut start = at;
    while start < chars.len() && chars[start].is_whitespace() {
        start += 1;
    }
    if start >= chars.len() {
        return None;
    }
    if let Some(&(_, close)) = QUOTE_PAIRS.iter().find(|(open, _)| *open == chars[start])
        && let Some(offset) = chars[start + 1..].iter().position(|&c| c == close)
        && offset > 0
    {
        let closer = start + 1 + offset;
        return Some(Token {
            text: chars[start + 1..closer].iter().collect(),
            edge: closer + 1,
            quoted: true,
        });
    }
    let mut end = start;
    while end < chars.len() && !chars[end].is_whitespace() && !is_cjk(chars[end]) {
        end += 1;
    }
    (end > start).then(|| Token {
        text: chars[start..end].iter().collect(),
        edge: end,
        quoted: false,
    })
}

/// The token that ends at `at` (before spaces); its edge is where it starts.
fn token_before(chars: &[char], at: usize) -> Option<Token> {
    let mut end = at;
    while end > 0 && chars[end - 1].is_whitespace() {
        end -= 1;
    }
    if end == 0 {
        return None;
    }
    if let Some(&(open, _)) = QUOTE_PAIRS
        .iter()
        .find(|(_, close)| *close == chars[end - 1])
        && end >= 2
        && let Some(opener) = chars[..=end - 2].iter().rposition(|&c| c == open)
        && opener + 2 < end
    {
        return Some(Token {
            text: chars[opener + 1..end - 1].iter().collect(),
            edge: opener,
            quoted: true,
        });
    }
    let mut start = end;
    while start > 0 && !chars[start - 1].is_whitespace() && !is_cjk(chars[start - 1]) {
        start -= 1;
    }
    (start < end).then(|| Token {
        text: chars[start..end].iter().collect(),
        edge: start,
        quoted: false,
    })
}

/// A token read as a pattern: wrappers peeled, wordless tokens refused.
fn pattern_of(token: &Token) -> Option<String> {
    let text = if token.quoted {
        token.text.as_str()
    } else {
        token.text.trim_matches(is_wrapper)
    };
    let valid = !text.is_empty()
        && text.chars().any(char::is_alphanumeric)
        && text.chars().count() <= PATTERN_CHAR_LIMIT;
    valid.then(|| text.to_owned())
}

/// Is a token a seeded noise word standing in the slot?
fn is_noise(token: &Token) -> bool {
    if token.quoted {
        return false;
    }
    let word = crate::engine::normalize_prompt(token.text.trim_matches(is_wrapper));
    seed::lexicon()
        .words_for_role(ROLE_SEARCH_NOISE)
        .contains(&word)
}

/// The pattern next to `at` (after it, or before it), skipping noise words.
fn pattern_beside(chars: &[char], at: usize, after: bool) -> Option<(Option<String>, usize)> {
    let mut cursor = at;
    for _ in 0..=NOISE_SKIP_LIMIT {
        let token = if after {
            token_after(chars, cursor)?
        } else {
            token_before(chars, cursor)?
        };
        if !is_noise(&token) {
            return Some((pattern_of(&token), token.edge));
        }
        cursor = token.edge;
    }
    None
}

/// Every offset `needle` starts at in `haystack`, on word boundaries.
fn offsets_of(haystack: &[char], needle: &[char]) -> Vec<usize> {
    (0..haystack.len())
        .filter(|&at| starts_at(haystack, needle, at))
        .collect()
}

/// The pattern one slot form reads out of the request, with the span it covers.
fn form_match(
    chars: &[char],
    lower: &[char],
    form: &seed::WordForm,
) -> Option<(String, (usize, usize))> {
    let before: Vec<char> = form.before_slot().trim().to_lowercase().chars().collect();
    let after: Vec<char> = form.after_slot().trim().to_lowercase().chars().collect();
    if !before.is_empty() {
        for at in offsets_of(lower, &before) {
            let Some((Some(pattern), end)) = pattern_beside(chars, at + before.len(), true) else {
                continue;
            };
            if after.is_empty() {
                return Some((pattern, (at, end)));
            }
            let mut next = end;
            while next < lower.len() && lower[next].is_whitespace() {
                next += 1;
            }
            if starts_at(lower, &after, next) {
                return Some((pattern, (at, next + after.len())));
            }
        }
        return None;
    }
    if after.is_empty() {
        return None;
    }
    offsets_of(lower, &after).into_iter().find_map(|at| {
        let (pattern, start) = pattern_beside(chars, at, false)?;
        pattern.map(|pattern| (pattern, (start, at + after.len())))
    })
}

/// A path-shaped token outside the matched form: the searched scope.
fn scope_of(chars: &[char], span: (usize, usize)) -> String {
    let outside: String = chars[..span.0]
        .iter()
        .chain(std::iter::once(&' '))
        .chain(chars[span.1..].iter())
        .collect();
    outside
        .split_whitespace()
        .map(|raw| raw.trim_matches(is_wrapper))
        .filter(|word| {
            !word.contains(':')
                && (word.contains('/')
                    || (super::write_request::looks_like_file_path(word)
                        && super::write_request::safe_relative_path(word)))
        })
        .map(|word| word.trim_end_matches('/'))
        .find(|scope| !scope.is_empty())
        .unwrap_or(CURRENT_SCOPE)
        .to_owned()
}

/// Is `pattern` one identifier?
fn is_identifier(pattern: &str) -> bool {
    let mut chars = pattern.chars();
    chars
        .next()
        .is_some_and(|first| first.is_alphabetic() || first == '_' || first == '$')
        && chars.all(|c| c.is_alphanumeric() || c == '_' || c == '$')
}

/// The workspace content search a request asks for, if it asks for one.
#[must_use]
pub fn content_search_for(task: &str) -> Option<ContentSearch> {
    let padded = format!(" {} ", crate::engine::normalize_prompt(task));
    if seed::lexicon().mentions_role_raw(ROLE_WEB_MEDIUM, &padded) {
        return None;
    }
    let chars: Vec<char> = task.chars().collect();
    let lower = lower_chars(&chars);
    let mut best: Option<(usize, String, (usize, usize))> = None;
    for form in seed::lexicon().role_word_forms(ROLE_SEARCH_FORM) {
        let Some((pattern, span)) = form_match(&chars, &lower, form) else {
            continue;
        };
        let weight = form.text.chars().count();
        if best
            .as_ref()
            .is_none_or(|(heaviest, _, _)| weight > *heaviest)
        {
            best = Some((weight, pattern, span));
        }
    }
    let (_, pattern, span) = best?;
    // A request that changes what it finds (rename, replace, remove the uses
    // of a name) is an edit, not a search: the pattern itself may be such a word.
    let rest = crate::engine::normalize_prompt(&task.replace(&pattern, " "));
    if EDIT_ROLES
        .iter()
        .any(|role| seed::lexicon().mentions_role(role, &rest))
    {
        return None;
    }
    Some(ContentSearch {
        scope: scope_of(&chars, span),
        identifier: is_identifier(&pattern),
        pattern,
    })
}

/// A regular expression matching `literal` exactly.
fn escaped_regex(literal: &str) -> String {
    let mut out = String::new();
    for character in literal.chars() {
        if "\\^$.*+?()[]{}|/".contains(character) {
            out.push('\\');
        }
        out.push(character);
    }
    out
}

/// The grep tool arguments for a search.
fn grep_arguments(search: &ContentSearch) -> String {
    let escaped = escaped_regex(&search.pattern);
    let pattern = if search.identifier {
        format!("\\b{escaped}\\b")
    } else {
        escaped
    };
    json!({ "path": search.scope, "pattern": pattern }).to_string()
}

/// The seeded `content_search` group of `data/seed/shell-intents.lino`:
/// the command template and the identifier and literal modes.
fn content_search_seed() -> &'static (String, String, String) {
    static SEED: OnceLock<(String, String, String)> = OnceLock::new();
    SEED.get_or_init(|| {
        let tree = parse_lino(SHELL_INTENTS_LINO);
        tree.children
            .first()
            .and_then(|root| {
                root.children
                    .iter()
                    .find(|group| group.name == "content_search")
            })
            .map(|group| {
                (
                    group.find_child_value("command").to_owned(),
                    group.find_child_value("identifier_mode").to_owned(),
                    group.find_child_value("literal_mode").to_owned(),
                )
            })
            .unwrap_or_default()
    })
}

/// The seeded shell command for a search (whole-word for an identifier,
/// fixed text for a literal).
fn shell_command(search: &ContentSearch) -> String {
    let (command, identifier_mode, literal_mode) = content_search_seed();
    let mode = if search.identifier {
        identifier_mode
    } else {
        literal_mode
    };
    command
        .replace(MODE_SLOT, mode)
        .replace(
            PATTERN_SLOT,
            &super::git_commit::shell_quote(&search.pattern),
        )
        .replace(SCOPE_SLOT, &super::git_commit::shell_quote(&search.scope))
}

/// The search results of the current turn: each grep or run result.
fn turn_results(messages: &[ChatMessage]) -> Vec<(Capability, String)> {
    let current_turn = messages
        .iter()
        .rposition(|message| message.role.eq_ignore_ascii_case("user"))
        .map_or(0, |index| index + 1);
    messages
        .iter()
        .enumerate()
        .skip(current_turn)
        .filter(|(_, message)| message.role.eq_ignore_ascii_case("tool"))
        .filter_map(|(index, message)| {
            super::progress::result_capability(messages, index)
                .filter(|capability| matches!(capability, Capability::Grep | Capability::Run))
                .map(|capability| (capability, message.content.plain_text()))
        })
        .collect()
}

/// A hit path relative to the working directory.
fn relative_path(path: &str, root: Option<&str>) -> String {
    let mut relative = path;
    if let Some(root) = root
        && let Some(rest) = relative.strip_prefix(root)
        && let Some(rest) = rest.strip_prefix('/')
    {
        relative = rest;
    }
    while let Some(rest) = relative.strip_prefix("./") {
        relative = rest;
    }
    relative.to_owned()
}

/// `digits:text` after a hit's line label.
fn numbered_text(rest: &str) -> Option<(String, String)> {
    let (number, text) = rest.split_once(':')?;
    (!number.is_empty() && number.bytes().all(|byte| byte.is_ascii_digit()))
        .then(|| (number.to_owned(), text.trim().to_owned()))
}

/// `path:digits:text` of one `grep -rn` line.
fn shell_hit(line: &str) -> Option<(String, String, String)> {
    line.char_indices()
        .filter(|&(at, character)| at > 0 && character == ':')
        .find_map(|(at, _)| {
            numbered_text(&line[at + 1..])
                .map(|(number, text)| (line[..at].to_owned(), number, text))
        })
}

/// The `(path, line, text)` hits of a grep tool result (grouped by file) or
/// `grep -rn` output.
fn hits_of(
    payload: &str,
    capability: Capability,
    root: Option<&str>,
) -> Vec<(String, String, String)> {
    let mut hits = Vec::new();
    let mut file: Option<String> = None;
    for line in payload.split('\n') {
        if capability == Capability::Grep {
            let quoted = line.trim_start();
            if quoted.len() != line.len() {
                let numbered = quoted
                    .split_once(' ')
                    .and_then(|(_, rest)| numbered_text(rest));
                if let (Some((number, text)), Some(file)) = (numbered, file.as_ref()) {
                    hits.push((file.clone(), number, text));
                }
            } else if let Some(path) = line.strip_suffix(':') {
                file = Some(relative_path(path, root));
            }
            continue;
        }
        if let Some((path, number, text)) = shell_hit(line) {
            hits.push((relative_path(&path, root), number, text));
        }
    }
    hits
}

/// The seeded answer for the observed hits.
fn answer_for(search: &ContentSearch, hits: &[(String, String, String)], language: &str) -> String {
    let values = [
        ("pattern", search.pattern.as_str()),
        ("scope", search.scope.as_str()),
    ];
    if hits.is_empty() {
        return seed::render_response("workspace_search_none", language, &values)
            .or_else(|| seed::render_response("workspace_search_none", "en", &values))
            .unwrap_or_default();
    }
    let listed = hits
        .iter()
        .map(|(path, line, text)| format!("- `{path}:{line}`: {text}"))
        .collect::<Vec<_>>()
        .join("\n");
    let count = hits.len().to_string();
    let all = [
        values[0],
        values[1],
        ("count", count.as_str()),
        ("hits", listed.as_str()),
    ];
    seed::render_response("workspace_search_hits", language, &all)
        .or_else(|| seed::render_response("workspace_search_hits", "en", &all))
        .unwrap_or_default()
}

/// The `workspace_search` arm: grep the named pattern, then answer from the hits.
pub(super) fn plan_workspace_search_step(
    task: &str,
    messages: &[ChatMessage],
    tool_names: &[&str],
) -> Option<AgenticPlan> {
    let search = content_search_for(task)?;
    if let Some((capability, raw)) = turn_results(messages).pop() {
        let language = crate::language::detect(task).slug();
        if raw.trim() == NO_FILES_FOUND {
            return Some(AgenticPlan::Final(answer_for(&search, &[], language)));
        }
        let Some(payload) = super::tool_result::normalized_payload(&raw) else {
            return Some(AgenticPlan::Final(super::tool_result::render(
                "grep", &raw, task,
            )));
        };
        let root = client_working_directory(messages);
        let hits = hits_of(&payload, capability, root.as_deref());
        return Some(AgenticPlan::Final(answer_for(&search, &hits, language)));
    }
    if let Some(grep) = tool_for(tool_names, Capability::Grep) {
        return Some(plan_one(grep, grep_arguments(&search)));
    }
    let shell = tool_for(tool_names, Capability::Run)?;
    Some(plan_one(
        shell,
        json!({ "command": shell_command(&search) }).to_string(),
    ))
}
