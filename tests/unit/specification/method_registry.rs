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
    let registry = MethodRegistry::shared();
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
        13,
        "there are exactly thirteen contextual override handlers"
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
fn task_decomposition_has_one_configured_contextual_dispatch_path() {
    let registry = MethodRegistry::shared();
    let entries = registry
        .methods
        .iter()
        .filter(|method| method.name == "task_decomposition")
        .collect::<Vec<_>>();
    assert_eq!(
        entries.len(),
        1,
        "the depth-aware handler must not also live in the regular specialized table"
    );
    assert_eq!(entries[0].surface, MethodSurface::Contextual);
}

#[test]
fn every_prelude_and_specialized_method_is_named_in_the_dispatch_table() {
    let registry = MethodRegistry::shared();
    let source = dispatch_source();
    let rule_backed = formal_ai::rule_interpreter::handler_names();
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
        // Each specialized handler appears as a `("name", try_...)` table entry,
        // or is a handler the rule interpreter runs from
        // `data/seed/handler-rules.lino` (issue #1085).
        let inline_needle = format!("(\"{}\",", method.name);
        let multiline_needle = format!("(\n        \"{}\",", method.name);
        assert!(
            source.contains(&inline_needle)
                || source.contains(&multiline_needle)
                || rule_backed.contains(&method.name.as_str()),
            "specialized method `{}` must be a real entry in HANDLER_FUNCTIONS or a \
             handler declared in data/seed/handler-rules.lino",
            method.name
        );
    }
}

#[test]
fn every_contextual_method_is_a_real_override_arm() {
    let registry = MethodRegistry::shared();
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
    let registry = MethodRegistry::shared();
    let specialized: Vec<&str> = registry
        .methods
        .iter()
        .filter(|m| m.surface == MethodSurface::Specialized)
        .map(|m| m.name.as_str())
        .collect();
    // Conversation controls precede generic URL handling; precedence is observable.
    assert_eq!(
        specialized.first().copied(),
        Some("conversation_control"),
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
    let registry = MethodRegistry::shared();
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
    let registry = MethodRegistry::shared();
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

// ---------------------------------------------------------------------------
// Corpus-wide dispatch closure (issue #559, R344).
//
// The registry is now the *sole* dispatch authority: the legacy hardcoded route
// mapper was removed once the corpus-wide parity certificate proved the registry
// was a behaviour-preserving replacement. This test preserves that certificate's
// invariant directly against the live registry, now with no second authority to
// compare against. The corpus is enumerated from live data, never a hand-kept
// list: every method name (a method is its own self-resolving route), every
// route→method alias (R336) — which includes the `write_program` intent the
// classifier emits directly — and every classifier route slug.
//
// Two facts are pinned. (1) Closure safety: no route the system can emit ever
// resolves to an *unregistered* method — the property that made the registry a
// safe drop-in for the retired table. (2) Coverage: every method-name route and
// every alias route resolves (these are exactly the routes the legacy authority
// also resolved, so the registry loses no coverage). Classifier slugs with no
// handler may legitimately stay unresolved, exactly as under the old certificate's
// four-way partition.
// ---------------------------------------------------------------------------

#[test]
fn the_registry_is_the_sole_authority_that_closes_over_the_route_corpus() {
    use formal_ai::route_method_alias::aliases;
    use formal_ai::seed::intent_routing;

    let registry = MethodRegistry::shared();

    // Routes that must resolve: every method name and every alias route.
    let mut must_resolve: Vec<String> = Vec::new();
    for method in &registry.methods {
        must_resolve.push(method.name.clone());
    }
    for alias in aliases() {
        must_resolve.push(alias.route.clone());
    }

    // The full vocabulary the system can ever emit (a superset of must_resolve).
    let mut corpus = must_resolve.clone();
    for intent in &intent_routing().intents {
        if !intent.slug.is_empty() {
            corpus.push(intent.slug.clone());
        }
    }
    corpus.sort_unstable();
    corpus.dedup();
    assert!(
        corpus.len() >= 40,
        "the route corpus should span the whole vocabulary, got {}",
        corpus.len()
    );

    // (1) Closure safety: whatever resolves, resolves to a registered method.
    for route in &corpus {
        if let Some(method) = registry.method_for_route(route) {
            assert!(
                registry.methods.iter().any(|m| m.name == method.name),
                "route `{route}` resolved to an unregistered method `{}`",
                method.name
            );
        }
    }

    // (2) Coverage: every method-name route and every alias route resolves.
    must_resolve.sort_unstable();
    must_resolve.dedup();
    for route in &must_resolve {
        assert!(
            registry.method_for_route(route).is_some(),
            "the sole dispatch authority must resolve route `{route}` to a method"
        );
    }
}

// ---------------------------------------------------------------------------
// Issue #1138, plan 07 leaves 4-6 and plan 12 leaf 2: one registry, three
// collections, one dispatch authority (R344).
//
// Plan 07 makes `learned_methods` executable at last precedence; plan 12 adds
// `heuristics`, which are never route targets. Both grow the same struct, so
// both are asserted here against the single declaration (plan 00 section 9 R16).
//
// Written before the leaves that make them pass (plan 14 wave T).

use formal_ai::method_registry::{LearnedMethod, LearnedMethodStatus};
use formal_ai::selection_heuristics::HeuristicRole;

/// The one adopted learned record the shipped seed carries.
const ADOPTED: &str = "learned_recursive_core_d21ca03aaabaf13d";

fn shipped_method(registry: &MethodRegistry) -> &LearnedMethod {
    registry
        .learned_methods
        .iter()
        .find(|method| method.name == ADOPTED)
        .expect("data/seed/learned-methods.lino carries the one adopted record")
}

#[test]
fn an_adopted_learned_method_is_dispatchable() {
    let seed = fs::read_to_string(repo_root().join("data/seed/learned-methods.lino"))
        .expect("learned-methods seed readable");
    let registry = MethodRegistry::from_store_with_learned_seed(&seed)
        .expect("an effect-qualified learned method loads");
    let adopted = shipped_method(&registry);
    assert_eq!(adopted.status, LearnedMethodStatus::Adopted);
    assert!(
        adopted.is_executable(),
        "every operation of the adopted record is already a recorder event kind the \
         recipe interpreter dispatches on, so it must bind"
    );
    assert!(
        adopted.to_recipe_program().is_ok(),
        "the learned operations project onto a recipe program; no second interpreter is \
         written"
    );

    let relevants = vec![format!("method:{ADOPTED}")];
    let ordered = registry.ordered_method_names_for_relevants(&relevants);
    assert!(
        ordered.iter().any(|name| name == ADOPTED),
        "an adopted learned method must appear in the one ordered selection path, or it \
         is knowledge nothing can reach: {ordered:?}"
    );
}

#[test]
fn learned_methods_rank_after_every_compiled_method() {
    let seed = fs::read_to_string(repo_root().join("data/seed/learned-methods.lino"))
        .expect("learned-methods seed readable");
    let registry = MethodRegistry::from_store_with_learned_seed(&seed)
        .expect("an effect-qualified learned method loads");
    let relevants = vec!["method:arithmetic".to_owned(), format!("method:{ADOPTED}")];
    let ordered = registry.ordered_method_names_for_relevants(&relevants);
    let learned_at = ordered.iter().position(|name| name == ADOPTED);
    let Some(learned_at) = learned_at else {
        panic!("the learned method must be reachable at all: {ordered:?}");
    };
    let compiled_names: Vec<&str> = registry
        .methods
        .iter()
        .map(|method| method.name.as_str())
        .collect();
    for (index, name) in ordered.iter().enumerate() {
        if compiled_names.contains(&name.as_str()) {
            assert!(
                index < learned_at,
                "compiled method `{name}` ranks after the learned method; learned \
                 methods are last, always (R331's signature and name are unchanged)"
            );
        }
    }
}

#[test]
fn an_ineffective_adoption_is_preserved_but_cannot_dispatch() {
    let seed = fs::read_to_string(repo_root().join("data/seed/learned-methods.lino"))
        .expect("learned-methods seed readable")
        .replace("status \"adopted\"", "status \"adopted_not_effective\"");
    let registry = MethodRegistry::from_store_with_learned_seed(&seed)
        .expect("the measured negative counterexample remains representable");
    let method = shipped_method(&registry);
    assert_eq!(method.status, LearnedMethodStatus::AdoptedNotEffective);
    assert!(
        method.is_executable(),
        "the negative result is about effect, not binding"
    );
    let ordered = registry.ordered_method_names_for_relevants(&[format!("method:{ADOPTED}")]);
    assert!(
        !ordered.iter().any(|name| name == ADOPTED),
        "changed-but-unverified experience is retained in the registry and never allowed to alter answers"
    );
}

#[test]
fn a_learned_method_with_an_unbound_operation_is_not_dispatched_and_is_named() {
    // An operation that binds to no recorder makes the whole method
    // non-executable; the method stays in the registry event as data and the
    // unbound operation is named. Silent skipping is forbidden.
    let seed = concat!(
        "learned_method \"learned_unbound_wave_t\"\n",
        "  algorithm_id \"algorithm_unbound_wave_t\"\n",
        "  evidence_id \"evidence_unbound_wave_t\"\n",
        "  operation \"need:status\"\n",
        "  operation \"operation_that_binds_to_nothing\"\n",
        "  support_trace_id \"trace_support_wave_t\"\n",
        "  held_out_trace_id \"trace_held_out_wave_t\"\n",
    );
    let registry = MethodRegistry::from_store_with_learned_seed(seed)
        .expect("a syntactically valid learned record parses");
    let unbound = registry
        .learned_methods
        .iter()
        .find(|method| method.name == "learned_unbound_wave_t")
        .expect("the record is present as data");
    assert!(
        !unbound.is_executable(),
        "one unbound operation makes the method non-executable"
    );
    let error = unbound
        .to_recipe_program()
        .expect_err("an unbound operation cannot produce a program");
    assert!(
        error.contains("operation_that_binds_to_nothing"),
        "the error must name the operation that did not bind, got {error:?}"
    );

    let relevants = vec!["method:learned_unbound_wave_t".to_owned()];
    assert!(
        !registry
            .ordered_method_names_for_relevants(&relevants)
            .iter()
            .any(|name| name == "learned_unbound_wave_t"),
        "a non-executable learned method is never dispatched"
    );
}

#[test]
fn an_answer_that_used_a_learned_method_says_so_in_the_trace() {
    // Plan 07 leaf 5 adds the fourth loop to `ordered_method_names_for_relevants`
    // plus the `method:learned` trace event, keeping the signature and the name
    // unchanged (R331).
    let source = fs::read_to_string(repo_root().join("src/method_registry.rs"))
        .expect("method_registry.rs readable");
    assert!(
        source.contains("method:learned"),
        "src/method_registry.rs must emit the `method:learned` trace event, so an answer \
         that used a learned method says so"
    );

    let registry = MethodRegistry::shared();
    let rendered = registry.to_links_notation();
    assert!(
        rendered.contains(ADOPTED),
        "the registry event names the adopted learned method: {rendered}"
    );
}

#[test]
fn a_heuristic_is_never_returned_by_method_for_route() {
    let registry = MethodRegistry::shared();
    assert!(
        !registry.heuristics.is_empty(),
        "plan 12 leaf 2 loads data/meta/selection-heuristics.lino into the registry"
    );
    for heuristic in &registry.heuristics {
        assert!(
            registry.method_for_route(&heuristic.name).is_none(),
            "`{}` is a heuristic, not a route target; a heuristic that answers a route \
             is a second dispatch authority",
            heuristic.name
        );
    }
    let ranking = registry.heuristics_for(HeuristicRole::Rank, "");
    assert!(
        !ranking.is_empty(),
        "the rank role must be seeded, since `least_action` is the migration identity"
    );
}

#[test]
fn the_registry_event_lists_every_heuristic_with_its_role_and_order() {
    let registry = MethodRegistry::shared();
    let rendered = registry.to_links_notation();
    for heuristic in &registry.heuristics {
        assert!(
            rendered.contains(&heuristic.name),
            "the registry event must list `{}`; one authority, one event",
            heuristic.name
        );
    }
    assert!(
        rendered.contains("heuristic_count"),
        "the event states how many heuristics the registry holds, beside the method and \
         learned counts"
    );
}
