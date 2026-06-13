//! Issue #412 (R6): coding-oracle fallback for uncatalogued languages.
//!
//! The verified catalog templates a fixed set of languages. For a language it
//! does not template — Kotlin, Swift, PHP — a canonical request like "write a
//! hello world program in Kotlin" used to dead-end on the unsupported answer.
//! The solver now treats the public knowledge bases (the Hello World Collection,
//! Rosetta Code, …) as cached external APIs and returns a reviewed snippet plus
//! its output and source attribution, exactly the "code + result" shape the
//! catalog produces.

use formal_ai::UniversalSolver;

#[test]
fn kotlin_hello_world_resolves_from_the_oracle() {
    let solver = UniversalSolver::default();
    let response = solver.solve("Write a hello world program in Kotlin");

    assert_eq!(
        response.intent, "write_program_oracle_hello_world_kotlin",
        "Kotlin hello world must resolve from the oracle, got: {} / {}",
        response.intent, response.answer
    );
    assert!(
        response.answer.contains("```kotlin"),
        "answer must carry a Kotlin code fence, got: {}",
        response.answer
    );
    assert!(
        response.answer.contains("println(\"Hello, World!\")"),
        "answer must contain the idiomatic Kotlin snippet, got: {}",
        response.answer
    );
    assert!(
        response.answer.contains("Hello World Collection")
            && response.answer.contains("helloworldcollection.de"),
        "answer must attribute its external source, got: {}",
        response.answer
    );
}

#[test]
fn swift_and_php_hello_world_resolve_from_the_oracle() {
    let solver = UniversalSolver::default();

    let swift = solver.solve("write me a hello world program in swift");
    assert_eq!(swift.intent, "write_program_oracle_hello_world_swift");
    assert!(swift.answer.contains("```swift"), "got: {}", swift.answer);

    let php = solver.solve("write a hello world program in php");
    assert_eq!(php.intent, "write_program_oracle_hello_world_php");
    assert!(php.answer.contains("```php"), "got: {}", php.answer);
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
