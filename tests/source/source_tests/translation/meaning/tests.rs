use super::*;

#[test]
fn slug_prefers_q_item_then_sense_then_wiktionary() {
    assert_eq!(MeaningId::from_item("Q42").slug(), "wikidata:Q42");
    assert_eq!(
        MeaningId::from_sense("L8485-S1").slug(),
        "wikidata-sense:L8485-S1"
    );
    assert_eq!(
        MeaningId::from_wiktionary_page("en", "hello").slug(),
        "wiktionary:en:hello"
    );
}

#[test]
fn equal_ids_carry_the_same_wikidata_pointer() {
    let a = MeaningId::from_item("Q1369");
    let b = MeaningId::from_item("Q1369");
    assert_eq!(a, b);
    // Different ids that happen to share a sense are not equal.
    let c = MeaningId::from_sense("L1-S1");
    assert_ne!(a, c);
}

#[test]
fn wikidata_backed_predicate_distinguishes_fallback_from_item() {
    assert!(MeaningId::from_item("Q42").is_wikidata_backed());
    assert!(MeaningId::from_sense("L1-S1").is_wikidata_backed());
    assert!(!MeaningId::from_wiktionary_page("en", "x").is_wikidata_backed());
}
