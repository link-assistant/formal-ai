#![cfg(test)]

use super::{parse_lino, parse_lino_checked};

const SHARED: &str = "meanings\n  example\n    shared-lexeme-fields lexical\n      part_of_speech noun\n    lexeme en\n      surface\n        text first\n        use-lexeme-fields lexical\n    lexeme es\n      surface\n        text second\n        use-lexeme-fields lexical\n";
const LEGACY: &str = "meanings\n  example\n    lexeme en\n      surface\n        text first\n        part_of_speech noun\n    lexeme es\n      surface\n        text second\n        part_of_speech noun\n";

#[test]
fn shared_fields_preserve_complete_legacy_tree_and_combined_roots() {
    let actual = parse_lino_checked(SHARED).expect("valid shared fields");
    let expected = parse_lino_checked(LEGACY).expect("valid legacy tree");
    assert_eq!(alloc::format!("{actual:?}"), alloc::format!("{expected:?}"));
    let combined = alloc::format!("{LEGACY}{SHARED}");
    let original = alloc::format!("{LEGACY}{LEGACY}");
    assert_eq!(
        alloc::format!("{:?}", parse_lino_checked(&combined)),
        alloc::format!("{:?}", parse_lino_checked(&original))
    );
}

#[test]
fn malformed_shared_fields_refuse_whole_documents() {
    for invalid in [
        SHARED.replace("use-lexeme-fields lexical", "use-lexeme-fields missing"),
        SHARED.replace("part_of_speech noun", "effect mutation"),
        SHARED.replace("lexeme es", "lexeme en"),
        SHARED.replace("meanings\n", "other-root\n"),
    ] {
        assert!(parse_lino_checked(&invalid).is_err());
        assert!(std::panic::catch_unwind(|| parse_lino(&invalid)).is_err());
    }
}
