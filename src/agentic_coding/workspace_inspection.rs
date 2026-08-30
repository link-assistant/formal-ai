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
//! This module supplies the missing admission reason. It answers one question —
//! *is this request asking about the workspace?* — and leaves picking the search
//! subject to [`super::shell_command`], which already knows how to do that for an
//! explicit search. Two things have to hold, and neither is a phrasing:
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
