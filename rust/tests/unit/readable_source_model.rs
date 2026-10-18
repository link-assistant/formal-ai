//! Shared source models must reconstruct without an installed source file.
#![cfg(feature = "meta-language")]
use formal_ai::readable_source::{
    read_source_links, source_from_links, source_to_links, verify_source_links, write_source_links,
};

#[test]
fn readable_models_restore_and_independently_reparse_all_three_source_roots() {
    let cases = [
        (
            "JavaScript",
            "// preserve Unicode 你好\nexport function keep(value) { return value; }\n",
            include_str!("../fixtures/source-model/JavaScript.lino"),
        ),
        (
            "TypeScript",
            "/** Keep the value. */\nexport function keep(value: string): string { return value; }\n",
            include_str!("../fixtures/source-model/TypeScript.lino"),
        ),
        (
            "Rust",
            "/// Keep the value.\npub fn keep(value: &str) -> &str { value }\n",
            include_str!("../fixtures/source-model/Rust.lino"),
        ),
    ];
    assert_eq!(cases.len(), 3);
    for (language, expected_source, notation) in cases {
        let retained = read_source_links(notation).expect("shared source document");
        assert_eq!(
            source_from_links(&retained).expect("source-free reconstruction"),
            expected_source
        );
        verify_source_links(&retained).expect("independent pinned grammar reparse");
        let actual = source_to_links(expected_source, language).expect("actual native grammar");
        assert_eq!(actual, retained, "shared JavaScript/native concrete model");
        let rewritten = write_source_links(&actual).expect("readable notation");
        assert_eq!(
            read_source_links(&rewritten).expect("source-free reload"),
            retained
        );
    }
}

#[test]
fn readable_models_reject_unrepresented_syntax_forged_labels_and_hidden_payloads() {
    let mut model = read_source_links(include_str!("../fixtures/source-model/JavaScript.lino"))
        .expect("shared document");
    model[2][0] = serde_json::json!("forged-module");
    assert!(verify_source_links(&model).is_err());
    assert!(source_to_links("fn broken( {", "Rust").is_err());
    assert!(source_to_links("return 1;", "UnknownLanguage").is_err());
    assert!(
        read_source_links(
            "(source (language \"Rust\") (syntax (form \"source_file\") \"opaque source\"))"
        )
        .is_err()
    );
    assert!(read_source_links("(source (language \"Rust\")").is_err());
    assert!(
        read_source_links(
            "(source (language \"Rust\") (syntax (form \"source_file\") (text \"\\x\")))"
        )
        .is_err()
    );
}

#[test]
fn readable_models_keep_comments_crlf_unicode_and_literal_percent_text() {
    let source = "// Привет हिन्दी 你好 %22\r\nexport function value() { return \"😀%22\"; }\r\n";
    let model = source_to_links(source, "JavaScript").expect("actual grammar");
    let notation = write_source_links(&model).expect("readable model");
    let reloaded = read_source_links(&notation).expect("source-free reload");
    assert_eq!(
        source_from_links(&reloaded).expect("exact restoration"),
        source
    );
    verify_source_links(&reloaded).expect("fresh grammar comparison");
    assert!(notation.contains("Привет हिन्दी 你好 %22"));
    assert!(notation.contains("😀%22"));
}

#[test]
fn structural_owners_survive_reload_but_never_replace_content_verification() {
    use formal_ai::readable_source::{owned_source_links, source_owner};
    let source = "pub fn retained() -> bool { true }\n";
    let content = source_to_links(source, "Rust").expect("actual grammar");
    let owned = owned_source_links("src/retained.rs", &content).expect("readable owner");
    let reloaded =
        read_source_links(&write_source_links(&owned).expect("write model")).expect("reload");
    assert_eq!(
        source_owner(&reloaded).expect("owner"),
        Some("src/retained.rs")
    );
    assert_eq!(source_from_links(&reloaded).expect("source-free"), source);
    verify_source_links(&reloaded).expect("independent grammar");
    for path in [
        "",
        "/outside.rs",
        "../outside.rs",
        "src/../outside.rs",
        "src//retained.rs",
        "C:/outside.rs",
        "src\\retained.rs",
    ] {
        assert!(owned_source_links(path, &content).is_err());
    }
    let mut ambiguous = owned.clone();
    ambiguous
        .as_array_mut()
        .expect("owned link")
        .push(content.clone());
    assert!(source_from_links(&ambiguous).is_err());
    let mut forged = owned;
    forged[2][2][0] = serde_json::json!("forged-module");
    assert!(verify_source_links(&forged).is_err());
}
