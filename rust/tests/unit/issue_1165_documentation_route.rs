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
    DocumentedCommand, documented_catalog_programs, documented_command_matches,
    documented_run_commands, language_has_documented_procedure,
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
            row(
                "python3 -X pycache_prefix=/tmp/formal-ai-pycache -m py_compile main.py",
                None
            ),
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
    assert_eq!(
        documented("java"),
        [
            row("javac Main.java", Some("javac HelloWorldApp.java")),
            row("java Main", Some("java HelloWorldApp"))
        ]
    );
    assert_eq!(
        documented("php"),
        [row("php -l main.php", None), row("php main.php", None)]
    );
}

/// The run contract a catalog row's documented program binds, as
/// `(file, [(role, command, source)])`.
#[cfg(feature = "meta-language")]
fn bound(language: &str) -> (String, Vec<(String, String, String)>) {
    let program = documented_catalog_programs()
        .into_iter()
        .find(|program| program.language == language)
        .unwrap_or_else(|| panic!("{language} is retired to the documentation route"))
        .rediscovered
        .unwrap_or_else(|reason| panic!("{language}: {reason}"));
    (
        program.contract.save_as,
        program
            .contract
            .commands
            .into_iter()
            .map(|command| (command.role.to_owned(), command.command, command.source))
            .collect(),
    )
}

#[cfg(feature = "meta-language")]
fn sourced(role: &str, command: &str, source: &str) -> (String, String, String) {
    (role.to_owned(), command.to_owned(), source.to_owned())
}

/// R1165-6: the catalog's file stem stays when the program declares what
/// the commands invoke (Kotlin); otherwise the name a captured command states
/// (Java's `HelloWorldApp`) or the name after an `entry_container` keyword
/// (Scala's `object hello`) binds in the file and every command, and each
/// command names its source.
#[cfg(feature = "meta-language")]
#[test]
fn the_documented_run_contract_of_every_bound_language() {
    const ORACLE: &str = "https://docs.oracle.com/javase/tutorial/getStarted/cupojava/unix.html";
    const KOTLINLANG: &str = "https://kotlinlang.org/docs/command-line.html";
    assert_eq!(
        bound("java"),
        (
            String::from("HelloWorldApp.java"),
            vec![
                sourced("check", "javac HelloWorldApp.java", ORACLE),
                sourced("run", "java HelloWorldApp", ORACLE),
            ]
        )
    );
    assert_eq!(
        bound("scala"),
        (
            String::from("hello.scala"),
            vec![
                sourced("check", "scalac hello.scala", "catalog"),
                sourced("run", "scala hello", "catalog"),
            ]
        )
    );
    assert_eq!(
        bound("kotlin"),
        (
            String::from("Main.kt"),
            vec![
                sourced(
                    "check",
                    "kotlinc Main.kt -include-runtime -d Main.jar",
                    KOTLINLANG
                ),
                sourced("run", "java -jar Main.jar", KOTLINLANG),
            ]
        )
    );
}

/// The deviation a documented program carries that its verification cannot
/// see: php.net's `echo` prints no line break and its line has no `PHP_EOL`;
/// every other retired program prints its trailing newline.
#[cfg(feature = "meta-language")]
#[test]
fn only_the_php_page_example_prints_no_trailing_newline() {
    let deviations: Vec<(String, String)> = documented_catalog_programs()
        .into_iter()
        .filter_map(|program| {
            let deviation = program.rediscovered.ok()?.deviation?;
            Some((program.language, deviation))
        })
        .collect();
    assert_eq!(
        deviations,
        [(String::from("php"), String::from("trailing_newline=absent"))]
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
/// rediscover a verified program for (R and Laravel have no capture; Bash
/// and Haskell have no grammar row, since meta-language ships none).
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
        "lua",
        "r",
        "laravel",
        "bash",
        "haskell",
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
            "swift",
            "scala",
            "java",
            "php",
            "lua"
        ]
    );
}

/// The derivation names what verified a documented program: the recorded
/// harness run only when the program is the one that run executed (the Rust
/// Book's equals the template the issue-8 harness ran, by `content_id`),
/// otherwise its page and the decomposition check, and the answer's
/// execution status says the same.
#[cfg(feature = "meta-language")]
#[test]
fn a_documented_program_cites_only_the_run_that_verified_it() {
    let rust = formal_ai::UniversalSolver::default().solve("write me hello world program in Rust");
    assert!(
        rust.links_notation.contains(
            "program_verification language=rust task=hello_world \
             content_id=0x544f96663d886356 verification=recorded"
        ),
        "{}",
        rust.links_notation
    );
    let page = "https://wiki.python.org/moin/BeginnersGuide/Programmers/SimpleExamples";
    let python =
        formal_ai::UniversalSolver::default().solve("write me hello world program in Python");
    assert!(
        python.links_notation.contains(&format!(
            "program_verification language=python task=hello_world \
             content_id=0x{:016x} verification=decomposition source={page}",
            formal_ai::discovery_production::fnv1a64(b"print('Hello, world!')")
        )),
        "{}",
        python.links_notation
    );
    assert!(
        python
            .links_notation
            .contains(&format!("execution_environment {page}")),
        "{}",
        python.links_notation
    );
}
