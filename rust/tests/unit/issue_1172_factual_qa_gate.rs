//! Issue #1172 R2, R6 and R7: subject-verified factual Q&A.
//!
//! R2 — a seeded fact answers only when the question's formalized subject
//! resolves to the record's `subject_qid` (and its relation to the record's
//! relation); word-boundary alias matching survives only as the last-resort
//! hint for records that carry no Q-id. R6 — "Compare X and Y" over two
//! seeded subjects aligns their facts by relation and states the difference;
//! one unseeded side is an honest gap that names it. R7 — a question over a
//! quoted, prompt-supplied passage is answered from that passage's own
//! sentence, with no source consulted, or honestly says the text does not
//! say. The browser twins are pinned in
//! `rust/tests/web/issue-1172-factual-qa-subject-gate.test.mjs`.

use formal_ai::event_log::EventLog;
use formal_ai::web_engine_core::normalize_prompt;
use formal_ai::{
    FormalAiEngine, fact_subject_gate, gated_fact_record, resolve_fact_subject,
    try_fact_comparison, try_prompt_text_question,
};

fn has_evidence(links: &[String], prefix: &str) -> bool {
    links.iter().any(|link| link.starts_with(prefix))
}

// ---- R2: the formalized subject's Q-id gates every seeded record ----------

#[test]
fn the_subject_slot_resolves_to_the_seeded_records_qid() {
    for (prompt, qid) in [
        ("What is the capital of the USA?", "Q30"),
        ("Какова столица США?", "Q30"),
        ("जापान की राजधानी क्या है?", "Q17"),
        ("美国的首都是什么？", "Q30"),
    ] {
        let gate = fact_subject_gate(prompt);
        assert_eq!(gate.qid.as_deref(), Some(qid), "{prompt}: {gate:?}");
        assert_eq!(
            gate.relation.as_deref(),
            Some("capital"),
            "{prompt}: {gate:?}"
        );
    }
    // Australia has no seeded record, so its subject resolves to nothing.
    assert_eq!(
        fact_subject_gate("What is the capital of Australia?").qid,
        None
    );
    // The relation's clause carries the subject: the trailing "not the USA"
    // must not lend the question its Q-id.
    assert_eq!(
        fact_subject_gate("What is the capital of Canada, not the USA?").qid,
        None
    );
}

#[test]
fn the_longest_label_wins_and_a_cross_qid_tie_resolves_to_nothing() {
    // "us" (Q30) is a whole word here, but "japan" is the longer label.
    assert_eq!(
        resolve_fact_subject("tell us the capital of japan").map(|subject| subject.qid),
        Some(String::from("Q17"))
    );
    assert_eq!(resolve_fact_subject("japan china"), None);
}

#[test]
fn the_gate_names_how_a_record_was_admitted() {
    let prompt = "What is the capital of the USA?";
    let (record, gate) =
        gated_fact_record(prompt, &normalize_prompt(prompt)).expect("USA is seeded");
    assert_eq!(record.slug, "fact_capital_usa");
    assert_eq!(gate, "subject_qid:Q30");

    // A record without a Q-id keeps the word-boundary alias hint.
    let prompt = "Who painted the Mona Lisa?";
    let (record, gate) =
        gated_fact_record(prompt, &normalize_prompt(prompt)).expect("Mona Lisa is seeded");
    assert_eq!(record.slug, "fact_mona_lisa_painter");
    assert_eq!(gate, "surface_hint");

    let prompt = "What is the capital of Canada, not the USA?";
    assert!(gated_fact_record(prompt, &normalize_prompt(prompt)).is_none());
}

#[test]
fn engine_answers_seeded_subjects_through_the_gate() {
    let usa = FormalAiEngine.answer("What is the capital of the USA?");
    assert_eq!(usa.intent, "fact_lookup");
    assert_eq!(
        usa.answer,
        "The capital of the United States is Washington, D.C."
    );
    assert!(
        has_evidence(&usa.evidence_links, "fact_lookup:subject_gate:"),
        "{:?}",
        usa.evidence_links
    );
}

// ---- R6: seeded comparisons ----------------------------------------------

#[test]
fn two_seeded_subjects_are_compared_on_aligned_relations() {
    for (prompt, left, right, expected) in [
        (
            "Compare Japan and Russia.",
            "Tokyo",
            "Moscow",
            "Japan and Russia, compared on their seeded facts:\n- capital: Japan — Tokyo; Russia — Moscow\nDifference in capital: Tokyo for Japan, Moscow for Russia.",
        ),
        (
            "Сравни Японию и Россию",
            "Токио",
            "Москва",
            "Япония и Россия в сравнении по сохранённым фактам:\n- столица: Япония — Токио; Россия — Москва\nРазличие (столица): Токио у Япония, Москва у Россия.",
        ),
        (
            "जापान और रूस की तुलना करें",
            "टोक्यो",
            "मास्को",
            "सहेजे गए तथ्यों के आधार पर जापान और रूस की तुलना:\n- राजधानी: जापान — टोक्यो; रूस — मास्को\nराजधानी में अंतर: जापान के लिए टोक्यो, रूस के लिए मास्को।",
        ),
        (
            "比较日本和俄罗斯",
            "东京",
            "莫斯科",
            "根据已存事实比较日本和俄罗斯：\n- 首都：日本 — 东京；俄罗斯 — 莫斯科\n首都不同：日本是东京，俄罗斯是莫斯科。",
        ),
    ] {
        let mut log = EventLog::new();
        let answer = try_fact_comparison(prompt, &mut log)
            .unwrap_or_else(|| panic!("{prompt} should compare two seeded subjects"));
        assert_eq!(answer.intent, "fact_comparison", "{prompt}");
        assert_eq!(answer.answer, expected, "{prompt}");
        assert!(answer.answer.contains(left), "{prompt}: {}", answer.answer);
        assert!(answer.answer.contains(right), "{prompt}: {}", answer.answer);
        assert!(
            has_evidence(&answer.evidence_links, "fact_comparison:difference:"),
            "{prompt}: {:?}",
            answer.evidence_links
        );
    }
}

#[test]
fn the_english_comparison_states_the_difference() {
    let answer = try_fact_comparison("Compare Japan and Russia.", &mut EventLog::new())
        .expect("Japan and Russia are seeded");
    assert_eq!(
        answer.answer,
        "Japan and Russia, compared on their seeded facts:\n\
         - capital: Japan — Tokyo; Russia — Moscow\n\
         Difference in capital: Tokyo for Japan, Moscow for Russia."
    );
}

#[test]
fn a_comparison_with_one_unseeded_side_names_the_missing_side() {
    let answer = try_fact_comparison("Compare Japan and Australia", &mut EventLog::new())
        .expect("one seeded side is an honest gap");
    assert_eq!(answer.intent, "fact_comparison_gap");
    assert_eq!(
        answer.answer,
        "I can compare Japan and Australia only from seeded facts, and I have none for Australia. What I have for Japan:\n- capital: Tokyo\nAdd facts for Australia (or allow a live lookup) to compare them."
    );
    assert!(answer.answer.contains("Australia"), "{}", answer.answer);
    assert!(answer.answer.contains("Tokyo"), "{}", answer.answer);
}

#[test]
fn subjects_without_seeded_facts_leave_the_comparison_alone() {
    for prompt in [
        "Compare Rust and Go for writing web servers.",
        "Compare Japan and Japan",
        "What is the capital of Japan?",
    ] {
        assert!(
            try_fact_comparison(prompt, &mut EventLog::new()).is_none(),
            "{prompt} must not be claimed as a seeded comparison"
        );
    }
}

#[test]
fn engine_routes_a_seeded_comparison() {
    let answer = FormalAiEngine.answer("Compare Japan and Russia.");
    assert_eq!(answer.intent, "fact_comparison", "{}", answer.answer);
    assert_eq!(
        answer.answer,
        "Japan and Russia, compared on their seeded facts:\n- capital: Japan — Tokyo; Russia — Moscow\nDifference in capital: Tokyo for Japan, Moscow for Russia."
    );
    assert!(answer.answer.contains("Moscow"), "{}", answer.answer);
}

// ---- R7: questions over prompt-supplied text -------------------------------

#[test]
fn a_question_over_a_quoted_passage_is_answered_from_its_sentence() {
    for (prompt, expected) in [
        (
            "Read this and answer: \"The meeting moved from Tuesday to Thursday at 3 pm in room 204.\" When and where is the meeting?",
            "The text answers this: «The meeting moved from Tuesday to Thursday at 3 pm in room 204.»",
        ),
        (
            "Given the text: 'The meeting moved from Tuesday to Thursday at 3 pm in room 204.' When is the meeting?",
            "The text answers this: «The meeting moved from Tuesday to Thursday at 3 pm in room 204.»",
        ),
        (
            "Given the text: \"Мы встретимся в четверг в комнате 204.\" Где встреча?",
            "Ответ есть в тексте: «Мы встретимся в четверг в комнате 204.»",
        ),
        (
            "根据文本：「会议改到星期四下午三点，在204房间。」会议在哪里？",
            "文本中的答案：「会议改到星期四下午三点，在204房间。」",
        ),
    ] {
        let answer = try_prompt_text_question(prompt, &mut EventLog::new())
            .unwrap_or_else(|| panic!("{prompt} carries its own text"));
        assert_eq!(
            answer.intent, "prompt_text_answer",
            "{prompt}: {}",
            answer.answer
        );
        assert_eq!(answer.answer, expected, "{prompt}");
        assert!(answer.answer.contains("204"), "{prompt}: {}", answer.answer);
        assert!(
            has_evidence(&answer.evidence_links, "prompt_text:network:"),
            "{prompt}: {:?}",
            answer.evidence_links
        );
    }
}

#[test]
fn the_sentence_covering_the_question_is_the_one_quoted() {
    let answer = try_prompt_text_question(
        "Given the text: \"The library opens at 9 am. The cafe closes at 6 pm. Parking is free on Sundays.\" When does the cafe close?",
        &mut EventLog::new(),
    )
    .expect("the text carries the answer");
    assert_eq!(
        answer.answer,
        "The text answers this: «The cafe closes at 6 pm.»"
    );
}

#[test]
fn nothing_in_the_text_covers_the_question_so_the_text_does_not_say() {
    let answer = try_prompt_text_question(
        "Given the text: 'The meeting moved to Thursday. Alice chairs it.' Who is the CEO?",
        &mut EventLog::new(),
    )
    .expect("a cued text question is always answered from the text");
    assert_eq!(answer.intent, "prompt_text_gap");
    assert_eq!(
        answer.answer,
        "The given text does not say: none of its sentences mentions ceo."
    );
}

#[test]
fn a_quoted_phrase_inside_an_instruction_is_not_a_text_to_read() {
    for prompt in [
        "Translate 'I would like to order a coffee please' into French",
        "Proofread 'the quick brown fox jumps over the lazy dog'",
        "What is the capital of the USA?",
    ] {
        assert!(
            try_prompt_text_question(prompt, &mut EventLog::new()).is_none(),
            "{prompt} must not be read as a prompt-supplied text"
        );
    }
}

#[test]
fn engine_answers_the_issue_example_from_the_prompt_text() {
    let answer = FormalAiEngine.answer(
        "Read this and answer: \"The meeting moved from Tuesday to Thursday at 3 pm in room 204.\" When and where is the meeting?",
    );
    assert_eq!(answer.intent, "prompt_text_answer", "{}", answer.answer);
    assert_eq!(
        answer.answer,
        "The text answers this: «The meeting moved from Tuesday to Thursday at 3 pm in room 204.»"
    );
}
