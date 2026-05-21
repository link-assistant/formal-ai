//! Translation-prompt parsing helpers.
//!
//! These are the surface-extraction routines that bridge the natural-
//! language prompt and the [`crate::translation::TranslationPipeline`].
//! Quoted-fragment extraction lives in [`crate::solver_helpers`]; this
//! module focuses on the unquoted variant introduced for issue #216
//! (`translate apple to russian`).

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

    extract_between_prefix_and_marker(trimmed, &lower, "translate ", " to ")
        .or_else(|| extract_between_prefix_and_marker(trimmed, &lower, "переведи ", " на "))
        .or_else(|| extract_hindi_unquoted_surface(trimmed, &lower))
        .or_else(|| extract_chinese_unquoted_surface(trimmed, &lower))
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

fn extract_hindi_unquoted_surface(original: &str, lower: &str) -> Option<String> {
    if !lower.contains("अनुवाद") {
        return None;
    }
    for target_marker in [" में अनुवाद", " मे अनुवाद"] {
        let Some(target_offset) = lower.find(target_marker) else {
            continue;
        };
        let before_target = &lower[..target_offset];
        for surface_marker in [" का ", " को "] {
            if let Some(surface_end) = before_target.rfind(surface_marker) {
                return clean_unquoted_surface(&original[..surface_end]);
            }
        }
    }
    None
}

fn extract_chinese_unquoted_surface(original: &str, lower: &str) -> Option<String> {
    const COMMAND_PREFIXES: &[&str] = &["把", "将"];
    const TRANSLATE_PREFIXES: &[&str] = &["翻译", "翻譯"];
    const COMMAND_MARKERS: &[&str] = &["翻译成", "翻译为", "翻译到", "翻譯成", "翻譯為", "翻譯到"];
    const TARGET_MARKERS: &[&str] = &["成", "为", "為", "到"];

    for prefix in COMMAND_PREFIXES {
        let Some(rest) = lower.strip_prefix(prefix) else {
            continue;
        };
        if let Some((marker_offset, _)) = first_marker(rest, COMMAND_MARKERS) {
            let start = prefix.len();
            let end = start + marker_offset;
            return clean_unquoted_surface(&original[start..end]);
        }
    }

    for prefix in TRANSLATE_PREFIXES {
        let Some(rest) = lower.strip_prefix(prefix) else {
            continue;
        };
        if let Some((marker_offset, _)) = first_marker(rest, TARGET_MARKERS) {
            let start = prefix.len();
            let end = start + marker_offset;
            return clean_unquoted_surface(&original[start..end]);
        }
    }

    None
}

fn first_marker<'a>(text: &str, markers: &'a [&str]) -> Option<(usize, &'a str)> {
    markers
        .iter()
        .filter_map(|marker| text.find(marker).map(|offset| (offset, *marker)))
        .min_by_key(|(offset, _)| *offset)
}

fn clean_unquoted_surface(candidate: &str) -> Option<String> {
    let cleaned = candidate.trim();
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
    fn extracts_unquoted_hindi_surface() {
        assert_eq!(
            extract_unquoted_translation_surface("apple का हिंदी में अनुवाद करो"),
            Some("apple".to_owned()),
        );
        assert_eq!(
            extract_unquoted_translation_surface("सेब को अंग्रेजी में अनुवाद करो"),
            Some("सेब".to_owned()),
        );
    }

    #[test]
    fn extracts_unquoted_chinese_surface() {
        assert_eq!(
            extract_unquoted_translation_surface("把 apple 翻译成中文"),
            Some("apple".to_owned()),
        );
        assert_eq!(
            extract_unquoted_translation_surface("将苹果翻译成英文"),
            Some("苹果".to_owned()),
        );
        assert_eq!(
            extract_unquoted_translation_surface("翻译 apple 成中文"),
            Some("apple".to_owned()),
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
