use super::try_write_program_from_oracle;
use crate::event_log::EventLog;

#[test]
fn resolves_a_hello_world_for_an_uncatalogued_language() {
    let mut log = EventLog::new();
    let answer = try_write_program_from_oracle(
        "write a hello world program in Kotlin",
        "write a hello world program in kotlin",
        Some("hello_world"),
        Some("Kotlin"),
        &mut log,
    )
    .expect("oracle should answer kotlin hello world");

    assert_eq!(answer.intent, "write_program_oracle_hello_world_kotlin");
    assert!(answer.answer.contains("```kotlin"));
    assert!(answer.answer.contains("println"));
    assert!(answer.answer.contains("Hello, World!"));
    // Source attribution is mandatory: the answer comes from a cached external
    // knowledge base, not the verified catalogue.
    assert!(answer.answer.contains("Hello World Collection"));
    assert!(answer.answer.contains("helloworldcollection.de"));
    assert_eq!(
        log.first_of("knowledge_source").map(|event| event.payload.as_str()),
        Some("hello-world-collection")
    );
}

#[test]
fn falls_back_to_task_alias_when_the_task_hint_is_missing() {
    let mut log = EventLog::new();
    // No explicit task hint — the handler recovers it from the prompt alias.
    let answer = try_write_program_from_oracle(
        "write a hello world program in swift",
        "write a hello world program in swift",
        None,
        Some("swift"),
        &mut log,
    )
    .expect("task alias recovery should find hello world");
    assert_eq!(answer.intent, "write_program_oracle_hello_world_swift");
}

#[test]
fn declines_without_a_language() {
    let mut log = EventLog::new();
    assert!(try_write_program_from_oracle(
        "write a hello world program",
        "write a hello world program",
        Some("hello_world"),
        None,
        &mut log,
    )
    .is_none());
    assert!(try_write_program_from_oracle(
        "write a hello world program in   ",
        "write a hello world program in   ",
        Some("hello_world"),
        Some("   "),
        &mut log,
    )
    .is_none());
}

#[test]
fn declines_for_a_catalogued_or_unknown_language() {
    let mut log = EventLog::new();
    // Klingon is not in any source — the caller keeps its existing path.
    assert!(try_write_program_from_oracle(
        "write a hello world program in klingon",
        "write a hello world program in klingon",
        Some("hello_world"),
        Some("klingon"),
        &mut log,
    )
    .is_none());
}
