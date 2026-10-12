//! Bind artifact type to the action/target header, excluding literal values.
use crate::agentic_coding::write_request::{
    first_action_cue_start, first_content_lead_end, ranked_bindings, tokens,
};

/// Preserve original byte positions while excluding destination and value bytes.
fn delivery_header<'a>(sentence: &'a str, target: &str) -> Option<(&'a str, String)> {
    let all = tokens(sentence);
    let action = first_action_cue_start(&all).unwrap_or(0);
    let binding = ranked_bindings(&all)
        .into_iter()
        .find(|item| item.path == target)?;
    let token = &all[binding.index];
    let start = token.start.checked_sub(action)?;
    let end = token.end.checked_sub(action)?;
    let raw = &sentence[action..];
    let mut masked = raw.to_owned();
    masked.replace_range(start..end, &" ".repeat(end - start));
    for segment in crate::normal_markov::quoted_segment_spans(&masked) {
        masked.replace_range(
            segment.start..segment.end,
            &" ".repeat(segment.end - segment.start),
        );
    }
    let mut lowered = String::new();
    let mut offsets = Vec::new();
    for (offset, character) in masked.char_indices() {
        let lower: String = character.to_lowercase().collect();
        offsets.extend(std::iter::repeat_n(offset, lower.len()));
        lowered.push_str(&lower);
    }
    let limit = first_content_lead_end(&lowered).map_or(masked.len(), |(start, _)| offsets[start]);
    Some((&raw[..limit], masked[..limit].to_owned()))
}

/// Source-type evidence is local to the actual action/target header.
pub(super) fn names_callable_artifact(sentence: &str, target: &str) -> bool {
    let Some((raw, header)) = delivery_header(sentence, target) else {
        return false;
    };
    let normalized = crate::engine::normalize_prompt(&header);
    let lexicon = crate::seed::lexicon();
    let operand = normalized.split_whitespace().find(|word| {
        lexicon.mentions_role("coding-source-artifact-kind", word)
            || lexicon.mentions_role("evidence-report-artifact-kind", word)
    });
    if operand.is_some_and(|word| lexicon.mentions_role("coding-source-artifact-kind", word))
        && (lexicon.mentions_role("coding_request_verb", &normalized)
            || lexicon.mentions_role("coding_member_add_action", &normalized))
    {
        return true;
    }
    let Some(call) = crate::agentic_coding::module_function::signature(&header) else {
        return false;
    };
    if !crate::agentic_coding::module_function::paths_in(&raw[call.at..])
        .iter()
        .any(|path| path == target)
    {
        return false;
    }
    let prefix = crate::engine::normalize_prompt(&header[..call.at]);
    lexicon.mentions_role("coding_request_verb", &prefix)
        || lexicon.mentions_role("coding_member_add_action", &prefix)
}
