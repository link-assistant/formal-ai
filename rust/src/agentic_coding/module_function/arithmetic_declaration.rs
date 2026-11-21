use super::*;

/// A finite declaration frame consumes source operands before literal routing.
pub(in super::super) fn closed_arithmetic_declaration(request: &str) -> bool {
    if !request.is_ascii() || crate::normal_markov::quote_fault(request).is_some() {
        return false;
    }
    let Some(declaration) = module_function_request(request) else {
        return false;
    };
    if declaration.test.is_some()
        || declaration.command.is_some()
        || declaration.parameters.len() != 2
        || declaration.parameters[0] == declaration.parameters[1]
    {
        return false;
    }
    let path_tokens = unquoted_path_tokens(request)
        .into_iter()
        .filter(|token| looks_like_file_path(clean_path_token(token.text)))
        .collect::<Vec<_>>();
    if path_tokens.len() != 1 || clean_path_token(path_tokens[0].text) != declaration.module {
        return false;
    }
    let Some(relative_start) = request[path_tokens[0].start..].find(&declaration.module) else {
        return false;
    };
    let path_start = path_tokens[0].start + relative_start;
    if !request[path_start + declaration.module.len()..]
        .chars()
        .all(|character| character.is_whitespace() || matches!(character, '.' | '!' | '?' | ';'))
    {
        return false;
    }
    let Some(relative_opening) = request[declaration.at..].find('(') else {
        return false;
    };
    let opening = declaration.at + relative_opening;
    let Some(relative_closing) = request[opening..].find(')') else {
        return false;
    };
    let closing = opening + relative_closing;
    if closing >= path_start {
        return false;
    }
    let prefix = &request[..declaration.at];
    let prefix_tokens = tokens(prefix);
    let Some(action_end) = first_action_cue_end(&prefix_tokens) else {
        return false;
    };
    if first_action_cue_start(&prefix_tokens) != Some(0) {
        return false;
    }
    let action = crate::engine::normalize_prompt(&prefix[..action_end]);
    if !["coding_request_verb", "coding_member_add_action"]
        .iter()
        .any(|role| {
            bare_surfaces(role)
                .iter()
                .any(|surface| crate::engine::normalize_prompt(surface) == action)
        })
    {
        return false;
    }
    let covered_words = |text: &str, roles: &[&str]| {
        let allowed = roles
            .iter()
            .flat_map(|role| bare_surfaces(role))
            .flat_map(|surface| {
                crate::engine::normalize_prompt(&surface)
                    .split_ascii_whitespace()
                    .map(str::to_owned)
                    .collect::<Vec<_>>()
            })
            .collect::<std::collections::BTreeSet<_>>();
        tokens(text)
            .iter()
            .all(|token| allowed.contains(&crate::engine::normalize_prompt(token.text)))
    };
    if !crate::seed::lexicon().mentions_role("coding_request_object", prefix)
        || !covered_words(
            &prefix[action_end..],
            &["request_function_word", "coding_request_object"],
        )
    {
        return false;
    }
    let suffix = &request[closing + 1..path_start];
    let suffix_tokens = tokens(suffix);
    let Some(returned) = suffix_tokens.iter().find(|token| {
        bare_surfaces("coding_return_action").iter().any(|surface| {
            crate::engine::normalize_prompt(surface) == crate::engine::normalize_prompt(token.text)
        })
    }) else {
        return false;
    };
    if !covered_words(&suffix[..returned.start], &["request_function_word"]) {
        return false;
    }
    let Ok(pattern) =
        regex::Regex::new(r"^\s*([A-Za-z_$][\w$]*)\s*([+*/%-])\s*([A-Za-z_$][\w$]*)\s+(.+?)\s*$")
    else {
        return false;
    };
    let Some(expression) = pattern.captures(&suffix[returned.end..]) else {
        return false;
    };
    if !declaration
        .parameters
        .iter()
        .any(|parameter| parameter == &expression[1])
        || !declaration
            .parameters
            .iter()
            .any(|parameter| parameter == &expression[3])
    {
        return false;
    }
    ["file_edit_target_cue", "file_write_destination_cue"]
        .iter()
        .flat_map(|role| bare_surfaces(role))
        .any(|surface| {
            crate::engine::normalize_prompt(&surface)
                == crate::engine::normalize_prompt(&expression[4])
        })
}
