//! Complete seeded additive scope; composers retain payload interpretation.
use super::literal_request::parse_write_contract;
use crate::agentic_coding::write_request::{
    bare_surfaces, clean_cue_token, clean_path_token, first_action_cue_end, first_action_cue_start,
    looks_like_file_path, tokens,
};
use crate::normal_markov::{quote_fault, quoted_segment_spans};

/// Mirrors `ownsAdditiveScope`; this syntactic preflight does not grant effect authority.
pub fn owns_additive_scope(request: &str) -> bool {
    if quote_fault(request).is_some() {
        return false;
    }
    let mut spans: Vec<(usize, usize)> = quoted_segment_spans(request)
        .into_iter()
        .map(|span| (span.start, span.end))
        .collect();
    if let Some(contract) = parse_write_contract(request) {
        if contract.target_span.end <= contract.payload.start
            && crate::agentic_coding::write_request::first_content_lead_end(
                &request[contract.target_span.end..].to_lowercase(),
            )
            .is_some()
        {
            spans.push((contract.target_span.end, contract.payload.end));
        }
    }
    if quoted_segment_spans(request).is_empty() {
        let words = tokens(request);
        let paths: Vec<_> = words
            .iter()
            .filter(|token| looks_like_file_path(&clean_path_token(token.text)))
            .collect();
        let mut leads: Vec<Vec<String>> = bare_surfaces("file_edit_line_lead")
            .into_iter()
            .map(|surface| {
                surface
                    .to_lowercase()
                    .split_whitespace()
                    .map(str::to_owned)
                    .collect()
            })
            .filter(|surface: &Vec<String>| !surface.is_empty())
            .collect();
        leads.sort_by_key(|surface| std::cmp::Reverse(surface.len()));
        let mut lead = None;
        for index in 0..words.len() {
            if let Some(surface) = leads.iter().find(|surface| {
                surface.iter().enumerate().all(|(offset, word)| {
                    words
                        .get(index + offset)
                        .is_some_and(|token| clean_cue_token(token.text) == *word)
                })
            }) {
                lead = Some(words[index + surface.len() - 1].end);
                break;
            }
        }
        if let (Some(end), [path]) = (lead, paths.as_slice()) {
            if end < path.start {
                let destinations = bare_surfaces("file_write_destination_cue");
                let segment = &request[end..path.start];
                if let Some(destination) = tokens(segment)
                    .into_iter()
                    .filter(|token| {
                        destinations.contains(
                            &token
                                .text
                                .to_lowercase()
                                .trim_end_matches(['.', '!', '?', ',', ':', ';'])
                                .to_owned(),
                        )
                    })
                    .next_back()
                {
                    spans.push((end, end + destination.start));
                }
            }
        }
    }
    for token in tokens(request) {
        if looks_like_file_path(&clean_path_token(token.text)) {
            spans.push((token.start, token.end));
        }
    }
    let owned_end = spans.iter().map(|(_, end)| *end).max().unwrap_or(0);
    if owned_end > 0
        && !request[owned_end..]
            .chars()
            .all(|character| " \t\n\r\u{000b}\u{000c}.!?。！？।,，:：;；".contains(character))
    {
        return false;
    }
    let mut bytes = request.as_bytes().to_vec();
    for (start, end) in spans {
        bytes[start..end].fill(b' ');
    }
    let Ok(view) = String::from_utf8(bytes) else {
        return false;
    };
    if view.chars().any(|character| {
        (character.is_whitespace() || character == '\u{feff}')
            && !" \t\n\r\u{000b}\u{000c}".contains(character)
    }) {
        return false;
    }
    let mut remaining = crate::engine::normalize_prompt(&view);
    if remaining
        .split(['.', '!', '?', '。', '！', '？', '।', ';', '；'])
        .filter(|part| !part.trim().is_empty())
        .count()
        > 1
    {
        return false;
    }
    let words = tokens(&remaining);
    if let Some((start, end)) = first_action_cue_start(&words).zip(first_action_cue_end(&words)) {
        remaining.replace_range(start..end, &" ".repeat(end - start));
    }
    let mut roles = vec![
        "request_function_word",
        "enumeration_cue",
        "file_edit_position_end",
        "file_edit_position_start",
        "file_edit_line_lead",
        "file_edit_blank_line",
        "file_write_destination_cue",
        "file_edit_target_cue",
        "file_edit_joiner_cue",
        "file_declared_noun",
        "file_contents_source_cue",
    ];
    if crate::agentic_coding::markdown_section::section_scope(request, &view.to_lowercase())
        .is_some()
    {
        roles.push("file_section_noun");
        let mut actions: Vec<String> = bare_surfaces("coding_member_add_action")
            .into_iter()
            .map(|surface| crate::engine::normalize_prompt(&surface))
            .collect();
        actions.sort_by_key(|surface| std::cmp::Reverse(surface.len()));
        if let Some(action) = actions.iter().find(|surface| {
            remaining.starts_with(surface.as_str())
                && remaining[surface.len()..]
                    .chars()
                    .next()
                    .is_none_or(|character| !character.is_alphanumeric() && character != '_')
        }) {
            remaining.replace_range(..action.len(), &" ".repeat(action.len()));
        }
    }
    let mut surfaces: Vec<String> = roles
        .iter()
        .flat_map(|role| bare_surfaces(role))
        .map(|surface| crate::engine::normalize_prompt(&surface))
        .filter(|surface| !surface.is_empty())
        .collect();
    surfaces.sort_by_key(|surface| std::cmp::Reverse(surface.len()));
    surfaces.dedup();
    let Ok(word) = regex::Regex::new(r"[\p{L}\p{N}_]") else {
        return false;
    };
    for surface in surfaces {
        let mut offset = 0;
        while let Some(found) = remaining[offset..].find(&surface) {
            let start = offset + found;
            let end = start + surface.len();
            let before = remaining[..start].chars().next_back();
            let after = remaining[end..].chars().next();
            let is_word = |character: Option<char>| {
                character.is_some_and(|value| word.is_match(&value.to_string()))
            };
            if !is_word(before) && !is_word(after) {
                remaining.replace_range(start..end, &" ".repeat(surface.len()));
            }
            offset = end;
        }
    }
    regex::Regex::new(r"^[\s.!?。！？।,，:：;；()\[\]{}\-—–]*$")
        .is_ok_and(|grammar| grammar.is_match(&remaining))
}
