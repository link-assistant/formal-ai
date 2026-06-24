//! Issue #559: route→method aliases as grounded link data (R336).
//!
//! The meta core resolves a work-unit leaf's route slug to a catalogued method.
//! Most slugs name a method directly; a few meta-language intents are coarser or
//! finer than the handler that serves them, and `data/meta/route-method-aliases.lino`
//! is the data bridge for those. These tests keep that bridge grounded: every
//! alias target must be a real method in the registry derived from the live
//! dispatch code, and every alias must be *necessary* (its route slug is not
//! already a method name), so the map can never drift into stale or redundant
//! entries. We also pin the concrete gap the map closes — a `write_program` leaf
//! now resolves to a method in the solution evidence — and the trace-only
//! contract: resolution enriches the audit, it does not change routing.

use formal_ai::intent_formalization::formalize_intent;
use formal_ai::meta_frame::{NeedLedger, ProblemFrame, WorkUnit};
use formal_ai::method_registry::MethodRegistry;
use formal_ai::route_method_alias::{aliases, method_for_alias};
use formal_ai::solution_evidence::SolutionEvidence;
use formal_ai::translation::formalize_prompt;

#[test]
fn alias_catalogue_is_non_empty_and_closes_the_write_program_gap() {
    let all = aliases();
    assert!(
        !all.is_empty(),
        "the alias catalogue must enumerate the known route→method divergences"
    );
    assert_eq!(
        method_for_alias("write_program"),
        Some("write_script"),
        "the write_program intent must alias to the registered write_script method"
    );
    for alias in all {
        assert!(
            !alias.rationale.is_empty(),
            "alias {} → {} must explain why it holds",
            alias.route,
            alias.method
        );
    }
}

#[test]
fn every_alias_target_is_a_registered_method() {
    let registry = MethodRegistry::from_dispatch();
    for alias in aliases() {
        assert!(
            registry.method_for_route(&alias.method).is_some(),
            "alias target `{}` (from route `{}`) must be a registered method",
            alias.method,
            alias.route
        );
    }
}

#[test]
fn every_alias_is_necessary_and_unique() {
    let registry = MethodRegistry::from_dispatch();
    let mut seen = std::collections::BTreeSet::new();
    for alias in aliases() {
        assert!(
            seen.insert(alias.route.clone()),
            "route `{}` must have exactly one alias record",
            alias.route
        );
        assert_ne!(
            alias.route, alias.method,
            "an alias must rename a route to a different method; `{}` is redundant",
            alias.route
        );
        // If the route slug already named a method, no alias would be needed: the
        // direct lookup would have matched it. So the route must not itself be a
        // registered method name.
        assert!(
            !registry.methods.iter().any(|m| m.name == alias.route),
            "route `{}` is already a registered method; the alias is redundant",
            alias.route
        );
    }
}

#[test]
fn method_for_route_resolves_directly_and_via_alias() {
    let registry = MethodRegistry::from_dispatch();
    // Direct: a route slug that names a method resolves to itself.
    assert_eq!(
        registry
            .method_for_route("translation")
            .map(|m| m.name.as_str()),
        Some("translation"),
        "a direct route slug must resolve to the identically named method"
    );
    // Aliased: write_program resolves to write_script via the link data.
    assert_eq!(
        registry
            .method_for_route("write_program")
            .map(|m| m.name.as_str()),
        Some("write_script"),
        "write_program must resolve through the alias to write_script"
    );
    // Unknown: an unroutable slug resolves to nothing.
    assert!(
        registry.method_for_route("zzqqx_no_such_route").is_none(),
        "an unknown route slug must not resolve to any method"
    );
}

#[test]
fn write_program_need_resolves_to_a_method_in_solution_evidence() {
    // The conjunction surfaces a translation need (direct) and a program-writing
    // need (aliased). Both must resolve to a catalogued method, so the evidence
    // reports every detected need as addressed (R334) rather than leaving the
    // program-writing need methodless.
    let prompt = "translate apple to Russian and write a hello world program in Python";
    let candidate = formalize_prompt(prompt, "en");
    let formalization = formalize_intent(prompt, "en", Some(&candidate));
    let frame = ProblemFrame::from_formalization(&formalization);
    let root = WorkUnit::from_formalization(&formalization, 4);
    let ledger = NeedLedger::resolve(&frame, &root);
    let registry = MethodRegistry::from_dispatch();
    let evidence = SolutionEvidence::assemble(&frame, &ledger, &registry);

    assert_eq!(
        evidence.resolved_to_method(),
        evidence.trails.len(),
        "every detected need must resolve to a catalogued method"
    );
    assert!(
        evidence
            .trails
            .iter()
            .any(|trail| trail.method_via_alias && trail.method.as_deref() == Some("write_script")),
        "the program-writing need must resolve to write_script via the alias"
    );
    let lino = evidence.to_links_notation();
    assert!(
        lino.contains("method_via_alias \"true\""),
        "alias provenance must appear in the evidence trace:\n{lino}"
    );
}
