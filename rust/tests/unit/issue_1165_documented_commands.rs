//! Verify documented command bindings using explicit publisher replays.

use formal_ai::discovery_production::{
    SourcedCommand, acquire_documentation_captures, command_procedures,
    documented_language_commands_from, documented_pairs, documented_run_commands_from,
};

#[allow(clippy::duplicate_mod)]
#[path = "../fixtures/coding-discovery/documentation_replay.rs"]
mod replay;

fn captures(
    language: &str,
    task: &str,
) -> Vec<formal_ai::discovery_production::DocumentationCapture> {
    acquire_documentation_captures(language, task, &replay::client())
}

const RUST_BOOK: &str = "https://doc.rust-lang.org/book/ch01-02-hello-world.html";
const TS_HANDBOOK: &str = "https://raw.githubusercontent.com/microsoft/TypeScript-Website/v2/packages/documentation/copy/en/handbook-v2/Basics.md";
const ORACLE: &str = "https://docs.oracle.com/javase/tutorial/getStarted/cupojava/unix.html";
const KOTLINLANG: &str = "https://kotlinlang.org/docs/command-line.html";

/// Every command the procedure rows derive, as
/// `(language, save_as, role, command, page)`.
const DERIVED: [(&str, &str, &str, &str, &str); 7] = [
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
    (
        "scala",
        "Main.scala",
        "run",
        "scala run Main.scala",
        "https://docs.scala-lang.org/scala3/book/taste-hello-world.html",
    ),
];

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
        ("scala", "Main.scala"),
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
            documented_language_commands_from(
                language,
                save_as,
                &captures(language, "hello_world")
            ),
            expected,
            "{language}"
        );
    }
    assert_eq!(
        documented_language_commands_from("go", "main.go", &captures("go", "hello_world")),
        Vec::<formal_ai::discovery_production::SourcedCommand>::new()
    );
}

/// R1165-6: the catalog rows run with the derived commands, for a task the
/// page never documented too (Kotlin `FizzBuzz`).
#[test]
fn a_derived_command_is_the_catalog_command_of_every_task() {
    let kotlin: Vec<String> = documented_run_commands_from("kotlin", &[])
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
    // Since 9b1ab09ca (PR #1614) a catalog FizzBuzz binds the caller's range
    // operands and asks for a range it was not given, so the request names
    // one -- the same prompt as the JS twin
    // (`rust/tests/web/issue-1165-documented-commands.test.mjs`).
    let response = formal_ai::UniversalSolver::default()
        .solve("Write a Kotlin program that prints FizzBuzz from 1 through 15");
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

/// Every publisher command claims provenance only when the page was read.
#[test]
fn command_procedure_references_cover_observed_publisher_commands() {
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
        let commands = documented_run_commands_from(&language, &captures(&language, &task));
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
}
