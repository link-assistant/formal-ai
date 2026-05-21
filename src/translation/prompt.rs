//! Translation-prompt parsing helpers.
//!
//! These are the surface-extraction routines that bridge the natural-
//! language prompt and the [`crate::translation::TranslationPipeline`].
//! Quoted-fragment extraction lives in [`crate::solver_helpers`]; this
//! module focuses on the unquoted variant introduced for issue #216
//! (`translate apple to russian`).

/// Extract the surface phrase from an unquoted translation prompt.
///
/// Recognises `translate <surface> to <language>` (English) and
/// `переведи <surface> на <language>` (Russian). Returns `None` when the
/// prompt does not match this shape — callers fall back to
/// `extract_quoted_phrase`.
///
/// The function is intentionally conservative: it only recognises the two
/// trigger verbs supported by `detect_target_language` and stops the
/// surface at the first directional preposition (`to` / `на`). This keeps
/// the extraction robust against the natural-language variations users
/// type (`translate apple to russian`, `переведи яблоко на английский`,
/// `translate apple to russian.`).
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
    let (verb, prep_offset) = if let Some(rest) = lower.strip_prefix("translate ") {
        ("translate ", rest.find(" to ")?)
    } else if let Some(rest) = lower.strip_prefix("переведи ") {
        ("переведи ", rest.find(" на ")?)
    } else {
        return None;
    };
    let verb_len = verb.len();
    let surface_lower = &lower[verb_len..verb_len + prep_offset];
    let original_surface = &trimmed[verb_len..verb_len + prep_offset];
    let cleaned = original_surface
        .trim()
        .trim_matches(['"', '\'', '«', '»', '`']);
    if cleaned.is_empty() || surface_lower.contains('"') || surface_lower.contains('\'') {
        return None;
    }
    Some(cleaned.to_owned())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn extracts_unquoted_english_surface() {
        assert_eq!(
            extract_unquoted_translation_surface("translate apple to russian"),
            Some("apple".to_owned()),
        );
    }

    #[test]
    fn preserves_capitalization() {
        assert_eq!(
            extract_unquoted_translation_surface("Translate Apple to Russian"),
            Some("Apple".to_owned()),
        );
    }

    #[test]
    fn extracts_unquoted_russian_surface() {
        assert_eq!(
            extract_unquoted_translation_surface("переведи яблоко на английский"),
            Some("яблоко".to_owned()),
        );
    }

    #[test]
    fn ignores_trailing_punctuation() {
        assert_eq!(
            extract_unquoted_translation_surface("translate apple to russian."),
            Some("apple".to_owned()),
        );
    }

    #[test]
    fn returns_none_for_quoted_prompts() {
        assert_eq!(
            extract_unquoted_translation_surface("translate \"apple\" to russian"),
            None,
        );
    }

    #[test]
    fn returns_none_without_verb() {
        assert_eq!(extract_unquoted_translation_surface("what is apple"), None,);
    }

    #[test]
    fn returns_none_without_preposition() {
        assert_eq!(
            extract_unquoted_translation_surface("translate apple"),
            None,
        );
    }
}
