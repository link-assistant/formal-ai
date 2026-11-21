//! Native localized seed preservation, without compiling the browser worker.

#[test]
fn a_language_without_an_opener_pool_keeps_its_complete_localized_seed() {
    let seed = crate::seed::response_for("unknown", "es").expect("Spanish unknown seed");
    assert!(!super::has_unknown_opener_pool("es"));
    assert_eq!(
        super::language_aware_unknown_answer("probe", crate::Language::Spanish),
        seed,
    );
}

#[test]
fn authored_pools_still_select_their_own_first_opener() {
    for language in crate::language::registered_languages() {
        if !super::has_unknown_opener_pool(language.slug()) {
            continue;
        }
        let seed = crate::seed::response_for("unknown", language.slug())
            .expect("a language with an opener pool has a localized unknown seed");
        assert_eq!(
            super::language_aware_unknown_answer("", language),
            seed,
            "{} must retain its complete seed when selecting the first opener",
            language.slug(),
        );
    }
}
