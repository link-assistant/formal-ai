//! Reading a request's `write_program` parameters — the task it names and the
//! implementation language it asks for.
//!
//! Split out of `intent_formalization.rs` so the routing veto and the
//! history-recovery reading stay side by side and readable.

use std::collections::BTreeMap;

use super::detected_program_modifiers;

/// The `write_program` parameters the *request being routed* names, or `None`
/// when the request is not a request to write a program at all.
///
/// Issue #906: "Create a file named hello.txt containing Hello World, in
/// JavaScript." names a *file* — a path and its content — and the task alias
/// table matched only because the content happens to read "hello world".
/// Answering it with `console.log("Hello, world!")` discards both the path and
/// the content and reports a program that was never asked for. A request whose
/// artefact is a file is not a `write_program` request; the file-write shape is
/// recognized by the same seed-driven parse that composes the plan
/// (`data/seed/meanings-file-write.lino`), so this declines exactly the requests
/// the file planner can actually carry out.
///
/// The veto is a *routing* decision about the current turn, which is why it does
/// not live in [`write_program_parameters`]: recovering the active program from
/// conversation history asks a different question — "which program are we
/// talking about?" — and a turn that also names a file still answers it.
pub(super) fn requested_write_program_parameters(
    raw: &str,
    normalized: &str,
) -> Option<BTreeMap<String, String>> {
    if crate::agentic_coding::general_planner::has_file_write_intent(&raw.to_lowercase()) {
        return None;
    }
    // The same reading, applied to quoted operands: "Replace \"Hello World\"
    // with \"Bye world\"" edits text, and the task alias table matches only
    // because the words being edited happen to read "hello world". The test is
    // whether the request still names the task once the quotes are removed:
    // "Write hello world in JavaScript and replace `Hello World` with `Bye JS`"
    // does, and so it is still a request to write a program.
    let quotes_carry_the_task = !asks_for_program(normalized)
        && crate::solver_handlers::names_a_quoted_replacement(raw)
        && {
            let outside = crate::engine::normalize_prompt(
                &crate::solver_handlers::text_outside_quoted_segments(raw),
            );
            crate::coding::program_task_by_alias(&outside).is_none()
        };
    if quotes_carry_the_task {
        return None;
    }
    write_program_parameters(normalized)
}

/// Does the request name a program *as its artefact*?
///
/// Issue #386: "write a &lt;program&gt;" is recognised by *meaning*, not a hardcoded
/// per-language word list. The prompt asks for a program when it evidences a
/// `program_kind` meaning (the artefact: program / script / code / function /
/// class) *and* a `program_request` meaning (the verb: write / create / show /
/// generate / make / build). The surface words for every language live once, in
/// `data/seed/meanings.lino`; this code understands the concepts.
fn asks_for_program(normalized: &str) -> bool {
    let lexicon = crate::seed::lexicon();
    lexicon.mentions_role(crate::seed::ROLE_PROGRAM_KIND, normalized)
        && lexicon.mentions_role(crate::seed::ROLE_PROGRAM_REQUEST, normalized)
}

/// The `write_program` parameters — task and language — that `normalized` names.
pub(super) fn write_program_parameters(normalized: &str) -> Option<BTreeMap<String, String>> {
    let task = crate::coding::program_task_by_alias(normalized);
    let language = requested_program_language(normalized);
    let mentions_program_request =
        crate::seed::lexicon().mentions_role(crate::seed::ROLE_PROGRAM_REQUEST, normalized);
    let asks_for_known_language_program = language
        .as_deref()
        .is_some_and(|language| mentions_program_request && known_write_program_language(language));
    if task.is_none() && !asks_for_program(normalized) && !asks_for_known_language_program {
        return None;
    }
    let mut parameters = BTreeMap::new();
    if let Some(task) = task {
        // Issue #358: modification phrases in the same turn lower the base task
        // through the data-backed substitution pipeline so composed requests can
        // resolve directly.
        let modifiers = detected_program_modifiers(normalized);
        let task_slug = crate::program_plan::resolve_task(task.slug, &modifiers);
        parameters.insert(String::from("task"), task_slug);
    }
    if let Some(language) = language {
        parameters.insert(String::from("language"), language);
    }
    Some(parameters)
}

fn known_write_program_language(language: &str) -> bool {
    crate::implementation_language::is_known(language)
}

/// The implementation language the request names, if any.
///
/// Issue #386 introduced the seed-driven positional scan that reads an unknown
/// language name after the modifier marker; issue #906 moved the whole question
/// — including what may legally fill that position — into
/// [`crate::implementation_language`], because "in the current directory" was
/// being read as the language `the`.
fn requested_program_language(normalized: &str) -> Option<String> {
    crate::implementation_language::requested(normalized)
}
