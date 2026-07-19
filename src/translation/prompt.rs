//! Translation-prompt parsing helpers.
//!
//! These are the surface-extraction routines that bridge the natural-
//! language prompt and the [`crate::translation::TranslationPipeline`].
//! Quoted-fragment extraction lives in `crate::solver_helpers`; this
//! module focuses on the unquoted variant introduced for issue #216
//! (`translate apple to russian`).
//!
//! Like the rest of issue #386, no surface word is hardcoded here. Every
//! literal the extractor matches — the verb frames, the directional markers,
//! the object particles — is projected from the language-independent meaning
//! lexicon (`data/seed/meanings-translation.lino`) by semantic *role* and by
//! the *slot* and *script* each word form occupies. The algorithm is the only
//! thing that lives in code: it reads each form's [`Slot`] to decide whether a
//! language is head-initial (a circumfix verb frame brackets the surface) or
//! head-final (the verb/object particles bound it), and partitions the markers
//! by script so the Devanagari and Han strategies never name a raw word.

use std::sync::OnceLock;

use crate::coding::{contains_cjk, contains_devanagari};
use crate::seed::{
    self, Slot, WordForm, ROLE_TRANSLATION_INTO_MARKER, ROLE_TRANSLATION_OBJECT_MARKER,
    ROLE_TRANSLATION_TARGET_DIRECTION, ROLE_TRANSLATION_UNQUOTED_FRAME,
};

/// Extract the surface phrase from an unquoted translation prompt.
///
/// Recognises the supported-language unquoted forms used by the solver,
/// including `translate <surface> to <language>`, `переведи <surface> на
/// <language>`, `<surface> का <language> में अनुवाद करो`, and `把 <surface>
/// 翻译成<language>`. Returns `None` when the prompt does not match this shape
/// — callers fall back to `extract_quoted_phrase`.
///
/// The function is intentionally conservative: it only recognises target
/// markers supported by `detect_target_language` and stops the surface at
/// the first directional marker. This keeps the extraction robust against
/// natural-language variations users type (`translate apple to russian`,
/// `переведи яблоко на английский`, `apple का हिंदी में अनुवाद करो`,
/// `把 apple 翻译成中文`).
///
/// Issue #216 reproduced this: `translate apple to russian` returned the
/// empty placeholder `[ru]` because `extract_quoted_phrase` returned
/// `None` and the handler defaulted to an empty surface. The fallback
/// here recovers the surface `apple` so the Wiktionary pipeline can
/// translate it.
#[must_use]
pub fn extract_unquoted_translation_surface(prompt: &str) -> Option<String> {
    let trimmed = prompt.trim_end_matches(['.', '!', '?', '。']);
    let lower = trimmed.to_lowercase();

    extract_circumfix_surface(trimmed, &lower)
        .or_else(|| extract_hindi_unquoted_surface(trimmed, &lower))
        .or_else(|| extract_chinese_unquoted_surface(trimmed, &lower))
        .or_else(|| extract_suffix_surface(trimmed, &lower))
}

/// Whether a structurally extracted source-first prompt also carries a
/// language-neutral translation action and a recognized target.
#[must_use]
pub fn is_source_first_translation_request(
    normalized: &str,
    target_detected: bool,
    surface_detected: bool,
) -> bool {
    target_detected
        && surface_detected
        && seed::lexicon()
            .words_for_role_in_languages(seed::ROLE_TRANSLATION_ACTION, &["en", "ru", "hi", "zh"])
            .iter()
            .any(|stem| normalized.contains(stem.as_str()))
}

/// Source-first extraction: a suffix frame follows the text and precedes the
/// target language (`<surface> - translate to <language>`). The separator is
/// presentation punctuation, not part of the source proposition.
fn extract_suffix_surface(original: &str, lower: &str) -> Option<String> {
    markers().suffix_frames.iter().find_map(|frame| {
        let offset = lower.rfind(frame)?;
        let target = lower[offset + frame.len()..].trim();
        if target.is_empty() {
            return None;
        }
        clean_unquoted_surface(&original[..offset])
    })
}

/// Head-initial extraction (English, Russian): each circumfix verb frame
/// brackets the surface — the before-slot literal is stripped from the front and
/// the after-slot directional marker bounds the surface on the right. Frames are
/// tried in lexicon declaration order (English, then Russian).
fn extract_circumfix_surface(original: &str, lower: &str) -> Option<String> {
    markers()
        .circumfix_frames
        .iter()
        .find_map(|(prefix, marker)| {
            extract_between_prefix_and_marker(original, lower, prefix, marker)
        })
}

fn extract_between_prefix_and_marker(
    original: &str,
    lower: &str,
    prefix: &str,
    marker: &str,
) -> Option<String> {
    let rest = lower.strip_prefix(prefix)?;
    let marker_offset = rest.find(marker)?;
    let start = prefix.len();
    let end = start + marker_offset;
    clean_unquoted_surface(&original[start..end])
}

/// Head-final Hindi extraction: a bare verb stem (अनुवाद) gates the strategy;
/// the target-and-verb compound (` में अनुवाद`) bounds the surface on the right,
/// and the object postposition (का, then को) marks where the surface ends.
fn extract_hindi_unquoted_surface(original: &str, lower: &str) -> Option<String> {
    let table = markers();
    if !table
        .hindi_verb_stems
        .iter()
        .any(|stem| lower.contains(stem))
    {
        return None;
    }
    for target_marker in &table.hindi_target_markers {
        let Some(target_offset) = lower.find(target_marker) else {
            continue;
        };
        let before_target = &lower[..target_offset];
        for surface_marker in &table.hindi_object_markers {
            if let Some(surface_end) = before_target.rfind(surface_marker) {
                return clean_unquoted_surface(&original[..surface_end]);
            }
        }
    }
    None
}

/// Head-final Chinese extraction: a disposal particle (把, 将) fronts the object
/// and a verb-and-target compound (翻译成, …) closes the surface; failing that, a
/// bare 翻译 verb stem is stripped from the front and the surface stops at the
/// first bare target-direction marker (成, 为, 為, 到).
fn extract_chinese_unquoted_surface(original: &str, lower: &str) -> Option<String> {
    let table = markers();
    for prefix in &table.chinese_command_prefixes {
        let Some(rest) = lower.strip_prefix(prefix) else {
            continue;
        };
        if let Some(marker_offset) = first_marker(rest, &table.chinese_command_markers) {
            let start = prefix.len();
            let end = start + marker_offset;
            return clean_unquoted_surface(&original[start..end]);
        }
    }

    for prefix in &table.chinese_translate_prefixes {
        let Some(rest) = lower.strip_prefix(prefix) else {
            continue;
        };
        if let Some(marker_offset) = first_marker(rest, &table.chinese_target_markers) {
            let start = prefix.len();
            let end = start + marker_offset;
            return clean_unquoted_surface(&original[start..end]);
        }
    }

    None
}

/// The byte offset of the earliest of `markers` found in `text`, if any.
fn first_marker(text: &str, markers: &[&str]) -> Option<usize> {
    markers.iter().filter_map(|marker| text.find(marker)).min()
}

fn clean_unquoted_surface(candidate: &str) -> Option<String> {
    let cleaned = candidate
        .trim()
        .trim_end_matches(['-', '–', '—', ':'])
        .trim_end();
    if cleaned.is_empty()
        || cleaned.chars().any(|character| {
            matches!(
                character,
                '"' | '\'' | '«' | '»' | '`' | '“' | '”' | '‘' | '’'
            )
        })
    {
        return None;
    }
    Some(cleaned.to_owned())
}

/// The projection of the translation-extraction markers out of the meaning
/// lexicon, built once. Each field is the language-independent role filtered to
/// the slot and script the corresponding strategy needs, in lexicon declaration
/// order — so the code names a *role* and a *shape*, never a surface word.
struct TranslationMarkers {
    /// Circumfix verb frames as `(before-slot prefix, after-slot marker)` — the
    /// head-initial English/Russian forms, declaration order.
    circumfix_frames: Vec<(&'static str, &'static str)>,
    /// Literal after-slot frames for source-first commands.
    suffix_frames: Vec<&'static str>,
    /// Hindi bare verb stems (Devanagari) whose presence gates the extractor.
    hindi_verb_stems: Vec<&'static str>,
    /// Hindi target-and-verb compounds (Devanagari): the right boundary.
    hindi_target_markers: Vec<&'static str>,
    /// Hindi object postpositions (Devanagari): right boundary, tried in order.
    hindi_object_markers: Vec<&'static str>,
    /// Chinese disposal particles (Han) that front the object: stripped prefix.
    chinese_command_prefixes: Vec<&'static str>,
    /// Chinese verb-and-target compounds (Han) that stop the surface.
    chinese_command_markers: Vec<&'static str>,
    /// Chinese bare verb stems (Han) stripped as a translate prefix.
    chinese_translate_prefixes: Vec<&'static str>,
    /// Chinese bare target-direction markers (Han) that stop the surface.
    chinese_target_markers: Vec<&'static str>,
}

/// Build (once) the marker projection from the meaning lexicon.
fn markers() -> &'static TranslationMarkers {
    static CACHE: OnceLock<TranslationMarkers> = OnceLock::new();
    CACHE.get_or_init(|| TranslationMarkers {
        circumfix_frames: circumfix_frames(ROLE_TRANSLATION_UNQUOTED_FRAME),
        suffix_frames: suffix_frames(ROLE_TRANSLATION_UNQUOTED_FRAME),
        hindi_verb_stems: bare_script_forms(ROLE_TRANSLATION_UNQUOTED_FRAME, contains_devanagari),
        hindi_target_markers: script_forms(ROLE_TRANSLATION_INTO_MARKER, contains_devanagari),
        hindi_object_markers: script_forms(ROLE_TRANSLATION_OBJECT_MARKER, contains_devanagari),
        chinese_command_prefixes: script_forms(ROLE_TRANSLATION_OBJECT_MARKER, contains_cjk),
        chinese_command_markers: script_forms(ROLE_TRANSLATION_INTO_MARKER, contains_cjk),
        chinese_translate_prefixes: bare_script_forms(
            ROLE_TRANSLATION_UNQUOTED_FRAME,
            contains_cjk,
        ),
        chinese_target_markers: script_forms(ROLE_TRANSLATION_TARGET_DIRECTION, contains_cjk),
    })
}

fn suffix_frames(role: &str) -> Vec<&'static str> {
    seed::lexicon()
        .role_word_forms(role)
        .into_iter()
        .filter(|form| form.slot() == Slot::Suffix)
        .map(WordForm::after_slot)
        .collect()
}

/// Every circumfix form of `role` as a `(before-slot, after-slot)` pair, in
/// lexicon declaration order.
fn circumfix_frames(role: &str) -> Vec<(&'static str, &'static str)> {
    seed::lexicon()
        .role_word_forms(role)
        .into_iter()
        .filter(|form| form.slot() == Slot::Circumfix)
        .map(|form| (form.before_slot(), form.after_slot()))
        .collect()
}

/// The surface text of every form of `role` whose text matches `script`, in
/// lexicon declaration order. The script filter is what excludes the
/// English/Russian "completeness" forms a head-final role also carries.
fn script_forms(role: &str, script: fn(&str) -> bool) -> Vec<&'static str> {
    seed::lexicon()
        .role_word_forms(role)
        .into_iter()
        .filter(|form| script(&form.text))
        .map(|form| form.text.as_str())
        .collect()
}

/// The surface text of every [`Slot::Bare`] form of `role` whose text matches
/// `script`, in lexicon declaration order.
fn bare_script_forms(role: &str, script: fn(&str) -> bool) -> Vec<&'static str> {
    seed::lexicon()
        .role_word_forms(role)
        .into_iter()
        .filter(|form| form.slot() == Slot::Bare && script(&form.text))
        .map(WordForm::before_slot)
        .collect()
}
