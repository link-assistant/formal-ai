//! The seed parser reads a delimiter the way Links Notation escapes it.
//!
//! Issue #715 is about code reaching a Links Notation artifact, and code
//! carries quotes. The notation escapes a delimiter by *doubling* it; the value
//! decoder never learned that rule, so a value carrying its own delimiter fell
//! back to raw text with the quotes still in it.

use formal_ai::substitution::SubstitutionRuleSet;

/// The notation quotes a value by doubling the delimiter inside it, so the
/// value's own quote is preserved. This is the shape the corpus already
/// carries — `data/cache/wikidata/property/P138.lino` writes `the subject''s
/// name` — which the reader silently failed to decode until it learned the
/// rule, handing callers back the raw line with the quotes still in it.
#[test]
fn the_reader_collapses_a_doubled_delimiter() {
    let source = "substitution_rules\n  id 'the subject''s name'\n";
    let set = SubstitutionRuleSet::from_links_notation(source)
        .expect("a doubled delimiter is how the notation escapes a quote");
    assert_eq!(
        set.id, "the subject's name",
        "a doubled delimiter must collapse to one quote, not survive as two"
    );
}
