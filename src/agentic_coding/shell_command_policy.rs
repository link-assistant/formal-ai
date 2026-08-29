//! Sentence scoping and command-policy classification for shell routing.
//!
//! A prompt that *mentions* a command is not a prompt that *asks for* one.
//! Hive Mind wraps every Codex request in caller policy — *"When running sudo
//! commands, run them in the background."* — and matching a run verb against a
//! command token message-wide planned a bare `sudo` for every such run
//! (issue #907). So routing reads one sentence at a time, and a sentence that
//! only governs commands never licenses the token it names.
//!
//! This is the classification half of [`super::shell_command`]; the routing
//! half calls [`sentence_spans`], [`states_a_command_policy`] and
//! [`named_shell_command_in_sentence`].

use crate::seed::{self, TerminalCommandVocabulary};

/// One sentence of a prompt: its trimmed text, and the byte range it occupies.
///
/// The ranges tile the prompt, so dropping a sentence and concatenating the rest
/// reproduces the original wording verbatim — which is what a route needs when
/// it consumes one clause of a request and hands the remainder on (issue #1066).
pub(super) struct Sentence<'a> {
    /// The sentence with its surrounding whitespace and terminator removed.
    pub(super) text: &'a str,
    /// Where the sentence sits in the prompt, terminator included.
    pub(super) span: std::ops::Range<usize>,
}
/// The prompt split into sentences.
///
/// Splitting keeps the original case and spacing, because a recovered command
/// carries its arguments through verbatim — `run cat notes.txt` has to survive
/// as one sentence, so a dot inside a token ends nothing, exactly as in
/// [`sentences_with_mood`].
pub(super) fn sentences(prompt: &str) -> Vec<Sentence<'_>> {
    let mut sentences = Vec::new();
    let mut span_start = 0;
    let mut start = 0;
    for (index, character) in prompt.char_indices() {
        if !matches!(
            character,
            '.' | '!' | '?' | ';' | '\n' | '。' | '！' | '？' | '；'
        ) {
            continue;
        }
        if character == '.'
            && prompt[index + character.len_utf8()..]
                .chars()
                .next()
                .is_some_and(char::is_alphanumeric)
        {
            continue;
        }
        let text = prompt[start..index].trim();
        let end = index + character.len_utf8();
        if !text.is_empty() {
            sentences.push(Sentence {
                text,
                span: span_start..end,
            });
            span_start = end;
        }
        start = end;
    }
    let tail = prompt[start..].trim();
    if !tail.is_empty() {
        sentences.push(Sentence {
            text: tail,
            span: span_start..prompt.len(),
        });
    }
    sentences
}
/// The prompt split into sentences, each as a slice of the original text.
pub(super) fn sentence_spans(prompt: &str) -> Vec<&str> {
    sentences(prompt)
        .into_iter()
        .map(|sentence| sentence.text)
        .collect()
}
/// Whether `sentence` states a *rule about* running commands rather than asking
/// for one to run.
///
/// The tell is a seed-declared conditional lead opening the clause: Hive Mind's
/// *"When running sudo commands, run them in the background"* tells the agent how
/// to treat a class of commands it may later choose to run. Nothing in it names a
/// command to run now, so no token inside it may become one.
pub(super) fn states_a_command_policy(sentence: &str) -> bool {
    let lower = sentence.to_lowercase();
    // A lead opens the rule, or qualifies it from the middle: *"Run commands
    // with sudo only when necessary"* is as much a rule as *"When running sudo
    // commands, …"*, and both name a class rather than an instance. The test
    // itself lives on the vocabulary, because the same lead marks the same thing
    // wherever it is read — web-search routing asks the same question of the
    // topic a search marker introduces (issue #1066).
    let carries_lead = seed::caller_context_vocabulary().carries_policy_lead(&lower);
    // A conditional opener alone does not make a sentence policy — plenty of
    // real requests carry one, and treating them as policy answers nothing at
    // all, which is strictly worse than answering imperfectly.
    //
    // The separator is *where* a command is named. A conditional clause runs to
    // its comma; whatever follows is the order the condition qualifies. So
    // *"If the build fails, run cargo test."* orders `cargo test` after the
    // condition and routes, while *"When running sudo commands, run them in the
    // background."* names only a pronoun there — it is telling the agent how to
    // treat a class of commands, not asking for one. A rule with no comma at all
    // (*"Never run rm outside the workspace."*) governs throughout and orders
    // nothing.
    carries_lead
        && lower
            .split_once(',')
            .is_none_or(|(_, ordered)| !orders_a_named_command(ordered))
}

/// Whether `clause` orders a concrete command: a run verb immediately followed
/// by a known shell token. This is the difference between asking for a command
/// and talking about commands.
fn orders_a_named_command(clause: &str) -> bool {
    let vocab = seed::terminal_command_vocabulary();
    let words: Vec<&str> = clause.split_whitespace().collect();
    words.iter().enumerate().any(|(index, word)| {
        let is_verb = vocab
            .run_verbs
            .iter()
            .any(|verb| verb == &normalize_command_word(word));
        is_verb
            && words.get(index + 1).is_some_and(|next| {
                let token = normalize_command_word(next);
                vocab.shell_tokens.iter().any(|known| known == &token)
            })
    })
}

/// Whether every sentence of `prompt` is caller policy, so nothing in it asks
/// for a command to run now.
///
/// The check is whole-prompt because `prefixed_shell_command` claims a message
/// that merely *starts* with `run`/`execute` before any sentence-level rule is
/// consulted; a policy filter applied only to `named_shell_command` would leave
/// that branch open. Requiring *every* sentence to be policy keeps a genuine
/// request that arrives alongside one — *"Never run rm outside the workspace.
/// Now run git status."* — reaching the router through its second sentence.
///
/// A conditional request keeps working, because [`states_a_command_policy`]
/// asks whether the sentence names a command rather than only how it opens:
/// *"If the build fails, run cargo test."* names `cargo` and routes, while the
/// reported *"When running sudo commands, run them in the background."* names
/// only a pronoun and does not.
pub(super) fn governs_commands_rather_than_requesting_one(prompt: &str) -> bool {
    let sentences = sentence_spans(prompt);
    !sentences.is_empty() && sentences.iter().all(|sentence| states_a_command_policy(sentence))
}

pub(super) fn named_shell_command_in_sentence(
    prompt: &str,
    vocab: &TerminalCommandVocabulary,
) -> Option<String> {
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
    if (has_verb || has_phrase)
        && let Some(word) = words.iter().find(|w| is_shell_token(w)) {
            return Some(normalize_command_word(word));
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
pub(super) fn normalize_command_word(word: &str) -> String {
    word.trim_matches('`')
        .chars()
        .take_while(|c| c.is_ascii_alphanumeric() || *c == '_' || *c == '-')
        .collect::<String>()
        .to_ascii_lowercase()
}

/// Whether a word is natural-language prose rather than a command argument. Used to
/// stop argument collection at the boundary between a command and the sentence around
/// it (e.g. `git status` stops before `in the current directory`).
pub(super) fn is_prose_word(word: &str) -> bool {
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
        // Quantifiers a described action opens with. "Execute nothing without
        // asking" and "Run all tests in the background" are sentences about
        // commands, not commands.
        "nothing",
        "anything",
        "everything",
        "something",
        "all",
        "any",
        "every",
        "each",
        "no",
    ];
    let normalized = word
        .trim_matches(|c: char| !c.is_ascii_alphanumeric())
        .to_ascii_lowercase();
    PROSE_WORDS.contains(&normalized.as_str())
}
