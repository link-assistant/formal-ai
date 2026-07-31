//! Language-independent recognition of computer-use requests (issue #707).
//!
//! Nothing here names a natural-language word. Every surface lives in
//! `data/seed/meanings-computer-use.lino`; this module only asks the seed
//! lexicon which meanings carrying the operation, resource, and capability-gap
//! roles are evidenced in a prompt, and returns their language-independent
//! slugs together with the position at which the evidence appeared. Ordering by
//! position is what lets an unseen request ("pack the notes and unpack them")
//! keep the order the speaker used, in any of the four supported languages.

use crate::seed::{
    lexicon, Meaning, ROLE_COMPUTER_USE_CAPABILITY_GAP_CUE, ROLE_COMPUTER_USE_OPERATION_CUE,
    ROLE_COMPUTER_USE_RESOURCE_CUE,
};

/// One recognised cue: the meaning slug, where it matched, and how long the
/// matched surface was (longer evidence wins when two meanings overlap).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Cue {
    pub slug: String,
    pub position: usize,
    pub length: usize,
}

/// Lower-case the prompt and turn every character that cannot be part of a word
/// into a space, so surface matching sees whole tokens regardless of
/// punctuation.
///
/// "Part of a word" means alphanumeric, a path character, **or a combining
/// mark**. The mark clause is not decoration: Devanagari viramas and nuktas are
/// not `is_alphanumeric`, so dropping them would split `नोट्स` into `नोट स` and
/// silently make every Hindi surface containing a conjunct unmatchable — a
/// whole language quietly failing recognition.
#[must_use]
pub fn normalize(prompt: &str) -> String {
    let mut normalized = String::with_capacity(prompt.len());
    for character in prompt.chars() {
        if character.is_alphanumeric()
            || character == '/'
            || character == '.'
            || is_combining_mark(character)
        {
            normalized.extend(character.to_lowercase());
        } else {
            normalized.push(' ');
        }
    }
    normalized.split_whitespace().collect::<Vec<_>>().join(" ")
}

/// The part of `prompt` that is the speaker's *instruction*, with embedded
/// literal payload removed.
///
/// A request may carry content it wants written somewhere — a Links Notation
/// record, a snippet, a quoted string. Words inside that payload are data the
/// speaker is transporting, not operations they are asking for, and reading
/// them as cues is how a planner hallucinates a plan out of an incidental
/// `order "90"` or `list_files_arg`. Two structural signals separate the two,
/// neither of them language-specific:
///
///   * an indented line continues a structured block, so it is payload; and
///   * a double-quoted span is a literal the speaker is quoting, not naming.
///
/// Recognition therefore runs over the remainder. A request written as ordinary
/// prose — one line, unquoted, in any of the four languages — is unaffected.
#[must_use]
pub fn instruction_surface(prompt: &str) -> String {
    let mut instruction = String::with_capacity(prompt.len());
    for line in prompt.lines() {
        if line.starts_with(' ') || line.starts_with('\t') {
            continue;
        }
        let mut quoted = false;
        for character in line.chars() {
            if character == '"' {
                quoted = !quoted;
                instruction.push(' ');
            } else if !quoted {
                instruction.push(character);
            }
        }
        instruction.push('\n');
    }
    instruction
}

/// Is `character` a combining mark that belongs to the word it follows?
///
/// Covers the Unicode combining blocks and the Brahmic dependent-sign ranges of
/// the scripts the seed lexicon uses; marks are classified `Mn`/`Mc` and are
/// therefore invisible to [`char::is_alphanumeric`].
const fn is_combining_mark(character: char) -> bool {
    matches!(character as u32,
        0x0300..=0x036F      // combining diacritical marks
        | 0x0483..=0x0489    // Cyrillic combining marks
        | 0x0591..=0x05BD    // Hebrew points
        | 0x0610..=0x061A | 0x064B..=0x065F | 0x0670 | 0x06D6..=0x06DC // Arabic marks
        | 0x0900..=0x0903 | 0x093A..=0x094F | 0x0951..=0x0957 | 0x0962..=0x0963 // Devanagari
        | 0x0981..=0x0983 | 0x09BC..=0x09CD  // Bengali
        | 0x0A01..=0x0A03 | 0x0A3C..=0x0A4D  // Gurmukhi
        | 0x0B01..=0x0B4D | 0x0C00..=0x0C4D | 0x0D00..=0x0D4D // Oriya, Telugu, Malayalam
        | 0x0E31..=0x0E3A | 0x0E47..=0x0E4E  // Thai
        | 0x1AB0..=0x1AFF | 0x1DC0..=0x1DFF | 0x20D0..=0x20F0 | 0xFE20..=0xFE2F
    )
}

/// Operations named by the prompt, in the order the speaker named them.
#[must_use]
pub fn operation_cues(normalized: &str) -> Vec<Cue> {
    let mut cues = role_cues(ROLE_COMPUTER_USE_OPERATION_CUE, normalized);
    cues.sort_by(|left, right| {
        left.position
            .cmp(&right.position)
            .then_with(|| right.length.cmp(&left.length))
            .then_with(|| left.slug.cmp(&right.slug))
    });
    cues
}

/// The resource the prompt is about, if exactly one is evidenced most
/// specifically. Longer evidence wins: an "inbox note" is not a "note".
#[must_use]
pub fn resource_cue(normalized: &str) -> Option<Cue> {
    role_cues(ROLE_COMPUTER_USE_RESOURCE_CUE, normalized)
        .into_iter()
        .max_by(|left, right| {
            left.length
                .cmp(&right.length)
                .then_with(|| right.position.cmp(&left.position))
        })
}

/// The named capability gap a prompt runs into, if any (for example
/// `gui_rendering`). The capability name is the meaning slug's suffix.
#[must_use]
pub fn capability_gap_cue(normalized: &str) -> Option<String> {
    role_cues(ROLE_COMPUTER_USE_CAPABILITY_GAP_CUE, normalized)
        .into_iter()
        .min_by_key(|cue| cue.position)
        .and_then(|cue| {
            cue.slug
                .strip_prefix("computer_use_gap_")
                .map(ToOwned::to_owned)
        })
}

fn role_cues(role: &str, normalized: &str) -> Vec<Cue> {
    lexicon()
        .meanings_with_role(role)
        .filter_map(|meaning| best_evidence(meaning, normalized))
        .collect()
}

fn best_evidence(meaning: &Meaning, normalized: &str) -> Option<Cue> {
    meaning
        .words()
        .filter_map(|surface| surface_position(normalized, surface).map(|at| (at, surface.len())))
        .min_by(|left, right| left.0.cmp(&right.0).then_with(|| right.1.cmp(&left.1)))
        .map(|(position, length)| Cue {
            slug: meaning.slug.clone(),
            position,
            length,
        })
}

/// Where `surface` occurs in `normalized`, using the project-wide contract:
/// CJK surfaces match as substrings, space-delimited surfaces match whole
/// tokens or phrases (mirrors `crate::seed::Lexicon::mentions_role`).
fn surface_position(normalized: &str, surface: &str) -> Option<usize> {
    if surface.is_empty() {
        return None;
    }
    if crate::coding::contains_cjk(surface) {
        return normalized.find(surface);
    }
    let mut search = 0;
    while let Some(offset) = normalized[search..].find(surface) {
        let start = search + offset;
        let end = start + surface.len();
        let starts_token = start == 0 || normalized.as_bytes()[start - 1] == b' ';
        let ends_token = end == normalized.len() || normalized.as_bytes()[end] == b' ';
        if starts_token && ends_token {
            return Some(start);
        }
        search = start + surface.len().max(1);
        if search >= normalized.len() {
            break;
        }
    }
    None
}
