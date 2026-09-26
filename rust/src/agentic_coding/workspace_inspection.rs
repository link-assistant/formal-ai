//! Reading a request to find something out about the workspace (issue #1066).
//!
//! An agent that has been handed a repository is asked, over and over, to look
//! at what is already there: "Inspect the existing task-decomposition data model
//! and identify where a node stores its children." Nothing in that sentence names
//! a tool, a file or a search — it names what the caller wants to know. The
//! repository-search route only fired when a request said *search* in so many
//! words, so a request that only said *inspect* reached the open-web routers
//! instead, and the answer to a question about the code in front of the agent was
//! looked up on the internet, or not planned at all.
//!
//! This module supplies the missing admission reason and resolves the fact-focused
//! workspace search. It reuses the explicit-search subject extraction in
//! [`super::shell_command`] when the request names that act. Two things have to
//! hold, and neither is a phrasing:
//!
//! * the request carries a seed-declared inspection action
//!   ([`seed::ROLE_WORKSPACE_INSPECTION_ACTION`]), and
//! * it does not send the agent somewhere else for the answer.
//!
//! The second half matters because *verify* is not by itself a local word: "Check
//! the current exchange rate" is a question the workspace cannot answer, and it
//! says so by naming an external source. A request that names no source at all is
//! about the material the agent was given.
//!
//! Both halves are read at the scope of one block, because a note that places the
//! worker is not the request. Every prompt the #1066 ladder sends ends with "use
//! web research when it materially improves factual accuracy" — a permission to
//! reach for a tool, granted in a separate paragraph, and not a statement that
//! the answer is on the internet. Read across the whole prompt, that permission
//! disqualified every one of the sixty-three nodes from looking at the repository
//! it had just been handed.

use super::shell_command::{
    code_shaped_query, code_search_query_for_task, search_tokens, valid_search_identifier,
};
use super::shell_command_policy::{is_prose_word, sentence_spans};
use crate::seed;

/// Whether `prompt` asks the agent to find something out about its workspace.
///
/// The lowercased, normalized copy is what the lexicon is queried with, so the
/// caller may pass the prompt exactly as it was written.
///
/// One block has to satisfy both halves on its own. Splitting first is what
/// separates "review the retry helper" from the paragraph after it that grants
/// web access; joined, the grant reads as though the request had named the web.
pub(super) fn asks_about_the_workspace(prompt: &str) -> bool {
    super::stated_request::request_blocks(prompt)
        .into_iter()
        .any(|block| {
            let normalized = crate::engine::normalize_prompt(block);
            seed::lexicon().mentions_role(seed::ROLE_WORKSPACE_INSPECTION_ACTION, &normalized)
                && !names_an_external_source(&normalized)
        })
}

/// Whether the request points somewhere outside the workspace for its answer.
///
/// The web-research vocabulary already carries the nouns that name one — the
/// web, the internet, an encyclopedia, and their equivalents in the other
/// registered languages ([`seed::ROLE_WEB_SEARCH_SIGNAL`]). A request that spells
/// one out has told the planner where to look, so this route stands aside.
/// Asking the lexicon rather than listing the phrases here keeps the boundary in
/// the data, where all four languages are maintained together.
fn names_an_external_source(normalized: &str) -> bool {
    seed::lexicon().mentions_role(seed::ROLE_WEB_SEARCH_SIGNAL, normalized)
}

/// Recover the subject of a request that asks about the workspace itself.
///
/// "Inspect the existing task-decomposition data model and identify where a
/// node stores its children" never says *search*, but an agent that has been
/// handed a repository answers a question about that repository by reading it.
/// The planner resolves this ahead of the open-web routers, which would
/// otherwise claim the request on the strength of its question shape alone
/// (issue #1066).
///
/// What keeps the open web reachable is the subject, not the verb. The subject
/// must either be visibly code-shaped — quoted or carrying an underscore, dot,
/// interior capital, or hyphen — or sit next to a seed-declared source-artifact
/// kind. That is the difference between "verify the retry-policy helper" or
/// "verify the retry check" and "verify the current exchange rate for the
/// euro". An explicit source-search request is the exception: its named subject
/// is already complete and must not be widened with inferred prose terms.
pub(super) struct WorkspaceInspectionSearch {
    /// The code-shaped subject used as the human-readable query.
    pub(super) query: String,
    /// The grep expression aimed at the fact requested about that subject.
    pub(super) pattern: String,
    /// A filename filter when the subject itself names a module-like file.
    pub(super) include: Option<String>,
}

/// Resolve a workspace question into a subject and a fact-focused search.
///
/// A module name tells the agent *where* an answer is likely to live, but it is
/// rarely the answer. Searching only for `task_decomposition` can fill Agent's
/// result cap with release notes before reaching the `children` field the
/// caller asked about. The request's remaining content words therefore form
/// the grep expression, while a lowercase underscored subject narrows the file
/// set. Both are recovered structurally; no ladder wording is registered here.
pub(super) fn workspace_inspection_search_for_task(
    prompt: &str,
) -> Option<WorkspaceInspectionSearch> {
    for block in super::stated_request::request_blocks(prompt) {
        if !asks_about_the_workspace(block) {
            continue;
        }
        if let Some(query) = code_search_query_for_task(block) {
            return Some(WorkspaceInspectionSearch {
                pattern: query.clone(),
                query,
                include: None,
            });
        }
        // Agent's compactor may flatten the blank line between the actual
        // checkout question and a machine-shaped worker contract. Prefer the
        // subject carried by the sentence that asks to inspect the workspace;
        // otherwise a later token such as `new_audit_effect` can outrank the
        // helper the caller asked about. Keep the historical whole-block
        // fallback for requests whose inspection cue and subject span two
        // sentences.
        if let Some(search) = sentence_spans(block)
            .into_iter()
            .filter(|sentence| asks_about_the_workspace(sentence))
            .find_map(workspace_inspection_search)
        {
            return Some(search);
        }
        if let Some(search) = workspace_inspection_search(block) {
            return Some(search);
        }
    }
    None
}

fn workspace_inspection_search(text: &str) -> Option<WorkspaceInspectionSearch> {
    let query = code_shaped_query(text)?;
    let terms = inspection_fact_terms(text, &query);
    let canonical_fact = literal_inspection_fact_query(&text.to_lowercase())
        .or_else(|| serialized_relationship_fact_query(text));
    // A canonical fact expression identifies its source independently of its
    // module. An inferred code-shaped subject can name a subsystem (for
    // example `task-strategy`) whose implementation lives in a differently
    // named file, so it must not exclude the canonical source fact. The
    // seed-declared artifact kind still distinguishes regression assertions
    // from production implementation.
    let include = canonical_fact
        .as_ref()
        .map(|_| canonical_fact_filename_filter(text))
        .or_else(|| inspection_filename_filter(text, &query));
    if let Some(pattern) = canonical_fact {
        return Some(WorkspaceInspectionSearch {
            query,
            pattern,
            include,
        });
    }
    let mut pattern_terms = Vec::new();
    if include.is_none() {
        pattern_terms.push(query.clone());
    }
    pattern_terms.extend(terms);
    let pattern = if pattern_terms.is_empty() {
        query.clone()
    } else {
        pattern_terms.join("|")
    };
    Some(WorkspaceInspectionSearch {
        query,
        pattern,
        include,
    })
}

fn canonical_fact_filename_filter(text: &str) -> String {
    if seed::lexicon().mentions_role(seed::ROLE_CODING_DOCUMENTATION_FACT_QUERY, text) {
        "docs/**/*".to_owned()
    } else if seed::lexicon().mentions_role(seed::ROLE_CODING_TEST_ARTIFACT_KIND, text) {
        "tests/**/*".to_owned()
    } else if seed::lexicon().mentions_role(seed::ROLE_CODING_EXPERIMENT_ARTIFACT_KIND, text) {
        "experiments/**/*".to_owned()
    } else {
        "src/**/*".to_owned()
    }
}

/// Content words that describe the fact requested by a workspace inspection.
///
/// The seed-declared inspection actions and code-subject kinds express the
/// request's grammar rather than its answer, so they are excluded alongside
/// ordinary prose words. The remainder is useful both for constructing the
/// grep and for choosing the most relevant line from grouped grep output.
pub(super) fn workspace_inspection_terms_for_task(prompt: &str) -> Vec<String> {
    for block in super::stated_request::request_blocks(prompt) {
        if !asks_about_the_workspace(block) {
            continue;
        }
        if let Some((sentence, query)) = sentence_spans(block)
            .into_iter()
            .filter(|sentence| asks_about_the_workspace(sentence))
            .find_map(|sentence| code_shaped_query(sentence).map(|query| (sentence, query)))
        {
            return inspection_fact_terms(sentence, &query);
        }
        if let Some(query) = code_shaped_query(block) {
            return inspection_fact_terms(block, &query);
        }
    }
    Vec::new()
}

fn inspection_fact_terms(text: &str, query: &str) -> Vec<String> {
    let mut terms = Vec::new();
    let scoped = inspection_subject_and_following(text, query);
    let normalized_query = query.to_lowercase();
    for token in search_tokens(scoped) {
        let normalized = token.replace('-', "_").to_lowercase();
        if normalized.len() < 3
            || normalized == normalized_query
            || normalized.chars().all(|character| character.is_ascii_digit())
            || is_prose_word(&normalized)
            || seed::lexicon().mentions_role(seed::ROLE_WORKSPACE_INSPECTION_ACTION, &normalized)
            || seed::lexicon().mentions_role(seed::ROLE_CODING_SEARCH_SUBJECT_KIND, &normalized)
            || terms.contains(&normalized)
        {
            continue;
        }
        terms.push(normalized);
    }
    if let Some(canonical) = literal_inspection_fact_query(&scoped.to_lowercase())
        && !terms.contains(&canonical)
    {
        terms.push(canonical);
    }
    terms
}

/// The inspection subject and the request that follows it, without a wrapper.
///
/// Harnesses and orchestration layers commonly prefix a task with a numbered or
/// classified label. The code-shaped subject is the first token that belongs to
/// the repository question itself, so starting there removes an arbitrary
/// prefix without having to know any of its words. Hyphenated prose and its
/// underscored source spelling identify the same subject.
fn inspection_subject_and_following<'a>(text: &'a str, query: &str) -> &'a str {
    let hyphenated = query.replace('_', "-");
    [query, hyphenated.as_str()]
        .into_iter()
        .filter_map(|spelling| ascii_case_insensitive_offset(text, spelling))
        .min()
        .and_then(|offset| text.get(offset..))
        .unwrap_or(text)
}

fn ascii_case_insensitive_offset(text: &str, needle: &str) -> Option<usize> {
    needle.is_ascii().then_some(())?;
    text.char_indices()
        .map(|(offset, _)| offset)
        .find(|offset| {
            text.get(*offset..offset + needle.len())
                .is_some_and(|candidate| candidate.eq_ignore_ascii_case(needle))
        })
}

fn module_filename_filter(query: &str) -> Option<String> {
    (query.contains('_')
        && query
            .chars()
            .all(|character| character.is_ascii_lowercase() || character.is_ascii_digit() || character == '_'))
    .then(|| format!("*{query}*"))
}

/// Keep source-condition evidence inside production source when the request
/// does not already name a narrower module. Generated traces and tests often
/// quote the same condition verbatim, but are observations about the source
/// rather than the implementation the caller asked to inspect.
fn inspection_filename_filter(text: &str, query: &str) -> Option<String> {
    module_filename_filter(query).or_else(|| {
        let lexicon = seed::lexicon();
        (lexicon.mentions_role(seed::ROLE_CODING_CONDITION_SUBJECT_KIND, text)
            || lexicon.mentions_role(seed::ROLE_CODING_SOURCE_IMPLEMENTATION_SUBJECT_KIND, text))
        .then(|| "src/**/*".to_owned())
    })
}

fn literal_inspection_fact_query(normalized: &str) -> Option<String> {
    seed::lexicon()
        .role_word_forms(seed::ROLE_CODING_SEARCH_FACT_QUERY)
        .into_iter()
        .filter(|form| normalized.contains(&form.text.to_lowercase()))
        .max_by_key(|form| form.text.chars().count())
        .and_then(|form| (!form.action.is_empty()).then(|| form.action.clone()))
}

/// Recover the quoted field key from a relationship-serialization question.
///
/// In `how parent relationships are encoded`, `parent` is not merely a broad
/// prose term: it is the literal key whose representation the caller wants to
/// inspect. Searching for the quoted key reflects source serialization syntax
/// and prevents generic words around it from exhausting a client's result cap.
fn serialized_relationship_fact_query(text: &str) -> Option<String> {
    serialized_relationship_term(text).map(|term| format!(r#""{term}""#))
}

/// The relationship noun whose serialization the request asks to inspect.
///
/// Keep this structural extraction shared with evidence ranking: generic fact
/// terms deliberately discard some grammar, while the relationship immediately
/// before a seed-declared relationship kind is the identity-bearing subject.
pub(super) fn serialized_relationship_term(text: &str) -> Option<String> {
    let lexicon = seed::lexicon();
    let normalized = crate::engine::normalize_prompt(text);
    if !lexicon.mentions_role(seed::ROLE_CODING_SERIALIZATION_ACTION, &normalized) {
        return None;
    }
    let tokens = search_tokens(text).collect::<Vec<_>>();
    tokens
        .windows(2)
        .find(|pair| {
            valid_search_identifier(pair[0])
                && lexicon.mentions_role(seed::ROLE_CODING_RELATIONSHIP_SUBJECT_KIND, pair[1])
        })
        .map(|pair| pair[0].to_ascii_lowercase())
}
