//! Natural-language read+write access to the entire associative memory across
//! the four supported languages (issue #529).
//!
//! These cover the Turing-complete substitution path (read every stored value,
//! rewrite the matching ones) and the natural-language append path, both driven
//! entirely by the seed lexicon rather than hardcoded per-language phrases.

use formal_ai::{execute_memory_query, MemoryEvent, MemoryStore};

fn seeded_store(content: &str) -> MemoryStore {
    MemoryStore::from_events(vec![MemoryEvent {
        id: String::from("seed-1"),
        kind: Some(String::from("message")),
        role: Some(String::from("user")),
        content: Some(content.to_owned()),
        conversation_id: Some(String::from("conv-1")),
        ..MemoryEvent::default()
    }])
}

/// A substitution must actually rewrite the stored value (read + write), not
/// merely record an intent, and must report the change in the prompt's language.
///
/// `old`/`new` are the operands the prompt should resolve to; the seed content
/// holds two occurrences of `old` so the applied count is 2.
fn assert_substitution(prompt: &str, old: &str, new: &str, expected_answer_fragment: &str) {
    let mut store = seeded_store(&format!("{old} keeps {old}"));
    let execution = execute_memory_query(prompt, &mut store, Some("conv-1"))
        .unwrap_or_else(|| panic!("substitution should be recognised: {prompt}"));

    assert!(execution.changed, "store should change: {prompt}");
    assert_eq!(execution.answer.intent, "memory_substitution", "{prompt}");
    assert!(
        execution.answer.answer.contains(expected_answer_fragment),
        "answer `{}` should contain `{expected_answer_fragment}`",
        execution.answer.answer
    );

    // The original event's content is rewritten in place; both occurrences flip.
    let rewritten = store
        .events()
        .iter()
        .find(|event| event.id == "seed-1")
        .expect("seed event survives");
    assert_eq!(
        rewritten.content.as_deref(),
        Some(format!("{new} keeps {new}").as_str()),
        "both occurrences should be rewritten: {prompt}"
    );

    // The audit event records the applied count without rewriting itself.
    let audit = store
        .events()
        .iter()
        .find(|event| event.kind.as_deref() == Some("memory_substitution"))
        .expect("audit event appended");
    assert!(
        audit
            .evidence
            .iter()
            .any(|tag| tag == "substitution:applied=2"),
        "audit should record applied=2: {:?}",
        audit.evidence
    );
}

#[test]
fn substitution_is_recognised_across_languages() {
    assert_substitution(
        "replace alpha with beta in memory",
        "alpha",
        "beta",
        "Replaced \"alpha\" with \"beta\" in memory (2 occurrence(s) updated).",
    );
    assert_substitution(
        "замени альфа на бета в памяти",
        "альфа",
        "бета",
        "Заменил \"альфа\" на \"бета\"",
    );
    // Hindi is SOV: "X की जगह Y रखो" (put Y in place of X) → old=X, new=Y.
    assert_substitution(
        "स्मृति में अल्फा की जगह बीटा रखो",
        "अल्फा",
        "बीटा",
        "स्मृति में \"अल्फा\" को \"बीटा\" से बदला",
    );
    assert_substitution(
        "在记忆中把阿尔法换成贝塔",
        "阿尔法",
        "贝塔",
        "已在记忆中将\"阿尔法\"替换为\"贝塔\"",
    );
    // The directive may also lead in Russian regardless of scope position.
    assert_substitution(
        "в памяти замени alpha на beta",
        "alpha",
        "beta",
        "Заменил \"alpha\" на \"beta\"",
    );
}

#[test]
fn substitution_requires_a_memory_scope() {
    // A bare coding-style "replace X with Y" must NOT be hijacked as a memory
    // write — without a memory scope phrase the request is left for other
    // handlers (here: no write happens, recall finds nothing to change).
    let mut store = seeded_store("alpha");
    let execution = execute_memory_query("replace alpha with beta", &mut store, Some("conv-1"));
    let changed = execution.is_some_and(|exec| exec.changed);
    assert!(
        !changed,
        "bare replace without scope must not mutate memory"
    );
    assert_eq!(
        store.events()[0].content.as_deref(),
        Some("alpha"),
        "memory must be untouched"
    );
}

/// Append must store the statement and confirm it in the prompt's language.
fn assert_append(prompt: &str, expected_statement: &str, expected_answer_fragment: &str) {
    let mut store = MemoryStore::from_events(Vec::new());
    let execution = execute_memory_query(prompt, &mut store, Some("conv-1"))
        .unwrap_or_else(|| panic!("append should be recognised: {prompt}"));

    assert!(execution.changed, "append should change store: {prompt}");
    assert_eq!(execution.answer.intent, "memory_write", "{prompt}");
    assert!(
        execution.answer.answer.contains(expected_answer_fragment),
        "answer `{}` should contain `{expected_answer_fragment}`",
        execution.answer.answer
    );

    let stored = store
        .events()
        .iter()
        .any(|event| event.content.as_deref() == Some(expected_statement));
    assert!(
        stored,
        "statement `{expected_statement}` should be stored: {prompt}"
    );
}

#[test]
fn append_is_recognised_across_languages() {
    assert_append(
        "remember that the sky is blue",
        "the sky is blue",
        "Recorded memory: the sky is blue",
    );
    assert_append(
        "запомни что небо синее",
        "небо синее",
        "Запомнил: небо синее",
    );
    assert_append("याद रखो कि आकाश नीला है", "आकाश नीला है", "स्मृति में सहेजा गया:");
    assert_append("记住天空是蓝色的", "天空是蓝色的", "已记住:");
}
