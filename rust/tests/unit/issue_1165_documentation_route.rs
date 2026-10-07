//! Issue #1165 (E130): what the documentation route derives beyond programs.
//!
//! R1165-6: the check and run commands a catalog row hard-codes are compared
//! with the command lines its documentation captures state, a documented
//! line counting as the catalog command when it is the same command with the
//! documented file name bound to the catalog's; every command no captured
//! page states is listed with no documented line. R1165-4: a language the
//! documentation route rediscovers a verified program for is known. The
//! browser twins are in `rust/tests/web/issue-1165-documentation-captures.test.mjs`.

use formal_ai::discovery_production::{
    DocumentedCommand, documented_command_matches, documented_run_commands,
    language_has_documented_procedure,
};

/// `(catalog command, documented line)` for a captured language.
fn documented(language: &str) -> Vec<(String, Option<String>)> {
    documented_run_commands(language, "hello_world")
        .into_iter()
        .map(
            |DocumentedCommand {
                 catalog,
                 documented,
             }| (catalog, documented),
        )
        .collect()
}

fn row(catalog: &str, documented: Option<&str>) -> (String, Option<String>) {
    (catalog.to_owned(), documented.map(str::to_owned))
}

/// R1165-6: the documented commands of every captured language, exactly.
///
/// The Rust Book states `./main` (its compile line is `rustc main.rs`, not the
/// catalog's `rustc main.rs -o main`), kotlinlang states both Kotlin commands
/// for `hello.kt`, and the TypeScript handbook states `tsc hello.ts`; the
/// other captured pages state none of their catalog commands.
#[test]
fn documented_commands_of_every_captured_language() {
    assert_eq!(
        documented("rust"),
        [
            row("rustc main.rs -o main", None),
            row("./main", Some("./main"))
        ]
    );
    assert_eq!(
        documented("kotlin"),
        [
            row(
                "kotlinc Main.kt -include-runtime -d Main.jar",
                Some("kotlinc hello.kt -include-runtime -d hello.jar"),
            ),
            row("java -jar Main.jar", Some("java -jar hello.jar")),
        ]
    );
    assert_eq!(
        documented("typescript"),
        [
            row("tsc hello.ts", Some("tsc hello.ts")),
            row("node hello.js", None)
        ]
    );
    assert_eq!(documented("go"), [row("go run main.go", None)]);
    assert_eq!(
        documented("scala"),
        [row("scalac Main.scala", None), row("scala Main", None)]
    );
    assert_eq!(
        documented("python"),
        [
            row("python3 -m py_compile main.py", None),
            row("python3 main.py", None)
        ]
    );
    assert_eq!(
        documented("javascript"),
        [row("node --check main.js", None), row("node main.js", None)]
    );
    assert_eq!(
        documented("c"),
        [row("gcc main.c -o main", None), row("./main", None)]
    );
    assert_eq!(
        documented("cpp"),
        [row("g++ main.cpp -o main", None), row("./main", None)]
    );
    assert_eq!(
        documented("csharp"),
        [row("dotnet build", None), row("dotnet run", None)]
    );
    assert_eq!(
        documented("ruby"),
        [row("ruby -c main.rb", None), row("ruby main.rb", None)]
    );
}

/// R1165-6: the documented file name is bound once and everywhere.
#[test]
fn a_documented_command_binds_the_file_name_consistently() {
    assert!(documented_command_matches(
        "kotlinc hello.kt -d hello.jar",
        "kotlinc Main.kt -d Main.jar",
        "Main.kt"
    ));
    assert!(!documented_command_matches(
        "kotlinc hello.kt -d world.jar",
        "kotlinc Main.kt -d Main.jar",
        "Main.kt"
    ));
    assert!(!documented_command_matches(
        "rustc main.rs",
        "rustc main.rs -o main",
        "main.rs"
    ));
    assert!(!documented_command_matches(
        "go run .",
        "go run main.go",
        "main.go"
    ));
}

/// R1165-4: the documentation route knows exactly the languages its captures
/// rediscover a verified program for (Scala's captured example breaks the
/// run contract; Java, PHP, R and Laravel have no capture).
#[cfg(feature = "meta-language")]
#[test]
fn the_documentation_route_knows_the_languages_it_rediscovers() {
    let languages = [
        "rust",
        "python",
        "javascript",
        "typescript",
        "go",
        "c",
        "cpp",
        "csharp",
        "ruby",
        "kotlin",
        "swift",
        "scala",
        "java",
        "php",
        "r",
        "laravel",
    ];
    let known: Vec<&str> = languages
        .into_iter()
        .filter(|language| language_has_documented_procedure(language))
        .collect();
    assert_eq!(
        known,
        [
            "rust",
            "python",
            "javascript",
            "typescript",
            "go",
            "c",
            "cpp",
            "csharp",
            "ruby",
            "kotlin",
            "swift"
        ]
    );
}
