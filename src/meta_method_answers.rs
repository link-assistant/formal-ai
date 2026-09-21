//! Conversation-facing answers for registry-backed dispatch.
//!
//! The dispatcher in `meta_method_dispatch.rs` decides which method answers;
//! this module renders the answers that speak about the conversation itself:
//! the response language the conversation has already established (plan 10
//! leaf 15, issue #724), a demonstration of a named response language, an
//! explanation of the previous turn, and the honest capability gap with its
//! stable request anchors. Split from `meta_method_dispatch.rs` so both stay
//! under the 1000-line gate enforced by `scripts/check-file-size.rs`.

use crate::engine::SymbolicAnswer;
use crate::event_log::EventLog;
use crate::method_registry::ResponseLanguageVariant;
use crate::solver::{ConversationRole, ConversationTurn};
use crate::solver_handlers::{
    finalize_simple, try_concept_lookup_with_response_language,
    try_pattern_inference_with_response_language,
};
use crate::translation::detect_response_language;

pub fn capability_gap(
    prompt: &str,
    log: &mut EventLog,
    needed: &str,
    missing: &str,
) -> SymbolicAnswer {
    log.append_fields(
        "capability_gap",
        &[("needed", needed), ("missing", missing)],
    );
    let mut body = seeded_runtime_text(
        "capability_gap",
        &[("needed", needed), ("missing", missing)],
    );
    let anchors = request_anchors(prompt);
    if !anchors.is_empty() {
        let rendered = anchors
            .iter()
            .map(|anchor| format!("`{anchor}`"))
            .collect::<Vec<_>>()
            .join(", ");
        log.append("capability_gap:request_anchors", anchors.join("|"));
        body.push_str(&seeded_runtime_text(
            "capability_gap_anchors",
            &[("anchors", &rendered)],
        ));
    }
    finalize_simple(
        prompt,
        log,
        "capability_gap",
        "response:capability_gap",
        &body,
        1.0,
    )
}

/// Stable machine-addressable values that must survive an honest capability
/// gap. A chat surface cannot open the requested workspace, but it must not
/// discard an exact commit, path or code identifier before handing the request
/// to a capable client.
fn request_anchors(prompt: &str) -> Vec<&str> {
    let mut anchors = Vec::new();
    for token in prompt.split(|character: char| {
        !(character.is_ascii_alphanumeric()
            || matches!(character, '/' | '.' | '_' | '-' | ':' | '@'))
    }) {
        let token = token.trim_matches(['.', ',', ':', ';']);
        let exact_commit =
            token.len() == 40 && token.chars().all(|character| character.is_ascii_hexdigit());
        let named_path =
            token.contains('/') && !token.starts_with("http://") && !token.starts_with("https://");
        let code_identifier = token.contains('_')
            && token
                .chars()
                .all(|character| character.is_ascii_alphanumeric() || character == '_');
        // A bare dotted filename (`alpha.txt`, `main.rs`) is an addressable
        // workspace object with no slash to name it by: a refusal that drops
        // it leaves the capable client nothing to open. The shape keeps the
        // anchors honest against prose: one dot, a stem with a letter (a
        // decimal number is not a filename), and a two-to-five character
        // extension (so `e.g` stays prose and `1.5.0` stays a version).
        let dotted_filename = !token.contains('/')
            && token.split('.').count() == 2
            && token.split('.').all(|part| !part.is_empty())
            && {
                let mut parts = token.split('.');
                let stem = parts.next().unwrap_or_default();
                let extension = parts.next().unwrap_or_default();
                (2..=5).contains(&extension.len())
                    && extension.chars().any(char::is_alphabetic)
                    && extension
                        .chars()
                        .all(|character| character.is_ascii_alphanumeric())
                    && stem.chars().next().is_some_and(char::is_alphanumeric)
                    && stem.chars().any(char::is_alphabetic)
            };
        if (exact_commit || named_path || code_identifier || dotted_filename)
            && !anchors.contains(&token)
        {
            anchors.push(token);
        }
    }
    anchors
}

/// The response language the conversation has already established, read from
/// the user's own turns (plan 10 leaf 15, issue #724).
///
/// The marker is the seed-grounded response-language role
/// ([`detect_response_language`]), so this holds no phrase table of its own and
/// reads the request the same way the demonstration route does. Only user turns
/// speak: an assistant turn merely obeyed.
pub fn established_response_language(history: &[ConversationTurn]) -> Option<&'static str> {
    history
        .iter()
        .rev()
        .filter(|turn| turn.role == ConversationRole::User)
        .find_map(|turn| detect_response_language(&turn.content.to_lowercase()))
}

pub fn response_language_demonstration(
    prompt: &str,
    normalized: &str,
    log: &mut EventLog,
) -> Option<SymbolicAnswer> {
    let target = detect_response_language(normalized)?;
    let canonical = crate::language::language_name(target).unwrap_or(target);
    let surface = crate::seed::lexicon()
        .meanings_with_role(crate::seed::ROLE_RESPONSE_LANGUAGE_MARKER)
        .find(|meaning| {
            meaning.defined_by.iter().any(|slug| {
                crate::language::language_for_concept_slug(slug)
                    == crate::language::from_slug(target)
            })
        })
        .and_then(|meaning| meaning.words().find(|word| normalized.contains(word)))
        .unwrap_or(canonical);
    let demonstration =
        crate::seed::response_for("greeting", target).unwrap_or_else(|| canonical.to_owned());
    log.append("language_to", target.to_owned());
    let body = seeded_runtime_text(
        "response_language_demonstration",
        &[
            ("canonical", canonical),
            ("surface", surface),
            ("demonstration", &demonstration),
        ],
    );
    Some(finalize_simple(
        prompt,
        log,
        "response_language_demonstration",
        "response:response_language_demonstration",
        &body,
        1.0,
    ))
}

pub fn explain_previous_turn(prompt: &str, log: &mut EventLog) -> SymbolicAnswer {
    let body = crate::solver_helpers::last_assistant_turn(log).map_or_else(
        || seeded_runtime_text("explain_previous_turn_empty", &[]),
        |previous| seeded_runtime_text("explain_previous_turn_answer", &[("previous", previous)]),
    );
    finalize_simple(
        prompt,
        log,
        "explain_previous_turn",
        "response:explain_previous_turn",
        &body,
        0.9,
    )
}

pub fn try_response_language_variant(
    variant: Option<ResponseLanguageVariant>,
    prompt: &str,
    normalized: &str,
    log: &mut EventLog,
    language: &str,
) -> Option<SymbolicAnswer> {
    match variant? {
        ResponseLanguageVariant::ConceptLookup => {
            try_concept_lookup_with_response_language(prompt, log, Some(language))
        }
        ResponseLanguageVariant::PatternInference => {
            try_pattern_inference_with_response_language(prompt, normalized, log, language)
        }
    }
}

fn seeded_runtime_text(intent: &str, values: &[(&str, &str)]) -> String {
    crate::seed::render_response(intent, "en", values).unwrap_or_else(|| intent.to_owned())
}
