//! Issue #1172 R2 and R7: the installed gate has no fixture claims and
//! questions about a supplied passage use its own sentences.

use formal_ai::event_log::EventLog;
use formal_ai::{FormalAiEngine, fact_subject_gate, try_prompt_text_question};

fn has_evidence(links: &[String], prefix: &str) -> bool {
    links.iter().any(|link| link.starts_with(prefix))
}

// The installed solver leaves publisher QIDs unresolved before retrieval.
// Explicit transport and comparison tests live in seed/issue_1172_live_fact_answer.rs.
#[test]
fn the_installed_gate_does_not_infer_a_publisher_id_from_a_replay() {
    for prompt in [
        "What is the capital of the USA?",
        "Какова столица США?",
        "जापान की राजधानी क्या है?",
        "美国的首都是什么？",
        "What is the capital of Australia?",
        "What is the capital of Canada, not the USA?",
    ] {
        let gate = fact_subject_gate(prompt);
        assert_eq!(gate.qid, None, "{prompt}: {gate:?}");
        assert_eq!(
            gate.relation.as_deref(),
            Some("capital"),
            "{prompt}: {gate:?}"
        );
    }
}

// ---- R7: questions over prompt-supplied text -------------------------------

#[test]
fn a_question_over_a_quoted_passage_is_answered_from_its_sentence() {
    for (prompt, expected) in [
        (
            "Read this and answer: \"The meeting moved from Tuesday to Thursday at 3 pm in room 204.\" When and where is the meeting?",
            "From the text — time «from Tuesday to Thursday at 3 pm»; place «in room 204».\nThe text answers this: «The meeting moved from Tuesday to Thursday at 3 pm in room 204.»\nAs a statement: subject «The meeting»; predicate «moved»; time «from Tuesday to Thursday at 3 pm»; place «in room 204»",
        ),
        (
            "Given the text: 'The meeting moved from Tuesday to Thursday at 3 pm in room 204.' When is the meeting?",
            "From the text — time «from Tuesday to Thursday at 3 pm».\nThe text answers this: «The meeting moved from Tuesday to Thursday at 3 pm in room 204.»\nAs a statement: subject «The meeting»; predicate «moved»; time «from Tuesday to Thursday at 3 pm»; place «in room 204»",
        ),
        (
            "Given the text: \"Мы встретимся в четверг в комнате 204.\" Где встреча?",
            "По тексту — место «в комнате 204».\nОтвет есть в тексте: «Мы встретимся в четверг в комнате 204.»\nКак утверждение: субъект «Мы»; предикат «встретимся»; время «в четверг»; место «в комнате 204»",
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
        "From the text — time «at 6 pm».\nThe text answers this: «The cafe closes at 6 pm.»\nAs a statement: subject «The cafe»; predicate «closes»; time «at 6 pm»"
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
        "From the text — time «from Tuesday to Thursday at 3 pm»; place «in room 204».\nThe text answers this: «The meeting moved from Tuesday to Thursday at 3 pm in room 204.»\nAs a statement: subject «The meeting»; predicate «moved»; time «from Tuesday to Thursday at 3 pm»; place «in room 204»"
    );
}

#[test]
fn the_covering_sentence_projects_into_statement_slots() {
    let mut log = EventLog::new();
    let answer = try_prompt_text_question(
        "Given the text: \"The library opens at 9 am. The cafe closes at 6 pm. Parking is free on Sundays.\" When is parking free?",
        &mut log,
    )
    .expect("the text carries the answer");
    assert_eq!(
        answer.answer,
        "From the text — time «on Sundays».\nThe text answers this: «Parking is free on Sundays.»\nAs a statement: subject «Parking»; predicate «is free»; time «on Sundays»",
        "an inflected weekday is a time; the copula joins the predicate"
    );
    let payloads: Vec<&str> = log
        .events()
        .iter()
        .filter(|event| event.kind.starts_with("prompt_text:"))
        .map(|event| event.payload.as_str())
        .collect();
    assert!(
        payloads.contains(&"subject=Parking;predicate=is free;time=on Sundays"),
        "{payloads:?}"
    );
    assert!(payloads.contains(&"time"), "{payloads:?}");
}

#[test]
fn an_asked_slot_the_question_already_says_is_not_claimed() {
    let answer = try_prompt_text_question(
        "Given the text: 'The meeting moved from Tuesday to Thursday at 3 pm in room 204.' Who moved the meeting?",
        &mut EventLog::new(),
    )
    .expect("the text carries the sentence");
    assert_eq!(answer.intent, "prompt_text_answer");
    assert_eq!(
        answer.answer,
        "The text answers this: «The meeting moved from Tuesday to Thursday at 3 pm in room 204.»\nAs a statement: subject «The meeting»; predicate «moved»; time «from Tuesday to Thursday at 3 pm»; place «in room 204»",
        "subject «The meeting» is what the question names, not who moved it"
    );
}

#[test]
fn a_sentence_without_a_seeded_adjunct_keeps_the_quote_alone() {
    let answer = try_prompt_text_question(
        "Given the text: 'The meeting moved to Thursday. Alice chairs it.' Who chairs it?",
        &mut EventLog::new(),
    )
    .expect("the text carries the sentence");
    assert_eq!(answer.answer, "The text answers this: «Alice chairs it.»");
}
