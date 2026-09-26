//! Plan 07 capability ratchets: only a measured improvement may be adopted,
//! and every adopted method must have a production read path.

use std::fs;
use std::path::Path;

use formal_ai::method_registry::{LearnedMethodStatus, MethodRegistry};
use lino_objects_codec::format::parse_indented;

const MEASURED_METHOD: &str = "learned_recursive_core_d21ca03aaabaf13d";

#[test]
fn adopted_methods_equal_methods_with_a_qualifying_effect() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("the repository root sits one level above the crate");
    let registry = MethodRegistry::shared();
    let adopted = registry
        .learned_methods
        .iter()
        .filter(|method| method.status == LearnedMethodStatus::Adopted)
        .count();
    let ledger = fs::read_to_string(root.join("data/meta/learning-adoption-ledger.lino"))
        .expect("adoption ledger readable");
    let qualifying = ledger
        .split("  adoption_effect\n")
        .skip(1)
        .filter(|record| {
            record
                .lines()
                .take_while(|line| line.starts_with("    "))
                .any(|line| line.trim() == "qualifies \"true\"")
        })
        .count();
    assert_eq!(adopted, qualifying);

    let ratchet = fs::read_to_string(root.join("data/meta/adoption-effect-ratchet.lino"))
        .expect("ratchet readable");
    parse_indented(&ratchet).expect("ratchet is valid Links Notation");
    assert!(ratchet.contains("measure adopted_items_with_qualifying_effect"));
    assert!(ratchet.contains("invariant \"equals adopted_items\""));
}

#[test]
fn effective_adoption_is_executable_and_the_unread_adoption_count_is_zero() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("the repository root sits one level above the crate");
    let registry = MethodRegistry::shared();
    let measured = registry
        .learned_method(MEASURED_METHOD)
        .expect("the measured adopted item remains discoverable");
    assert_eq!(measured.status, LearnedMethodStatus::Adopted);
    assert!(
        measured.is_executable(),
        "binding and effect are separate facts"
    );
    assert!(
        registry
            .ordered_method_names_for_relevants(&[format!("method:{MEASURED_METHOD}")])
            .contains(&MEASURED_METHOD.to_owned()),
        "the effect-qualified item must enter the registry's final dispatch loop"
    );

    let ratchet = fs::read_to_string(root.join("data/meta/adoption-effect-ratchet.lino"))
        .expect("ratchet readable");
    let unread = ratchet
        .split("measure learned_items_never_read_back")
        .nth(1)
        .and_then(|tail| {
            tail.lines()
                .find(|line| line.trim_start().starts_with("value "))
        })
        .expect("unread-item measure")
        .trim();
    assert_eq!(unread, "value 0");

    assert!(ratchet.contains("measure adopted_items\n    direction up\n    value 1"));
    assert!(
        ratchet.contains(
            "measure adopted_items_with_qualifying_effect\n    direction up\n    value 1"
        )
    );
}
