use super::*;

#[test]
fn code_translation_routes_through_the_code_meta_language_not_direct_pairs() {
    // #526: code translation must go source -> code meaning -> target, so
    // language pairs that were never wired as a direct `(source, target)` arm
    // still translate. Python -> JavaScript, Rust -> Go, and JavaScript ->
    // TypeScript never had a hardcoded pair; they work because both legs share
    // the same `function:add:binary_sum` code meaning. This is the property
    // that keeps the translator at O(N) formalizers + O(N) renderers instead of
    // the O(N * N) direct table the issue forbids.
    let add_sources: &[(&str, &str)] = &[
        ("python", "def add(a, b): return a + b"),
        ("rust", "fn add(a: i32, b: i32) -> i32 { a + b }"),
        ("javascript", "function add(a, b) { return a + b; }"),
    ];
    // (target language slug, substring that proves the target rendering).
    let targets: &[(&str, &str)] = &[
        ("python", "def add"),
        ("rust", "fn add"),
        ("javascript", "function add"),
        ("typescript", "b: number"),
        ("go", "func add"),
    ];

    let mut shared_meaning: Option<String> = None;
    for (source_lang, source_code) in add_sources {
        for (target_lang, target_marker) in targets {
            if source_lang == target_lang {
                continue;
            }
            let response = answer(&format!(
                "Translate `{source_code}` from {source_lang} to {target_lang}"
            ));
            let (fence, body) = match *target_lang {
                "python" => ("python", "def add(a, b):\n    return a + b"),
                "rust" => ("rust", "fn add(a: i32, b: i32) -> i32 {\n    a + b\n}"),
                "javascript" => ("javascript", "function add(a, b) {\n    return a + b;\n}"),
                "typescript" => (
                    "typescript",
                    "function add(a: number, b: number): number {\n    return a + b;\n}",
                ),
                "go" => ("go", "func add(a int, b int) int {\n    return a + b\n}"),
                other => panic!("missing documented renderer for {other}"),
            };
            assert_eq!(
                response.answer,
                format!(
                    "Translated `{source_code}` from {source_lang} to {target_lang}:\n\n```{fence}\n{body}\n```"
                )
            );
            assert_eq!(
                response.intent,
                format!("translate_{source_lang}_to_{target_lang}"),
                "code translation should route through the translation handler for \
                 {source_lang}->{target_lang}, got {}: {}",
                response.intent,
                response.answer,
            );
            assert!(
                response.answer.contains(target_marker),
                "{source_lang}->{target_lang} should render the target-language add \
                 function (looking for {target_marker:?}), got: {}",
                response.answer,
            );
            assert!(
                response.answer.contains("a + b"),
                "{source_lang}->{target_lang} must preserve the add semantics, got: {}",
                response.answer,
            );
            // Every add function, in every source language, collapses to the
            // same meta-language meaning link.
            let meaning = meaning_link(&response).to_owned();
            match &shared_meaning {
                None => shared_meaning = Some(meaning),
                Some(expected) => assert_eq!(
                    &meaning, expected,
                    "every add-function translation must share one code meaning link, \
                     got {meaning} for {source_lang}->{target_lang} vs {expected}",
                ),
            }
        }
    }
}

#[test]
fn rust_javascript_code_translation_round_trips_through_code_meaning() {
    let rust_source = "fn add(a: i32, b: i32) -> i32 { a + b }";
    let rust_to_js = answer(&format!(
        "Translate `{rust_source}` from Rust to JavaScript"
    ));
    assert_eq!(
        rust_to_js.answer,
        "Translated `fn add(a: i32, b: i32) -> i32 { a + b }` from rust to javascript:\n\n```javascript\nfunction add(a, b) {\n    return a + b;\n}\n```"
    );
    assert_eq!(
        rust_to_js.intent, "translate_rust_to_javascript",
        "Rust->JavaScript code translation should route through the translation handler, got {}: {}",
        rust_to_js.intent, rust_to_js.answer,
    );
    assert!(
        rust_to_js.answer.contains("function add"),
        "Rust->JavaScript should render an add function, got: {}",
        rust_to_js.answer,
    );
    assert!(
        rust_to_js.answer.contains("return a + b"),
        "Rust->JavaScript should preserve add semantics, got: {}",
        rust_to_js.answer,
    );
    assert!(
        rust_to_js
            .evidence_links
            .iter()
            .any(|link| link == "language_from:rust"),
        "Rust->JavaScript should record source language, got {:?}",
        rust_to_js.evidence_links,
    );
    assert!(
        rust_to_js
            .evidence_links
            .iter()
            .any(|link| link == "language_to:javascript"),
        "Rust->JavaScript should record target language, got {:?}",
        rust_to_js.evidence_links,
    );

    let javascript_source = "function add(a, b) { return a + b; }";
    let js_to_rust = answer(&format!(
        "Translate `{javascript_source}` from JavaScript to Rust"
    ));
    assert_eq!(
        js_to_rust.answer,
        "Translated `function add(a, b) { return a + b; }` from javascript to rust:\n\n```rust\nfn add(a: i32, b: i32) -> i32 {\n    a + b\n}\n```"
    );
    assert_eq!(
        js_to_rust.intent, "translate_javascript_to_rust",
        "JavaScript->Rust code translation should route through the translation handler, got {}: {}",
        js_to_rust.intent, js_to_rust.answer,
    );
    assert!(
        js_to_rust.answer.contains("fn add"),
        "JavaScript->Rust should render an add function, got: {}",
        js_to_rust.answer,
    );
    assert!(
        js_to_rust.answer.contains("a + b"),
        "JavaScript->Rust should preserve add semantics, got: {}",
        js_to_rust.answer,
    );
    assert!(
        js_to_rust
            .evidence_links
            .iter()
            .any(|link| link == "language_from:javascript"),
        "JavaScript->Rust should record source language, got {:?}",
        js_to_rust.evidence_links,
    );
    assert!(
        js_to_rust
            .evidence_links
            .iter()
            .any(|link| link == "language_to:rust"),
        "JavaScript->Rust should record target language, got {:?}",
        js_to_rust.evidence_links,
    );
    assert_eq!(
        meaning_link(&rust_to_js),
        meaning_link(&js_to_rust),
        "#526: Rust->JavaScript->Rust must preserve the same code meaning link",
    );
}

#[test]
fn untranslatable_concepts_are_flagged() {
    let response = answer("Translate 'тоска' to English in one word");
    assert!(
        response
            .evidence_links
            .iter()
            .any(|link| link.starts_with("translation_gap:")),
        "translation gaps must be marked explicitly, not papered over"
    );
}

#[test]
fn issue_386_define_in_links_notation_resolves_to_the_links_notation_concept() {
    // Issue #386: the `try_translation` request-gate recognises a
    // "define <phrase> in links notation" command from *meaning* — the
    // `definition_command` verb and the `links_notation_format` markers seeded in
    // data/seed/meanings-translation.lino — rather than the hardcoded literals it
    // used before. The refactor is behaviour-preserving: across the full dispatch
    // pipeline the `concept_lookup` handler answers these prompts first (the phrase
    // "links notation" names a known concept), so the define-gate's routing never
    // changes the observable answer. This test locks that public contract so a
    // future dispatch-order change can't silently alter it; the seed→code wiring of
    // the gate itself is locked by the lib test
    // `define_in_links_roles_expose_the_scanned_surfaces` in src/seed/meanings.rs.
    let cases: &[&str] = &[
        "define `apple` in links notation",
        "define \"apple\" in links notation",
        "define `apple` в links notation",
        "define apple in links notation",
    ];
    for prompt in cases {
        let response = answer(prompt);
        assert_eq!(
            response.intent, "concept_lookup",
            "define-in-links prompt should resolve to the Links Notation concept for {prompt:?}, got {}: {}",
            response.intent, response.answer,
        );
        assert!(
            response.answer.starts_with("Links Notation (data-format):"),
            "expected the Links Notation concept definition for {prompt:?}, got: {}",
            response.answer,
        );
    }
}
