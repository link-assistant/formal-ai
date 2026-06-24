//! Issue #559: route→method aliases as first-class link data.
//!
//! The meta core decomposes a request into work-unit leaves, each carrying a
//! *route* slug from the meaning record — the meta-language intent vocabulary
//! (`write_program`, `software_project_plan`, …). It resolves that leaf against
//! the [`MethodRegistry`](crate::method_registry::MethodRegistry), whose names
//! are the *method* vocabulary derived from the live dispatch code (`write_script`,
//! `software_project`, …). The two vocabularies mostly coincide, so a route slug
//! usually names a method directly. A few meta-language intents are coarser or
//! finer than the handler that resolves them, though, and for those the direct
//! lookup finds nothing — making the solution evidence (R334) report a need as
//! unaddressed when it is in fact served.
//!
//! This module supplies the missing bridge as data, not code: it loads
//! `data/meta/route-method-aliases.lino` and exposes the alias each divergent
//! route slug resolves to. The map is grounded — a specification test asserts
//! every alias target is a real registered method and every alias is necessary
//! (the route slug is not already a method name) — so it can never drift from the
//! live catalogue. Resolution is trace-only: it enriches the evidence join, it
//! does not change routing or the produced answer (R13).

use crate::seed::parser::parse_lino;
use std::sync::OnceLock;

/// One route→method alias: a meta-language intent slug and the registered method
/// that resolves it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RouteMethodAlias {
    /// The route slug as it appears in a work-unit leaf / ledger row.
    pub route: String,
    /// The registered method name this route resolves to.
    pub method: String,
    /// Why the alias holds — kept beside the data so the map explains itself.
    pub rationale: String,
}

const ALIASES_LINO: &str = include_str!("../data/meta/route-method-aliases.lino");

/// The route→method alias catalogue, parsed once from the embedded link data.
#[must_use]
pub fn aliases() -> &'static [RouteMethodAlias] {
    static CELL: OnceLock<Vec<RouteMethodAlias>> = OnceLock::new();
    CELL.get_or_init(load_aliases)
}

fn load_aliases() -> Vec<RouteMethodAlias> {
    let tree = parse_lino(ALIASES_LINO);
    let mut out = Vec::new();
    for record in &tree.children {
        if record.find_child_value("record_type") != "route_method_alias" {
            continue;
        }
        let route = record.find_child_value("route").to_owned();
        let method = record.find_child_value("method").to_owned();
        if route.is_empty() || method.is_empty() {
            continue;
        }
        out.push(RouteMethodAlias {
            route,
            method,
            rationale: record.find_child_value("rationale").to_owned(),
        });
    }
    out
}

/// Resolve a route slug to the method name it aliases, if an alias is recorded.
#[must_use]
pub fn method_for_alias(route: &str) -> Option<&'static str> {
    aliases()
        .iter()
        .find(|alias| alias.route == route)
        .map(|alias| alias.method.as_str())
}
