//! Seed-driven recognition helpers shared by execution-capability surfaces.

use crate::language::detect as detect_language;
use crate::seed;

/// Preserve an implementation target explicitly named by a request even when
/// it is not in the local language catalog yet.
///
/// Programming-language and compiler names conventionally travel unchanged
/// through translated prose. A title-cased ASCII token is therefore useful
/// evidence without requiring a product list: a newly published target can be
/// named honestly before discovery teaches the catalog how to execute it.
pub fn explicit_named_execution_target(prompt: &str) -> Option<String> {
    let language = detect_language(prompt).slug();
    let marker_heads = seed::response_values_for("code_execution_request_markers", language)
        .unwrap_or_default()
        .into_iter()
        .filter_map(|marker| {
            marker
                .split(|character: char| !character.is_ascii_alphanumeric())
                .find(|token| !token.is_empty())
                .map(str::to_lowercase)
        })
        .collect::<Vec<_>>();
    prompt
        .split(|character: char| !character.is_ascii_alphanumeric())
        .filter(|token| token.len() > 1)
        .filter(|token| {
            let mut characters = token.chars();
            characters.next().is_some_and(char::is_uppercase) && characters.all(char::is_lowercase)
        })
        .rfind(|token| {
            !marker_heads
                .iter()
                .any(|head| head == &token.to_lowercase())
        })
        .map(str::to_owned)
}
