//! Coreference-resolution records loaded from `data/seed/coreference.lino`.
//!
//! The seed encodes a tiny pronoun catalogue plus an antecedent catalogue:
//!
//! - `pronoun "<token>"` lists `contexts` (substrings that must appear in the
//!   normalized prompt for that pronoun to fire) and an optional `starts_with`
//!   guard for the prompt-initial case. Multiple pronouns can coexist —
//!   adding a Russian or Hindi pronoun requires only a new seed entry.
//! - `antecedent "<DisplayName>"` lists multilingual `aliases` (substrings
//!   we look for in the prior conversation turn), an optional Wikidata
//!   Q-ID, the `intent` label finalize logs, and the canonical response
//!   `body` the handler returns.
//!
//! The matching contract is intentionally narrow so the handler stays
//! deterministic:
//!
//! 1. At least one `pronoun.contexts` substring (or `starts_with` prefix)
//!    must appear in the normalized prompt.
//! 2. The prior user turn must contain at least one antecedent alias.
//!
//! The first antecedent whose aliases match is returned, and the caller
//! emits the response body verbatim.

use super::parser::parse_lino;
use super::COREFERENCE_LINO;

/// A pronoun and the contexts that opt prompts into coreference mode.
#[derive(Debug, Clone, Default)]
pub struct Pronoun {
    pub token: String,
    pub contexts: Vec<String>,
    pub starts_with: Vec<String>,
}

/// An antecedent (subject) that a pronoun can resolve against.
#[derive(Debug, Clone, Default)]
pub struct Antecedent {
    pub display_name: String,
    pub aliases: Vec<String>,
    pub wikidata: String,
    pub intent: String,
    pub body: String,
}

/// Top-level coreference-seed bundle.
#[derive(Debug, Clone, Default)]
pub struct CoreferenceSeeds {
    pub pronouns: Vec<Pronoun>,
    pub antecedents: Vec<Antecedent>,
}

impl CoreferenceSeeds {
    /// Return `true` when the normalized prompt contains any pronoun's
    /// context substring (or matches its prompt-initial prefix).
    #[must_use]
    pub fn matches_pronoun(&self, normalized: &str) -> bool {
        self.pronouns.iter().any(|pronoun| {
            pronoun
                .contexts
                .iter()
                .any(|context| !context.is_empty() && normalized.contains(context.as_str()))
                || pronoun
                    .starts_with
                    .iter()
                    .any(|prefix| !prefix.is_empty() && normalized.starts_with(prefix.as_str()))
        })
    }

    /// Pick the first antecedent whose alias appears in the prior turn.
    #[must_use]
    pub fn pick_antecedent(&self, previous_turn_lower: &str) -> Option<&Antecedent> {
        self.antecedents.iter().find(|antecedent| {
            antecedent
                .aliases
                .iter()
                .any(|alias| !alias.is_empty() && previous_turn_lower.contains(alias.as_str()))
        })
    }
}

#[must_use]
pub fn coreference_seeds() -> CoreferenceSeeds {
    let tree = parse_lino(COREFERENCE_LINO);
    let mut seeds = CoreferenceSeeds::default();
    let Some(root) = tree.children.iter().find(|c| c.name == "coreference") else {
        return seeds;
    };
    for child in &root.children {
        match child.name.as_str() {
            "pronoun" => {
                if child.id.is_empty() {
                    continue;
                }
                let contexts = child
                    .children
                    .iter()
                    .filter(|c| c.name == "context")
                    .map(|c| c.id.to_lowercase())
                    .filter(|c| !c.is_empty())
                    .collect();
                let starts_with = child
                    .children
                    .iter()
                    .filter(|c| c.name == "starts_with")
                    .map(|c| c.id.to_lowercase())
                    .filter(|c| !c.is_empty())
                    .collect();
                seeds.pronouns.push(Pronoun {
                    token: child.id.clone(),
                    contexts,
                    starts_with,
                });
            }
            "antecedent" => {
                if child.id.is_empty() {
                    continue;
                }
                let body = child.find_child_value("body").to_owned();
                if body.is_empty() {
                    continue;
                }
                let aliases = child
                    .children
                    .iter()
                    .filter(|c| c.name == "alias")
                    .map(|c| c.id.to_lowercase())
                    .filter(|a| !a.is_empty())
                    .collect();
                seeds.antecedents.push(Antecedent {
                    display_name: child.id.clone(),
                    aliases,
                    wikidata: child.find_child_value("wikidata").to_owned(),
                    intent: child.find_child_value("intent").to_owned(),
                    body,
                });
            }
            _ => {}
        }
    }
    seeds
}
