//! Issue #1173 R1173-3 recurrence formalization and explicit test replay.

use formal_ai::coding_recurrence::cache::{
    RecurrenceValue, evaluate_recurrence_expression, records_from, recurrence_fold,
    recurrence_source_match, render_cached_recurrence, source_recurrence_candidate,
    verified_recurrence_record,
};
use formal_ai::coding_task_spec::recognise;

const REPLAY: &str =
    include_str!("../../fixtures/coding-discovery/wikifunctions/recurrence-replay.lino");

/// The en, hi and ru prompts of routing probes p223, p224 and p225.
const PROBES: [&str; 3] = [
    "Write a Python function that returns the factorial of n",
    "Python में एक फ़ंक्शन लिखो जो n का फ़ैक्टोरियल लौटाए",
    "Напиши функцию на Python, которая возвращает факториал n",
];

const FACTORIAL: &str = "def factorial(n):\n    return 1 if (n == 0) else (n * factorial(n - 1))";

#[test]
fn the_reader_reads_every_committed_record_and_verifies_it_against_its_source_tests() {
    let records = records_from(REPLAY);
    assert_eq!(
        records
            .iter()
            .map(|record| (record.zid.as_str(), record.identifier.as_str()))
            .collect::<Vec<_>>(),
        [("Z13835", "fibonacci"), ("Z13667", "factorial")]
    );
    for record in &records {
        assert!(verified_recurrence_record(record), "{}", record.zid);
        assert_eq!(record.source_tests.len(), 4, "{}", record.zid);
    }
    assert_eq!(
        render_cached_recurrence(&records[0]).as_deref(),
        Some(
            "def fibonacci(n):\n    return n if (n <= 1) else (fibonacci(n - 1) + fibonacci(n - 2))"
        )
    );
    assert_eq!(
        render_cached_recurrence(&records[1]).as_deref(),
        Some(FACTORIAL)
    );
    for (record, expected) in records.iter().zip([55, 3_628_800]) {
        let recurrence = record.recurrence.as_ref().expect("a known expression");
        assert_eq!(
            evaluate_recurrence_expression(
                &recurrence.expression,
                &recurrence.expression,
                &recurrence.parameter,
                10,
                0,
            ),
            Some(RecurrenceValue::Number(expected)),
            "{}",
            record.zid
        );
    }
}

#[test]
fn the_fold_drops_marks_as_the_browser_fold_does() {
    // The values `recurrenceFold` computes for the same inputs.
    assert_eq!(
        recurrence_fold(PROBES[1]),
        "python म एक फकशन लख ज n क फकटरयल लटए"
    );
    assert_eq!(recurrence_fold("फक्टोरियल"), "फकटरयल");
    assert_eq!(recurrence_fold("Un(1,−1)"), "un 1 1");
    assert_eq!(recurrence_fold("! (math)"), "! math");
    assert_eq!(recurrence_fold("Fibonacci数列"), "fibonacci数列");
}

#[test]
fn every_probe_names_the_factorial_record_and_gets_its_source_candidate() {
    let records = records_from(REPLAY);
    for prompt in PROBES {
        assert_eq!(
            recurrence_source_match(prompt, &records).map(|record| record.zid.as_str()),
            Some("Z13667"),
            "{prompt}"
        );
        let spec = recognise(prompt).unwrap_or_else(|| panic!("{prompt} is not recognised"));
        let candidate = source_recurrence_candidate(&spec, &records)
            .unwrap_or_else(|| panic!("{prompt} gets no source recurrence"));
        assert_eq!(candidate.kind, "wikifunctions_recurrence", "{prompt}");
        assert_eq!(candidate.id, "Z13863", "{prompt}");
        assert_eq!(candidate.code.as_deref(), Some(FACTORIAL), "{prompt}");
        assert_eq!(candidate.callable_name.as_deref(), Some("factorial"));
        assert_eq!(candidate.source_tests.len(), 4, "{prompt}");
    }
    if let Some(unrelated) = recognise("Write a Python function that returns the sum of a list") {
        assert!(source_recurrence_candidate(&unrelated, &records).is_none());
    }
}
