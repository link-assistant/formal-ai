//! Issue #1165 R1165-6: no language row hard-codes a compile or run command
//! that a captured documentation page states.
//!
//! The policy seed's `command_procedure` rows name, per language and role,
//! the captured page and the `command_verb` its command line starts with;
//! `documented_language_commands` derives the command from that page with
//! the page's file name bound to the catalog's, and the catalog's language
//! rows take it from there. The browser twins are in
//! `rust/tests/web/issue-1165-documented-commands.test.mjs`.

use formal_ai::discovery_production::{
    SourcedCommand, command_procedures, documented_language_commands, documented_pairs,
    documented_run_commands,
};

const RUST_BOOK: &str = "https://doc.rust-lang.org/book/ch01-02-hello-world.html";
const TS_HANDBOOK: &str = "https://raw.githubusercontent.com/microsoft/TypeScript-Website/v2/packages/documentation/copy/en/handbook-v2/Basics.md";
const ORACLE: &str = "https://docs.oracle.com/javase/tutorial/getStarted/cupojava/unix.html";
const KOTLINLANG: &str = "https://kotlinlang.org/docs/command-line.html";

/// Every command the procedure rows derive, as
/// `(language, save_as, role, command, page)`.
const DERIVED: [(&str, &str, &str, &str, &str); 6] = [
    ("rust", "main.rs", "run", "./main", RUST_BOOK),
    (
        "typescript",
        "hello.ts",
        "check",
        "tsc hello.ts",
        TS_HANDBOOK,
    ),
    ("java", "Main.java", "check", "javac Main.java", ORACLE),
    ("java", "Main.java", "run", "java Main", ORACLE),
    (
        "kotlin",
        "Main.kt",
        "check",
        "kotlinc Main.kt -include-runtime -d Main.jar",
        KOTLINLANG,
    ),
    ("kotlin", "Main.kt", "run", "java -jar Main.jar", KOTLINLANG),
];

/// The catalog's language table, read as source text.
const LANGUAGES_SOURCE: &str = include_str!("../../src/coding/catalog/languages.rs");

/// R1165-6: each `command_procedure` row derives the catalog command from
/// its captured page, the page's file name bound to the catalog's
/// (kotlinlang's `hello.kt` and Oracle's `HelloWorldApp` become `Main`).
#[test]
fn each_command_procedure_row_derives_the_catalog_command_from_its_page() {
    for (language, save_as) in [
        ("rust", "main.rs"),
        ("typescript", "hello.ts"),
        ("java", "Main.java"),
        ("kotlin", "Main.kt"),
    ] {
        let expected: Vec<SourcedCommand> = DERIVED
            .iter()
            .filter(|row| row.0 == language)
            .map(|&(_, _, role, command, page)| SourcedCommand {
                role,
                command: command.to_owned(),
                source: page.to_owned(),
            })
            .collect();
        assert_eq!(
            documented_language_commands(language, save_as),
            expected,
            "{language}"
        );
    }
    assert_eq!(
        documented_language_commands("go", "main.go"),
        Vec::<formal_ai::discovery_production::SourcedCommand>::new()
    );
}

/// R1165-6: the catalog rows run with the derived commands, for a task the
/// page never documented too (Kotlin `FizzBuzz`).
#[test]
fn a_derived_command_is_the_catalog_command_of_every_task() {
    let kotlin: Vec<String> = documented_run_commands("kotlin", "hello_world")
        .into_iter()
        .map(|command| command.catalog)
        .collect();
    assert_eq!(
        kotlin,
        [
            "kotlinc Main.kt -include-runtime -d Main.jar",
            "java -jar Main.jar"
        ]
    );
    let response =
        formal_ai::UniversalSolver::default().solve("Write a Kotlin program that prints FizzBuzz");
    let recipe = response
        .execution_recipe
        .expect("a program answer carries its execution recipe");
    assert_eq!(recipe.path, "Main.kt");
    assert_eq!(
        recipe.commands,
        [
            "kotlinc Main.kt -include-runtime -d Main.jar",
            "java -jar Main.jar"
        ]
    );
}

/// The source text of one row of the language table, from its `slug` line
/// to the next row.
fn row_text(slug: &str) -> &'static str {
    let opening = format!("slug: \"{slug}\",");
    let start = LANGUAGES_SOURCE
        .find(&opening)
        .unwrap_or_else(|| panic!("no row for {slug}"));
    let rest = &LANGUAGES_SOURCE[start..];
    rest.find("ProgramLanguage {")
        .map_or(rest, |end| &rest[..end])
}

/// R1165-6: every command a captured page states is derived by a
/// `command_procedure` row, and the language table states none of them.
#[test]
fn no_language_row_hard_codes_a_command_a_captured_page_states() {
    let procedures: Vec<(String, String)> = command_procedures()
        .into_iter()
        .map(|row| (row.language, row.role))
        .collect();
    assert_eq!(
        procedures,
        DERIVED
            .iter()
            .map(|row| (row.0.to_owned(), row.2.to_owned()))
            .collect::<Vec<_>>()
    );
    let mut stated = 0;
    for (task, language) in documented_pairs() {
        let commands = documented_run_commands(&language, &task);
        let last = commands.len().saturating_sub(1);
        for (index, command) in commands.iter().enumerate() {
            if command.documented.is_none() {
                continue;
            }
            stated += 1;
            let role = if index == last { "run" } else { "check" };
            assert!(
                procedures.contains(&(language.clone(), role.to_owned())),
                "{language} {role}: its page states it, so a command_procedure row must derive it"
            );
        }
    }
    assert!(stated >= DERIVED.len());
    for (language, _, _, command, _) in DERIVED {
        assert!(
            !row_text(language).contains(&format!("\"{command}\"")),
            "the {language} row still states \"{command}\""
        );
    }
}
