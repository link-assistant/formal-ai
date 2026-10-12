use super::selected;

/// The registered family selects only its explicit paths, never a later family or an unknown output.
#[test]
fn registered_projection_follows_declared_family_sources() {
    let rules = "notation-rules\n  family any\n    rule shared-lexeme-fields\n    files data/seed/example.lino\n  family unrelated\n    files data/seed/other.lino\n";
    assert!(selected(rules, "example.lino"));
    assert!(!selected(rules, "other.lino"));
    assert!(!selected(rules, "unknown.lino"));
}
