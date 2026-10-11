#![cfg(all(test, not(feature = "meta-language")))]

#[test]
fn unavailable_parser_never_mints_clean_ast() {
    assert!(super::rust_constant_rename_candidate("const OLD: i32 = 1;", "OLD", "NEW").is_none());
}
