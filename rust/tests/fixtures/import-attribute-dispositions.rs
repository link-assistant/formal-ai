#[test]
fn json_import_attributes_have_explicit_noform_dispositions_without_erasure() {
    use formal_ai::rust_projection::{ProjectionOutcome, project, projection_from};
    use std::collections::BTreeSet;
    let projection = projection_from(formal_ai::seed::GRAMMAR_PROJECTION_RULES_LINO).unwrap();
    for label in ["javascript", "typescript"] {
        let source = "import data from './数据.json' with { type: 'json' };\n";
        let network = formal_ai::grammar_kinds::parse_network(label, source);
        assert_eq!(network.reconstruct_text(), source);
        let kinds: BTreeSet<String> = formal_ai::grammar_kinds::named_source_kinds(label, source)
            .into_iter()
            .map(|(kind, _)| kind)
            .collect();
        for kind in ["import_statement", "import_attribute"] {
            assert!(kinds.contains(kind), "{label} must parse the real {kind}");
            assert!(projection.is_noform(kind, "rust"));
            assert!(!projection.is_ruled(kind, "rust"));
            assert!(!projection.is_refused(kind, "rust"));
        }
        assert!(projection.coverage_kinds(&kinds, "rust").is_empty());
        match project(label, "rust", source) {
            ProjectionOutcome::Refused { refusals } => {
                assert!(
                    refusals
                        .iter()
                        .any(|refusal| refusal.construct == "import_statement")
                );
            }
            other => panic!("attributes cannot disappear into rendered Rust: {other:?}"),
        }
        assert_eq!(network.reconstruct_text(), source);
    }
}

#[test]
fn grammar_kind_lookup_normalizes_spelling_and_refuses_duplicate_or_colliding_dispositions() {
    use formal_ai::rust_projection::projection_from;
    let seed = formal_ai::seed::GRAMMAR_PROJECTION_RULES_LINO;
    let projection = projection_from(seed).unwrap();
    for (external, canonical) in [
        ("import_statement", "import-statement"),
        ("import_attribute", "import-attribute"),
    ] {
        assert!(projection.is_noform(external, "rust"));
        assert_eq!(
            projection.is_noform(external, "rust"),
            projection.is_noform(canonical, "rust")
        );
    }
    assert!(projection.is_refused("_", "rust"));
    assert!(projection.is_refused("-", "rust"));
    assert!(projection.is_refused("field_identifier", "rust"));
    assert_eq!(
        projection.is_refused("field_identifier", "rust"),
        projection.is_refused("field-identifier", "rust")
    );
    assert!(projection.is_ruled("binary_expression", "javascript"));
    assert_eq!(
        projection.is_ruled("binary_expression", "javascript"),
        projection.is_ruled("binary-expression", "javascript")
    );
    for (kind, target, disposition) in [
        ("import-attribute", "rust", "grammar-projection-noform"),
        ("import_attribute", "rust", "grammar-projection-noform"),
        ("import-attribute", "any", "grammar-projection-refusal"),
    ] {
        let collision = format!(
            "{seed}(629: 1 (meta: (t: semantic) (n: 1) (term: {kind}) (def: {target}) (lang: {disposition})))\n"
        );
        assert!(
            projection_from(&collision).is_err(),
            "{kind}:{target}:{disposition}"
        );
    }
    let extended = format!(
        "{seed}(629: 1 (meta: (t: semantic) (n: 1) (term: import-attribute) (def: typescript) (lang: grammar-projection-noform)))\n"
    );
    let extended = projection_from(&extended).unwrap();
    assert!(extended.is_noform("import_attribute", "typescript"));
    assert!(!extended.is_noform("unobserved_attribute", "rust"));
}
