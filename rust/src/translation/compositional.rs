//! Complete lexical-span composition. Unknown or ambiguous units fail closed.
//! This preserves surface order; it does not infer syntax or contextual senses.

pub(super) fn target_surface_is_complete(surface: &str, target: &str) -> bool {
    let text = surface.trim();
    if !text.chars().any(char::is_alphabetic)
        || crate::formalization::text_statements::structural_rule_values(
            "target-surface",
            "reject-fragment",
        )
        .iter()
        .any(|fragment| text.contains(fragment.as_str()))
    {
        return false;
    }
    let cyrillic = text.chars().any(|c| ('\u{0400}'..='\u{052f}').contains(&c));
    let devanagari = text.chars().any(|c| ('\u{0900}'..='\u{097f}').contains(&c));
    let han = text.chars().any(|c| ('\u{3400}'..='\u{9fff}').contains(&c));
    match target {
        "en" | "es" | "fr" | "de" | "it" | "pt" => !cyrillic && !devanagari && !han,
        "ru" => cyrillic && !devanagari && !han,
        "hi" => devanagari && !cyrillic && !han,
        "zh" => han && !cyrillic && !devanagari,
        _ => true,
    }
}

pub(super) fn compose_spans(
    surface: &str,
    mut lookup: impl FnMut(&str) -> Option<(String, Vec<String>)>,
) -> Option<(String, Vec<String>)> {
    let parts: Vec<&str> = surface
        .split_inclusive(|c: char| c.is_whitespace() || c == '-' || c == ',')
        .collect();
    if !(2..=8).contains(&parts.len()) {
        return None;
    }
    let mut result = String::new();
    let mut provenance = Vec::new();
    let mut index = 0;
    while index < parts.len() {
        let mut found = None;
        for end in ((index + 1)..=parts.len()).rev() {
            let span = parts[index..end].concat();
            let core = span.trim_end_matches(|c: char| c.is_whitespace() || c == '-' || c == ',');
            if core.is_empty() {
                continue;
            }
            if let Some((translated, evidence)) = lookup(core) {
                let suffix = &span[core.len()..];
                found = Some((end, translated + suffix, evidence));
                break;
            }
        }
        let (end, translated, evidence) = found?;
        result.push_str(&translated);
        provenance.extend(evidence);
        index = end;
    }
    Some((result, provenance))
}

pub(super) fn translate_known_spans<T: super::HttpClient + ?Sized>(
    pipeline: &super::TranslationPipeline<'_, T>,
    surface: &str,
    source_lang: &str,
    target_lang: &str,
) -> Option<(String, Vec<String>)> {
    compose_spans(surface, |span| {
        if let Some((slug, translated)) =
            super::seed_compositional_translation(span, source_lang, target_lang)
        {
            return target_surface_is_complete(translated, target_lang).then(|| {
                (
                    translated.to_owned(),
                    vec![format!("seed:{slug}:{source_lang}->{target_lang}:{span}")],
                )
            });
        }
        // Only single units recurse: a failed phrase cannot refetch itself.
        if span
            .chars()
            .any(|c| c.is_whitespace() || c == '-' || c == ',')
        {
            return None;
        }
        let translated = pipeline.translate(span, source_lang, target_lang).ok()?;
        if translated.candidates.len() != 1 {
            return None;
        }
        Some((
            translated.primary_surface()?.to_owned(),
            translated.provenance,
        ))
    })
}

pub(super) fn fill_missing_candidates<T: super::HttpClient + ?Sized>(
    pipeline: &super::TranslationPipeline<'_, T>,
    surface: &str,
    source_lang: &str,
    target_lang: &str,
    candidates: &mut Vec<super::WiktionaryCandidate>,
    provenance: &mut Vec<String>,
) {
    if !candidates.is_empty() {
        return;
    }
    if let Some((translated, evidence)) =
        translate_known_spans(pipeline, surface, source_lang, target_lang)
    {
        provenance.extend(evidence);
        provenance.push(format!(
            "complete-lexical-spans:{source_lang}->{target_lang}"
        ));
        candidates.push(super::WiktionaryCandidate {
            surface: translated,
            qualifier: None,
        });
    }
}

#[cfg(test)]
#[path = "../../tests/source/source_tests/translation/compositional-tests.rs"]
mod tests;
