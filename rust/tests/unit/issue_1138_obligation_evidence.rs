//! Issue #1138 B5 (plan 05, leaf 18): ten prompts, five languages, no silent discard.
//!
//! Each prompt names an artifact in its first clause and, in its second, a
//! *check* no composer can read an artifact out of. Today that second clause is
//! discarded by a `continue`; it must become an obligation node that decomposes,
//! and the session may not finalize while it is unattempted.
//!
//! The five seeded wordings and the five held-out paraphrases differ on the
//! surface and must produce the same obligation tree shape: one `FileBytes`
//! node and one check node. These strings live here and in
//! `data/benchmarks/`, never in `src/` or `data/seed/`.

use formal_ai::execution_evidence::{Evidence, EvidenceSource, ObservationKind};
use formal_ai::intent_formalization::formalize_intent;
use formal_ai::meta_frame::ProblemFrame;
use formal_ai::obligation_ledger::{
    ObligationExpectation, ObligationLedger, ObligationNode, ObligationOutcome, ObligationStep,
    next_step,
};
use formal_ai::recursive_execution::DEFAULT_SPLIT_DEPTH_BOUND;
use formal_ai::translation::formalize_prompt;

/// The seeded wordings: the enumeration cue, the artifact, then the check.
const SEEDED: &[(&str, &str)] = &[
    (
        "en",
        "Two things. First, create file notes/attribution.md containing Gemfile.lock. Second, confirm the first line of that file is exactly Gemfile.lock.",
    ),
    (
        "ru",
        "Две вещи. Сначала создай файл notes/attribution.md с содержимым Gemfile.lock. Затем подтверди, что первая строка этого файла — ровно Gemfile.lock.",
    ),
    (
        "hi",
        "दो काम। पहले notes/attribution.md फ़ाइल बनाओ जिसमें Gemfile.lock हो। फिर पुष्टि करो कि उस फ़ाइल की पहली पंक्ति ठीक Gemfile.lock है।",
    ),
    (
        "zh",
        "两件事。首先，创建文件 notes/attribution.md，内容为 Gemfile.lock。然后确认该文件的第一行正好是 Gemfile.lock。",
    ),
    (
        "es",
        "Dos cosas. Primero, crea el archivo notes/attribution.md con el contenido Gemfile.lock. Después confirma que la primera línea de ese archivo es exactamente Gemfile.lock.",
    ),
];

/// The held-out paraphrases: different surface wording, same obligation shape.
const HELD_OUT: &[(&str, &str)] = &[
    (
        "en",
        "I need two things done. Write Gemfile.lock into notes/attribution.md. After that, check that its opening line reads Gemfile.lock and nothing else.",
    ),
    (
        "ru",
        "Нужно сделать две вещи. Запиши Gemfile.lock в notes/attribution.md. После этого проверь, что его первая строка — это Gemfile.lock и ничего больше.",
    ),
    (
        "hi",
        "दो चीज़ें चाहिए। notes/attribution.md में Gemfile.lock लिखो। उसके बाद जाँचो कि उसकी पहली पंक्ति सिर्फ़ Gemfile.lock है।",
    ),
    (
        "zh",
        "需要完成两件事。把 Gemfile.lock 写入 notes/attribution.md。之后检查它的首行只有 Gemfile.lock。",
    ),
    (
        "es",
        "Hay dos cosas que hacer. Escribe Gemfile.lock en notes/attribution.md. Luego revisa que su primera línea sea solo Gemfile.lock.",
    ),
];

fn ledger_for(prompt: &str, language: &str) -> ObligationLedger {
    let candidate = formalize_prompt(prompt, language);
    let formalization = formalize_intent(prompt, language, Some(&candidate));
    let frame = ProblemFrame::from_formalization(&formalization);
    ObligationLedger::for_frame(&frame, prompt, DEFAULT_SPLIT_DEPTH_BOUND)
}

/// The shape both wordings must produce: a file-bytes node and a check node.
fn tree_shape(root: &ObligationNode) -> (usize, usize) {
    let mut leaves = Vec::new();
    root.collect_leaves(&mut leaves);
    let file_bytes = leaves
        .iter()
        .filter(|node| matches!(node.expectation, ObligationExpectation::FileBytes { .. }))
        .count();
    let checks = leaves
        .iter()
        .filter(|node| {
            matches!(
                node.expectation,
                ObligationExpectation::SymbolicCheck { .. }
                    | ObligationExpectation::OutputHash { .. }
                    | ObligationExpectation::CommandExit { .. }
            )
        })
        .count();
    (file_bytes, checks)
}

/// The check clause is a node in every one of the five languages. Nothing is
/// dropped because no composer could read an artifact out of it.
#[test]
fn a_second_clause_without_an_artifact_is_never_silently_dropped() {
    for (language, prompt) in SEEDED {
        let root = ObligationNode::build(prompt, DEFAULT_SPLIT_DEPTH_BOUND);
        let mut leaves = Vec::new();
        root.collect_leaves(&mut leaves);
        assert!(
            leaves.len() >= 2,
            "{language}: both clauses must be nodes, got {} for {prompt:?}",
            leaves.len()
        );
        assert!(
            leaves.iter().all(|node| !node.clause.trim().is_empty()),
            "{language}: every node must carry the clause the user wrote"
        );
        assert!(
            leaves
                .iter()
                .any(|node| matches!(node.expectation, ObligationExpectation::FileBytes { .. })),
            "{language}: the artifact clause must derive a file-bytes expectation"
        );
    }
}

/// The held-out paraphrase produces the same tree shape as its seeded sibling.
#[test]
fn a_held_out_paraphrase_produces_the_same_obligation_tree_shape() {
    for ((language, seeded), (held_out_language, held_out)) in SEEDED.iter().zip(HELD_OUT.iter()) {
        assert_eq!(
            language, held_out_language,
            "the two tables must line up language for language"
        );
        let seeded_shape = tree_shape(&ObligationNode::build(seeded, DEFAULT_SPLIT_DEPTH_BOUND));
        let held_out_shape =
            tree_shape(&ObligationNode::build(held_out, DEFAULT_SPLIT_DEPTH_BOUND));
        assert_eq!(
            seeded_shape, held_out_shape,
            "{language}: the paraphrase must produce the same obligation shape as the seeded wording"
        );
        assert_eq!(
            seeded_shape,
            (1, 1),
            "{language}: the shape is one file-bytes node and one check node"
        );
    }
}

/// While anything is unattempted the session may not finalize, in any language.
#[test]
fn the_session_does_not_finalize_while_any_obligation_is_unattempted() {
    for (language, prompt) in SEEDED {
        let ledger = ledger_for(prompt, language);
        assert!(
            ledger.unattempted_count() >= 1,
            "{language}: nothing was observed yet, so something is unattempted"
        );
        assert!(
            !ledger.every_obligation_discharged(),
            "{language}: an unattempted obligation must keep the session open"
        );
        let step = next_step(prompt, &[]).expect("work remains while an obligation is unattempted");
        assert!(
            !matches!(step, ObligationStep::ReportGap { .. }),
            "{language}: a gap may not be reported while a split is still available"
        );
    }
}

/// When nothing can be observed and nothing is left to split, the answer names
/// the clause, its byte span and the reason — never a completion sentence.
#[test]
fn a_gap_is_reported_with_its_clause_and_byte_span_not_as_completion() {
    let (language, prompt) = SEEDED[0];
    let mut ledger = ledger_for(prompt, language);
    let mut leaves = Vec::new();
    ledger.root.collect_leaves(&mut leaves);
    let spans: Vec<(usize, usize)> = leaves.iter().map(|node| node.span).collect();

    // Drive every node to its terminal state without supplying any observation.
    for _ in 0..DEFAULT_SPLIT_DEPTH_BOUND {
        if ledger.every_obligation_discharged() {
            break;
        }
        ledger.observe(&Evidence::observed(
            "true",
            vec![String::from("true")],
            Some(0),
            b"",
            ObservationKind::CommandExit,
            EvidenceSource::Engine,
        ));
    }

    let lino = ledger.to_links_notation();
    for (start, end) in spans {
        assert!(
            lino.contains(&start.to_string()) && lino.contains(&end.to_string()),
            "every obligation's byte span must be reported:\n{lino}"
        );
    }
    assert!(
        ledger.unsatisfiable_count() + ledger.unattempted_count() >= 1,
        "an unobserved obligation is reported, never converted into completion prose"
    );
}

/// A bare `ok` from a client-owned tool hashes to the wrong value: it refutes a
/// file-bytes expectation rather than satisfying it. This replaces the `"ok"`
/// shortcut the multiple-obligation test used to rely on.
#[test]
fn a_bare_ok_tool_result_does_not_satisfy_a_file_bytes_expectation() {
    let (language, prompt) = SEEDED[0];
    let mut ledger = ledger_for(prompt, language);
    let record = Evidence::from_tool_result(
        "write_file notes/attribution.md",
        "ok",
        EvidenceSource::Harness,
    );
    assert_eq!(
        record.exit_code, None,
        "a harness tool result carries no exit code"
    );
    ledger.observe(&record);

    let mut leaves = Vec::new();
    ledger.root.collect_leaves(&mut leaves);
    for leaf in &leaves {
        assert!(
            !matches!(leaf.outcome, ObligationOutcome::Satisfied { .. }),
            "a bare ok may never satisfy a file-bytes expectation: {leaf:?}"
        );
    }
    assert_eq!(
        ledger.satisfied_count(),
        0,
        "the gate is not weakened: no obligation is satisfied by an unobserved claim"
    );
}
