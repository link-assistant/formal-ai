//! Recognise a prose request to list the files in the current place.
//!
//! Issue #865 reported *"List me files here"* answered by a web search. The
//! detector this module replaces matched a hardcoded array of complete English
//! phrases — *"list files"*, *"show me the files"*, *"what files"* — and no
//! entry had that word order, so an ordinary request fell through to the
//! internet. Enumerating phrases cannot converge: every new way of asking is a
//! new entry, and the array was English-only besides.
//!
//! A listing request is instead recognised from the **parts** it is composed
//! of, declared per language in `data/seed/shell-intents.lino`: a listing verb
//! (or a question word), a noun naming what is listed, and a phrase scoping the
//! request to the current place. Any word order that carries all three is a
//! listing request, so held-out paraphrases route without a seed edit, and the
//! natural language lives in seed data rather than in this file.

use crate::seed::{self, DirectoryListingVocabulary};

/// Whether `prompt` asks, in prose, to list the files in the current place.
pub(super) fn asks_for_directory_listing(prompt: &str) -> bool {
    let lower = prompt.to_lowercase();
    let vocabulary = seed::shell_intent_vocabulary().directory_listing;
    composes_a_listing_request(&lower, &vocabulary)
}

/// Whether the lowercased `prompt` combines the three parts of a listing
/// request. The verb and the question word are alternatives — *"show the files
/// here"* and *"which files are here?"* ask the same thing — while the object
/// and the scope are both required, so *"list the running processes"* stays
/// with its own intent.
///
/// The objects name a *collection* (files, contents, entries) and deliberately
/// exclude the bare *directory*/*folder* the scopes are built from: otherwise
/// *"show me the current directory"* would satisfy the object and the scope with
/// the same two words and list a directory the user asked to be told the name
/// of. A request that names only the container — *"what's in this directory"* —
/// is carried by the `ls` cue table next to this vocabulary.
fn composes_a_listing_request(prompt: &str, vocabulary: &DirectoryListingVocabulary) -> bool {
    let mentions_any = |parts: &[String]| parts.iter().any(|part| mentions(prompt, part));
    (mentions_any(&vocabulary.verbs) || mentions_any(&vocabulary.questions))
        && mentions_any(&vocabulary.objects)
        && mentions_any(&vocabulary.scopes)
}

/// Whether `text` mentions `phrase` as a whole word.
///
/// Word boundaries matter because the parts are single words often contained in
/// longer ones: *list* inside *checklist*, *see* inside *seed*. Scripts written
/// without spaces have no boundaries to check, so for those the phrase is a
/// plain substring — the same rule [`crate::seed::caller_context_vocabulary`]
/// applies to its own word lists.
fn mentions(text: &str, phrase: &str) -> bool {
    if phrase.is_empty() {
        return false;
    }
    if phrase.chars().any(is_unspaced_script) {
        return text.contains(phrase);
    }
    let mut searched = 0;
    while let Some(offset) = text[searched..].find(phrase) {
        let start = searched + offset;
        let end = start + phrase.len();
        let before = text[..start].chars().next_back();
        let after = text[end..].chars().next();
        if !before.is_some_and(char::is_alphanumeric) && !after.is_some_and(char::is_alphanumeric) {
            return true;
        }
        // Advance by one *character*, not one byte: a phrase that starts with a
        // multi-byte letter (Cyrillic, Devanagari) would otherwise leave
        // `searched` inside that letter and panic on the next slice.
        searched = start + phrase.chars().next().map_or(1, char::len_utf8);
    }
    false
}

/// Whether `character` belongs to a script written without spaces between words.
const fn is_unspaced_script(character: char) -> bool {
    matches!(character, '\u{3400}'..='\u{9fff}' | '\u{f900}'..='\u{faff}')
}

/// The scope phrases whose words are never a listed directory.
const LISTED_SCOPE_ROLES: [&str; 5] = [
    "local_path_scope_current",
    "local_path_scope_desktop",
    "local_path_scope_home",
    "capability_container_scope",
    "capability_workspace_scope",
];
const PLACE_ROLE: &str = "statement_place_preposition";
const OPERAND_WRAPPERS: &[char] = &[
    '`', '"', '\'', ',', ';', ':', '!', '?', '(', ')', '[', ']', '{', '}',
];
const CURRENT_DIRECTORY: &str = ".";

/// A relative path a listing may name: no flag, no parent step, no URL (the
/// operand characters admit no colon).
fn is_directory_operand(token: &str) -> bool {
    !token.is_empty()
        && !token.starts_with('-')
        && !token.split('/').any(|part| part == "..")
        && token
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || matches!(c, '.' | '_' | '/' | '-'))
}

/// The directory a listing request names (PR #1188 T98, gap G29).
///
/// "List the files in src." listed the workspace root. The operand is the
/// path-shaped word right after a seeded place preposition
/// (`statement_place_preposition`), unless that word is prose or part of a
/// seeded scope phrase ("in this directory"); `.` otherwise.
pub(super) fn listed_directory(task: &str) -> String {
    let lexicon = seed::lexicon();
    let excluded: Vec<String> = LISTED_SCOPE_ROLES
        .iter()
        .flat_map(|role| lexicon.words_for_role(role))
        .flat_map(|surface| {
            surface
                .to_lowercase()
                .split_whitespace()
                .map(str::to_owned)
                .collect::<Vec<_>>()
        })
        .collect();
    let places: Vec<String> = lexicon
        .words_for_role(PLACE_ROLE)
        .iter()
        .map(|surface| surface.to_lowercase())
        .collect();
    let words: Vec<&str> = task
        .split_whitespace()
        .map(|word| word.trim_matches(OPERAND_WRAPPERS).trim_end_matches('.'))
        .collect();
    words
        .windows(2)
        .filter(|pair| places.contains(&pair[0].to_lowercase()))
        .map(|pair| pair[1].trim_end_matches('/'))
        .find(|candidate| {
            is_directory_operand(candidate)
                && !super::shell_command_policy::is_prose_word(candidate)
                && !excluded.contains(&candidate.to_lowercase())
        })
        .unwrap_or(CURRENT_DIRECTORY)
        .to_owned()
}
