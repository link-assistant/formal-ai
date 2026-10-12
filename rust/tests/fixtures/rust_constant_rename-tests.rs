#![cfg(all(test, feature = "meta-language"))]

use super::*;
#[test]
fn source_bytes_and_utf16_spans_are_owned() {
    let source = "// 😀\nconst OLD: i32 = 1; fn value() -> i32 { OLD }";
    let candidate = rust_constant_rename_candidate(source, "OLD", "NEW").unwrap();
    let expected = source.replace("OLD", "NEW");
    assert_eq!(candidate.content, expected);
    assert_eq!(
        candidate.spans[0][0],
        source[..source.find("OLD").unwrap()].encode_utf16().count()
    );
    assert_eq!(candidate.spans.len(), 2);
    assert!(!candidate.write_authority);
    assert_eq!(candidate.compilation, "pending");
}
#[test]
fn shadow_private_module_macro_and_collision_refuse() {
    for source in [
        "pub const OLD: i32 = 1;",
        "const OLD: i32 = 1; fn f(OLD: i32) -> i32 { OLD }",
        "const OLD: i32 = 1; mod nested {}",
        "const OLD: i32 = 1; fn f() { println!(\"{}\", OLD); }",
        "const OLD: i32 = 1; const NEW: i32 = 2;",
        "const OLD: i32 = ;",
    ] {
        assert!(rust_constant_rename_candidate(source, "OLD", "NEW").is_none());
    }
}
