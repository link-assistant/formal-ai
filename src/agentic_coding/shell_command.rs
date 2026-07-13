//! Resolve a user turn into the concrete shell command the agentic loop should run.
//!
//! Split out of [`super::planner`] (issue #676): the agentic planner used to know
//! only the hardcoded `ls`, so `execute pwd` and every other seed shell token fell
//! through to the *unknown* fallback. The two data-driven strategies here — a named
//! command backed by `data/seed/terminal-commands.lino`, and a natural-language
//! directory-listing request — make the whole seed vocabulary reachable. Keeping the
//! matching prose (`PROSE_WORDS`, listing phrases) in one module also keeps the
//! planner file under the repository line budget.

use crate::seed::{self, TerminalCommandVocabulary};

/// Resolve a user turn into the concrete shell command the agentic loop should run.
///
/// Two data-driven strategies, in order of specificity:
///
/// 1. **Named command** ([`named_shell_command`]): when the prompt pairs a run/execute
///    verb (or terminal/shell phrase) with a known shell token from
///    `data/seed/terminal-commands.lino` — e.g. *"execute pwd"*, *"run git status"*,
///    *«запусти ls»* — emit that command (with its flag/path/sub-command arguments).
///    This is what makes `pwd` (issue #676) and every other seed token reachable, not
///    just the hardcoded `ls` the fallback used to know.
///
/// 2. **Natural-language directory listing** ([`asks_for_directory_listing`]): when the
///    prompt asks, in prose, to see the files in the current place — e.g. *"give me a
///    list of files in current folder"*, *"what files are here?"* — resolve to `ls`.
///
/// The vocabulary lives in seed data, so a maintainer retunes coverage by editing a
/// `.lino` file rather than this function, upholding the project rule against hardcoded
/// natural language in the solver.
pub(super) fn shell_command_for_task(prompt: &str) -> Option<String> {
    let vocab = seed::terminal_command_vocabulary();
    named_shell_command(prompt, &vocab)
        .or_else(|| asks_for_directory_listing(prompt).then(|| String::from("ls")))
}

/// Extract an explicit shell command named in the prompt, backed by the seed
/// [`TerminalCommandVocabulary`].
///
/// A prompt names a command when it carries *run context* — a run/execute verb, a
/// Chinese run verb, or a terminal/shell phrase — together with a recognized shell
/// token (`pwd`, `git`, `cargo`, …). Two shapes are handled:
///
/// * **Verb-adjacent** (`execute pwd`, `run git status`): the shell token immediately
///   follows a run verb, so the token *and its trailing arguments* are the command —
///   arguments run until a natural-language word ([`is_prose_word`]) ends them, so
///   `run git status` → `git status` but `run ls then stop` → `ls`.
/// * **Mentioned** (`Run the ls command to list files`): the token appears with run
///   context but is not directly after the verb, so only the single token is emitted
///   (`ls`) — the surrounding words are prose describing the request, not arguments.
fn named_shell_command(prompt: &str, vocab: &TerminalCommandVocabulary) -> Option<String> {
    let lower = prompt.to_ascii_lowercase();
    let has_phrase = vocab.terminal_phrases.iter().any(|p| lower.contains(p));
    let has_cjk_verb = vocab.cjk_run_verbs.iter().any(|v| lower.contains(v));

    // Word tokens of the original prompt, preserving case so command arguments
    // (paths, flags, filenames) survive intact.
    let words: Vec<&str> = prompt
        .split(|c: char| c.is_whitespace())
        .filter(|w| !w.is_empty())
        .collect();

    let is_run_verb = |word: &str| {
        let normalized = normalize_command_word(word);
        vocab.run_verbs.iter().any(|v| v == &normalized)
    };
    let is_shell_token = |word: &str| {
        let normalized = normalize_command_word(word);
        !normalized.is_empty() && vocab.shell_tokens.iter().any(|t| t == &normalized)
    };
    let has_verb = words.iter().any(|w| is_run_verb(w)) || has_cjk_verb;

    // Shape 1: a shell token directly after a run verb — the command plus its arguments.
    for (index, word) in words.iter().enumerate() {
        if index == 0 || !is_shell_token(word) {
            continue;
        }
        if is_run_verb(words[index - 1]) {
            return Some(collect_command(&words[index..]));
        }
    }

    // Shape 2: a shell token mentioned anywhere, given run context — the token alone.
    if has_verb || has_phrase {
        if let Some(word) = words.iter().find(|w| is_shell_token(w)) {
            return Some(normalize_command_word(word));
        }
    }

    None
}

/// Assemble a command from a token slice that starts at the command word: keep the
/// command and every following argument until a natural-language word ends it.
fn collect_command(words: &[&str]) -> String {
    let mut parts = vec![normalize_command_word(words[0])];
    for word in &words[1..] {
        if is_prose_word(word) {
            break;
        }
        let trimmed = word.trim_matches(|c: char| c == '`' || c == ',' || c == '.');
        if trimmed.is_empty() {
            break;
        }
        parts.push(trimmed.to_owned());
    }
    parts.join(" ")
}

/// Normalize a raw prompt word to a bare command token: lowercase, keeping only the
/// leading run of command-name characters (so ``` `pwd` ```, `pwd.` and `pwd,` all
/// normalize to `pwd`).
fn normalize_command_word(word: &str) -> String {
    word.trim_matches('`')
        .chars()
        .take_while(|c| c.is_ascii_alphanumeric() || *c == '_' || *c == '-')
        .collect::<String>()
        .to_ascii_lowercase()
}

/// Whether a word is natural-language prose rather than a command argument. Used to
/// stop argument collection at the boundary between a command and the sentence around
/// it (e.g. `git status` stops before `in the current directory`).
fn is_prose_word(word: &str) -> bool {
    const PROSE_WORDS: &[&str] = &[
        "command",
        "commands",
        "to",
        "in",
        "into",
        "on",
        "the",
        "a",
        "an",
        "and",
        "then",
        "please",
        "for",
        "of",
        "that",
        "which",
        "so",
        "this",
        "these",
        "those",
        "here",
        "there",
        "me",
        "us",
        "you",
        "it",
        "from",
        "at",
        "with",
        "will",
        "would",
        "can",
        "could",
        "should",
        "using",
        "via",
        "inside",
        "within",
        "output",
        "result",
        "results",
        "contents",
        "content",
        "directory",
        "folder",
        "folders",
        "file",
        "files",
        "currently",
        "again",
        "also",
        "just",
        "now",
    ];
    let normalized = word
        .trim_matches(|c: char| !c.is_ascii_alphanumeric())
        .to_ascii_lowercase();
    PROSE_WORDS.contains(&normalized.as_str())
}

/// Whether `prompt` asks, in natural language, to list the files in the current place.
///
/// Resolves prose directory-listing requests to `ls`: a listing phrase (or a
/// which-files question) scoped to the current/working/this directory or folder.
fn asks_for_directory_listing(prompt: &str) -> bool {
    let lower = prompt.to_ascii_lowercase();

    let asks_for_listing = contains_any(
        &lower,
        &[
            "list files",
            "list the files",
            "list all files",
            "list local files",
            "list of files",
            "list of all files",
            "list directory",
            "list the directory",
            "list the contents",
            "listing of files",
            "directory listing",
            "directory contents",
            "folder contents",
            "contents of this directory",
            "contents of the current directory",
            "contents of this folder",
            "contents of the current folder",
            "show files",
            "show the files",
            "show me the files",
            "see the files",
        ],
    );
    let asks_which_files = contains_any(
        &lower,
        &[
            "what files",
            "which files",
            "files are in",
            "files in the",
            "files in this",
            "files in current",
            "files in the current",
            "files exist",
            "files are here",
            "files are there",
        ],
    );
    let local_scope = contains_any(
        &lower,
        &[
            "here",
            "current directory",
            "working directory",
            "current working directory",
            "this directory",
            "the directory",
            "current folder",
            "this folder",
            "the folder",
            "local files",
        ],
    );

    (asks_for_listing || asks_which_files) && local_scope
}

fn contains_any(text: &str, phrases: &[&str]) -> bool {
    phrases.iter().any(|phrase| text.contains(phrase))
}
