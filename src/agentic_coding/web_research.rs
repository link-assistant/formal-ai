//! Multi-step web research for agentic clients (issue #687).
//!
//! Intent recognition is delegated to the same meaning-lexicon detector used by
//! the universal solver. This module only adds agentic sequencing (search, rank
//! a source, fetch, answer) and resolves a seed-declared contextual reference
//! against prior user turns. No natural-language vocabulary lives here.

use serde_json::json;

use super::planner::{fetch_arguments, plan_one, tool_for, AgenticPlan, Capability, Progress};
use crate::engine::FormalAiEngine;
use crate::protocol::ChatMessage;
use crate::seed::{self, Slot};
use crate::world_model::Context;
use crate::world_model_context::{ContextHierarchy, ExternalLookup, InheritancePolicy};

/// The subject this turn should research, when the request states one.
///
/// The subject is read from one block of the request rather than from the whole
/// of it, for the reason [`super::stated_request`] states: a note that says
/// where the worker is and how to report is not work, so nothing is read out of
/// it. Every prompt the issue-#1066 ladder sends carries such a note, and the
/// subject ran straight through the blank line into it -- a leaf that asked
/// about "a two-node decomposition at depth one" researched "a two node
/// decomposition at depth one this is recursive binary tree node 1 2 1 2 1 at
/// depth 5 solve only this node s task in this fresh temporary repository ...",
/// a query no source on earth answers.
///
/// The blocks are tried in order, so a request that states its subject before
/// its framing keeps the subject it stated.
pub(super) fn web_research_query_for(messages: &[ChatMessage]) -> Option<String> {
    let task = latest_user_text(messages)?;
    let query = super::stated_request::request_blocks(&task)
        .into_iter()
        .find_map(|block| {
            seed_research_subject(block)
                .or_else(|| seed_slotted_subject(block, seed::ROLE_DEFINITION_EXAMPLE_REQUEST))
                .or_else(|| crate::solver_handlers::detect_web_search_query(block))
                .or_else(|| seed_definition_subject(block))
                .or_else(|| seed_unresolved_question_subject(block))
        })?;
    if is_context_reference(&query) {
        topic_from_history(messages)
    } else {
        Some(query)
    }
}

/// Promote a request only after every specialized local route and completed
/// tool result has declined it. Keeping this separate from explicit research
/// preserves those higher-priority outcomes while closing the open-world gap.
///
/// This is the last route in the planner, so it is the one that decides what a
/// request nothing else understood is searched for -- and it used to search for
/// all of it. A prompt whose second block only places the worker was sent to
/// the open web with that block attached, blank line and all: "Verify the
/// current exchange rate between the euro and the yen.\n\nWork only in this
/// checkout." Issue #1066 found every one of its sixty-three ladder nodes
/// carrying such a block. The query is therefore the first block the engine
/// itself cannot resolve, for the reason [`super::stated_request`] states,
/// and the whole request stays the query when no single block qualifies --
/// which is what a one-block prompt always is.
pub(super) fn unresolved_web_research_query_for(messages: &[ChatMessage]) -> Option<String> {
    let task = latest_user_text(messages)?;
    // Once every specialized local route has declined, any unresolved request
    // is an open-world research task. This is deliberately intent-driven rather
    // than punctuation-driven: instructions can require missing knowledge just
    // as questions do. Conversation-meta requests remain local because searching
    // the public web cannot recover private dialog history.
    let unresolved = |text: &str| {
        matches!(
            FormalAiEngine.answer(text).intent.as_str(),
            "unknown" | "web_search"
        )
    };
    let preceding = messages
        .get(..messages.len().saturating_sub(1))
        .unwrap_or(&[]);
    if !unresolved(&task) || is_conversation_meta_request(&task, preceding) {
        return None;
    }
    let stated = super::stated_request::request_blocks(&task)
        .into_iter()
        .find(|block| unresolved(block))
        .unwrap_or(task.as_str());
    Some(trim_question_punctuation(stated.trim()))
}

/// Whether this turn has already entered a successful search → fetch research
/// sequence. Generic tool results retain their friendly renderer; only a
/// search that produced usable output proves that an unresolved fallback owns
/// the rest of this turn.
pub(super) fn has_successful_search_result(messages: &[ChatMessage]) -> bool {
    Progress::scan(messages)
        .latest_successful_output(Capability::Search)
        .is_some()
}

pub(super) fn is_definition_followup(task: &str) -> bool {
    seed::lexicon().mentions_role(
        seed::ROLE_DEFINITION_ANTECEDENT_FOLLOWUP,
        &crate::engine::normalize_prompt(task),
    )
}

pub(super) fn definition_followup_topic(messages: &[ChatMessage], task: &str) -> Option<String> {
    // A compound one-turn prompt can state the antecedent before the follow-up.
    let normalized = crate::engine::normalize_prompt(task);
    let mut forms = seed::lexicon().words_for_role(seed::ROLE_DEFINITION_ANTECEDENT_FOLLOWUP);
    forms.sort_by_key(|form| std::cmp::Reverse(form.chars().count()));
    if let Some(prefix) = forms.into_iter().find_map(|form| {
        let form = crate::engine::normalize_prompt(&form);
        normalized
            .find(&form)
            .map(|position| normalized[..position].trim())
            .filter(|prefix| !prefix.is_empty())
    }) {
        let mut antecedent = prefix
            .trim_end_matches(|character: char| {
                character.is_whitespace()
                    || character.is_ascii_punctuation()
                    || matches!(character, '？' | '。')
            })
            .trim()
            .to_owned();
        let mut continuations =
            seed::lexicon().words_for_role(seed::ROLE_CLAUSE_CONTINUATION_MARKER);
        continuations.sort_by_key(|marker| std::cmp::Reverse(marker.chars().count()));
        for marker in continuations {
            let marker = crate::engine::normalize_prompt(&marker);
            if antecedent == marker {
                antecedent.clear();
                break;
            }
            if antecedent.ends_with(&format!(" {marker}")) {
                antecedent.truncate(antecedent.len() - marker.len() - 1);
                break;
            }
        }
        if !antecedent.is_empty() {
            return crate::solver_handlers::detect_web_search_query(&antecedent)
                .or_else(|| seed_prefix_subject(&antecedent, seed::ROLE_RESEARCH_QUESTION_OPENER))
                .or(Some(antecedent));
        }
    }
    topic_from_history(messages)
}

pub(super) fn definition_followup_clarification(task: &str) -> String {
    let language = crate::language::detect(task).slug();
    seed::localized_response("definition_followup_clarify", language).unwrap_or_default()
}

/// A word-meaning question can carry a sentence as context while its subject is
/// still only a pronoun ("what does *this* mean in …"). When that sentence
/// contains no earlier discourse, searching the pronoun on the public web is a
/// category error. Reuse the same meanings-driven clarification as a bare
/// follow-up and let a later turn supply the missing antecedent.
pub(super) fn contextual_reference_clarification(task: &str) -> Option<String> {
    let query = crate::concepts::extract_concept_query(task)?;
    query.context.as_ref()?;
    is_context_reference(&query.term).then(|| definition_followup_clarification(task))
}

/// Extract the subject carried by a seed-declared research imperative. The
/// shared web detector deliberately rejects pronouns as standalone searches;
/// the agentic planner accepts them here because it can resolve them against
/// conversation history before creating a tool call.
fn seed_research_subject(task: &str) -> Option<String> {
    seed_prefix_subject(task, seed::ROLE_WEB_SEARCH_IMPERATIVE_LEAD)
}

fn seed_definition_subject(task: &str) -> Option<String> {
    let normalized = crate::engine::normalize_prompt(task);
    let lexicon = seed::lexicon();
    lexicon
        .role_word_forms(seed::ROLE_RESEARCH_QUESTION_OPENER)
        .into_iter()
        .filter(|form| form.slot() == Slot::Prefix)
        .filter(|form| {
            lexicon.mentions_role(
                seed::ROLE_DEFINITION_COMMAND,
                &crate::engine::normalize_prompt(form.before_slot()),
            )
        })
        .find_map(|form| {
            let prefix = crate::engine::normalize_prompt(form.before_slot());
            normalized
                .strip_prefix(&prefix)
                .map(trim_question_punctuation)
                .filter(|subject| !subject.trim().is_empty())
        })
}

/// Extract a subject from a role whose surface declares an ellipsis slot.
///
/// This is deliberately slot-driven rather than language-driven: English and
/// Russian put the request before the subject, Hindi puts it after, and Chinese
/// wraps it. Adding another language is therefore a seed-data change only.
fn seed_slotted_subject(task: &str, role: &str) -> Option<String> {
    let normalized = crate::engine::normalize_prompt(task);
    seed::lexicon()
        .role_word_forms(role)
        .into_iter()
        .find_map(|form| {
            let before = crate::engine::normalize_prompt(form.before_slot());
            let after = crate::engine::normalize_prompt(form.after_slot());
            let subject = match form.slot() {
                Slot::Prefix => normalized.strip_prefix(&before),
                Slot::Suffix => normalized.strip_suffix(&after),
                Slot::Circumfix => normalized
                    .strip_prefix(&before)
                    .and_then(|body| body.strip_suffix(&after)),
                Slot::Bare => None,
            }?;
            let subject = trim_question_punctuation(subject);
            (!subject.is_empty()).then_some(subject)
        })
}

/// Recover an open-world question whose output instruction follows its question
/// mark (for example, "What is X? Answer in English."). The ordinary web intent
/// detector sees the seeded opener but deliberately leaves broad factual
/// questions to the symbolic solver. Only promote that subject after the local
/// engine reports it unresolved, so locally known facts keep their established
/// route.
fn seed_unresolved_question_subject(task: &str) -> Option<String> {
    let subject = seed_prefix_subject(task, seed::ROLE_RESEARCH_QUESTION_OPENER)?;
    matches!(
        FormalAiEngine.answer(task).intent.as_str(),
        "unknown" | "web_search"
    )
    .then_some(subject)
}

fn seed_prefix_subject(task: &str, role: &str) -> Option<String> {
    let normalized = crate::engine::normalize_prompt(task);
    seed::lexicon()
        .role_word_forms(role)
        .into_iter()
        .filter(|form| form.slot() == Slot::Prefix)
        .find_map(|form| {
            let prefix = crate::engine::normalize_prompt(form.before_slot());
            normalized
                .strip_prefix(&prefix)
                .map(trim_question_punctuation)
                .filter(|subject| !subject.is_empty())
        })
}

/// How many search → fetch rounds one question may take.
///
/// One round can only answer a question whose every aspect happens to sit on
/// the pages the first search returned. Issue #781's question does not: the
/// requirement, the part that meets it, and where to get it are three different
/// documents, and the third is only findable once the first two are read. The
/// bound exists because the loop's own stopping rule — no aspect of the question
/// left uncovered — can be unreachable when the missing fact simply is not on
/// the open web, and a research loop must terminate either way. Three rounds
/// fit inside the driver's turn budget with the fetches they imply.
const MAX_RESEARCH_ROUNDS: usize = 3;

pub(super) fn plan_web_research_step(
    messages: &[ChatMessage],
    tool_names: &[&str],
    query: &str,
) -> Option<AgenticPlan> {
    let progress = Progress::scan(messages);
    if let Some(failure) = progress.latest_failure()
        && matches!(failure.capability, Capability::Search | Capability::Fetch) {
            return Some(AgenticPlan::Final(super::tool_result::render_failure(
                failure.capability.registry_id(),
                &failure.detail,
                query,
            )));
        }
    // `completed` is in arrival order, so the most recent result says which
    // phase this round is in. `done` cannot: it stays true from round one
    // onward, which is exactly why the old single-round shape could not deepen.
    match progress.last() {
        None => tool_for(tool_names, Capability::Search)
            .map(|tool| plan_one(tool, json!({ "query": query }).to_string())),
        Some(Capability::Search) => Some(
            plan_fetches(tool_names, &progress)
                .unwrap_or_else(|| AgenticPlan::Final(final_answer(query, &progress))),
        ),
        Some(Capability::Fetch) => Some(
            plan_fetches(tool_names, &progress)
                .or_else(|| plan_deeper_round(tool_names, &progress, query))
                .unwrap_or_else(|| AgenticPlan::Final(final_answer(query, &progress))),
        ),
        // The last completed call belongs to some other route -- a workspace
        // grep, a file read, a shell command. A further search can still be
        // worth issuing, but composing the answer here is not: `final_answer`
        // would speak for research that never ran, and it would speak over a
        // result the agent already has in hand. Issue #1066 measured what that
        // costs. Nine of the sixty-three ladder nodes searched their own
        // repository, were handed what it said, and reported "the tool returned
        // no content" on top of it, because `last` saw the grep and this arm
        // treated it as a research round that had come back empty.
        Some(_) => plan_deeper_round(tool_names, &progress, query),
    }
}

/// Read the sources the latest search returned, skipping any already read.
///
/// Skipping is what keeps a multi-round loop from stalling: a refined search
/// usually returns some of the same pages, and re-reading them would burn the
/// turn budget while adding no evidence. When nothing new remains, the round has
/// no work and the caller falls through to answering.
fn plan_fetches(tool_names: &[&str], progress: &Progress) -> Option<AgenticPlan> {
    let tool = tool_for(tool_names, Capability::Fetch)?;
    let output = progress.search_output.as_deref()?;
    let already: std::collections::BTreeSet<&str> = progress
        .attempted_fetches
        .iter()
        .map(String::as_str)
        .collect();
    research_urls(output)
        .into_iter()
        .find(|url| !already.contains(url.as_str()))
        .map(|url| plan_one(tool, fetch_arguments(&url)))
}

/// Search again for the part of the question the evidence has not covered.
///
/// The refinement is the uncovered aspects *alone*, not the original question
/// repeated. Re-issuing the whole question returns the whole first result set
/// again; dropping the aspects already grounded is what makes the second search
/// reach documents the first could not. Returns `None` when the question is
/// fully covered or the round budget is spent — both mean it is time to answer.
fn plan_deeper_round(tool_names: &[&str], progress: &Progress, query: &str) -> Option<AgenticPlan> {
    if progress.count(Capability::Search) >= MAX_RESEARCH_ROUNDS {
        return None;
    }
    let open = uncovered_aspects(query, progress);
    // Another round is only worth spending on a gap we can actually name, and
    // this rule is deliberately strict about that: exactly one aspect of a
    // several-aspect question is unsupported by everything read so far.
    //
    // The strictness is not tuning, it is a response to how weak the underlying
    // signal is. Token coverage is a poor proxy for "answered": a page that
    // genuinely answers "when are elections in usa" does not repeat every word
    // of the question, and how many words it echoes varies with the language it
    // is written in — the same page shape leaves two aspects open in Hindi and
    // none in English. Anything looser turns that variation into wasted rounds
    // on questions that were already answered.
    //
    // A single open aspect is different in kind. It is the shape of a real
    // follow-up — the specifications were found and only the warranty is
    // missing — and it yields a refinement worth issuing, because the refined
    // query is that aspect alone rather than a restatement of the question.
    //
    // The cost is that genuinely partial answers with two or more gaps are
    // returned as-is instead of researched further. That is the intended
    // trade: answering with what was actually found beats spending a round on
    // a guess about what is missing.
    //
    // This also makes the loop terminate on its own, independently of the round
    // budget: uncovered aspects only ever shrink as evidence accumulates.
    if open.len() != 1 || aspects_of(query).len() < 3 {
        return None;
    }
    let tool = tool_for(tool_names, Capability::Search)?;
    Some(plan_one(
        tool,
        json!({ "query": open.join(" ") }).to_string(),
    ))
}

/// The aspects of `query` that no fetched page supports.
///
/// This is the loop's open-question signal, and it carries no vocabulary: an
/// aspect is a content token of the question, and it is covered when some page
/// actually mentions it. That is deliberately the same symbolic, non-neural
/// notion of aboutness [`relevance`] ranks sentences with, applied to the
/// question instead of the answer.
///
/// Scripts that do not space-separate words tokenize to one long token, which
/// would report the whole question uncovered forever. For those the aspect is
/// the ideograph, matching the fallback [`relevance`] already uses.
fn uncovered_aspects(query: &str, progress: &Progress) -> Vec<String> {
    if progress.fetched_pages.is_empty() {
        return Vec::new();
    }
    let evidence = progress
        .fetched_pages
        .iter()
        .map(|(_, text)| text.to_lowercase())
        .collect::<Vec<_>>()
        .join(" ");
    aspects_of(query)
        .into_iter()
        .filter(|aspect| !evidence.contains(&aspect.to_lowercase()))
        .collect()
}

/// Shortest token treated as an aspect. One-character latin tokens are
/// initials and stray letters, not aspects of a question.
const MIN_ASPECT_CHARS: usize = 2;

fn aspects_of(query: &str) -> Vec<String> {
    if crate::coding::contains_cjk(query) {
        return query
            .chars()
            .filter(|character| character.is_alphanumeric())
            .map(|character| character.to_string())
            .collect();
    }
    let mut seen = std::collections::BTreeSet::new();
    query
        .split(|character: char| !character.is_alphanumeric())
        .filter(|token| token.chars().count() >= MIN_ASPECT_CHARS)
        .map(str::to_lowercase)
        .filter(|token| seen.insert(token.clone()))
        .collect()
}

/// Maximum fallback length when no sentence overlaps the question. Even a
/// short fetch can be a whole page made of terse navigation labels, so length
/// is never treated as proof that a payload is already an answer.
const MAX_EXTRACT_CHARS: usize = 600;

/// How many sentences an extract keeps. Enough for a claim plus its immediate
/// qualification, short enough to stay an answer rather than a transcript.
const EXTRACT_SENTENCES: usize = 3;

fn final_answer(query: &str, progress: &Progress) -> String {
    if !progress.fetched_pages.is_empty() {
        return progress
            .fetched_pages
            .iter()
            .map(|(url, evidence)| {
                format!(
                    "{}\n\n{}: {url}",
                    extract_answer(query, evidence.trim()),
                    seed_text("web_research_source_label")
                )
            })
            .collect::<Vec<_>>()
            .join("\n\n");
    }
    if !progress.attempted_fetches.is_empty() {
        // Search-result snippets and URLs are discovery metadata, not a
        // substitute for pages that every fetch attempt failed to read.
        return render_seed_text("web_research_no_content", "query", query);
    }
    let evidence = progress
        .fetched_text
        .as_deref()
        .or(progress.search_output.as_deref())
        .unwrap_or_default()
        .trim();
    if evidence.is_empty() {
        return render_seed_text("web_research_no_content", "query", query);
    }
    let source = progress
        .search_output
        .as_deref()
        .and_then(preferred_url)
        .map_or_else(String::new, |url| {
            format!("\n\n{}: {url}", seed_text("web_research_source_label"))
        });
    format!("{}{source}", extract_answer(query, evidence))
}

/// Reduce fetched evidence to the sentences that actually bear on `query`.
///
/// A fetch tool returns the whole page; answering with it verbatim is what made
/// issue #771's session unreadable. Sentences are scored by symbolic token
/// overlap with the query — the same non-neural similarity the ranker uses — and
/// the best few are returned in document order so the extract still reads as
/// prose. Scoring is deterministic and carries no natural-language vocabulary,
/// so it works in every supported language — see [`relevance`] for how the
/// space-less scripts are handled.
fn extract_answer(query: &str, evidence: &str) -> String {
    let evidence = structurally_complete_prose(evidence);
    let sentences = crate::summarization::formalize(&evidence);
    let mut scored: Vec<(usize, f32, &str)> = sentences
        .iter()
        .enumerate()
        .map(|(position, statement)| {
            (
                position,
                relevance(query, &statement.text),
                statement.text.as_str(),
            )
        })
        .filter(|(_, score, _)| *score > 0.0)
        .collect();
    if scored.is_empty() {
        // Nothing overlaps the query: fall back to the head of the document
        // rather than the whole of it, so the answer stays bounded either way.
        return truncate_chars(&evidence, MAX_EXTRACT_CHARS);
    }
    // Rank by relevance, keep the best few, then restore document order.
    scored.sort_by(|left, right| {
        right
            .1
            .partial_cmp(&left.1)
            .unwrap_or(std::cmp::Ordering::Equal)
            .then(left.0.cmp(&right.0))
    });
    scored.truncate(EXTRACT_SENTENCES);
    scored.sort_by_key(|(position, _, _)| *position);
    scored
        .iter()
        .map(|(_, _, text)| *text)
        .collect::<Vec<_>>()
        .join(" ")
}

/// Prefer complete prose blocks when evidence mixes prose with page furniture.
///
/// Search providers often flatten headings, controls, and neighbouring links
/// into short newline-delimited blocks beside the useful snippet. A complete
/// sentence has a script-level terminal; navigation fragments generally do
/// not. The structural signal works across languages and avoids maintaining a
/// blocklist of provider-specific button labels. If the provider returns no
/// complete block, preserve all evidence so terse but legitimate answers are
/// still available to the ordinary relevance ranker.
fn structurally_complete_prose(evidence: &str) -> String {
    let blocks: Vec<&str> = evidence
        .split("\n\n")
        .map(str::trim)
        .filter(|block| !block.is_empty())
        .collect();
    let prose: Vec<&str> = blocks
        .iter()
        .copied()
        .filter(|block| block.chars().next_back().is_some_and(is_sentence_terminal))
        .collect();
    if prose.is_empty() {
        evidence.trim().to_owned()
    } else {
        prose.join("\n")
    }
}

const fn is_sentence_terminal(character: char) -> bool {
    matches!(character, '.' | '!' | '?' | '。' | '…' | '।' | '॥')
}

/// How much `sentence` bears on `query`, in `0.0..=1.0`.
///
/// Bag-of-words cosine is the primary measure. It tokenizes on non-alphanumeric
/// boundaries, which is exactly right for the space-separated scripts but scores
/// every Chinese sentence 0.0: a run of Han characters with no spaces is a
/// single token, so query and sentence never share one. The codebase's existing
/// answer for that (see `coding::catalog::contains_cjk` and its callers) is to
/// match on characters rather than words, so that is the fallback here.
fn relevance(query: &str, sentence: &str) -> f32 {
    let cosine = crate::probability::symbolic_cosine_similarity(query, sentence);
    if cosine > 0.0 || !crate::coding::contains_cjk(query) {
        return cosine;
    }
    character_overlap(query, sentence)
}

/// The fraction of the query's distinct ideographs that `sentence` also uses.
///
/// Punctuation and spacing are ignored, so the score reflects shared content
/// characters only. Common function characters inflate it slightly, which costs
/// nothing here because the score is only ever used to rank sentences of the
/// same document against each other.
fn character_overlap(query: &str, sentence: &str) -> f32 {
    let sentence: std::collections::BTreeSet<char> =
        sentence.chars().filter(|c| c.is_alphanumeric()).collect();
    let query: std::collections::BTreeSet<char> =
        query.chars().filter(|c| c.is_alphanumeric()).collect();
    if query.is_empty() {
        return 0.0;
    }
    let shared = query.iter().filter(|c| sentence.contains(c)).count();
    #[expect(
        clippy::cast_precision_loss,
        reason = "character counts are far below f32's exact-integer range"
    )]
    {
        shared as f32 / query.len() as f32
    }
}

/// Truncate to at most `max` characters on a char boundary, appending an
/// ellipsis when shortened.
fn truncate_chars(value: &str, max: usize) -> String {
    let value = value.trim();
    if value.chars().count() <= max {
        return value.to_owned();
    }
    let head: String = value.chars().take(max.saturating_sub(1)).collect();
    format!("{}…", head.trim_end())
}

fn seed_text(key: &str) -> String {
    seed::agent_info()
        .remove(key)
        .unwrap_or_else(|| key.to_owned())
}

fn render_seed_text(key: &str, name: &str, value: &str) -> String {
    let mut placeholder = String::with_capacity(name.len() + 2);
    placeholder.push('{');
    placeholder.push_str(name);
    placeholder.push('}');
    seed_text(key).replace(&placeholder, value)
}

/// Rank URLs instead of blindly fetching the first search result. Government
/// and education hosts are authoritative for public facts; otherwise preserve
/// the search provider's ordering. The complete fetched URL is retained in the
/// final answer for auditability.
pub(super) fn preferred_url(text: &str) -> Option<String> {
    research_urls(text).into_iter().next()
}

/// Bound the breadth of one research round while retaining independent sources.
/// The first authoritative host is moved to the front; the search provider's
/// ranking determines the remaining order. Three captures are enough to
/// triangulate a claim without turning a single question into an unbounded crawl.
const MAX_RESEARCH_SOURCES: usize = 3;

fn research_urls(text: &str) -> Vec<String> {
    let mut urls = urls_in(text);
    let mut seen = std::collections::BTreeSet::new();
    urls.retain(|url| seen.insert(url.clone()));
    if let Some(position) = urls.iter().position(|url| authoritative_host(url)) {
        urls.swap(0, position);
    }
    urls.truncate(MAX_RESEARCH_SOURCES);
    urls
}

fn urls_in(text: &str) -> Vec<String> {
    text.split_whitespace()
        .filter(|token| token.starts_with("http://") || token.starts_with("https://"))
        .map(|token| {
            token
                .trim_end_matches(['.', ',', ';', ')', ']', '"', '\''])
                .to_owned()
        })
        .collect()
}

fn authoritative_host(url: &str) -> bool {
    let host = url
        .split_once("://")
        .map_or(url, |(_, rest)| rest)
        .split('/')
        .next()
        .unwrap_or_default()
        .split(':')
        .next()
        .unwrap_or_default()
        .to_ascii_lowercase();
    let mut labels = host.rsplit('.');
    let terminal = labels.next().unwrap_or_default();
    terminal == "gov"
        || terminal == "edu"
        || (terminal == "uk" && labels.next().is_some_and(|label| label == "gov"))
}

fn is_context_reference(query: &str) -> bool {
    let normalized = crate::engine::normalize_prompt(query);
    seed::lexicon()
        .role_word_forms(seed::ROLE_NON_REFERENTIAL_SUBJECT)
        .into_iter()
        .any(|form| match form.slot() {
            Slot::Bare => crate::engine::normalize_prompt(&form.text) == normalized,
            Slot::Prefix => {
                normalized.starts_with(&crate::engine::normalize_prompt(form.before_slot()))
            }
            Slot::Suffix | Slot::Circumfix => false,
        })
}

fn topic_from_history(messages: &[ChatMessage]) -> Option<String> {
    let latest = messages
        .iter()
        .rposition(|message| message.role.eq_ignore_ascii_case("user"))?;
    let mut hierarchy = ContextHierarchy::new();
    hierarchy.insert(Context::new("conversation")).ok()?;
    let mut parent_id = String::from("conversation");

    for (index, message) in messages[..latest].iter().enumerate() {
        let context_id = format!("conversation:turn:{}", index + 1);
        let mut context = Context::new(&context_id);
        if message.role.eq_ignore_ascii_case("user") {
            let text = message.content.user_request_text();
            if !super::report_issue::is_report_intent(&text)
                && !is_conversation_meta_request(&text, &messages[..index])
            {
                let topic = crate::solver_handlers::detect_web_search_query(&text)
                    .or_else(|| seed_prefix_subject(&text, seed::ROLE_RESEARCH_QUESTION_OPENER))
                    .unwrap_or_else(|| trim_question_punctuation(&text));
                if !topic.trim().is_empty() && !is_context_reference(&topic) {
                    context.assert_link("research_topic", &topic);
                }
            }
        }
        hierarchy
            .nest(context, &parent_id, InheritancePolicy::Full)
            .ok()?;
        parent_id = context_id;
    }

    hierarchy
        .nest(
            Context::new("conversation:current"),
            &parent_id,
            InheritancePolicy::Full,
        )
        .ok()?;
    hierarchy
        .resolve(
            "conversation:current",
            "research_topic",
            ExternalLookup::Denied,
        )
        .ok()?
        .links
        .first()
        .map(|link| link.to.clone())
}

fn is_conversation_meta_request(prompt: &str, preceding: &[ChatMessage]) -> bool {
    let history = preceding
        .iter()
        .filter_map(crate::protocol::chat_message_to_turn)
        .collect::<Vec<_>>();
    crate::solve_with_history(prompt, &history).intent == "summarize_conversation"
}

fn trim_question_punctuation(text: &str) -> String {
    text.trim()
        .trim_end_matches(['?', '？', '؟', '¿', '.', '!', '。'])
        .trim()
        .to_owned()
}

fn latest_user_text(messages: &[ChatMessage]) -> Option<String> {
    crate::protocol::latest_user_request(messages)
}
