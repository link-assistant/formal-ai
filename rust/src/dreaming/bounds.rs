//! Bounds on what dreaming may learn as a standing requirement (issue #1559):
//! `js/server/standing-requirement-bounds.mjs`.
//!
//! A learned rule is one statement a user made, never a whole message and never
//! a system or developer instruction a client delivered as the first user
//! message. The rule is bounded in length, may not name one task (a repository,
//! branch, issue or pull-request number, working-directory path), and its topic
//! is the subject of the statement — a pronoun or function word of any supported
//! language is never a topic. Every list and the length limit are data in
//! `data/meta/dreaming-lexicon.lino`.

use super::cues::requirement_cues;
use super::lexicon::lexicon;

/// The rule length limit used when the lexicon configures none.
pub const DEFAULT_RULE_MAX_CHARS: usize = 240;

const SENTENCE_END: [char; 3] = ['.', '!', '?'];
const HARD_SENTENCE_END: [char; 5] = ['\n', '。', '！', '？', '।'];
const LEADING_WRAPPERS: [char; 9] = ['(', '[', '{', '<', '"', '\'', '`', '*', '_'];
const TRAILING_WRAPPERS: [char; 15] = [
    ')', ']', '}', '>', '"', '\'', '`', ',', ';', ':', '!', '?', '.', '*', '_',
];

fn is_digits(text: &str) -> bool {
    !text.is_empty() && text.chars().all(|character| character.is_ascii_digit())
}

/// The configured rule length limit in characters.
#[must_use]
pub fn rule_max_chars() -> usize {
    lexicon()
        .rule_max_chars
        .last()
        .map(|raw| raw.trim())
        .filter(|raw| is_digits(raw))
        .and_then(|raw| raw.parse::<usize>().ok())
        .filter(|limit| *limit > 0)
        .unwrap_or(DEFAULT_RULE_MAX_CHARS)
}

fn starts_with_opener(lowered: &str, opener: &str) -> bool {
    if opener.is_empty() || !lowered.starts_with(opener) {
        return false;
    }
    // A CJK opener carries no word boundary; a spaced one must end on one.
    if !opener.chars().any(char::is_whitespace) {
        return true;
    }
    lowered[opener.len()..]
        .chars()
        .next()
        .is_none_or(|next| !next.is_alphanumeric())
}

/// The text opens the way a system or developer instruction does ("You are an
/// AI issue solver…"), so it configures one session and is never a learned
/// requirement.
#[must_use]
pub fn is_session_instruction(content: &str) -> bool {
    let lowered = content.trim().to_lowercase();
    lexicon()
        .instruction_openers
        .iter()
        .any(|opener| starts_with_opener(&lowered, opener))
}

/// Split `content` into sentences: always at line breaks and CJK/Devanagari
/// terminators, at `.`, `!`, `?` when whitespace or the end follows.
#[must_use]
pub fn statement_sentences(content: &str) -> Vec<String> {
    let chars = content.chars().collect::<Vec<_>>();
    let mut sentences = Vec::new();
    let mut current = String::new();
    let flush = |current: &mut String, sentences: &mut Vec<String>| {
        let sentence = current.trim();
        if !sentence.is_empty() {
            sentences.push(sentence.to_owned());
        }
        current.clear();
    };
    for (index, character) in chars.iter().copied().enumerate() {
        if HARD_SENTENCE_END.contains(&character) {
            if character != '\n' {
                current.push(character);
            }
            flush(&mut current, &mut sentences);
            continue;
        }
        current.push(character);
        let next = chars.get(index + 1);
        if SENTENCE_END.contains(&character) && next.is_none_or(|next| next.is_whitespace()) {
            flush(&mut current, &mut sentences);
        }
    }
    flush(&mut current, &mut sentences);
    sentences
}

fn stripped_token(raw: &str) -> &str {
    raw.trim_start_matches(|character| LEADING_WRAPPERS.contains(&character))
        .trim_end_matches(|character| TRAILING_WRAPPERS.contains(&character))
}

fn is_url_naming_one_task(token: &str) -> bool {
    match token.find("://") {
        Some(at) if at > 0 => {
            token[at + 3..]
                .split('/')
                .filter(|segment| !segment.is_empty())
                .count()
                >= 3
        }
        _ => false,
    }
}

fn is_path(token: &str) -> bool {
    if token.starts_with("~/") || token.starts_with("./") || token.starts_with("../") {
        return true;
    }
    if token.starts_with('/') && token[1..].contains('/') {
        return true;
    }
    let chars = token.chars().take(3).collect::<Vec<_>>();
    chars.len() == 3 && chars[0].is_ascii_alphabetic() && chars[1] == ':' && chars[2] == '\\'
}

fn is_repository_slug(token: &str) -> bool {
    let parts = token.split('/').collect::<Vec<_>>();
    if parts.len() != 2 {
        return false;
    }
    let slug_part = |part: &&str| {
        !part.is_empty()
            && part
                .chars()
                .all(|character| character.is_ascii_alphanumeric() || "._-".contains(character))
            && part
                .chars()
                .any(|character| character.is_ascii_alphabetic())
            && !part.starts_with('.')
    };
    parts.iter().all(slug_part)
        && parts.iter().any(|part| {
            part.chars()
                .any(|character| character.is_ascii_digit() || "-_.".contains(character))
        })
}

fn has_commit_like_segment(token: &str) -> bool {
    token
        .split(|character: char| !character.is_ascii_alphanumeric())
        .any(|segment| {
            let lowered = segment.to_ascii_lowercase();
            lowered.len() >= 7
                && lowered
                    .chars()
                    .all(|character| character.is_ascii_digit() || ('a'..='f').contains(&character))
                && lowered.chars().any(|character| character.is_ascii_digit())
                && lowered
                    .chars()
                    .any(|character| ('a'..='f').contains(&character))
        })
}

/// The text names one task — a URL into a repository, a working-directory
/// path, an `owner/repo` slug, an issue or pull-request number, or a
/// branch/commit hash.
#[must_use]
pub fn contains_task_identifier(text: &str) -> bool {
    let references = &lexicon().task_reference_words;
    let mut parts = Vec::new();
    for raw in text.split_whitespace() {
        let token = stripped_token(raw);
        if token.is_empty() {
            continue;
        }
        if is_url_naming_one_task(token)
            || is_path(token)
            || is_repository_slug(token)
            || has_commit_like_segment(token)
        {
            return true;
        }
        if token.strip_prefix('#').is_some_and(is_digits) {
            return true;
        }
        parts.extend(
            token
                .split(['-', '#', '/'])
                .filter(|part| !part.is_empty())
                .map(str::to_lowercase),
        );
    }
    parts.iter().enumerate().any(|(index, part)| {
        references.contains(part) && parts.get(index + 1).is_some_and(|next| is_digits(next))
    })
}

/// Short enough, not a session instruction, and naming no single task.
#[must_use]
pub fn is_bounded_rule(text: &str) -> bool {
    let rule = text.trim();
    !rule.is_empty()
        && rule.chars().count() <= rule_max_chars()
        && !is_session_instruction(rule)
        && !contains_task_identifier(rule)
}

/// The first sentence of `content` that carries a requirement cue and stays
/// inside the bounds. A message opening like a session instruction yields
/// nothing at all.
#[must_use]
pub fn bounded_statement(content: &str) -> Option<String> {
    if is_session_instruction(content) {
        return None;
    }
    let cues = requirement_cues();
    statement_sentences(content).into_iter().find(|sentence| {
        let lowered = sentence.to_lowercase();
        cues.iter().any(|cue| lowered.contains(cue.as_str())) && is_bounded_rule(sentence)
    })
}

/// A pronoun or function word of a supported language.
#[must_use]
pub fn is_topic_stopword(word: &str) -> bool {
    lexicon()
        .topic_stopwords
        .iter()
        .any(|stopword| stopword == word)
}

/// An answer-form verb ("answer", "reply") — a valid topic of a requirement
/// about every answer.
#[must_use]
pub fn is_answer_form_cue(word: &str) -> bool {
    lexicon().answer_form_cues.iter().any(|cue| cue == word)
}

/// A topic is not empty and not made only of pronouns and function words — an
/// answer-form verb stays a valid topic.
#[must_use]
pub fn is_valid_topic(topic: &str) -> bool {
    let lowered = topic.to_lowercase();
    let words = lowered.split_whitespace().collect::<Vec<_>>();
    !words.is_empty()
        && words
            .iter()
            .any(|word| is_answer_form_cue(word) || !is_topic_stopword(word))
}

/// A script written without spaces between words.
fn is_unspaced_script(character: char) -> bool {
    matches!(
        character,
        '\u{3040}'..='\u{30ff}'
            | '\u{3400}'..='\u{4dbf}'
            | '\u{4e00}'..='\u{9fff}'
            | '\u{f900}'..='\u{faff}'
    )
}

/// The topic inside one run of an unspaced script ("你必须总是用中文回答") —
/// the earliest answer-form verb it contains, else what is left after its
/// leading pronouns, function words and requirement cues (two or more
/// characters).
fn unspaced_topic(token: &str, cues: &[String]) -> Option<String> {
    let mut best: Option<(usize, &str)> = None;
    for cue in &lexicon().answer_form_cues {
        if cue.is_empty() {
            continue;
        }
        let Some(at) = token.find(cue.as_str()) else {
            continue;
        };
        let better = best.is_none_or(|(best_at, best_cue)| {
            at < best_at || (at == best_at && cue.chars().count() > best_cue.chars().count())
        });
        if better {
            best = Some((at, cue.as_str()));
        }
    }
    if let Some((_, cue)) = best {
        return Some(cue.to_owned());
    }
    let prefixes = lexicon()
        .topic_stopwords
        .iter()
        .chain(cues.iter())
        .filter(|prefix| !prefix.is_empty())
        .collect::<Vec<_>>();
    let mut rest = token;
    loop {
        let longest = prefixes
            .iter()
            .filter(|prefix| rest.starts_with(prefix.as_str()))
            .max_by_key(|prefix| prefix.chars().count());
        match longest {
            Some(prefix) => rest = &rest[prefix.len()..],
            None => break,
        }
    }
    (rest.chars().count() >= 2).then(|| rest.to_owned())
}

/// The subject of the statement — its first answer-form verb or first
/// significant word (three or more characters, not a number, a stopword or a
/// requirement cue). A run of an unspaced script is read by `unspaced_topic`.
#[must_use]
pub fn statement_topic(statement: &str) -> Option<String> {
    let cues = requirement_cues();
    for raw in statement.split_whitespace() {
        let token = raw
            .trim_matches(|character: char| !character.is_alphanumeric())
            .to_lowercase();
        if token.is_empty() {
            continue;
        }
        if is_answer_form_cue(&token) {
            return Some(token);
        }
        if token.chars().any(is_unspaced_script) {
            if let Some(topic) = unspaced_topic(&token, cues) {
                return Some(topic);
            }
            continue;
        }
        if token.chars().count() >= 3
            && !is_digits(&token)
            && !is_topic_stopword(&token)
            && !cues.contains(&token)
        {
            return Some(token);
        }
    }
    None
}
