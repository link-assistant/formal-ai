use super::*;

pub fn record_decomposition(
    log: &mut EventLog,
    prompt: &str,
    max_depth: u8,
) -> Vec<DecomposedSubImpulse> {
    if max_depth == 0 {
        return Vec::new();
    }

    // Multiple explicit questions are independently actionable even when the
    // whole prompt happens to match the first one. Previously that whole-prompt
    // match (for example, `capabilities`) suppressed decomposition and silently
    // discarded every following question. Mixed sentences keep their existing
    // whole-prompt semantics: "Hello. Prove ..." is one proof request, and a
    // greeting followed by an identity question remains an identity request.
    let independent_parts = independent_actionable_segments(prompt);
    if independent_parts.len() > 1
        && independent_parts.iter().all(|part| {
            part.trim_end()
                .chars()
                .next_back()
                .is_some_and(|ch| matches!(ch, '?' | '？'))
        })
    {
        return record_sub_impulses(log, independent_parts, true);
    }
    // A question asking the assistant's opinion is one utterance: the clause
    // after its lead ("Как ты думаешь, …") is what the opinion is about, not a
    // second request (the browser worker splits only at sentence ends).
    if formal_ai::rule_interpreter::handler_matches("opinion_question", prompt) {
        return Vec::new();
    }

    if courtesy_context_for_single_directive(&independent_parts) {
        return Vec::new();
    }

    let language = detect_language(prompt);
    let whole_intent = formalize_intent(prompt, language.slug(), None);
    if (whole_intent.route.is_none() || whole_intent.kind == IntentKind::Courtesy)
        && independent_parts.len() > 1
    {
        return record_sub_impulses(log, independent_parts, true);
    }

    let lower = prompt.to_lowercase();
    let triggers = [" and ", " with tests", " with benchmarks", "; "];
    if !triggers.iter().any(|trigger| lower.contains(trigger)) {
        return Vec::new();
    }

    let parts: Vec<String> = prompt
        .split([',', ';'])
        .flat_map(|chunk| chunk.split(" and "))
        .flat_map(|chunk| chunk.split(" with "))
        .map(str::trim)
        .filter(|chunk| !chunk.is_empty())
        .map(str::to_owned)
        .collect();
    record_sub_impulses(log, parts, false)
}

/// Return true when the normalized prompt asks for a script or code to be
/// *authored* — the author verb ([`ROLE_SCRIPT_AUTHORING_VERB`], carried by the
/// `write` meaning) paired with a script-or-code artifact noun
/// ([`ROLE_SCRIPT_OR_CODE_ARTIFACT`], carried by `script` and `code`) — in any
/// supported language. No natural-language word is hardcoded here; the lexicon
/// answers which surface forms evidence each role.
///
/// Defers to the parametric write-program route for prompts that name the broad
/// program genus ([`ROLE_PROGRAM_GENUS`]) or the canonical hello-world archetype
/// ([`ROLE_HELLO_WORLD_REFERENCE`]), so those keep their richer formalization
/// instead of collapsing into a bare script.
///
/// Defers again — see [`names_no_task_beyond_the_minimal_script`] — when the
/// prompt names a task the minimal script cannot render.
///
/// [`ROLE_SCRIPT_AUTHORING_VERB`]: crate::seed::ROLE_SCRIPT_AUTHORING_VERB
/// [`ROLE_SCRIPT_OR_CODE_ARTIFACT`]: crate::seed::ROLE_SCRIPT_OR_CODE_ARTIFACT
/// [`ROLE_PROGRAM_GENUS`]: crate::seed::ROLE_PROGRAM_GENUS
/// [`ROLE_HELLO_WORLD_REFERENCE`]: crate::seed::ROLE_HELLO_WORLD_REFERENCE
pub fn is_write_script_request(prompt: &str, normalized: &str) -> bool {
    use crate::seed::{
        ROLE_HELLO_WORLD_REFERENCE, ROLE_PROGRAM_GENUS, ROLE_SCRIPT_AUTHORING_VERB,
        ROLE_SCRIPT_OR_CODE_ARTIFACT,
    };
    let lexicon = crate::seed::lexicon();
    // The parametric write-program route owns the broad program genus and the
    // canonical hello-world archetype; step aside for those.
    if lexicon.mentions_role(ROLE_PROGRAM_GENUS, normalized)
        || lexicon.mentions_role(ROLE_HELLO_WORLD_REFERENCE, normalized)
    {
        return false;
    }
    // Author a script: the write verb plus a script-or-code artifact noun.
    if !(lexicon.mentions_role(ROLE_SCRIPT_AUTHORING_VERB, normalized)
        && lexicon.mentions_role(ROLE_SCRIPT_OR_CODE_ARTIFACT, normalized))
    {
        return false;
    }
    names_no_task_beyond_the_minimal_script(prompt, normalized)
}
