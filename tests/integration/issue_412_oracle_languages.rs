//! Issue #412 (R6): coding-oracle fallback for uncatalogued languages.
//!
//! The verified catalog templates a fixed set of languages. For a language it
//! does not template — Swift, Bash, Lua, Haskell — a canonical request like
//! "write a hello world program in Swift" used to dead-end on the unsupported
//! answer. The solver now treats the public knowledge bases (the Hello World
//! Collection, Rosetta Code, …) as cached external APIs and returns a reviewed
//! snippet plus its output and source attribution, exactly the "code + result"
//! shape the catalog produces.
//!
//! The set is not fixed forever, and that is the point: Kotlin was this file's
//! headline example until issue #921 catalogued it, and PHP followed it out
//! under issue #1021, so the first tests below now assert the graduation instead
//! of the fallback. A language leaving the oracle is the catalog growing, which
//! is the outcome the fallback exists to make unnecessary.

use formal_ai::UniversalSolver;

/// Kotlin used to be the headline example here, because the catalog did not
/// template it. Issue #921 added it — the hive-mind#2158 production matrix
/// dispatched a Kotlin Hello World and could not be answered — so Kotlin has
/// *graduated* from the oracle to the catalog.
///
/// That is the direction this handler was built for: its own module doc says it
/// fires only when the catalog "does not template" a language and is "purely
/// additive … only ever supplies an answer the caller would otherwise not have".
/// A catalogued language taking the catalog route is the fallback correctly
/// standing down, so the guarantee is asserted here rather than deleted.
#[test]
fn kotlin_graduated_from_the_oracle_to_the_catalog() {
    let solver = UniversalSolver::default();
    let response = solver.solve("Write a hello world program in Kotlin");

    assert_eq!(
        response.intent, "write_program",
        "Kotlin is catalogued now, so it must take the catalog route, got: {} / {}",
        response.intent, response.answer
    );
    assert!(
        response.answer.contains("```kotlin"),
        "answer must carry a Kotlin code fence, got: {}",
        response.answer
    );
    assert!(
        response.answer.contains("fun main()"),
        "answer must contain the catalogued Kotlin template, got: {}",
        response.answer
    );
    // The catalog templates Kotlin but no kotlinc verified it, so the answer
    // must not borrow the "compiled and ran" claim the verified languages carry.
    assert!(
        !response.answer.contains("compiled and ran"),
        "an unverified toolchain must not be reported as executed, got: {}",
        response.answer
    );
}

/// Swift is still uncatalogued, so it still resolves from the oracle: the
/// fallback keeps its job for every language the catalog does not template.
///
/// PHP shared this test until issue #1021 catalogued it, and rewriting what was
/// left was the chance to show the answer rather than a substring of it (R234-2,
/// issue #960): the oracle route is the one place an answer carries an external
/// attribution, and a reader cannot check that the attribution is honest -- a
/// cached snippet, credited to where it came from -- from `contains("```swift")`.
#[test]
fn swift_hello_world_resolves_from_the_oracle() {
    let solver = UniversalSolver::default();

    let swift = solver.solve("write me a hello world program in swift");
    assert_eq!(swift.intent, "write_program_oracle_hello_world_swift");
    assert_eq!(
        swift.answer,
        "Here is a minimal Swift program (hello world):\n\
         \n\
         ```swift\n\
         print(\"Hello, World!\")\n\
         ```\n\
         \n\
         Output:\n\
         ```text\n\
         Hello, World!\n\
         ```\n\
         Source: Hello World Collection \
         (http://helloworldcollection.de/#Swift), cached locally as a popular \
         example."
    );
}

/// PHP shared this file's fallback example with Swift until issue #1021, which
/// asked for a PHP request (issue #723) to be answered by generalization rather
/// than by a per-prompt fix. Cataloguing PHP is that generalization: the eleven
/// task templates every other catalogued language carries, verified by `php -l`
/// and executed in the issue-8 harness, so PHP graduates from the oracle the way
/// Kotlin did.
#[test]
fn php_graduated_from_the_oracle_to_the_catalog() {
    let solver = UniversalSolver::default();
    let response = solver.solve("write a hello world program in php");

    assert_eq!(
        response.intent, "write_program",
        "PHP is catalogued now, so it must take the catalog route, got: {} / {}",
        response.intent, response.answer
    );
    assert!(
        response.answer.contains("```php"),
        "answer must carry a PHP code fence, got: {}",
        response.answer
    );
    assert!(
        response.answer.contains("<?php"),
        "answer must contain the catalogued PHP template, got: {}",
        response.answer
    );
    // Unlike Kotlin, a real `php` toolchain verified this one, so the verified
    // execution status is the honest claim to carry.
    assert!(
        response.answer.contains("compiled and ran"),
        "the verified PHP toolchain must be reported as executed, got: {}",
        response.answer
    );
}

#[test]
fn catalogued_languages_still_use_the_verified_catalog() {
    let solver = UniversalSolver::default();
    let response = solver.solve("write a hello world program in Rust");

    // Rust is templated by the verified catalog, so it must NOT route through
    // the oracle — its "compiled and ran" guarantee stays intact.
    assert_eq!(
        response.intent, "write_program",
        "catalog languages must keep the verified route, got: {} / {}",
        response.intent, response.answer
    );
    assert!(
        response.answer.contains("compiled and ran"),
        "catalog answer must keep its verified execution status, got: {}",
        response.answer
    );
}
