//! Question-necessity protocol regressions (issue #920).

use std::collections::BTreeMap;

use formal_ai::event_log::EventLog;
use formal_ai::question_necessity::{
    authorize_question, classify_question, enforce_questions, policy_summary, NecessityTrace,
    QuestionAuthorization, QuestionClass, QuestionRefusal,
};
use formal_ai::{SolverConfig, SymbolicAnswer, UniversalSolver};

fn has_evidence(answer: &SymbolicAnswer, kind: &str) -> bool {
    answer
        .evidence_links
        .iter()
        .any(|link| link.starts_with(&format!("{kind}:")))
}

fn necessity_trace(answer: &SymbolicAnswer) -> Vec<&str> {
    answer
        .links_notation
        .lines()
        .filter(|line| line.contains("question_necessity:"))
        .collect()
}

#[test]
fn issue_920_clarification_has_a_replayable_three_stage_necessity_trace() {
    let config = SolverConfig {
        temperature: 0.7,
        guess_probability: 0.0,
        questioning_rigor: 1.0,
        ..SolverConfig::default()
    };
    let first = UniversalSolver::new(config).solve("apple is a fruit");
    let replay = UniversalSolver::new(config).solve("apple is a fruit");

    assert_eq!(first.intent, "clarify_interpretation");
    assert_eq!(first.answer.matches('?').count(), 1, "{}", first.answer);
    for kind in [
        "question_necessity:memory",
        "question_necessity:workspace",
        "question_necessity:sources",
        "question_necessity:classification",
        "question_necessity:authorized",
        "question_necessity:asked",
    ] {
        assert!(
            has_evidence(&first, kind),
            "missing {kind} from {:?}",
            first.evidence_links
        );
    }
    assert_eq!(necessity_trace(&first), necessity_trace(&replay));
}

#[test]
fn issue_920_factual_unknown_is_not_asked_after_bounded_research() {
    let answer = UniversalSolver::new(SolverConfig {
        questioning_rigor: 0.8,
        offline: true,
        ..SolverConfig::default()
    })
    .solve("How should snorflax be calibrated for teal silence");

    assert_eq!(answer.intent, "unknown");
    assert!(
        !answer
            .answer
            .contains("Which one source or missing fact should I use?"),
        "a factual unknown must be researched and logged, not delegated: {}",
        answer.answer
    );
    assert_eq!(
        answer.answer.matches('?').count(),
        1,
        "only the requirement-level issue-report choice may remain: {}",
        answer.answer
    );
    assert!(has_evidence(&answer, "question_necessity:refused"));
    assert!(has_evidence(
        &answer,
        "question_necessity:research_required"
    ));
    assert!(has_evidence(&answer, "question_necessity:asked"));
    let refused = answer
        .evidence_links
        .iter()
        .position(|link| link.starts_with("question_necessity:refused:"))
        .expect("refusal event");
    let research = answer
        .evidence_links
        .iter()
        .position(|link| link.starts_with("question_necessity:research_required:"))
        .expect("research handoff event");
    assert!(refused < research, "{:?}", answer.evidence_links);
}

#[test]
fn issue_920_question_without_a_complete_trace_is_refused() {
    assert_eq!(
        authorize_question(QuestionClass::Requirement, &NecessityTrace::default(), 0),
        QuestionAuthorization::Refused(QuestionRefusal::MissingTrace)
    );
}

#[test]
fn issue_920_classification_and_budget_are_loaded_from_seed_data() {
    assert_eq!(
        classify_question("Which one source or missing fact should I use?").class,
        QuestionClass::Factual
    );
    assert_eq!(
        classify_question("Would you like me to report this issue?").class,
        QuestionClass::Requirement
    );

    let summary = policy_summary();
    assert_eq!(summary.required_stages, ["memory", "workspace", "sources"]);
    assert_eq!(summary.maximum_questions_per_answer, 1);
    assert_eq!(summary.ratchet_metric, "questions_per_100_tasks");
    assert_eq!(summary.ratchet_direction, "down");
    assert_eq!(summary.ratchet_maximum, 60);
}

#[test]
fn issue_920_requirement_greetings_are_preserved_with_a_trace() {
    for question in [
        "How may I help you?",
        "Чем могу помочь?",
        "मैं आपकी क्या मदद कर सकता हूँ?",
        "请问有什么可以帮您的?",
        "¿Cómo puedo ayudarte?",
    ] {
        assert_eq!(
            classify_question(question).class,
            QuestionClass::Requirement,
            "{question}"
        );
    }

    let answer = UniversalSolver::default().solve("Hi");
    assert_eq!(answer.answer, "Hi, how may I help you?");
    assert!(has_evidence(&answer, "question_necessity:asked"));
}

#[test]
fn issue_920_agent_authored_seed_is_preserved_byte_for_byte() {
    assert_eq!(
        include_str!("../../data/seed/question-necessity.lino"),
        include_str!(
            "../../docs/case-studies/issue-920/self-hosting-authorship/question-necessity.lino"
        )
    );
}

#[test]
fn issue_920_listed_questions_are_enforced_without_punctuation() {
    let body = "Plan.\n\nClarifying questions:\nPlease clarify:\n1. First input\n2. Second input";
    let mut log = EventLog::new();
    let enforced = enforce_questions(body, &mut log);
    assert!(enforced.contains("1. First input"));
    assert!(!enforced.contains("2. Second input"));
}

#[test]
fn issue_920_question_marks_in_quotes_code_and_urls_are_not_questions() {
    let body = "`why?` \"who?\" https://example.test/?q=x Should I continue?";
    let mut log = EventLog::new();
    let enforced = enforce_questions(body, &mut log);
    assert!(!enforced.contains("Should I continue?"));
    assert_eq!(
        log.events()
            .iter()
            .filter(|event| event.kind == "question_necessity:refused")
            .count(),
        1
    );
}

#[test]
fn issue_920_duplicate_question_presentations_share_one_identity() {
    let body = "Still needed from you:\n- First input\n- Second input\n\nClarifying questions:\nPlease clarify:\n1. First input\n2. Second input";
    let mut log = EventLog::new();
    let enforced = enforce_questions(body, &mut log);
    assert!(!enforced.contains("Still needed from you:"));
    assert!(enforced.contains("1. First input"));
    assert!(!enforced.contains("2. Second input"));
}

#[test]
fn issue_920_proof_followups_keep_only_the_smallest_requirement_question() {
    let answer = UniversalSolver::new(SolverConfig {
        guess_probability: 0.05,
        follow_up_probability: 0.95,
        ..SolverConfig::default()
    })
    .solve("Prove the Riemann hypothesis");

    let asked = answer
        .evidence_links
        .iter()
        .filter(|link| link.starts_with("question_necessity:asked:"))
        .count();
    assert_eq!(
        asked, 1,
        "answer: {}\nevidence: {:?}",
        answer.answer, answer.evidence_links
    );
    assert!(has_evidence(&answer, "question_necessity:refused"));
    assert!(answer.answer.contains("Clarifying questions:"));
    let clarification_footer = answer
        .answer
        .split_once("Clarifying questions:")
        .expect("clarification footer")
        .1;
    let numbered_questions = clarification_footer
        .lines()
        .filter(|line| {
            let line = line.trim_start();
            line.chars()
                .next()
                .is_some_and(|value| value.is_ascii_digit())
                && line.contains(". ")
        })
        .count();
    assert_eq!(numbered_questions, 1, "{}", answer.answer);
    assert!(!answer.answer.contains("Still needed from you:"));
}

fn benchmark_records() -> Vec<BTreeMap<String, String>> {
    let fixture = include_str!("../../data/benchmarks/question-necessity-suite.lino");
    let mut records = Vec::new();
    let mut current = BTreeMap::new();
    for line in fixture.lines() {
        if line.trim().is_empty() {
            continue;
        }
        if !line.starts_with(' ') {
            if !current.is_empty() {
                records.push(current);
                current = BTreeMap::new();
            }
            current.insert(String::from("record_id"), line.to_owned());
            continue;
        }
        let (name, value) = line
            .trim()
            .split_once(' ')
            .expect("benchmark fields have a value");
        current.insert(name.to_owned(), value.trim_matches('"').to_owned());
    }
    if !current.is_empty() {
        records.push(current);
    }
    records
}

#[test]
fn issue_920_question_necessity_benchmark_ratchets_down() {
    let records = benchmark_records();
    let suite = records
        .iter()
        .find(|record| {
            record.get("record_type").map(String::as_str) == Some("question_necessity_suite")
        })
        .expect("benchmark suite record");
    let cases = records
        .iter()
        .filter(|record| {
            record.get("record_type").map(String::as_str) == Some("question_necessity_case")
        })
        .collect::<Vec<_>>();
    assert_eq!(
        cases.len(),
        suite["case_count"].parse::<usize>().expect("case count")
    );

    let mut total_questions = 0;
    for case in &cases {
        let mut config = SolverConfig::default();
        if let Some(value) = case.get("questioning_rigor") {
            config.questioning_rigor = value.parse().expect("questioning rigor");
        }
        if let Some(value) = case.get("guess_probability") {
            config.guess_probability = value.parse().expect("guess probability");
        }
        if let Some(value) = case.get("follow_up_probability") {
            config.follow_up_probability = value.parse().expect("follow-up probability");
        }
        if let Some(value) = case.get("offline") {
            config.offline = value.parse().expect("offline flag");
        }
        let answer = UniversalSolver::new(config).solve(&case["prompt"]);
        let questions = answer
            .evidence_links
            .iter()
            .filter(|link| link.starts_with("question_necessity:asked:"))
            .count();
        assert_eq!(
            questions,
            case["expected_questions"]
                .parse::<usize>()
                .expect("expected question count"),
            "case {} answered with: {}",
            case["record_id"],
            answer.answer
        );
        total_questions += questions;
    }

    let questions_per_100_tasks = total_questions * 100 / cases.len();
    let maximum = suite["maximum"].parse::<usize>().expect("suite maximum");
    assert!(questions_per_100_tasks <= maximum);
    assert_eq!(maximum, policy_summary().ratchet_maximum);
}
