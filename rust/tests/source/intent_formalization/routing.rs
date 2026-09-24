//! Intent-route selection for the mirror formalizer.
//!
//! Split out of `intent_formalization.rs` at the 1000-line gate. The order
//! here is the order the retired planner arms had: write_program parameters
//! first, then the conversational families' declared role surfaces, then the
//! seeded keyword/phrase/token/combo rows.

use crate::engine::WRITE_PROGRAM_INTENT;
use crate::seed;

use super::{contains_token, write_program_parameters};

pub(super) struct MatchedRoute {
    pub(super) slug: String,
    pub(super) response_link: String,
}

pub(super) fn route_for_prompt(normalized: &str) -> Option<MatchedRoute> {
    if write_program_parameters(normalized).is_some() {
        return Some(MatchedRoute {
            slug: String::from(WRITE_PROGRAM_INTENT),
            response_link: String::from("response:write_program"),
        });
    }
    // Before the table: the conversational families' blocks sat ahead of the
    // write_program family's, so their rows claimed `hello` before
    // write_program's `keyword hello` (a row shadowed since it was written).
    // The declared role surfaces keep that precedence for the retired rows.
    declared_role_surface_route(normalized).or_else(|| {
        seed::intent_routing()
            .intents
            .iter()
            .find(|route| matches_route(normalized, route))
            .map(|route| MatchedRoute {
                slug: route.slug.clone(),
                response_link: route.response_link.clone(),
            })
    })
}

/// The conversational families' bare whole prompts (`hi`, `how are you`,
/// `who are you`) retired onto seeded roles (issue #1138 plan 10 leaf 20):
/// the prompt has no object to derive from, so the decision is the declared
/// roles' surface inventories under the same whole-prompt equality the
/// exact-match rows had. The declaration lives on the family's own block
/// (`role_surface <role>`), so a future family retires its rows by seeding
/// the role and declaring it — no code change. Compound courtesy
/// (`привет как дела`) is a surface of its own family's role, exactly as it
/// was a phrase row.
fn declared_role_surface_route(normalized: &str) -> Option<MatchedRoute> {
    seed::intent_routing()
        .intents
        .iter()
        .filter(|route| !route.role_surfaces.is_empty())
        .find(|route| {
            route.role_surfaces.iter().any(|role| {
                seed::lexicon()
                    .words_for_role(role)
                    .iter()
                    .any(|word| word.as_str() == normalized)
            })
        })
        .map(|route| MatchedRoute {
            slug: route.slug.clone(),
            response_link: route.response_link.clone(),
        })
}

fn matches_route(normalized: &str, route: &seed::IntentRoute) -> bool {
    route.keywords.iter().any(|keyword| normalized == keyword)
        || route.phrases.iter().any(|phrase| normalized == phrase)
        || route
            .tokens
            .iter()
            .any(|token| contains_token(normalized, token))
        || route.combos.iter().any(|combo| {
            !combo.is_empty() && combo.iter().all(|token| contains_token(normalized, token))
        })
}
