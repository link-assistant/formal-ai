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
    CodePartKind, DecomposeError, LITERAL_SLOT, ParameterBindings, ProseLink, decompose_code_node,
    decompose_code_node_from, decomposed_links_notation, generalize_examples,
    generalized_links_notation, recompose_for_requirement, recomposition_links_notation,
};
use formal_ai::coding_research_learning::{
    CodingResearchApproval, DecomposedProcedureSource, adopt_decomposed_procedure,
};
use formal_ai::web_formalize::{PageBlock, formalize_page_with_context};

const RUST_HELLO: &str = "fn main() {\n    println!(\"Hello, world!\");\n}\n";
const GO_HELLO: &str =
    "package main\n\nimport \"fmt\"\n\nfunc main() {\n    fmt.Println(\"Hello, world!\")\n}\n";
const KOTLIN_HELLO: &str = "fun main() {\n    println(\"Hello, world!\")\n}\n";
const SWIFT_HELLO: &str = "print(\"Hello, world!\")\n";
const PYTHON_HELLO: &str = "print(\"Hello, world!\")\n";

const PARTS_SEED: &str = include_str!("../../../data/seed/code-example-parts.lino");

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

/// R1164-9: Pascal stays held out. The seed carries no Pascal vocabulary
/// and no stored program of any language, so even a fully bound
/// requirement cannot produce a Pascal program until a decomposed Free
/// Pascal example contributes one.
#[test]
fn pascal_is_held_out_with_no_stored_program() {
    assert!(
        !PARTS_SEED.contains("program_shape"),
        "no program template is stored"
    );
    assert!(
        !PARTS_SEED.contains("part_language pascal"),
        "no Pascal vocabulary is stored"
    );
    let examples = [decompose_code_node(RUST_HELLO, "rust", &[]).expect("rust decomposes")];
    let procedure = generalize_examples(&examples);
    let mut bindings = BTreeMap::new();
    bindings.insert("output_literal".to_owned(), "Hello, Formal AI!".to_owned());
    bindings.insert("output_call".to_owned(), "writeln".to_owned());
    let error = recompose_for_requirement(&procedure, &ParameterBindings(bindings), "pascal")
        .expect_err("no Pascal example contributed a body");
    assert!(matches!(error, DecomposeError::ParseFailed(message) if message.contains("pascal")));
}

/// R1164-6: a target no example contributed a body for is an error naming
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

/// R1164-6: recomposition is built from the aligned parts -- the target's
/// own example body with the bound literal in its slot -- so recomposing a
/// Rust example for its own literal returns that example byte for byte, and
/// a held-out literal changes nothing but the literal.
#[test]
fn recomposition_builds_from_the_decomposed_example_body() {
    let rust = decompose_code_node(RUST_HELLO, "rust", &[]).expect("rust decomposes");
    assert_eq!(
        rust.program_body,
        RUST_HELLO.replace("Hello, world!", LITERAL_SLOT)
    );
    let go = decompose_code_node(GO_HELLO, "go", &[]).expect("go decomposes");
    assert!(
        go.program_body.contains("import \"fmt\""),
        "the import literal is not the slot"
    );
    let procedure = generalize_examples(&[rust, go]);
    assert_eq!(procedure.program_bodies.len(), 2);
    let same = recompose_for_requirement(&procedure, &ParameterBindings::default(), "rust")
        .expect("rust body");
    assert_eq!(same.source, RUST_HELLO);
    let mut bindings = BTreeMap::new();
    bindings.insert("output_literal".to_owned(), "Hello, Formal AI!".to_owned());
    let held_out =
        recompose_for_requirement(&procedure, &ParameterBindings(bindings), "go").expect("go body");
    assert_eq!(
        held_out.source,
        GO_HELLO.replace("Hello, world!", "Hello, Formal AI!")
    );
}

/// R1164-6: the aligned `output_call` is checked against the body -- a
/// requirement naming a call the example does not make is refused -- and a
/// language no example contributed is refused even though it has a grammar.
#[test]
fn recomposition_refuses_an_unaligned_call_or_language() {
    let examples = [
        decompose_code_node(RUST_HELLO, "rust", &[]).expect("rust decomposes"),
        decompose_code_node(GO_HELLO, "go", &[]).expect("go decomposes"),
    ];
    let procedure = generalize_examples(&examples);
    let mut bindings = BTreeMap::new();
    bindings.insert("output_literal".to_owned(), "Hello, Formal AI!".to_owned());
    bindings.insert("output_call".to_owned(), "eprintln!".to_owned());
    recompose_for_requirement(&procedure, &ParameterBindings(bindings), "rust")
        .expect_err("the rust body does not make eprintln!");
    let error = recompose_for_requirement(&procedure, &ParameterBindings::default(), "kotlin")
        .expect_err("no kotlin example contributed");
    assert!(matches!(error, DecomposeError::ParseFailed(message) if message.contains("kotlin")));
}

/// R1164-6: a bound literal is escaped for the quote that opens its slot.
#[test]
fn recomposition_escapes_the_bound_literal_for_its_quote() {
    let procedure = generalize_examples(&[
        decompose_code_node(RUST_HELLO, "rust", &[]).expect("rust decomposes")
    ]);
    let mut bindings = BTreeMap::new();
    bindings.insert("output_literal".to_owned(), "say \"hi\"".to_owned());
    let recomposition = recompose_for_requirement(&procedure, &ParameterBindings(bindings), "rust")
        .expect("rust body");
    assert!(recomposition.source.contains(r#"println!("say \"hi\"");"#));
}

/// R1164-6: parts read from a page's code carry that page's URL, so the
/// recomposition cites the code's source, not only the prose's.
#[test]
fn code_parts_carry_the_page_url() {
    const PAGE_URL: &str = "https://go.dev/tour/welcome/1";
    let node = decompose_code_node_from(GO_HELLO, "go", &[], PAGE_URL).expect("go decomposes");
    assert!(node.parts.iter().all(|part| part.source_url == PAGE_URL));
    let procedure = generalize_examples(&[node]);
    let recomposition = recompose_for_requirement(&procedure, &ParameterBindings::default(), "go")
        .expect("go body");
    assert_eq!(recomposition.part_source_urls, vec![PAGE_URL.to_owned()]);
}

/// R1164-2: every registered language with seed vocabulary decomposes its
/// Hello World into an output operation and the printed literal.
#[test]
fn registered_languages_decompose_output_and_literal() {
    for (language, source, call) in [
        (
            "javascript",
            "console.log(\"Hello, world!\");\n",
            "console.log",
        ),
        (
            "typescript",
            "console.log(\"Hello, world!\");\n",
            "console.log",
        ),
        (
            "java",
            "public class Main {\n    public static void main(String[] args) {\n        System.out.println(\"Hello, world!\");\n    }\n}\n",
            "System.out.println",
        ),
        (
            "csharp",
            "class Program {\n    static void Main() {\n        System.Console.WriteLine(\"Hello, world!\");\n    }\n}\n",
            "Console.WriteLine",
        ),
        (
            "c",
            "#include <stdio.h>\n\nint main(void) {\n    puts(\"Hello, world!\");\n    return 0;\n}\n",
            "puts",
        ),
        (
            "cpp",
            "#include <iostream>\n\nint main() {\n    std::cout << \"Hello, world!\" << std::endl;\n}\n",
            "std::cout",
        ),
        ("ruby", "puts \"Hello, world!\"\n", "puts"),
        ("php", "<?php\necho \"Hello, world!\";\n", "echo"),
        ("r", "print(\"Hello, world!\")\n", "print"),
    ] {
        let node = decompose_code_node(source, language, &[])
            .unwrap_or_else(|error| panic!("{language} must decompose: {error:?}"));
        assert!(
            node.parts
                .iter()
                .any(|part| part.kind == CodePartKind::OutputOperation && part.source_text == call),
            "{language} finds {call}"
        );
        assert!(
            node.parts
                .iter()
                .any(|part| part.kind == CodePartKind::StringLiteral
                    && part.source_text == "Hello, world!"),
            "{language} finds the literal"
        );
        assert!(
            node.program_body.contains(LITERAL_SLOT),
            "{language} slots its literal"
        );
    }
}

/// R1164-7: `adopt_decomposed_procedure` fills the provenance the
/// decomposer leaves empty and goes through the existing gate -- an
/// unexecuted procedure is refused for execution, a non-commercial license
/// is refused by name, and missing provenance never reaches the gate.
#[test]
fn adopt_decomposed_procedure_goes_through_the_gate() {
    let procedure = generalize_examples(&[
        decompose_code_node(RUST_HELLO, "rust", &[]).expect("rust decomposes"),
        decompose_code_node(GO_HELLO, "go", &[]).expect("go decomposes"),
    ]);
    let records = procedure.procedure_step_records();
    assert_eq!(records.first().map(|record| record.ordinal), Some(1));
    let source = DecomposedProcedureSource {
        source_url: "https://doc.rust-lang.org/book/ch01-02-hello-world.html".to_owned(),
        sha256: "a".repeat(64),
        fetched_at: "1759795200".to_owned(),
        license_name: "MIT".to_owned(),
        license_url: "https://opensource.org/licenses/MIT".to_owned(),
    };
    let approval = CodingResearchApproval::granted("maintainer");
    let refusal =
        adopt_decomposed_procedure(&procedure, "print a greeting", &source, None, &approval)
            .expect_err("an unexecuted procedure may not be adopted");
    assert_eq!(refusal.reason, "coding_research_execution_missing");

    let non_commercial = DecomposedProcedureSource {
        license_name: "CC BY-NC-SA 4.0".to_owned(),
        ..source
    };
    let refusal = adopt_decomposed_procedure(
        &procedure,
        "print a greeting",
        &non_commercial,
        None,
        &approval,
    )
    .expect_err("a non-commercial license blocks adoption");
    assert!(
        refusal.reason.contains("CC BY-NC-SA 4.0"),
        "{}",
        refusal.reason
    );

    let refusal = adopt_decomposed_procedure(
        &procedure,
        "print a greeting",
        &DecomposedProcedureSource::default(),
        None,
        &approval,
    )
    .expect_err("missing provenance never reaches the gate");
    assert_eq!(
        refusal.reason,
        "coding_research_decomposed_procedure_incomplete"
    );
}
