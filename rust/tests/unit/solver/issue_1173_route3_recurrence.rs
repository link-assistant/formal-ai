//! Issue #1173 R1173-3, routing probes p223-p225: a Python function request
//! that names a recurrence the committed cache
//! `js/source-cache/wikifunctions-recurrences.lino` holds is answered natively
//! from that cache, as the browser's `trySourceRecurrenceSynthesis` answers
//! it, instead of ending in the skill gap. The reader is
//! `formal_ai::coding_recurrence::cache`, the twin of
//! `js/worker/formal_ai_worker_recurrence.js`.

use formal_ai::coding_recurrence::cache::{
    RecurrenceValue, evaluate_recurrence_expression, recurrence_fold, recurrence_source_match,
    recurrence_source_records, render_cached_recurrence, source_recurrence_candidate,
    verified_recurrence_record,
};
use formal_ai::coding_task_spec::recognise;
use formal_ai::{SolverConfig, UniversalSolver};

/// The en, hi and ru prompts of routing probes p223, p224 and p225.
const PROBES: [&str; 3] = [
    "Write a Python function that returns the factorial of n",
    "Python में एक फ़ंक्शन लिखो जो n का फ़ैक्टोरियल लौटाए",
    "Напиши функцию на Python, которая возвращает факториал n",
];

const FACTORIAL: &str = "def factorial(n):\n    return 1 if (n == 0) else (n * factorial(n - 1))";

#[test]
fn the_reader_reads_every_committed_record_and_verifies_it_against_its_source_tests() {
    let records = recurrence_source_records();
    assert_eq!(
        records
            .iter()
            .map(|record| (record.zid.as_str(), record.identifier.as_str()))
            .collect::<Vec<_>>(),
        [("Z13835", "fibonacci"), ("Z13667", "factorial")]
    );
    for record in records {
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
    for prompt in PROBES {
        assert_eq!(
            recurrence_source_match(prompt).map(|record| record.zid.as_str()),
            Some("Z13667"),
            "{prompt}"
        );
        let spec = recognise(prompt).unwrap_or_else(|| panic!("{prompt} is not recognised"));
        let candidate = source_recurrence_candidate(&spec)
            .unwrap_or_else(|| panic!("{prompt} gets no source recurrence"));
        assert_eq!(candidate.kind, "wikifunctions_recurrence", "{prompt}");
        assert_eq!(candidate.id, "Z13863", "{prompt}");
        assert_eq!(candidate.code.as_deref(), Some(FACTORIAL), "{prompt}");
        assert_eq!(candidate.callable_name.as_deref(), Some("factorial"));
        assert_eq!(candidate.source_tests.len(), 4, "{prompt}");
    }
    if let Some(unrelated) = recognise("Write a Python function that returns the sum of a list") {
        assert!(source_recurrence_candidate(&unrelated).is_none());
    }
}

/// The page the cached factorial recurrence was formalized from, as the
/// answer cites it.
const SOURCE: &str = "- https://www.wikifunctions.org/w/api.php?action=wikilambda_fetch&format=json&zids=Z13863&language=en (CC0-1.0)";

/// The seeded `coding_synthesis_verified_answer` and
/// `coding_synthesis_sources_heading` responses in the probe's language, with
/// the cached recurrence and its four source tests bound.
fn expected_answer(language: &str) -> String {
    let (lead, status, heading) = match language {
        "ru" => (
            "Ниже приведён выведенный артефакт Python, собранный из найденных частей и проверенный в изолированной рабочей среде:",
            "Статус выполнения: тесты пройдены в изолированной ограниченной рабочей среде агента.\nКоманда проверки: `python3 solution.py`\nРезультат тестов: пройдено 4/4 исполняемых проверок.\nИзоляция: временная рабочая среда агента без унаследованного окружения и с ограниченным бюджетом команд.",
            "Источники:",
        ),
        "hi" => (
            "यह खोजे गए भागों से पुनर्निर्मित और पृथक workspace में सत्यापित Python artifact है:",
            "Execution status: सीमित पृथक agent workspace में tests pass हुए।\nजाँच command: `python3 solution.py`\nTest outcome: 4/4 executable checks pass हुए।\nWorkspace isolation: inherited environment के बिना अस्थायी agent workspace और सीमित command budget।",
            "स्रोत:",
        ),
        _ => (
            "Here is a derived Python artifact reconstructed from discovered parts and verified in an isolated workspace:",
            "Execution status: tests passed in isolated bounded agent workspace.\nCheck command: `python3 solution.py`\nTest outcome: 4/4 executable checks passed.\nWorkspace isolation: temporary agent workspace with no inherited environment beyond a constructed temporary directory, and a bounded command budget.",
            "Sources:",
        ),
    };
    format!("{lead}\n\n```python\n{FACTORIAL}\n```\n\n{status}\n{heading}\n{SOURCE}")
}

#[test]
fn the_native_solver_answers_the_probes_with_the_source_recurrence() {
    let solver = UniversalSolver::new(SolverConfig {
        offline: true,
        ..SolverConfig::default()
    });
    for (prompt, language) in PROBES.into_iter().zip(["en", "hi", "ru"]) {
        let answer = solver.solve(prompt);
        assert_eq!(
            answer.intent, "write_program",
            "{prompt}: {}",
            answer.answer
        );
        assert_eq!(answer.answer, expected_answer(language), "{prompt}");
    }
}
