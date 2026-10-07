//! Issue #1164 (E129): code examples as formal knowledge.
//!
//! Retrieved examples are parsed into the meta language, decomposed into
//! meaningful parts, generalized across languages by deduplication, and
//! recomposed for a new requirement. The tests pin the contract in two
//! sections -- decompose and recompose -- exactly as the issue's test plan
//! splits them, and one test walks the full internet path: the #1163
//! formalizer extracts the code block from a captured Hello World page,
//! and the decomposer reads it from there.
//!
//! Meanings derive from CST node kinds plus the seed vocabulary (R1164-3);
//! a slug without a registered grammar is named in an
//! `UnknownGrammar` error and never guessed (R1164-1). Pascal is held out
//! exactly that way today: no grammar row exists, so decompose refuses it
//! while the seed already carries the vocabulary and shape that will serve
//! it the day the row lands (R1164-9).
//!
//! Compiling and running the recomposed programs (R1164-8's execution
//! half, the fpc run for Pascal) is the gated workspace execution named in
//! the issue; here the recompositions are proved structurally -- the held
//! out literal is present, the call is the language's own from the seed,
//! and the meta-language parse accepts the source.

#![cfg(feature = "meta-language")]

use std::collections::BTreeMap;

use formal_ai::code_example_knowledge::{
    CodePartKind, DecomposeError, ParameterBindings, ProseLink, decompose_code_node,
    decomposed_links_notation, generalize_examples, generalized_links_notation,
    recompose_for_requirement, recomposition_links_notation,
};
use formal_ai::web_formalize::{PageBlock, formalize_page_with_context};

const RUST_HELLO: &str = "fn main() {\n    println!(\"Hello, world!\");\n}\n";
const GO_HELLO: &str =
    "package main\n\nimport \"fmt\"\n\nfunc main() {\n    fmt.Println(\"Hello, world!\")\n}\n";
const KOTLIN_HELLO: &str = "fun main() {\n    println(\"Hello, world!\")\n}\n";
const SWIFT_HELLO: &str = "print(\"Hello, world!\")\n";
const PYTHON_HELLO: &str = "print(\"Hello, world!\")\n";

const KOTLIN_PAGE: &str =
    include_str!("../fixtures/coding-discovery/issue-1164/kotlin-hello-world.html");
const RUST_PAGE: &str =
    include_str!("../fixtures/coding-discovery/issue-1164/rust-hello-world.html");
const GO_PAGE: &str = include_str!("../fixtures/coding-discovery/issue-1164/go-hello-world.html");
const SWIFT_PAGE: &str =
    include_str!("../fixtures/coding-discovery/issue-1164/swift-hello-world.html");
const SCALA_PAGE: &str =
    include_str!("../fixtures/coding-discovery/issue-1164/scala-hello-world.html");

/// R1164-2: a Rust Hello World decomposes into entry point, output
/// operation, and string literal parts with real CST node kinds.
#[test]
fn decompose_rust_hello_world_produces_entry_output_literal() {
    let node = decompose_code_node(RUST_HELLO, "rust", &[]).expect("rust grammar is registered");
    assert_eq!(node.language_slug, "rust");
    let kinds = |kind: CodePartKind| node.parts.iter().filter(|part| part.kind == kind).count();
    assert_eq!(kinds(CodePartKind::EntryPoint), 1);
    assert_eq!(kinds(CodePartKind::OutputOperation), 1);
    assert_eq!(kinds(CodePartKind::StringLiteral), 1);
    let entry = node
        .parts
        .iter()
        .find(|part| part.kind == CodePartKind::EntryPoint)
        .expect("entry part");
    assert_eq!(entry.source_text, "main");
    assert_ne!(entry.cst_node_kind, "");
    let output = node
        .parts
        .iter()
        .find(|part| part.kind == CodePartKind::OutputOperation)
        .expect("output part");
    assert_eq!(output.source_text, "println!");
    let literal = node
        .parts
        .iter()
        .find(|part| part.kind == CodePartKind::StringLiteral)
        .expect("literal part");
    assert_eq!(literal.source_text, "Hello, world!");
    assert_eq!(literal.cst_node_kind, "string_literal");
}

/// R1164-2: Go decomposes the same way, plus its import.
#[test]
fn decompose_go_hello_world_produces_entry_output_literal() {
    let node = decompose_code_node(GO_HELLO, "go", &[]).expect("go grammar is registered");
    assert!(
        node.parts
            .iter()
            .any(|part| part.kind == CodePartKind::EntryPoint)
    );
    assert!(node.parts.iter().any(
        |part| part.kind == CodePartKind::OutputOperation && part.source_text == "fmt.Println"
    ));
    assert!(node.parts.iter().any(
        |part| part.kind == CodePartKind::StringLiteral && part.source_text == "Hello, world!"
    ));
}

/// R1164-2: script languages have no entry point.
#[test]
fn decompose_python_script_has_no_entry_point() {
    let node =
        decompose_code_node(PYTHON_HELLO, "python", &[]).expect("python grammar is registered");
    assert!(
        node.parts
            .iter()
            .any(|part| part.kind == CodePartKind::OutputOperation && part.source_text == "print")
    );
    assert!(node.parts.iter().any(
        |part| part.kind == CodePartKind::StringLiteral && part.source_text == "Hello, world!"
    ));
    assert!(
        node.parts
            .iter()
            .all(|part| part.kind != CodePartKind::EntryPoint)
    );
}

/// R1164-1/R1164-9: an unregistered slug is named in the error, never
/// guessed. Zig has no grammar in the dependency; Pascal is the held-out
/// language -- no grammar row is registered yet, so it refuses today and
/// answers the need honestly.
#[test]
fn unknown_grammar_slug_returns_unknown_grammar_error() {
    let error =
        decompose_code_node("print(\"hello\")", "zig", &[]).expect_err("zig has no grammar");
    assert_eq!(error, DecomposeError::UnknownGrammar("zig".to_owned()));
    let error = decompose_code_node("program HelloWorld;\nbegin\nend.", "pascal", &[])
        .expect_err("pascal grammar row is not registered yet");
    assert_eq!(error, DecomposeError::UnknownGrammar("pascal".to_owned()));
}

/// The full internet path: the #1163 formalizer extracts the code block
/// from a captured page, the decomposer reads it from there, and prose
/// links from the page's commands become build and run parts carrying the
/// prose's source URL (R1164-4).
#[test]
fn decompose_reads_the_code_block_of_a_captured_page() {
    let (_, blocks) = formalize_page_with_context(KOTLIN_PAGE.as_bytes(), Some("text/html"), None);
    let code = blocks
        .iter()
        .find_map(|(_, block)| match block {
            PageBlock::CodeBlock { language, text } if language == "kotlin" => Some(text.clone()),
            _ => None,
        })
        .expect("kotlin code block in the captured page");
    let prose = vec![
        ProseLink {
            relation: "compile with".to_owned(),
            text: "kotlinc hello.kt -include-runtime -d hello.jar".to_owned(),
            source_url: "https://kotlinlang.org/docs/command-line.html".to_owned(),
        },
        ProseLink {
            relation: "run_with".to_owned(),
            text: "java -jar hello.jar".to_owned(),
            source_url: "https://kotlinlang.org/docs/command-line.html".to_owned(),
        },
    ];
    let node = decompose_code_node(&code, "kotlin", &prose).expect("kotlin grammar is registered");
    assert!(
        node.parts
            .iter()
            .any(|part| part.kind == CodePartKind::EntryPoint && part.source_text == "main")
    );
    assert!(node
        .parts
        .iter()
        .any(|part| part.kind == CodePartKind::OutputOperation && part.source_text == "println"));
    let build = node
        .parts
        .iter()
        .find(|part| part.kind == CodePartKind::BuildCommand)
        .expect("build command from prose");
    assert_eq!(
        build.source_text,
        "kotlinc hello.kt -include-runtime -d hello.jar"
    );
    assert_eq!(
        build.source_url,
        "https://kotlinlang.org/docs/command-line.html"
    );
    let run = node
        .parts
        .iter()
        .find(|part| part.kind == CodePartKind::RunCommand)
        .expect("run command from prose");
    assert_eq!(run.source_text, "java -jar hello.jar");
    assert_eq!(
        run.source_url,
        "https://kotlinlang.org/docs/command-line.html"
    );
}

/// Every captured fixture decomposes (R1164-2 across languages).
#[test]
fn every_captured_fixture_decomposes() {
    for (page, language, call) in [
        (RUST_PAGE, "rust", "println!"),
        (GO_PAGE, "go", "fmt.Println"),
        (KOTLIN_PAGE, "kotlin", "println"),
        (SWIFT_PAGE, "swift", "print"),
        (SCALA_PAGE, "scala", "println"),
    ] {
        let (_, blocks) = formalize_page_with_context(page.as_bytes(), Some("text/html"), None);
        let code = blocks
            .iter()
            .find_map(|(_, block)| match block {
                PageBlock::CodeBlock { text, .. } => Some(text.clone()),
                _ => None,
            })
            .unwrap_or_default();
        let node = decompose_code_node(&code, language, &[])
            .unwrap_or_else(|error| panic!("{language} fixture must decompose: {error:?}"));
        assert!(
            node.parts
                .iter()
                .any(|part| part.kind == CodePartKind::OutputOperation && part.source_text == call),
            "{language} finds its seed output call"
        );
        assert!(
            node.parts
                .iter()
                .any(|part| part.kind == CodePartKind::StringLiteral
                    && part.source_text == "Hello, world!"),
            "{language} finds the literal"
        );
    }
    // Swift has no entry point at all (the page says so and the seed agrees).
    let swift =
        decompose_code_node(SWIFT_HELLO, "swift", &[]).expect("swift grammar is registered");
    assert!(
        swift
            .parts
            .iter()
            .all(|part| part.kind != CodePartKind::EntryPoint)
    );
}

/// R1164-5: aligning the Rust and Go examples shares the
/// entry-output-literal structure (Rust has no import, so the shared
/// structure excludes it) and binds the literal as a per-language
/// parameter.
#[test]
fn generalize_rust_and_go_produces_shared_structure_with_literal_parameter() {
    let examples = [
        decompose_code_node(RUST_HELLO, "rust", &[]).expect("rust decomposes"),
        decompose_code_node(GO_HELLO, "go", &[]).expect("go decomposes"),
    ];
    let procedure = generalize_examples(&examples);
    assert!(procedure.id.starts_with("generalized:"));
    assert!(
        procedure
            .shared_structure
            .contains(&CodePartKind::EntryPoint)
    );
    assert!(
        procedure
            .shared_structure
            .contains(&CodePartKind::OutputOperation)
    );
    assert!(
        procedure
            .shared_structure
            .contains(&CodePartKind::StringLiteral)
    );
    assert!(!procedure.shared_structure.contains(&CodePartKind::Import));
    let literal = procedure
        .parameters
        .iter()
        .find(|parameter| parameter.name == "output_literal")
        .expect("literal parameter");
    assert_eq!(
        literal.per_language.get("rust").map(String::as_str),
        Some("Hello, world!")
    );
    assert_eq!(
        literal.per_language.get("go").map(String::as_str),
        Some("Hello, world!")
    );
    let call = procedure
        .parameters
        .iter()
        .find(|parameter| parameter.name == "output_call")
        .expect("call parameter");
    assert_eq!(
        call.per_language.get("rust").map(String::as_str),
        Some("println!")
    );
    assert_eq!(
        call.per_language.get("go").map(String::as_str),
        Some("fmt.Println")
    );
}

/// R1164-6/R1164-8 (structural half): a held-out literal recomposes into
/// Rust source that carries it, calls the language's own output macro, and
/// parses through the meta-language CST bridge.
#[test]
fn recompose_with_formal_ai_literal_yields_rust_source() {
    let examples = [
        decompose_code_node(RUST_HELLO, "rust", &[]).expect("rust decomposes"),
        decompose_code_node(GO_HELLO, "go", &[]).expect("go decomposes"),
    ];
    let procedure = generalize_examples(&examples);
    let mut bindings = BTreeMap::new();
    bindings.insert("output_literal".to_owned(), "Hello, Formal AI!".to_owned());
    let recomposition = recompose_for_requirement(&procedure, &ParameterBindings(bindings), "rust")
        .expect("rust shape");
    assert!(
        recomposition
            .source
            .contains("println!(\"Hello, Formal AI!\")")
    );
    assert!(recomposition.source.contains("fn main()"));
    assert_eq!(recomposition.language_slug, "rust");
    assert!(decompose_code_node(&recomposition.source, "rust", &[]).is_ok());
}

/// R1164-6: the same procedure recomposes for Go from the one binding.
#[test]
fn recompose_with_formal_ai_literal_yields_go_source() {
    let examples = [
        decompose_code_node(RUST_HELLO, "rust", &[]).expect("rust decomposes"),
        decompose_code_node(GO_HELLO, "go", &[]).expect("go decomposes"),
    ];
    let procedure = generalize_examples(&examples);
    let mut bindings = BTreeMap::new();
    bindings.insert("output_literal".to_owned(), "Hello, Formal AI!".to_owned());
    let recomposition = recompose_for_requirement(&procedure, &ParameterBindings(bindings), "go")
        .expect("go shape");
    assert!(
        recomposition
            .source
            .contains("fmt.Println(\"Hello, Formal AI!\")")
    );
    assert!(recomposition.source.contains("package main"));
    assert!(decompose_code_node(&recomposition.source, "go", &[]).is_ok());
}

/// R1164-9 (static half): the seed already carries Pascal's program shape
/// and output call, so the moment a grammar row is registered the held-out
/// language recomposes; the answer today is the honest `UnknownGrammar` from
/// decompose, and this pins the shape that waits for it.
#[test]
fn pascal_shape_is_ready_for_its_grammar_row() {
    let examples = [decompose_code_node(RUST_HELLO, "rust", &[]).expect("rust decomposes")];
    let procedure = generalize_examples(&examples);
    let mut bindings = BTreeMap::new();
    bindings.insert("output_literal".to_owned(), "Hello, Formal AI!".to_owned());
    let recomposition =
        recompose_for_requirement(&procedure, &ParameterBindings(bindings), "pascal")
            .expect("pascal shape is seeded");
    assert!(
        recomposition
            .source
            .contains("writeln('Hello, Formal AI!')")
    );
    assert!(recomposition.source.contains("program HelloWorld;"));
}

/// R1164-6: a target with no program shape in the seed is an error naming
/// the language, never a guessed program.
#[test]
fn recompose_refuses_a_language_without_a_shape() {
    let examples = [decompose_code_node(RUST_HELLO, "rust", &[]).expect("rust decomposes")];
    let procedure = generalize_examples(&examples);
    let error = recompose_for_requirement(&procedure, &ParameterBindings::default(), "zig")
        .expect_err("zig has no shape");
    assert!(matches!(error, DecomposeError::ParseFailed(message) if message.contains("zig")));
}

/// R1164-6: the recomposition carries the source URL of every contributing
/// part.
#[test]
fn recomposed_parts_carry_all_source_urls() {
    let prose = vec![ProseLink {
        relation: "compile with".to_owned(),
        text: "kotlinc hello.kt -include-runtime -d hello.jar".to_owned(),
        source_url: "https://kotlinlang.org/docs/command-line.html".to_owned(),
    }];
    let examples = [
        decompose_code_node(RUST_HELLO, "rust", &[]).expect("rust decomposes"),
        decompose_code_node(KOTLIN_HELLO, "kotlin", &prose).expect("kotlin decomposes"),
    ];
    let procedure = generalize_examples(&examples);
    assert!(
        procedure
            .source_urls
            .contains(&"https://kotlinlang.org/docs/command-line.html".to_owned())
    );
    let recomposition =
        recompose_for_requirement(&procedure, &ParameterBindings::default(), "rust")
            .expect("rust shape");
    assert!(
        recomposition
            .part_source_urls
            .contains(&"https://kotlinlang.org/docs/command-line.html".to_owned())
    );
}

/// R1164-10: every record is expressible as a Links Notation document
/// matching the schema seed.
#[test]
fn records_render_as_links_notation() {
    // The prose link is the part that carries a source URL, so the
    // recomposition below has a `part_source_url` to render.
    let prose = vec![ProseLink {
        relation: "compile with".to_owned(),
        text: "rustc main.rs".to_owned(),
        source_url: "https://doc.rust-lang.org/book/ch01-02-hello-world.html".to_owned(),
    }];
    let node = decompose_code_node(RUST_HELLO, "rust", &prose).expect("rust decomposes");
    let notation = decomposed_links_notation(&node);
    assert!(notation.contains("decomposed_code_node"));
    assert!(notation.contains("language_slug rust"));
    assert!(notation.contains("kind entry_point"));
    assert!(notation.contains("cst_node_kind"));

    let procedure = generalize_examples(&[node]);
    let notation = generalized_links_notation(&procedure);
    assert!(notation.contains("generalized_procedure"));
    assert!(notation.contains("shared_structure"));
    assert!(notation.contains("name output_literal"));

    let mut bindings = BTreeMap::new();
    bindings.insert("output_literal".to_owned(), "Hello, Formal AI!".to_owned());
    let recomposition = recompose_for_requirement(&procedure, &ParameterBindings(bindings), "rust")
        .expect("rust shape");
    let notation = recomposition_links_notation(&recomposition);
    assert!(notation.contains("code_recomposition"));
    assert!(notation.contains("language_slug rust"));
    assert!(notation.contains("part_source_url"));
}

/// R1164-7 (static half): adoption goes through the existing gate -- the
/// step records carry empty license fields on purpose, because only the
/// approval gate fills them.
#[test]
fn adoption_steps_leave_license_fields_to_the_gate() {
    let examples = [
        decompose_code_node(RUST_HELLO, "rust", &[]).expect("rust decomposes"),
        decompose_code_node(GO_HELLO, "go", &[]).expect("go decomposes"),
    ];
    let procedure = generalize_examples(&examples);
    let records = procedure.procedure_step_records();
    assert_eq!(records.len(), procedure.shared_structure.len());
    for record in &records {
        assert_eq!(record.license_name, "");
        assert_eq!(record.license_url, "");
        assert_eq!(record.source_id, procedure.id);
    }
}
