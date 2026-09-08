//! Seed-declared intent routing, read from the seed links network.
//!
//! Issue #1085 (D1.2): `intent_routing()` walks the projected
//! `data/seed/intent-routing.lino` document through link queries;
//! `intent_routing_from` keeps the tree parser it replaced so a test can show
//! both read the same routes.

use super::parser::parse_lino;

/// Intent routing record from `data/seed/intent-routing.lino`.
///
/// Match semantics (mirrored in `src/web/formal_ai_worker.js`):
/// - `keywords`: exact match of the entire normalized prompt
/// - `phrases`: exact match of the entire normalized prompt (kept as a
///   separate label so multi-word entries are easy to spot in `.lino`)
/// - `tokens`: any single whitespace-separated token equals the value
/// - `combos`: every token in the combo appears as a whitespace-separated
///   token in the prompt (in any order)
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct IntentRoute {
    pub id: String,
    pub slug: String,
    pub response_link: String,
    pub keywords: Vec<String>,
    pub phrases: Vec<String>,
    pub tokens: Vec<String>,
    pub combos: Vec<Vec<String>>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct IntentRouting {
    pub intents: Vec<IntentRoute>,
    pub article_prefixes: Vec<String>,
    pub trace_prefixes: Vec<String>,
}

#[must_use]
pub fn intent_routing() -> IntentRouting {
    let network = crate::seed_links::network();
    let mut routing = IntentRouting::default();
    let Some(root) = network.top_level(INTENT_ROUTING_PATH).into_iter().next() else {
        return routing;
    };
    for child in network.nodes_under(&root.index) {
        match child.to.as_str() {
            "intent" => routing.intents.push(IntentRoute {
                id: network
                    .value_of(&child.index)
                    .unwrap_or_default()
                    .to_owned(),
                slug: network
                    .field(&child.index, "slug")
                    .unwrap_or_default()
                    .to_owned(),
                response_link: network
                    .field(&child.index, "response_link")
                    .unwrap_or_default()
                    .to_owned(),
                keywords: network.field_values(&child.index, "keyword"),
                phrases: network.field_values(&child.index, "phrase"),
                tokens: network.field_values(&child.index, "token"),
                combos: network
                    .field_values(&child.index, "combo")
                    .iter()
                    .map(|combo| {
                        combo
                            .split('+')
                            .map(str::trim)
                            .filter(|part| !part.is_empty())
                            .map(ToOwned::to_owned)
                            .collect()
                    })
                    .collect(),
            }),
            "article" => routing.article_prefixes.push(
                network
                    .value_of(&child.index)
                    .unwrap_or_default()
                    .to_owned(),
            ),
            "trace_prefix" => routing.trace_prefixes.push(
                network
                    .value_of(&child.index)
                    .unwrap_or_default()
                    .to_owned(),
            ),
            _ => {}
        }
    }
    routing
}

/// Repository path of the intent routing seed, its name in the seed links network.
pub const INTENT_ROUTING_PATH: &str = "data/seed/intent-routing.lino";

/// Parse intent routing from a document's text; the tree parser the network
/// replaced, kept so a test can show both read the same routes.
#[must_use]
pub fn intent_routing_from(text: &str) -> IntentRouting {
    let tree = parse_lino(text);
    let mut routing = IntentRouting::default();
    if let Some(root) = tree.children.first() {
        for child in &root.children {
            match child.name.as_str() {
                "intent" => {
                    let mut keywords = Vec::new();
                    let mut phrases = Vec::new();
                    let mut tokens = Vec::new();
                    let mut combos = Vec::new();
                    for entry in &child.children {
                        match entry.name.as_str() {
                            "keyword" => keywords.push(entry.id.clone()),
                            "phrase" => phrases.push(entry.id.clone()),
                            "token" => tokens.push(entry.id.clone()),
                            "combo" => combos.push(
                                entry
                                    .id
                                    .split('+')
                                    .map(str::trim)
                                    .filter(|s| !s.is_empty())
                                    .map(ToOwned::to_owned)
                                    .collect(),
                            ),
                            _ => {}
                        }
                    }
                    routing.intents.push(IntentRoute {
                        id: child.id.clone(),
                        slug: child.find_child_value("slug").to_string(),
                        response_link: child.find_child_value("response_link").to_string(),
                        keywords,
                        phrases,
                        tokens,
                        combos,
                    });
                }
                "article" => routing.article_prefixes.push(child.id.clone()),
                "trace_prefix" => routing.trace_prefixes.push(child.id.clone()),
                _ => {}
            }
        }
    }
    routing
}
