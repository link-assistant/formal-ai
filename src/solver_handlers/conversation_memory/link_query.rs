//! The link operand domain of the substitution query language, over memory.
//!
//! The text domain reaches the product through `agentic_coding::code_artifact`,
//! which lowers a request into a rewrite over a file's bytes. The link domain
//! had no such route: `parse_link_substitution_query` and `matched_links` were
//! public library API with no caller in `src/`, so the half of the query
//! language that operates on links was unreachable from the CLI that owns the
//! links. This is that route — `formal-ai memory query` is the surface where the
//! store *is* an associative store, so it is where link-cli's own syntax applies
//! to link-cli's own operand domain.
//!
//! Reads are served; writes are refused. That asymmetry is not an omission, and
//! [`write_boundary`] states why in the answer itself rather than leaving the
//! caller to infer it from a silent failure.

use std::fmt::Write as _;

use super::super::finalize_simple;

use crate::engine::SymbolicAnswer;
use crate::event_log::EventLog;
use crate::link_store::{memory_events_to_link_records, DoubletLink};
use crate::links_substitution_query::{
    link_substitution_effect, parse_link_substitution_query, render_link, LinkRewriteProgram,
};
use crate::memory::MemoryStore;
use crate::substitution::CrudEvent;

/// Reads do not step, so this only bounds a parse. It matches the bound
/// `agentic_coding::code_artifact` gives the text domain, so neither dialect is
/// the more permissive one.
const MAX_REWRITE_STEPS: usize = 100_000;

pub(super) fn try_link_substitution_query(
    prompt: &str,
    store: &MemoryStore,
    log: &mut EventLog,
) -> Option<SymbolicAnswer> {
    let program = explicit_link_query(prompt)?;
    log.append(
        "link_substitution_query",
        crate::links_substitution_query::render_link_substitution_query(&program),
    );

    let effects: Vec<CrudEvent> = program.rules.iter().map(link_substitution_effect).collect();
    if let Some(effect) = effects.iter().find(|effect| **effect != CrudEvent::Read) {
        log.append("link_substitution_rejected", format!("{effect:?}"));
        return Some(finalize_simple(
            prompt,
            log,
            "memory_link_query_rejected",
            "response:memory_link_query_rejected",
            &write_boundary(*effect),
            0.9,
        ));
    }

    let links = projected_links(store);
    let matched = program.matched_links(&links);
    log.append("link_substitution_matched", matched.len().to_string());
    Some(finalize_simple(
        prompt,
        log,
        "memory_link_query",
        "response:memory_link_query",
        &render_matches(&matched, links.len()),
        0.9,
    ))
}

/// Accept the query itself as the request.
///
/// Mirrors the text domain's recogniser in `code_artifact`: the whole turn must
/// be the query, and parsing is the recognition. A prompt that merely opens with
/// a parenthesis but does not parse falls through to natural-language recall
/// rather than being answered with a parse error, so ordinary prose is never
/// captured by this route.
fn explicit_link_query(prompt: &str) -> Option<LinkRewriteProgram> {
    let trimmed = prompt.trim();
    if !trimmed.starts_with('(') {
        return None;
    }
    let program = parse_link_substitution_query(trimmed, MAX_REWRITE_STEPS).ok()?;
    (!program.rules.is_empty()).then_some(program)
}

/// The doublet projection of everything in the store.
fn projected_links(store: &MemoryStore) -> Vec<DoubletLink> {
    memory_events_to_link_records(store.events())
        .into_iter()
        .flat_map(|record| record.links)
        .collect()
}

fn render_matches(matched: &[DoubletLink], total: usize) -> String {
    let mut body = format!("matched {} of {total} links", matched.len());
    for link in matched {
        let _ = write!(body, "\n{}", render_link(link));
    }
    body
}

/// Why a link-level write cannot be honoured here.
///
/// The projection is derived and one-way: `memory_event_to_link_record`
/// content-addresses each record id from the event's own canonical form, so an
/// edited link has no inverse back to the event that produced it, and
/// `LinkStore`'s only write is `append_memory_event` — an event, not a link.
/// Accepting the query and quietly rewriting a projection nothing reads back
/// would report success for a change the store never made, so the boundary is
/// named instead.
fn write_boundary(effect: CrudEvent) -> String {
    let attempted = match effect {
        CrudEvent::Create => "create",
        CrudEvent::Delete => "delete",
        CrudEvent::Update => "update",
        CrudEvent::Read | CrudEvent::Manual => "change",
    };
    format!(
        "This query would {attempted} links, which memory cannot apply: the doublet view is a \
         one-way projection of memory events, so an edited link has no way back to the event it \
         came from. Reads are supported here -- ((($i: $s $t)) (($i: $s $t))) matches every link. \
         To change memory, write to it in natural language, which appends an event the projection \
         is then derived from."
    )
}
