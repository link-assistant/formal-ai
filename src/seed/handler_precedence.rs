//! Specialized-handler precedence loaded from `data/seed/handler-precedence.lino`
//! (issue #663; rank links added by issue #1138 plan 09 leaf 41).
//!
//! Handler precedence is behaviour — the order in which the universal solver
//! tries its specialized handlers, first-match-wins — and behaviour belongs in
//! seed data ("Data Is The Interface"), not in a Rust constant. Each row is a
//! `handler <name>` block whose `rank` link carries its dispatch precedence:
//! the loaders below sort by rank, so document position carries no meaning and
//! an operator edit is changing a rank value, not moving a row. Rows marked
//! `browser_only true` name handlers only the browser worker runs. Only *value*
//! tokens are grounded, never the head slug a row names, so the precedence
//! stays invisible to the seed's meaning-closure audit.
//! [`super::super::solver_dispatch`] joins that order with the Rust function
//! pointers (which must stay code) and asserts the two are an exact permutation
//! of each other, so a seed edit can never silently drop or duplicate a handler.
//!
//! The JavaScript worker loads the synced deployment copy and derives its
//! dispatch from the same rank-sorted precedence through the wasm parser
//! export, and a routing-parity fixture pins the shared precedence invariants
//! across the Rust and browser surfaces.

use super::parser::parse_lino;

/// Ordered specialized-handler names, in dispatch precedence order (first wins).
///
/// Sorted by the rank links the shipped `data/seed/handler-precedence.lino`
/// declares. Built once from the seed links network.
///
/// `specialized_handlers()` asks for the precedence on every dispatch, so
/// rebuilding it from the network per call is per-request work that grows with
/// the seed (issue #1085 D1.2).
#[must_use]
pub fn handler_precedence() -> &'static [String] {
    static CELL: std::sync::OnceLock<Vec<String>> = std::sync::OnceLock::new();
    CELL.get_or_init(load_handler_precedence)
}

fn load_handler_precedence() -> Vec<String> {
    let network = crate::seed_links::network();
    let Some(root) = network
        .top_level(HANDLER_PRECEDENCE_PATH)
        .into_iter()
        .next()
    else {
        return Vec::new();
    };
    let rows = network
        .nodes_under(&root.index)
        .into_iter()
        .map(|row| {
            (
                network.field(&row.index, "rank").map(str::to_owned),
                network.value_of(&row.index).unwrap_or_default().to_owned(),
            )
        })
        .collect();
    order_by_rank_links(rows)
}

/// Sort `(rank link, handler name)` rows into dispatch order: ascending rank.
///
/// A row without a rank link, a rank that is not an unsigned integer, or two
/// rows sharing a rank are all seed errors that panic loudly — any of them
/// would put part of the dispatch order back on document position, the exact
/// ambiguity rank links exist to remove (plan 09 leaf 41).
fn order_by_rank_links(rows: Vec<(Option<String>, String)>) -> Vec<String> {
    let mut ranked: Vec<(u32, String)> = Vec::with_capacity(rows.len());
    for (rank, name) in rows {
        let Some(rank) = rank else {
            panic!(
                "handler-precedence.lino row `{name}` declares no rank link; \
                 precedence is rank links, so every row must carry one"
            );
        };
        let parsed: u32 = rank.parse().unwrap_or_else(|_| {
            panic!(
                "handler-precedence.lino row `{name}` declares rank `{rank}`, \
                 which is not an unsigned integer"
            )
        });
        ranked.push((parsed, name));
    }
    ranked.sort_by(|left, right| left.0.cmp(&right.0).then_with(|| left.1.cmp(&right.1)));
    for pair in ranked.windows(2) {
        assert_ne!(
            pair[0].0, pair[1].0,
            "handler-precedence.lino rows `{}` and `{}` share rank {}; a shared rank \
             would decide the dispatch order by document position",
            pair[0].1, pair[1].1, pair[0].0
        );
    }
    ranked.into_iter().map(|(_, name)| name).collect()
}

/// Repository path of the precedence seed, its name in the seed links network.
pub const HANDLER_PRECEDENCE_PATH: &str = "data/seed/handler-precedence.lino";

/// The precedence rows the seed marks `browser_only true`: handlers only the
/// browser worker runs (issue #1138 B9, plan 09 leaf 13).
///
/// The native dispatcher
/// skips them when it joins the order to its function pointers, while the
/// worker's registry keeps them — one vocabulary, with the phase a row runs in
/// declared in the seed rather than hidden on either surface.
#[must_use]
pub fn browser_only_handlers() -> &'static [String] {
    static CELL: std::sync::OnceLock<Vec<String>> = std::sync::OnceLock::new();
    CELL.get_or_init(|| {
        let network = crate::seed_links::network();
        let Some(root) = network
            .top_level(HANDLER_PRECEDENCE_PATH)
            .into_iter()
            .next()
        else {
            return Vec::new();
        };
        network
            .nodes_under(&root.index)
            .into_iter()
            .filter(|row| {
                network
                    .field(&row.index, "browser_only")
                    .is_some_and(|mark| mark == "true")
            })
            .filter_map(|row| network.value_of(&row.index).map(str::to_owned))
            .collect()
    })
}

/// Parse a handler-precedence document into its rank-ordered handler names.
///
/// Exposed so tests can swap rank links in a fixture and observe the routing
/// change (`routing_precedence_from_seed`). Reads the same `handler <name>` +
/// `rank <n>` row shape the network loader reads, and orders by the same
/// rank-links rule as the network loader.
#[must_use]
pub fn handler_precedence_from(seed: &str) -> Vec<String> {
    let tree = parse_lino(seed);
    let Some(root) = tree.children.first() else {
        return Vec::new();
    };
    let rows = root
        .children
        .iter()
        .filter(|child| child.name == "handler")
        .map(|child| {
            let rank = child.find_child_value("rank");
            (
                (!rank.is_empty()).then(|| rank.to_owned()),
                child.id.clone(),
            )
        })
        .collect();
    order_by_rank_links(rows)
}
