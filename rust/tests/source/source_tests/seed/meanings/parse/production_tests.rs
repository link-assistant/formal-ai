use super::{FACET_KINDS, parse_semantic_facets};
use crate::seed::parser::parse_lino;

#[test]
fn registered_facets_merge_native_and_wrapped_targets() {
    for kind in FACET_KINDS {
        let input = format!(
            "{kind} first_target\nfacet {kind}\n  target first_target\n  target second_target\n"
        );
        let facets = parse_semantic_facets(&parse_lino(&input));
        assert_eq!(facets.len(), 1, "{kind} must produce one merged facet");
        assert_eq!(facets[0].kind, *kind);
        assert_eq!(facets[0].meanings, ["first_target", "second_target"]);
    }
}

#[test]
fn unrelated_seed_fields_do_not_become_semantic_facets() {
    let input = "role memory_inspection\ngloss descriptive_text\nnotation\n";
    assert!(parse_semantic_facets(&parse_lino(input)).is_empty());
}
