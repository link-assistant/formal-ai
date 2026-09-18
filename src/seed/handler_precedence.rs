//! Specialized-handler precedence loaded from `data/seed/handler-precedence.lino`
//! (issue #663).
//!
//! Handler precedence is behaviour — the order in which the universal solver
//! tries its specialized handlers, first-match-wins — and behaviour belongs in
//! seed data ("Data Is The Interface"), not in a Rust constant. This loader
//! reads the ordered handler rows — each row is a bare handler name, so the
//! precedence stays invisible to the seed's meaning-closure audit (only *value*
//! tokens are grounded, never the head slug a row names).
//! [`super::super::solver_dispatch`] joins that order with the Rust function
//! pointers (which must stay code) and asserts the two are an exact permutation
//! of each other, so a seed edit can never silently drop or duplicate a handler.
//!
//! The JavaScript worker loads the synced deployment copy through
//! `src/web/seed_loader.js`, and a routing-parity fixture pins the shared
//! precedence invariants across the Rust and browser surfaces.

use super::parser::parse_lino;

/// Ordered specialized-handler names, in dispatch precedence order (first wins),
/// as declared by the shipped `data/seed/handler-precedence.lino`.
/// Built once from the seed links network.
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
    network
        .nodes_under(&root.index)
        .into_iter()
        .map(|row| row.to.clone())
        .collect()
}

/// Repository path of the precedence seed, its name in the seed links network.
pub const HANDLER_PRECEDENCE_PATH: &str = "data/seed/handler-precedence.lino";

/// The guard note a precedence row carries when only the browser worker runs
/// it (plan 09 leaf 13).
const BROWSER_ONLY_MARK: &str = "browser_only";

/// The precedence rows the seed marks `browser_only`: handlers only the browser
/// worker runs (issue #1138 B9, plan 09 leaf 13). The native dispatcher skips
/// them when it joins the order to its function pointers, while the worker's
/// registry keeps them — one vocabulary, with the phase a row runs in declared
/// in the seed rather than hidden on either surface.
#[must_use]
pub fn browser_only_handlers() -> &'static [String] {
    static CELL: std::sync::OnceLock<Vec<String>> = std::sync::OnceLock::new();
    CELL.get_or_init(|| {
        crate::seed::seed_files()
            .into_iter()
            .find(|(path, _)| *path == HANDLER_PRECEDENCE_PATH)
            .map(|(_, text)| text)
            .map(|text| {
                text.lines()
                    .filter(|line| line.contains(BROWSER_ONLY_MARK))
                    .filter_map(|line| {
                        let name = line.split_whitespace().next()?;
                        (name.chars().next().is_some_and(|first| first != '#'))
                            .then(|| name.to_owned())
                    })
                    .collect()
            })
            .unwrap_or_default()
    })
}

/// Parse an arbitrary handler-precedence document into its ordered handler names.
///
/// Exposed so tests can reorder rows in a fixture and observe the routing change
/// (`routing_precedence_from_seed`).
#[must_use]
pub fn handler_precedence_from(seed: &str) -> Vec<String> {
    let tree = parse_lino(seed);
    let Some(root) = tree.children.first() else {
        return Vec::new();
    };
    // Each row under the root is a bare handler name (comments already stripped
    // by the parser), in dispatch precedence order.
    root.children
        .iter()
        .map(|child| child.name.clone())
        .collect()
}
