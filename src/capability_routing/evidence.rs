//! Surface-evidence helpers for the capability router.
//!
//! The predicates that read a prompt's characters and the seed's role surfaces
//! -- URL and path shapes, clock times, quoted spans, scope nouns, web hosts --
//! one layer beside the derivation that consumes them. The router owns what a
//! capability is; this module owns what counts as evidence.

use super::*;

/// Whether any surface of `role` occurs in `prompt`.
///
/// Both the token-bounded and the raw-substring readings are consulted, because
/// the roles this module shares with older recognisers record some surfaces as
/// whole phrases and some as inflectable stems.
pub(super) fn evidences(role: &str, normalized: &str) -> bool {
    let lexicon = seed::lexicon();
    lexicon.mentions_role(role, normalized) || lexicon.mentions_role_raw(role, normalized)
}

/// How *specific* the evidence for `role` in `normalized` is: the character
/// length of the longest surface of the role the prompt carries, or zero.
///
/// A longer surface is a more specific observation about the same sentence.
/// "echo" is evidence of composing; "echo the contents of" is evidence of
/// retrieving, and it is the same four letters plus fifteen more, so it is the
/// reading the sentence actually supports. Without this, act resolution would
/// depend on the order the acts happen to be declared in, which is exactly the
/// declaration-order dependence plan 10 refuses in the table itself.
pub(super) fn evidence_strength(role: &str, normalized: &str) -> usize {
    seed::lexicon()
        .words_for_role(role)
        .into_iter()
        .filter(|word| normalized.contains(word.as_str()))
        .map(|word| word.chars().count())
        .max()
        .unwrap_or(0)
}

/// Whether a token parses as an absolute URL.
pub(super) fn is_url(token: &str) -> bool {
    let token = token.trim_matches(|character: char| !character.is_alphanumeric());
    token
        .split_once("://")
        .is_some_and(|(scheme, rest)| matches!(scheme, "http" | "https") && !rest.is_empty())
}

/// Whether a token reads as a workspace path in this request.
///
/// A dotted API member and a filename have the same punctuation. A
/// lower-to-upper boundary in the left side (`TypeName.member`) is structural
/// evidence for a qualified symbol, unless the surrounding request explicitly
/// places that token in a filesystem scope. This prevents a documentation
/// subject from becoming a read-file request without maintaining an extension
/// allowlist in Rust.
pub(super) fn is_path(token: &str, normalized: &str) -> bool {
    let token = token.trim_matches(|character: char| matches!(character, ',' | ';' | '"' | '\''));
    if is_url(token) {
        return false;
    }
    if token.contains('/') || token.contains('\\') {
        return true;
    }
    token.rsplit_once('.').is_some_and(|(stem, extension)| {
        !stem.is_empty()
            && (1..=5).contains(&extension.chars().count())
            && extension.chars().all(char::is_alphanumeric)
            && extension.chars().any(char::is_alphabetic)
            && !is_web_host_suffix(extension)
            && (!looks_like_qualified_member(stem, extension) || has_workspace_scope(normalized))
    })
}

/// The punctuation a token may carry at its edges and still be read as a
/// *pattern*. Wider than [`is_path`]'s trim: a pattern is frequently quoted or
/// followed by a colon ("find files matching *.lino:"), and the wildcard inside
/// is the evidence, not the edges. The sentence marks are here because `?` is
/// also a question mark: "work?" ends a question, it does not carry a
/// single-character wildcard.
pub(super) fn trim_pattern_token(token: &str) -> &str {
    token.trim_matches(|character: char| {
        matches!(
            character,
            ',' | ';'
                | '"'
                | '\''
                | '`'
                | ':'
                | '('
                | ')'
                | '.'
                | '\u{3002}'
                | '?'
                | '!'
                | '\u{00BF}'
                | '\u{00A1}'
        )
    })
}

/// Whether a token carries a wildcard character anchored to file-shaped
/// structure, so it names a *pattern* for files to match rather than one file
/// to act on.
///
/// The anchor is the honest part of the reading: a glob wildcard is written
/// against a filename (`*.rs`, `src/**/*.ts`, `config.?`), so the token must
/// also carry a path separator or a dotted extension. A bare `*` floating
/// between spaces is not a filename at all — in "find y: 7 * y = 84" it is the
/// multiplication operator, and reading it as a workspace pattern would turn
/// an arithmetic request into a glob refusal (the no-silent-unknown canary).
pub(super) fn is_pattern_token(token: &str) -> bool {
    let token = trim_pattern_token(token);
    if token.is_empty() {
        return false;
    }
    let has_wildcard = token.contains('*') || token.contains('?') || token.contains('[');
    if !has_wildcard {
        return false;
    }
    token.contains('/')
        || token.contains('\\')
        || (token.contains('.') && token.chars().any(char::is_alphanumeric))
}

/// How many distinct tokens of `prompt` read as workspace paths.
pub(super) fn path_token_count(prompt: &str, normalized: &str) -> usize {
    prompt
        .split_whitespace()
        .filter(|token| is_path(token, normalized))
        .count()
}

/// A dotted identifier whose left side carries a type/module-style word
/// boundary and whose right side is an identifier, rather than an extension.
pub(super) fn looks_like_qualified_member(stem: &str, member: &str) -> bool {
    if stem.contains('.')
        || !stem
            .chars()
            .all(|character| character.is_alphanumeric() || character == '_')
        || !member
            .chars()
            .all(|character| character.is_alphanumeric() || character == '_')
    {
        return false;
    }
    stem.chars()
        .zip(stem.chars().skip(1))
        .any(|(left, right)| left.is_lowercase() && right.is_uppercase())
}

/// Whether `extension` is a registrable domain suffix rather than a file
/// extension, so `amazon.in` is a site and `main.rs` is a file.
pub(super) fn is_web_host_suffix(extension: &str) -> bool {
    let extension = extension.to_lowercase();
    seed::lexicon()
        .words_for_role(ROLE_CAPABILITY_WEB_HOST_SUFFIX)
        .contains(&extension)
}

/// Whether the prompt carries a clock time: digits, a colon, digits.
pub(super) fn has_clock_time(prompt: &str) -> bool {
    let characters: Vec<char> = prompt.chars().collect();
    characters.iter().enumerate().any(|(index, character)| {
        *character == ':'
            && index > 0
            && characters[index - 1].is_ascii_digit()
            && characters.get(index + 1).is_some_and(char::is_ascii_digit)
    })
}

/// Whether the prompt carries a balanced quoted span of more than one token.
pub(super) fn has_quoted_span(prompt: &str) -> bool {
    for delimiter in ['"', '\u{201c}', '\u{00ab}'] {
        let closing = match delimiter {
            '\u{201c}' => '\u{201d}',
            '\u{00ab}' => '\u{00bb}',
            other => other,
        };
        if let Some(open) = prompt.find(delimiter) {
            let rest = &prompt[open + delimiter.len_utf8()..];
            if let Some(close) = rest.find(closing)
                && rest[..close].split_whitespace().count() > 1
            {
                return true;
            }
        }
    }
    false
}

/// Whether the request carries *explicit content*: a quoted span, or a seeded
/// content introducer (`containing`, `с текстом`, `内容为`, `con el texto`).
///
/// Quotation marks are one way a person hands over a literal; naming the
/// content with the phrase the language uses for it is the other, and a request
/// that carries one carries the same object as a request that carries the other.
pub(super) fn has_explicit_content(prompt: &str, normalized: &str) -> bool {
    has_quoted_span(prompt) || evidences(ROLE_CAPABILITY_CONTENT_INTRODUCER, normalized)
}

/// Whether the prompt names a filesystem container: the desktop, the home
/// directory, the current directory, or the folder/file/path nouns themselves.
/// A possessive is not consulted, which is the single change that makes
/// `on my desktop` and `on desktop` one request.
pub(super) fn has_container_scope(normalized: &str) -> bool {
    [
        ROLE_LOCAL_PATH_SCOPE_DESKTOP,
        ROLE_LOCAL_PATH_SCOPE_HOME,
        ROLE_LOCAL_PATH_SCOPE_CURRENT,
        ROLE_CAPABILITY_CONTAINER_SCOPE,
    ]
    .iter()
    .any(|role| evidences(role, normalized))
}

/// Whether the prompt scopes its effect to the machine the task is being done
/// on. Wider than [`has_container_scope`]: the code, the repository and the
/// project are the workspace without being a directory the request names.
pub(super) fn has_workspace_scope(normalized: &str) -> bool {
    has_container_scope(normalized) || evidences(ROLE_CAPABILITY_WORKSPACE_SCOPE, normalized)
}

/// Whether the prompt scopes its effect to the open web, either by naming it or
/// by asking for what is true *now*, which nothing but a live source can answer
/// (plan 10 leaf 12's `freshness: live` qualifier).
pub(super) fn has_web_scope(normalized: &str) -> bool {
    evidences(ROLE_CAPABILITY_WEB_SCOPE, normalized) || is_freshness_live(normalized)
}

/// Whether the request names the open web itself -- the web, the internet, a
/// search engine, or what is true right now.
///
/// The planner asks this before letting the table answer a bare term at the end
/// of its cascade: at that position every route that reads the conversation and
/// the workspace has already declined, and a request that never mentioned the
/// open web is one the symbolic engine should still get its turn at.
#[must_use]
pub fn names_open_web(prompt: &str) -> bool {
    has_web_scope(&normalize_prompt(prompt))
}

/// The `freshness: live` qualifier: the request is about what is true at the
/// moment it is asked, so a stored answer cannot satisfy it (issue #720).
#[must_use]
pub fn is_freshness_live(normalized: &str) -> bool {
    evidences(ROLE_CAPABILITY_FRESHNESS_LIVE, normalized)
}

/// Whether the subject is a piece of the assistant's own surface.
pub(super) fn is_self_surface(normalized: &str) -> bool {
    evidences(ROLE_CAPABILITY_SELF_SURFACE_NOUN, normalized)
        || evidences(ROLE_ASSISTANT_MECHANISM_INQUIRY, normalized)
        || is_prior_turn_reference(normalized)
}

/// Whether the prompt is, in full, a seeded clarification utterance (issue #29).
///
/// A bare "what do you mean" or "не понял" is a comprehension turn, and the
/// table reads those same short words as a question about the assistant's
/// surface (`self` is in "what do *you* mean"). Whole-surface equality -- the
/// issue #1095 rule that a turn which *is* a surface carries the role while a
/// request merely *containing* it keeps its own meaning -- is what lets the
/// seed's clarification class keep its own utterances.
#[must_use]
pub fn is_bare_clarification(prompt: &str) -> bool {
    let cleaned = crate::web_engine_core::normalize_prompt(prompt);
    !cleaned.is_empty()
        && seed::lexicon()
            .words_for_role(crate::seed::ROLE_CLARIFICATION_REQUEST)
            .iter()
            .any(|surface| surface == &cleaned)
}

/// Whether the object of the request is the assistant's *previous turn*: the
/// closed class of comprehension-failure surfaces (issue #721).
///
/// This is the one class with no structural signal at all -- "that went over my
/// head" names nothing the characters can be read for -- so it is grounded in a
/// seed role in five languages rather than in a Rust branch, and none of the
/// three reported strings is among its surfaces.
#[must_use]
pub fn is_prior_turn_reference(normalized: &str) -> bool {
    evidences(ROLE_CAPABILITY_PRIOR_TURN_REFERENCE, normalized)
}

/// The literal content a request hands over, and the path it hands it to.
///
/// Both halves are read from the same two sources the `quoted_content` object
/// is derived from -- a balanced quoted span, or a seeded content introducer --
/// so the planner writes what the table said was there rather than guessing
/// again with a second rule.
#[must_use]
pub fn explicit_content(prompt: &str) -> Option<String> {
    if let Some(span) = quoted_span(prompt) {
        return Some(span);
    }
    let characters: Vec<char> = prompt.chars().collect();
    let lowered: Vec<char> = prompt.chars().flat_map(char::to_lowercase).collect();
    if lowered.len() != characters.len() {
        // A case fold that changes the character count would misplace every
        // offset below, so the request is left to the planner's own extractor.
        return None;
    }
    let mut best: Option<usize> = None;
    for surface in seed::lexicon().words_for_role(ROLE_CAPABILITY_CONTENT_INTRODUCER) {
        let needle: Vec<char> = surface.chars().collect();
        if needle.is_empty() || needle.len() > lowered.len() {
            continue;
        }
        for start in 0..=lowered.len() - needle.len() {
            if lowered[start..start + needle.len()] == needle[..] {
                let end = start + needle.len();
                if best.is_none_or(|current| end > current) {
                    best = Some(end);
                }
            }
        }
    }
    let Some(end) = best else {
        return assigned_content(prompt);
    };
    let tail: String = characters[end..].iter().collect();
    let tail = tail.trim().trim_matches(|character: char| {
        matches!(
            character,
            ':' | '"' | '\u{201c}' | '\u{201d}' | '.' | '\u{3002}'
        )
    });
    (!tail.trim().is_empty()).then(|| tail.trim().to_owned())
}

/// The content of a request that names its destination first: the text after
/// the assignment connector that follows the destination path.
///
/// "set the contents of note.txt to hello" and "pon el contenido de note.txt en
/// hello" are the same request, and neither quotes its content nor introduces
/// it. The connector is only read *after* the path, because on its own it is
/// far too common a word to be evidence of anything.
pub(super) fn assigned_content(prompt: &str) -> Option<String> {
    let path = first_path(prompt)?;
    let after = prompt.split_once(path.as_str()).map(|(_, tail)| tail)?;
    let lowered = after.to_lowercase();
    let mut best: Option<usize> = None;
    for surface in seed::lexicon().words_for_role(ROLE_CAPABILITY_CONTENT_ASSIGNMENT) {
        if let Some(position) = lowered.find(surface.as_str()) {
            let end = position + surface.len();
            if best.is_none_or(|current| end < current) && after.is_char_boundary(end) {
                best = Some(end);
            }
        }
    }
    let content = after[best?..].trim().trim_matches('"');
    (!content.is_empty()).then(|| content.to_owned())
}

/// The first balanced quoted span of more than one token, if any.
pub(super) fn quoted_span(prompt: &str) -> Option<String> {
    for delimiter in ['"', '\u{201c}', '\u{00ab}'] {
        let closing = match delimiter {
            '\u{201c}' => '\u{201d}',
            '\u{00ab}' => '\u{00bb}',
            other => other,
        };
        if let Some(open) = prompt.find(delimiter) {
            let rest = &prompt[open + delimiter.len_utf8()..];
            if let Some(close) = rest.find(closing) {
                let inner = &rest[..close];
                if inner.split_whitespace().count() > 1 {
                    return Some(inner.to_owned());
                }
            }
        }
    }
    None
}

/// The first token of `prompt` that reads as a workspace path.
#[must_use]
pub fn first_path(prompt: &str) -> Option<String> {
    let normalized = normalize_prompt(prompt);
    prompt
        .split_whitespace()
        .find(|token| is_path(token, &normalized))
        .map(|token| {
            token
                .trim_matches(|character: char| {
                    matches!(character, ',' | ';' | '"' | '\'' | '(' | ')' | '\u{3002}')
                })
                .to_owned()
        })
}

/// The first token of `prompt` that parses as an absolute URL.
#[must_use]
pub fn first_url(prompt: &str) -> Option<String> {
    prompt
        .split_whitespace()
        .find(|token| is_url(token))
        .map(|token| {
            token
                .trim_matches(|character: char| matches!(character, ',' | ';' | '"' | '\'' | '.'))
                .to_owned()
        })
}
