//! Surface-formatting helper shared by the translation pipeline.
//!
//! Translation is a meaning-preserving operation, but writers expect the
//! target's *typography* to mirror the source's typography: a lowercase,
//! unpunctuated source should not return a capitalized, fully-punctuated
//! target. This helper applies that contract mechanically.
//!
//! Style references: Chicago Manual of Style 5.10 (mid-sentence quoted
//! fragments preserve their original capitalization); Garner's Modern
//! English Usage "Capitalization"; Розенталь §3 (русский) on quoted
//! fragments.

const TERMINAL_PUNCTUATION: &[char] = &['?', '!', '.', '。', '？', '！', '．'];

/// Copy the source fragment's leading case and terminal punctuation onto
/// the target so that:
///
/// - `как у тебя дела?` (lowercase, `?`) → target keeps lowercase + `?`
/// - `Как у тебя дела?` (uppercase, `?`) → target keeps uppercase + `?`
/// - `как дела` (lowercase, no terminal) → target keeps lowercase, drops `?`
#[must_use]
pub fn match_source_formatting(target: &str, source: &str) -> String {
    let target_trimmed = target.trim();
    if target_trimmed.is_empty() {
        return String::new();
    }
    let source_trimmed = source.trim();

    let source_terminal = source_trimmed
        .chars()
        .next_back()
        .filter(|character| TERMINAL_PUNCTUATION.contains(character));
    let target_no_terminal: String = target_trimmed
        .trim_end_matches(|character: char| TERMINAL_PUNCTUATION.contains(&character))
        .to_owned();
    let with_terminal = match source_terminal {
        Some(character) => format!("{target_no_terminal}{character}"),
        None => target_no_terminal,
    };

    let Some(source_first_letter) = source_trimmed
        .chars()
        .find(|character| character.is_alphabetic())
    else {
        return with_terminal;
    };

    let Some((idx, target_first_letter)) = with_terminal
        .char_indices()
        .find(|(_, character)| character.is_alphabetic())
    else {
        return with_terminal;
    };

    if source_first_letter.is_lowercase() && target_first_letter.is_uppercase() {
        return splice_first_letter(&with_terminal, idx, target_first_letter, true);
    }
    if source_first_letter.is_uppercase() && target_first_letter.is_lowercase() {
        return splice_first_letter(&with_terminal, idx, target_first_letter, false);
    }
    with_terminal
}

fn splice_first_letter(source: &str, idx: usize, first_letter: char, to_lowercase: bool) -> String {
    let mut result = String::with_capacity(source.len());
    result.push_str(&source[..idx]);
    if to_lowercase {
        for character in first_letter.to_lowercase() {
            result.push(character);
        }
    } else {
        for character in first_letter.to_uppercase() {
            result.push(character);
        }
    }
    result.push_str(&source[idx + first_letter.len_utf8()..]);
    result
}

#[path = "../source_tests/translation/formatting/tests.rs"]
mod tests;
