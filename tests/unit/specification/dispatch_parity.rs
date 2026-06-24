//! Issue #559 (R344): the registry-vs-legacy dispatch parity certificate.
//!
//! These tests pin the retire-parity invariant: across the *entire* route
//! vocabulary the system can emit — not just the routes one prompt happens to
//! produce — the data-driven method registry never contradicts the legacy dispatch
//! authority. While that holds, the registry is a behaviour-preserving drop-in for
//! the hardcoded `specialized_handler_name` table, which is the precondition for
//! retiring that table in a later phase. A new route that the registry resolved
//! differently (or failed to resolve) than the legacy would flip
//! `is_retire_safe()` to false and fail here.

use formal_ai::dispatch_parity::{record_dispatch_parity, DispatchParity};
use formal_ai::event_log::EventLog;
use formal_ai::method_registry::MethodRegistry;
use formal_ai::selection::SelectionAgreement;

#[test]
fn the_registry_contradicts_the_legacy_on_zero_routes() {
    let parity = DispatchParity::audit();
    assert_eq!(
        parity.contradiction_count(),
        0,
        "the registry must never pick a different real method than a valid legacy \
         selection, nor fail to resolve a route the legacy resolves"
    );
    assert!(
        parity.is_retire_safe(),
        "zero contradictions means the registry is a behaviour-preserving drop-in \
         for the legacy dispatch authority"
    );
}

#[test]
fn every_audited_route_is_accounted_for_in_exactly_one_class() {
    let parity = DispatchParity::audit();
    let classified = parity.agreement_count()
        + parity.rescue_count()
        + parity.unresolved_count()
        + parity.contradiction_count();
    assert_eq!(
        parity.route_count(),
        classified,
        "every route is classified as agree, rescue, unresolved, or contradict — \
         nothing is dropped or double-counted"
    );
    assert!(
        parity.route_count() > 0,
        "the corpus must be non-empty (it is derived from live data)"
    );
}

#[test]
fn the_corpus_has_no_duplicate_routes() {
    let parity = DispatchParity::audit();
    let mut routes: Vec<&str> = parity.routes.iter().map(|r| r.route.as_str()).collect();
    let total = routes.len();
    routes.sort_unstable();
    routes.dedup();
    assert_eq!(
        routes.len(),
        total,
        "the audited route corpus is de-duplicated"
    );
}

#[test]
fn the_registry_covers_every_registered_method_as_a_self_resolving_route() {
    // The strongest sense of "the registry can drive selection": every handler in
    // the live catalogue is reachable as a route that resolves to itself, with the
    // legacy authority agreeing. If a method existed that the registry could not
    // name from its own route, it could not drive selection for that method.
    let parity = DispatchParity::audit();
    let registry = MethodRegistry::from_dispatch();
    for method in &registry.methods {
        let entry = parity
            .routes
            .iter()
            .find(|route| route.route == method.name)
            .unwrap_or_else(|| panic!("method `{}` must appear as an audited route", method.name));
        assert_eq!(
            entry.registry_method.as_deref(),
            Some(method.name.as_str()),
            "route `{}` must resolve to itself through the registry",
            method.name
        );
        assert_eq!(
            entry.agreement,
            SelectionAgreement::Agree,
            "a method named as its own route must have both authorities agree: {}",
            method.name
        );
    }
}

#[test]
fn write_program_is_resolved_by_the_registry_through_its_alias() {
    // `write_program` is the one classifier intent whose slug is not itself a
    // registered method: the legacy catch-all names no real handler, but the
    // registry rescues it through the route→method alias to `write_script`. This
    // proves the alias bridge participates in the corpus-wide parity proof.
    let parity = DispatchParity::audit();
    let entry = parity
        .routes
        .iter()
        .find(|route| route.route == "write_program")
        .expect("write_program is part of the audited corpus");
    assert_eq!(entry.legacy_method, None, "the legacy names no real method");
    assert_eq!(
        entry.registry_method.as_deref(),
        Some("write_script"),
        "the registry resolves write_program to write_script via its alias"
    );
    assert_eq!(entry.agreement, SelectionAgreement::RegistryRescues);
}

#[test]
fn the_certificate_serializes_as_links_notation() {
    let parity = DispatchParity::audit();
    let lino = parity.to_links_notation();
    assert!(lino.contains("dispatch_parity"), "header record present");
    assert!(
        lino.contains("contradiction_count \"0\""),
        "the zero-contradiction count is serialized"
    );
    assert!(
        lino.contains("retire_safe \"true\""),
        "the retire-parity verdict is serialized"
    );
    assert!(
        lino.contains("record_type \"route_parity\""),
        "each route is serialized as its own record"
    );
}

#[test]
fn recording_the_certificate_appends_the_trace_events() {
    let mut log = EventLog::new();
    let parity = record_dispatch_parity(&mut log);
    let kinds: Vec<&str> = log.events().iter().map(|event| event.kind).collect();
    assert!(
        kinds.contains(&"dispatch_parity"),
        "the serialized certificate is appended as a trace event"
    );
    assert!(
        kinds.contains(&"dispatch_parity:contradictions"),
        "the contradiction count is appended as a compact, auditable event"
    );
    let contradictions = log
        .events()
        .iter()
        .find(|event| event.kind == "dispatch_parity:contradictions")
        .expect("the contradiction-count event was appended");
    assert_eq!(
        contradictions.payload, "0",
        "the trace records zero contradictions"
    );
    assert!(parity.is_retire_safe());
}
