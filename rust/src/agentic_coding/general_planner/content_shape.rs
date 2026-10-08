//! What the content of a general write request describes.
//!
//! A clause the write grammar picks as content is not always the file's bytes:
//! it may describe code to author, or an addition to the file. Split out of
//! `general_planner.rs` to keep that file under its warning band.

use super::super::write_request::{
    clean_path_token, first_action_cue_end, first_action_cue_start, first_content_lead_end,
    looks_like_file_path, tokens,
};
use crate::seed;

/// Unquoted content that names a code construct (`a function multiply(a, b)`,
/// seeded `coding_request_object`) is a description of code to write, not the
/// file's bytes: writing it as the whole file destroyed `math.mjs` (PR #1188
/// dogfooding). Content a seeded content lead introduces (`containing`, `with
/// exactly this content:`) is bytes whatever it mentions.
pub(super) fn describes_code_to_author(request: &str, content: &str) -> bool {
    if content.is_empty()
        || crate::normal_markov::quoted_segments(request)
            .iter()
            .any(|segment| segment.contains(content))
    {
        return false;
    }
    (first_content_lead_end(&request.to_lowercase()).is_none()
        && seed::lexicon().mentions_role(
            "coding_request_object",
            &crate::engine::normalize_prompt(content),
        ))
        || asks_to_author_code(&prose_around(request, content))
}

/// `Add <content> to <file>` -- the write verb is the seeded add action and
/// the content comes before the file -- names an addition to that file, never
/// its whole new content.
///
/// Writing it as the file replaced `m.test.mjs` with the sentence "an
/// assertion that add(2, 2) equals 4" (PR #1188 dogfooding). Content a seeded
/// content lead introduces (`containing`, `with exactly this content:`) is
/// still the file's bytes.
pub(super) fn names_an_addition(request: &str, content: &str, target: &str) -> bool {
    let toks = tokens(request);
    let (Some(start), Some(end)) = (first_action_cue_start(&toks), first_action_cue_end(&toks))
    else {
        return false;
    };
    if content.is_empty() || first_content_lead_end(&request.to_lowercase()).is_some() {
        return false;
    }
    request
        .find(content)
        .zip(request.find(target))
        .is_some_and(|(at, file)| at < file)
        && seed::lexicon().mentions_role(
            seed::ROLE_CODING_MEMBER_ADD_ACTION,
            &crate::engine::normalize_prompt(&request[start..end]),
        )
}

/// The request itself asks to write a code construct (`Write a Python
/// function add(a, b) … in add.py and run it with 2 and 3`), so whatever clause
/// the write grammar picks as content (`2 and 3.`) is not the file's bytes.
fn asks_to_author_code(prose: &str) -> bool {
    let normalized = crate::engine::normalize_prompt(prose);
    let lexicon = seed::lexicon();
    lexicon.mentions_role("coding_request_object", &normalized)
        && lexicon.mentions_role(seed::ROLE_CODING_REQUEST_VERB, &normalized)
}

/// The request without the content and its file paths: a path such as
/// `learned-program-rules.lino` names a file, not the code to author.
fn prose_around(request: &str, content: &str) -> String {
    let rest = request.replace(content, " ");
    tokens(&rest)
        .iter()
        .filter(|token| !looks_like_file_path(clean_path_token(token.text)))
        .map(|token| token.text)
        .collect::<Vec<_>>()
        .join(" ")
}
