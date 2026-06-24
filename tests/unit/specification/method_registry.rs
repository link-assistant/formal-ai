//! Issue #559: the method registry as grounded link data and live method
//! selection data (R331).
//!
//! The registry enumerates every method an atomic work-unit leaf can route to.
//! These tests keep the data *grounded in the live code*: the registry is built
//! from the dispatch constants, and we assert every derived name actually appears
//! in `src/solver_dispatch.rs` (and vice-versa for the contextual surface), so the
//! catalogue-as-data can never drift from the methods that really run. We also
//! pin the Links Notation serialization and the order the live executor consumes.

use std::fs;
use std::path::{Path, PathBuf};

use formal_ai::method_registry::{MethodRegistry, MethodSurface};

fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).to_path_buf()
}

fn dispatch_source() -> String {
    let path = repo_root().join("src/solver_dispatch.rs");
    fs::read_to_string(&path).unwrap_or_else(|error| panic!("solver_dispatch.rs readable: {error}"))
}

#[test]
fn registry_covers_all_dispatch_surfaces() {
    let registry = MethodRegistry::from_dispatch();
    assert_eq!(
        registry.count_on(MethodSurface::Prelude),
        5,
        "there are exactly five prelude methods"
    );
    assert!(
        registry.count_on(MethodSurface::Specialized) >= 40,
        "the specialized surface should enumerate the full ordered table, got {}",
        registry.count_on(MethodSurface::Specialized)
    );
    assert_eq!(
        registry.count_on(MethodSurface::Contextual),
        5,
        "there are exactly five contextual override handlers"
    );
    assert_eq!(
        registry.method_count(),
        registry.count_on(MethodSurface::Prelude)
            + registry.count_on(MethodSurface::Specialized)
            + registry.count_on(MethodSurface::Contextual),
        "every method must belong to exactly one surface"
    );
}

#[test]
fn every_prelude_and_specialized_method_is_named_in_the_dispatch_table() {
    let registry = MethodRegistry::from_dispatch();
    let source = dispatch_source();
    for method in registry
        .methods
        .iter()
        .filter(|m| m.surface == MethodSurface::Prelude)
    {
        assert!(
            source.contains(&format!("\"{}\"", method.name)),
            "prelude method `{}` must be a real entry in PRELUDE_METHOD_NAMES",
            method.name
        );
    }
    for method in registry
        .methods
        .iter()
        .filter(|m| m.surface == MethodSurface::Specialized)
    {
        // Each specialized handler appears as a `("name", try_...)` table entry.
        let needle = format!("(\"{}\",", method.name);
        assert!(
            source.contains(&needle),
            "specialized method `{}` must be a real entry in SPECIALIZED_HANDLERS",
            method.name
        );
    }
}

#[test]
fn every_contextual_method_is_a_real_override_arm() {
    let registry = MethodRegistry::from_dispatch();
    let source = dispatch_source();
    for method in registry
        .methods
        .iter()
        .filter(|m| m.surface == MethodSurface::Contextual)
    {
        // Each contextual handler is dispatched by a `"name" =>` match arm in
        // `try_contextual_override`; if the arm is removed the registry is wrong.
        let needle = format!("\"{}\" =>", method.name);
        assert!(
            source.contains(&needle),
            "contextual method `{}` must be a real arm in try_contextual_override",
            method.name
        );
    }
}

#[test]
fn specialized_order_follows_table_precedence() {
    let registry = MethodRegistry::from_dispatch();
    let specialized: Vec<&str> = registry
        .methods
        .iter()
        .filter(|m| m.surface == MethodSurface::Specialized)
        .map(|m| m.name.as_str())
        .collect();
    // The first table entry is the http_fetch handler; precedence is observable.
    assert_eq!(
        specialized.first().copied(),
        Some("http_fetch"),
        "the ordered table must lead with the first dispatch entry"
    );
    let orders: Vec<usize> = registry
        .methods
        .iter()
        .filter(|m| m.surface == MethodSurface::Specialized)
        .map(|m| m.order)
        .collect();
    assert!(
        orders.windows(2).all(|w| w[0] + 1 == w[1]),
        "specialized order must be a dense 0-based precedence sequence"
    );
}

#[test]
fn registry_order_starts_with_prelude_and_promotes_relevant_methods() {
    let registry = MethodRegistry::from_dispatch();
    let ordered = registry.ordered_method_names_for_relevants(&[
        "handler:write_program".to_owned(),
        "route:translation".to_owned(),
    ]);
    assert_eq!(
        &ordered[..5],
        [
            "diagnostic",
            "nl_tool",
            "behavior_rules",
            "feature_capability",
            "playwright_script",
        ],
        "prelude methods must always run first"
    );
    assert_eq!(
        ordered[5], "write_script",
        "handler:write_program should resolve through the route-method alias"
    );
    assert_eq!(
        ordered[6], "translation",
        "route:translation should promote the translation method"
    );
    let first_http = ordered
        .iter()
        .position(|name| name == "http_fetch")
        .expect("full specialized table appended after promoted methods");
    assert!(
        first_http > 6,
        "regular table entries should follow prelude and relevant promotions: {ordered:?}"
    );
}

#[test]
fn registry_serializes_to_grounded_links_notation() {
    let registry = MethodRegistry::from_dispatch();
    let lino = registry.to_links_notation();
    assert!(
        lino.contains("record_type \"method_registry\""),
        "the registry must declare its record_type:\n{lino}"
    );
    assert!(
        lino.contains("record_type \"method\""),
        "every method must serialize as its own record:\n{lino}"
    );
    assert!(
        lino.contains(&format!("method_count \"{}\"", registry.method_count())),
        "the registry must record its method count:\n{lino}"
    );
    assert!(
        lino.contains(&format!(
            "prelude_count \"{}\"",
            registry.count_on(MethodSurface::Prelude)
        )),
        "the registry must record its prelude count:\n{lino}"
    );
    assert!(
        lino.contains("surface \"prelude\"")
            && lino.contains("surface \"specialized\"")
            && lino.contains("surface \"contextual\""),
        "all dispatch surfaces must be represented:\n{lino}"
    );
    for method in &registry.methods {
        assert!(
            lino.contains(&format!("name \"{}\"", method.name)),
            "method {} must appear in the trace:\n{lino}",
            method.name
        );
    }
}
