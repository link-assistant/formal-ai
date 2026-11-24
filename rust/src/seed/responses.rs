//! Localized response records, variants, fallbacks and template rendering.

use super::parser::split_pipe_list;
use super::{RESPONSE_FILES, parse_lino};

/// A single response variant for an intent in a particular language.
#[derive(Debug, Clone)]
pub struct ResponseRecord {
    pub id: String,
    pub intent: String,
    pub language: String,
    pub text: String,
    pub variants: Vec<String>,
}

/// Parse `multilingual-responses.lino` into structured records.
#[must_use]
pub fn multilingual_responses() -> Vec<ResponseRecord> {
    let mut out = Vec::new();
    for source in RESPONSE_FILES {
        let tree = parse_lino(source);
        if let Some(root) = tree.children.first() {
            for entry in root.children.iter().filter(|c| c.name == "response") {
                let intent = entry.find_child_value("intent").to_string();
                let language = entry.find_child_value("language").to_string();
                let text = entry.find_child_value("text").to_string();
                let variants = entry
                    .children
                    .iter()
                    .filter(|child| child.name == "variant")
                    .map(|child| child.id.clone())
                    .collect();
                if intent.is_empty() || language.is_empty() {
                    continue;
                }
                out.push(ResponseRecord {
                    id: entry.id.clone(),
                    intent,
                    language,
                    text,
                    variants,
                });
            }
        }
    }
    out
}

/// Look up a localized response by intent and language, returning `None` if
/// the seed has no matching record.
#[must_use]
pub fn response_for(intent: &str, language: &str) -> Option<String> {
    for record in multilingual_responses() {
        if record.intent == intent && record.language == language {
            return Some(record.text);
        }
    }
    None
}

/// Look up a localized response and select one of its declared variants with
/// a prompt-stable hash.
///
/// Replaying the same prompt is deterministic while semantically equivalent
/// phrasings are not collapsed to one canned answer.
#[must_use]
pub fn response_variant_for(intent: &str, language: &str, prompt: &str) -> Option<String> {
    multilingual_responses()
        .into_iter()
        .find(|record| record.intent == intent && record.language == language)
        .map(|record| {
            if record.variants.is_empty() {
                return record.text;
            }
            let hash = prompt
                .trim()
                .as_bytes()
                .iter()
                .fold(0xcbf2_9ce4_8422_2325_u64, |hash, byte| {
                    (hash ^ u64::from(*byte)).wrapping_mul(0x0000_0100_0000_01b3)
                });
            let variant_count = u64::try_from(record.variants.len()).unwrap_or(u64::MAX);
            let index = usize::try_from(hash % variant_count).unwrap_or_default();
            record.variants[index].clone()
        })
}

/// Look up one localized response and fill its `{name}` slots in one pass.
///
/// The lookup is [`localized_response`]'s; filling once means a value that
/// itself contains braces — a user's task text, a quoted list item — is never
/// re-filled (issue #918).
#[must_use]
pub fn render_localized_once(intent: &str, language: &str, values: &[(&str, &str)]) -> String {
    fill_template_once(
        &localized_response(intent, language).unwrap_or_default(),
        values,
    )
}

/// Fill the `{name}` slots of `template` in a single left-to-right pass;
/// a slot whose name is not among `values` stays as written.
#[must_use]
pub fn fill_template_once(template: &str, values: &[(&str, &str)]) -> String {
    let mut out = String::with_capacity(template.len());
    let mut rest = template;
    while let Some(open) = rest.find('{') {
        out.push_str(&rest[..open]);
        let after = &rest[open + 1..];
        let filled = after.find('}').and_then(|close| {
            values
                .iter()
                .find(|(name, _)| after.get(..close) == Some(*name))
                .map(|(_, value)| (close, *value))
        });
        if let Some((close, value)) = filled {
            out.push_str(value);
            rest = &after[close + 1..];
        } else {
            out.push('{');
            rest = after;
        }
    }
    out.push_str(rest);
    out
}

/// Look up one response and substitute its named template fields.
#[must_use]
pub fn render_response(intent: &str, language: &str, values: &[(&str, &str)]) -> Option<String> {
    response_for(intent, language).map(|mut rendered| {
        for (name, value) in values {
            rendered = rendered.replace(&format!("{{{name}}}"), value);
        }
        rendered
    })
}

/// Look up a localized response, applying the registry's `explicit_gap`
/// fallback policy (`data/seed/languages.lino`).
///
/// Issue #706: a language that the detection registry knows but a given intent
/// has no localized text for is a *gap*, not an English prompt. Handlers that
/// used to write `response_for(intent, language).or_else(|| response_for(intent,
/// "en"))` silently answered a Spanish speaker in English; going through this
/// helper instead prefers the seed's `language unknown` variant — the record
/// that says out loud that the language is unsupported — before falling back to
/// English as a last resort.
#[must_use]
pub fn localized_response(intent: &str, language: &str) -> Option<String> {
    if let Some(text) = response_for(intent, language) {
        return Some(text);
    }
    if crate::language::from_slug(language).is_some()
        && let Some(text) = response_for(intent, "unknown")
    {
        return Some(text);
    }
    response_for(intent, "en")
}

/// Look up a localized response whose `text` is a native reference list.
///
/// Scalar text is returned as a one-element vector, which keeps this helper
/// useful for seed records that may migrate between scalar and list forms.
#[must_use]
pub fn response_values_for(intent: &str, language: &str) -> Option<Vec<String>> {
    response_for(intent, language).map(|text| split_pipe_list(&text))
}
