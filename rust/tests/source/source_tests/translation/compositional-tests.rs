use super::{compose_spans, target_surface_is_complete};
#[test]
fn every_span_requires_target_evidence() {
    let lookup = |source: &str| match source {
        "red" => Some(("красный".to_owned(), vec!["lexical:red".to_owned()])),
        "orchard" => Some(("сад".to_owned(), vec!["lexical:orchard".to_owned()])),
        _ => None,
    };
    assert_eq!(
        compose_spans("red orchard", lookup).unwrap().0,
        "красный сад"
    );
    assert!(compose_spans("red unknown", lookup).is_none());
    assert_eq!(
        compose_spans("red-orchard", lookup).unwrap().0,
        "красный-сад"
    );
}
#[test]
fn diagnostics_and_foreign_script_are_not_target_output() {
    assert!(target_surface_is_complete("Web programmer", "en"));
    assert!(!target_surface_is_complete("-- Английский-->", "en"));
    assert!(!target_surface_is_complete("response:translate", "en"));
    assert!(!target_surface_is_complete("programmer неизвестно", "en"));
    assert!(!target_surface_is_complete("", "ru"));
}
