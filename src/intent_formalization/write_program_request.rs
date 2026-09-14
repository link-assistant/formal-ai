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
    // Issue #862 / #1021: "мне нужен код" names code as the artefact it wants
    // produced, and nothing else. It is the same request as "write me some
    // code" with a different asking verb, and the asking verbs live in the
    // lexicon under [`crate::seed::ROLE_SCRIPT_AUTHORING_VERB`] rather than
    // under the narrower `program_request`. Reading it as a `write_program`
    // request with no parameters is what lets the honest dead end
    // (`program_skill_gap::Shape::RequestUnspecified`) answer it — the
    // alternative, and what happened before, is a web search for the words.
    let asks_for_bare_code =
        task.is_none() && language.is_none() && names_code_and_nothing_else(normalized);
    if task.is_none()
        && !asks_for_program(normalized)
        && !asks_for_known_language_program
        && !asks_for_bare_code
    {
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

/// Does the request name code as its artefact — and name nothing else?
///
/// The recogniser this guards answers "you named neither a task nor a
/// language". It may only fire when that is *all* the request left
/// unanswered, so it subtracts everything it can account for — the asking verb
/// ([`crate::seed::ROLE_SCRIPT_AUTHORING_VERB`]), the artefact noun
/// ([`crate::seed::ROLE_SCRIPT_OR_CODE_ARTIFACT`] or, for "a program" rather
/// than "code", [`crate::seed::ROLE_PROGRAM_GENUS`]), and the closed-class words
/// a request is built out of ([`crate::seed::ROLE_REQUEST_FUNCTION_WORD`]) —
/// and requires nothing to be left over.
///
/// That subtraction is what separates "I need code" from "give me the code of
/// this repository" and "I need a code review": both of those name something
/// besides the code — a repository, a review — so the code word is a qualifier
/// rather than the artefact, and this route stands aside for whichever route
/// owns the thing that was named. No surface word is compared here; the
/// lexicon answers which words each role contributes, in every language it
/// carries.
fn names_code_and_nothing_else(normalized: &str) -> bool {
    use crate::seed::{
        ROLE_PROGRAM_GENUS, ROLE_REQUEST_FUNCTION_WORD, ROLE_SCRIPT_AUTHORING_VERB,
        ROLE_SCRIPT_OR_CODE_ARTIFACT,
    };
    let lexicon = crate::seed::lexicon();
    let names_the_artefact = lexicon.mentions_role(ROLE_SCRIPT_OR_CODE_ARTIFACT, normalized)
        || lexicon.mentions_role(ROLE_PROGRAM_GENUS, normalized);
    if !(lexicon.mentions_role(ROLE_SCRIPT_AUTHORING_VERB, normalized) && names_the_artefact) {
        return false;
    }
    let mut accounted: Vec<String> = [
        ROLE_SCRIPT_AUTHORING_VERB,
        ROLE_SCRIPT_OR_CODE_ARTIFACT,
        ROLE_PROGRAM_GENUS,
        ROLE_REQUEST_FUNCTION_WORD,
    ]
    .into_iter()
    .flat_map(|role| lexicon.words_for_role(role))
    .filter(|word| !word.trim().is_empty())
    .collect();
    // Longest first, so "give me" is subtracted as the one act it is before
    // "me" is subtracted as a pronoun.
    accounted.sort_by_key(|word| std::cmp::Reverse(word.chars().count()));
    nothing_is_left(normalized, &accounted)
}

/// Whether `normalized` is left empty once every surface in `accounted` is
/// subtracted from it.
fn nothing_is_left(normalized: &str, accounted: &[String]) -> bool {
    let mut text = normalized.to_owned();
    for phrase in accounted.iter().filter(|word| word.contains(' ')) {
        text = text.replace(phrase.as_str(), " ");
    }
    text.split_whitespace().all(|token| {
        let token = token.trim_matches(|character: char| !character.is_alphanumeric());
        token.is_empty() || is_accounted_for(token, accounted)
    })
}

/// Whether one token is accounted for by `accounted`.
///
/// Chinese is written without spaces between words, so a single whitespace
/// token there carries a whole clause; for those the surfaces are subtracted as
/// substrings until nothing remains, which is the same contract
/// [`crate::coding::contains_cjk`] draws everywhere else in the solver.
fn is_accounted_for(token: &str, accounted: &[String]) -> bool {
    if accounted.iter().any(|word| word == token) {
        return true;
    }
    if !crate::coding::contains_cjk(token) {
        return false;
    }
    let mut rest = token.to_owned();
    while let Some(word) = accounted
        .iter()
        .find(|word| crate::coding::contains_cjk(word) && rest.contains(word.as_str()))
    {
        rest = rest.replace(word.as_str(), "");
        if rest.is_empty() {
            return true;
        }
    }
    rest.is_empty()
}
