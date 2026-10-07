//! Issue #1164 (E129): the held-out Pascal path and the execution half of
//! recomposition (R1164-8, R1164-9, R1164-13).
//!
//! Pascal stays held out of the seed: `data/seed/code-example-parts.lino`
//! carries no Pascal vocabulary and no program of any language. The grammar
//! row in `data/seed/program-cst-grammars.lino` (meta-language 0.58 ships
//! tree-sitter-pascal) lets the decomposer parse Pascal, the Free Pascal
//! Quick Start page supplies the example, and the RTL `WriteLn` page's prose
//! (the seed's `print` relation) names the output call. Everything a
//! recomposed Pascal program contains therefore comes from the
//! documentation, and every part cites the page it came from.
//!
//! The execution half compiles and runs each recomposition with the
//! language's own toolchain where that toolchain is installed: `rustc`
//! always (the suite runs under cargo), `go` and `fpc` when they are on
//! `PATH`, and a missing toolchain is reported on stderr instead of
//! guessed. The live test fetches the real pages and is gated by
//! `FORMAL_AI_LIVE_FETCH`.

#![cfg(feature = "meta-language")]

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::atomic::{AtomicUsize, Ordering};

use formal_ai::code_example_knowledge::{
    CodePartKind, CodeRecomposition, DecomposedCodeNode, ParameterBindings, ProseLink,
    decompose_code_node, decompose_code_node_from, generalize_examples, recompose_for_requirement,
};
use formal_ai::source_fetch::{CurlSourceTransport, SourceTransport};
use formal_ai::web_formalize::{PageBlock, formalize_page_with_context};

const FPC_QUICK_START: &str = "https://www.freepascal.org/_new/docs/quick-start/";
const FPC_WRITELN: &str = "https://www.freepascal.org/docs-html/rtl/system/writeln.html";
const PASCAL_PAGE: &str =
    include_str!("../fixtures/coding-discovery/issue-1164/pascal-quick-start.html");
const PARTS_SEED: &str = include_str!("../../../data/seed/code-example-parts.lino");

const RUST_HELLO: &str = "fn main() {\n    println!(\"Hello, world!\");\n}\n";
const GO_HELLO: &str =
    "package main\n\nimport \"fmt\"\n\nfunc main() {\n    fmt.Println(\"Hello, world!\")\n}\n";

/// The held-out literal every recomposition prints.
const HELD_OUT: &str = "Hello, Formal AI!";

static NEXT_WORKSPACE: AtomicUsize = AtomicUsize::new(0);

/// The prose the two Free Pascal pages supply.
///
/// The RTL page names the call that writes to standard output, the Quick
/// Start page the compile step.
fn pascal_prose() -> Vec<ProseLink> {
    vec![
        ProseLink {
            relation: "print".to_owned(),
            text: "WriteLn".to_owned(),
            source_url: FPC_WRITELN.to_owned(),
        },
        ProseLink {
            relation: "compile with".to_owned(),
            text: "fpc hello.pas".to_owned(),
            source_url: FPC_QUICK_START.to_owned(),
        },
    ]
}

/// Every code block of a page, in order, through the #1163 formalizer.
fn code_blocks(page: &[u8]) -> Vec<String> {
    let (_, blocks) = formalize_page_with_context(page, Some("text/html"), None);
    blocks
        .into_iter()
        .filter_map(|(_, block)| match block {
            PageBlock::CodeBlock { text, .. } => Some(text),
            _ => None,
        })
        .collect()
}

/// The page's Pascal example, found by its parts.
///
/// The first block the Pascal grammar decomposes into an output operation,
/// rather than a block picked by its language tag.
fn pascal_example(page: &[u8], prose: &[ProseLink]) -> DecomposedCodeNode {
    code_blocks(page)
        .iter()
        .filter_map(|code| decompose_code_node_from(code, "pascal", prose, FPC_QUICK_START).ok())
        .find(|node| {
            node.parts
                .iter()
                .any(|part| part.kind == CodePartKind::OutputOperation)
        })
        .expect("the Free Pascal page carries a Pascal example with an output call")
}

fn recompose(node: DecomposedCodeNode, language: &str) -> CodeRecomposition {
    let procedure = generalize_examples(&[node]);
    let mut bindings = BTreeMap::new();
    bindings.insert("output_literal".to_owned(), HELD_OUT.to_owned());
    recompose_for_requirement(&procedure, &ParameterBindings(bindings), language)
        .unwrap_or_else(|error| panic!("{language} recomposes: {error:?}"))
}

/// R1164-9: the Free Pascal documentation alone yields the held-out program.
///
/// No Pascal row sits in the part vocabulary; without the RTL
/// page's prose the example has no output operation, with it the call and
/// the compile step cite their pages, and recomposition slots the held-out
/// literal into the documentation's own body.
#[test]
fn held_out_pascal_is_recomposed_from_the_free_pascal_documentation() {
    assert!(!PARTS_SEED.contains("part_language pascal"));
    assert_eq!(
        code_blocks(PASCAL_PAGE.as_bytes()),
        [
            "program Hello;\n\nbegin\n  WriteLn('Hello, Free Pascal!');\nend.",
            "fpc hello.pas",
        ]
    );
    let bare = decompose_code_node(
        "program Hello;\n\nbegin\n  WriteLn('Hello, Free Pascal!');\nend.",
        "pascal",
        &[],
    )
    .expect("the pascal grammar row is registered");
    assert!(
        bare.parts
            .iter()
            .all(|part| part.kind != CodePartKind::OutputOperation),
        "no stored vocabulary names a Pascal output call"
    );

    let node = pascal_example(PASCAL_PAGE.as_bytes(), &pascal_prose());
    let cited = |kind: CodePartKind| -> Vec<(String, String)> {
        node.parts
            .iter()
            .filter(|part| part.kind == kind)
            .map(|part| (part.source_text.clone(), part.source_url.clone()))
            .collect()
    };
    assert_eq!(
        cited(CodePartKind::OutputOperation),
        [("WriteLn".to_owned(), FPC_WRITELN.to_owned())]
    );
    assert_eq!(
        cited(CodePartKind::StringLiteral),
        [("Hello, Free Pascal!".to_owned(), FPC_QUICK_START.to_owned())]
    );
    assert_eq!(
        cited(CodePartKind::BuildCommand),
        [("fpc hello.pas".to_owned(), FPC_QUICK_START.to_owned())]
    );

    let recomposition = recompose(node, "pascal");
    assert_eq!(
        recomposition.source,
        "program Hello;\n\nbegin\n  WriteLn('Hello, Formal AI!');\nend."
    );
    assert!(
        recomposition
            .part_source_urls
            .contains(&FPC_WRITELN.to_owned())
    );
    assert!(
        recomposition
            .part_source_urls
            .contains(&FPC_QUICK_START.to_owned())
    );
}

/// Whether `program` answers its version probe.
fn toolchain(program: &str, probe: &str) -> bool {
    Command::new(program)
        .arg(probe)
        .output()
        .is_ok_and(|output| output.status.success())
}

fn workspace(label: &str) -> PathBuf {
    let directory = std::env::temp_dir().join(format!(
        "formal-ai-issue-1164-{label}-{}-{}",
        std::process::id(),
        NEXT_WORKSPACE.fetch_add(1, Ordering::Relaxed)
    ));
    std::fs::create_dir_all(&directory).expect("create the workspace");
    directory
}

/// Run a command in `directory` and return its stdout.
///
/// A failing command fails the test with its stderr.
fn run(command: &mut Command, directory: &Path, what: &str) -> String {
    let output = command
        .current_dir(directory)
        .output()
        .unwrap_or_else(|error| panic!("{what} starts: {error}"));
    assert!(
        output.status.success(),
        "{what} succeeds: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    String::from_utf8(output.stdout).expect("UTF-8 output")
}

/// Compile and run a recomposition with its documented toolchain.
///
/// `None` when the toolchain is not installed here.
fn compile_and_run(recomposition: &CodeRecomposition) -> Option<String> {
    let directory = workspace(&recomposition.language_slug);
    let stdout = match recomposition.language_slug.as_str() {
        "rust" => {
            if !toolchain("rustc", "--version") {
                return None;
            }
            std::fs::write(directory.join("main.rs"), &recomposition.source).expect("write");
            run(
                Command::new("rustc").args(["--edition=2024", "-o", "hello", "main.rs"]),
                &directory,
                "rustc",
            );
            run(
                &mut Command::new(directory.join("hello")),
                &directory,
                "the Rust program",
            )
        }
        "go" => {
            if !toolchain("go", "version") {
                return None;
            }
            std::fs::write(directory.join("hello.go"), &recomposition.source).expect("write");
            run(
                Command::new("go").args(["run", "hello.go"]),
                &directory,
                "go run",
            )
        }
        "pascal" => {
            if !toolchain("fpc", "-iV") {
                return None;
            }
            std::fs::write(directory.join("hello.pas"), &recomposition.source).expect("write");
            // The documentation's own compile step: `fpc hello.pas`.
            run(Command::new("fpc").arg("hello.pas"), &directory, "fpc");
            run(
                &mut Command::new(directory.join("hello")),
                &directory,
                "the Pascal program",
            )
        }
        other => panic!("no toolchain is wired for {other}"),
    };
    let _ = std::fs::remove_dir_all(&directory);
    Some(stdout)
}

/// R1164-8: every recomposition compiles and prints the held-out literal.
///
/// Each language whose example built the procedure runs where its toolchain
/// is installed (`rustc` always is under cargo).
#[test]
fn recomposed_programs_compile_and_print_the_held_out_literal() {
    let recompositions = [
        recompose(
            decompose_code_node(RUST_HELLO, "rust", &[]).expect("rust decomposes"),
            "rust",
        ),
        recompose(
            decompose_code_node(GO_HELLO, "go", &[]).expect("go decomposes"),
            "go",
        ),
        recompose(
            pascal_example(PASCAL_PAGE.as_bytes(), &pascal_prose()),
            "pascal",
        ),
    ];
    for recomposition in &recompositions {
        let Some(stdout) = compile_and_run(recomposition) else {
            eprintln!(
                "skipped the {} run: its toolchain is not installed",
                recomposition.language_slug
            );
            continue;
        };
        assert_eq!(stdout, format!("{HELD_OUT}\n"));
    }
}

fn live_fetch_requested() -> bool {
    matches!(
        std::env::var("FORMAL_AI_LIVE_FETCH")
            .unwrap_or_default()
            .trim()
            .to_ascii_lowercase()
            .as_str(),
        "1" | "true" | "yes" | "on"
    )
}

/// R1164-13: the held-out Pascal path over the live Free Pascal pages.
///
/// Fetches the documentation, decomposes its example with the RTL page's
/// prose, recomposes the held-out literal, and runs it through `fpc` when
/// it is installed. Gated by `FORMAL_AI_LIVE_FETCH`.
#[test]
fn live_held_out_pascal_hello_world_from_official_docs() {
    if !live_fetch_requested() {
        eprintln!("skipped: set FORMAL_AI_LIVE_FETCH=1 to fetch the Free Pascal documentation");
        return;
    }
    let transport = CurlSourceTransport;
    let reference = transport
        .get(FPC_WRITELN)
        .expect("the RTL WriteLn page answers");
    let reference = String::from_utf8_lossy(&reference);
    assert!(
        reference.contains("WriteLn") && reference.contains("standard output"),
        "the RTL page still says WriteLn writes to standard output"
    );
    let page = transport
        .get(FPC_QUICK_START)
        .expect("the Quick Start page answers");
    let recomposition = recompose(pascal_example(&page, &pascal_prose()), "pascal");
    assert!(
        recomposition
            .source
            .contains(&format!("WriteLn('{HELD_OUT}')")),
        "{}",
        recomposition.source
    );
    let Some(stdout) = compile_and_run(&recomposition) else {
        eprintln!("skipped the fpc run: fpc is not installed");
        return;
    };
    assert_eq!(stdout, format!("{HELD_OUT}\n"));
}
