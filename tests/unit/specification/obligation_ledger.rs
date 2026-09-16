//! Issue #1138 B5 (plan 05, leaves 4–9): obligations discharged by observations.
//!
//! `ObligationOutcome::Satisfied` carries an `Evidence` and nothing else, so the
//! type system — not a convention — forbids a satisfied obligation without an
//! observation. `need_ledger_with_execution` is the single place
//! `NeedStatus::Satisfied` may be produced, an unrelated observation discharges
//! nothing (R710-R4), and a clause with no derivable expectation becomes a node
//! rather than a discard (R710-R9).

use std::fs;
use std::path::{Path, PathBuf};

use formal_ai::execution_evidence::{Evidence, EvidenceSource, ObservationKind};
use formal_ai::intent_formalization::formalize_intent;
use formal_ai::meta_frame::ProblemFrame;
use formal_ai::obligation_ledger::{
    ObligationExpectation, ObligationLedger, ObligationNode, ObligationOutcome, ObligationStep,
    clauses_with_spans, next_step,
};
use formal_ai::recursive_execution::DEFAULT_SPLIT_DEPTH_BOUND;
use formal_ai::translation::formalize_prompt;
use walkdir::WalkDir;

/// The two-clause request plan 05 is written around: the first clause names an
/// artifact, the second names a check no composer can read an artifact out of.
const REQUEST: &str = "Two things. First, create file notes/attribution.md containing Gemfile.lock. \
Second, confirm the first line of that file is exactly Gemfile.lock.";

fn ledger_for(request: &str) -> ObligationLedger {
    let candidate = formalize_prompt(request, "en");
    let formalization = formalize_intent(request, "en", Some(&candidate));
    let frame = ProblemFrame::from_formalization(&formalization);
    ObligationLedger::for_frame(&frame, request, DEFAULT_SPLIT_DEPTH_BOUND)
}

fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).to_path_buf()
}

fn src_files() -> Vec<(String, String)> {
    let mut files = Vec::new();
    for entry in WalkDir::new(repo_root().join("src"))
        .into_iter()
        .filter_map(Result::ok)
        .filter(|entry| entry.file_type().is_file())
    {
        if entry.path().extension().and_then(|e| e.to_str()) != Some("rs") {
            continue;
        }
        let relative = entry
            .path()
            .strip_prefix(repo_root())
            .unwrap_or(entry.path())
            .display()
            .to_string()
            .replace('\\', "/");
        files.push((
            relative,
            fs::read_to_string(entry.path()).unwrap_or_default(),
        ));
    }
    files
}

fn file_bytes_record(path: &str, bytes: &[u8]) -> Evidence {
    Evidence::observed(
        format!("cat {path}"),
        vec![String::from("cat"), path.to_owned()],
        Some(0),
        bytes,
        ObservationKind::FileBytes,
        EvidenceSource::LocalProcess,
    )
}

/// `Satisfied` has exactly one field and it is the record. There is no
/// constructor that reaches it without one.
#[test]
fn satisfied_is_unconstructible_without_an_execution_record() {
    let mut ledger = ledger_for(REQUEST);
    let record = file_bytes_record("notes/attribution.md", b"Gemfile.lock\n");
    let discharged = ledger.observe(record.clone());
    assert!(
        discharged.is_some(),
        "an observation that answers a node's expectation must discharge it"
    );

    let mut leaves = Vec::new();
    ledger.root.collect_leaves(&mut leaves);
    let satisfied: Vec<&ObligationNode> = leaves
        .iter()
        .copied()
        .filter(|node| matches!(node.outcome, ObligationOutcome::Satisfied { .. }))
        .collect();
    assert_eq!(
        satisfied.len(),
        1,
        "exactly the node whose expectation was answered may become satisfied"
    );
    match &satisfied[0].outcome {
        ObligationOutcome::Satisfied { record: held } => assert_eq!(
            held.evidence_id, record.evidence_id,
            "the satisfied variant carries the very record that discharged it"
        ),
        other => panic!("expected a satisfied outcome carrying its record, got {other:?}"),
    }
}

/// `NeedStatus::Satisfied` may be produced in exactly one place in `src/`, and
/// that place is `need_ledger_with_execution`.
#[test]
fn need_ledger_with_execution_is_the_only_producer_of_satisfied() {
    let mut producers: Vec<String> = Vec::new();
    for (path, text) in src_files() {
        for line in text.lines() {
            let code = line.split("//").next().unwrap_or(line);
            if !code.contains("NeedStatus::Satisfied") {
                continue;
            }
            // A comparison or a slug match reads the variant; only an
            // assignment or a construction produces it.
            let produces = code.contains("status: NeedStatus::Satisfied")
                || code.contains("status = NeedStatus::Satisfied")
                || code.contains("=> NeedStatus::Satisfied");
            if produces {
                producers.push(path.clone());
            }
        }
    }
    producers.sort_unstable();
    producers.dedup();
    assert_eq!(
        producers,
        vec![String::from("src/obligation_ledger.rs")],
        "NeedStatus::Satisfied must be produced only by need_ledger_with_execution"
    );
}

/// A result nothing expected clears nothing: `observe` returns `None` and no
/// node changes (R710-R4).
#[test]
fn an_unrelated_observation_discharges_nothing() {
    let mut ledger = ledger_for(REQUEST);
    let before = ledger.clone();
    let unrelated = Evidence::observed(
        "echo hello",
        vec![String::from("echo"), String::from("hello")],
        Some(0),
        b"hello\n",
        ObservationKind::CommandExit,
        EvidenceSource::LocalProcess,
    );
    assert_eq!(
        ledger.observe(unrelated),
        None,
        "no node expected this command, so nothing may be discharged"
    );
    assert_eq!(
        ledger, before,
        "an unrelated observation must leave the ledger unchanged"
    );
}

/// The second clause carries no artifact. It becomes a node with its byte span,
/// never a `continue` (R710-R9).
#[test]
fn a_clause_with_no_derivable_expectation_becomes_a_node_not_a_discard() {
    let root = ObligationNode::build(REQUEST, DEFAULT_SPLIT_DEPTH_BOUND);
    let mut leaves = Vec::new();
    root.collect_leaves(&mut leaves);
    assert!(
        leaves.len() >= 2,
        "both clauses must appear as nodes, got {}",
        leaves.len()
    );

    let check_clause = leaves
        .iter()
        .find(|node| node.clause.contains("first line"))
        .expect("the check clause must be a node of the tree");
    assert!(
        check_clause.span.1 > check_clause.span.0,
        "the node must carry the clause's byte span, got {:?}",
        check_clause.span
    );

    let lino = root.to_links_notation();
    assert!(
        lino.contains("first line"),
        "the undischarged clause must appear in the projection:\n{lino}"
    );
    assert!(
        lino.contains(&format!("{}", check_clause.span.0)),
        "the projection must carry the clause's byte span:\n{lino}"
    );
}

/// An underivable node is split before it is called unsatisfiable: `Decompose`
/// precedes `ReportGap`.
#[test]
fn an_underivable_node_is_split_before_it_is_called_unsatisfiable() {
    let step = next_step(REQUEST, &[]).expect("a two-clause request leaves work to do");
    assert!(
        matches!(
            step,
            ObligationStep::Observe(_) | ObligationStep::Decompose(_)
        ),
        "an underivable clause is decomposed before any gap is reported, got {step:?}"
    );
    assert!(
        !matches!(step, ObligationStep::ReportGap { .. }),
        "ReportGap may not be the first step while a split is still available"
    );
}

/// The recursion is bounded by the bound that already exists; no new constant is
/// invented in this module.
#[test]
fn the_split_is_bounded_by_the_existing_split_depth_bound() {
    let root = ObligationNode::build(REQUEST, DEFAULT_SPLIT_DEPTH_BOUND);
    let mut leaves = Vec::new();
    root.collect_leaves(&mut leaves);
    for leaf in &leaves {
        assert!(
            leaf.depth <= DEFAULT_SPLIT_DEPTH_BOUND,
            "node {} reached depth {} beyond the existing bound {DEFAULT_SPLIT_DEPTH_BOUND}",
            leaf.node_id,
            leaf.depth
        );
    }

    let module = fs::read_to_string(repo_root().join("src/obligation_ledger.rs"))
        .expect("src/obligation_ledger.rs should be readable");
    for line in module.lines() {
        let code = line.split("//").next().unwrap_or(line);
        assert!(
            !(code.contains("const") && code.contains("DEPTH")),
            "no second split-depth bound may be declared here: {line}"
        );
    }
}

/// An observation that contradicts the expectation reopens the node; it never
/// finishes it.
#[test]
fn a_refuted_observation_reopens_the_node_instead_of_finishing_it() {
    let mut ledger = ledger_for(REQUEST);
    let wrong = file_bytes_record("notes/attribution.md", b"Cargo.lock\n");
    ledger.observe(wrong);

    let mut leaves = Vec::new();
    ledger.root.collect_leaves(&mut leaves);
    let refuted = leaves
        .iter()
        .find(|node| matches!(node.outcome, ObligationOutcome::Refuted { .. }))
        .expect("a contradicting observation must refute the node it answered");
    assert!(
        !refuted.discharged(),
        "a refuted node is still open; it may not be treated as finished"
    );
    assert!(
        !ledger.every_obligation_discharged(),
        "a refuted node keeps the session from finalizing"
    );
}

/// Plan 05 leaf 1: the cues that cut a multi-clause request into its clauses are
/// seed rows in all five languages, never Rust literals — and each of them
/// actually cuts.
///
/// The clause splitter is the only thing standing between an enumerated second
/// obligation and the first clause swallowing it, so a language whose cue is
/// missing has no second obligation at all. The surfaces are read back out of
/// the lexicon and used to build the request the assertion splits, so the pin
/// cannot pass against a cue the splitter does not consult.
#[test]
fn the_enumeration_cues_are_seeded_in_five_languages() {
    const REQUIRED: &[(&str, &[&str])] = &[
        ("en", &["first", "second", "then", "after that"]),
        ("ru", &["сначала", "затем", "после этого"]),
        ("hi", &["पहले", "फिर", "उसके बाद"]),
        ("zh", &["首先", "然后", "之后"]),
        ("es", &["primero", "después", "luego"]),
    ];
    for (language, surfaces) in REQUIRED {
        let seeded = formal_ai::seed::lexicon()
            .words_for_role_in_languages(formal_ai::seed::ROLE_ENUMERATION_CUE, &[language]);
        for surface in *surfaces {
            assert!(
                seeded.iter().any(|word| word == surface),
                "{language}: the enumeration cue {surface:?} must be a seed row, \
                 not a literal in src/; the language has {seeded:?}"
            );
        }
        let opening = surfaces[0];
        let following = surfaces[surfaces.len() - 1];
        let request = format!("{opening} alpha. {following} beta.");
        let clauses = clauses_with_spans(&request);
        assert!(
            clauses.len() >= 2,
            "{language}: {request:?} must cut into at least two clauses, got {clauses:?}"
        );
    }
}

/// The session may not finish while anything is unattempted.
#[test]
fn every_obligation_discharged_is_false_while_any_node_is_unattempted() {
    let ledger = ledger_for(REQUEST);
    assert!(
        ledger.unattempted_count() >= 1,
        "a fresh ledger has unattempted obligations"
    );
    assert_eq!(
        ledger.satisfied_count(),
        0,
        "nothing may be satisfied before anything was observed"
    );
    assert!(
        !ledger.every_obligation_discharged(),
        "an unattempted obligation means the session is not done"
    );
    assert_eq!(
        ObligationExpectation::Underivable {
            reason: String::from("no_artifact_in_clause"),
        }
        .to_links_notation()
        .is_empty(),
        false,
        "an underivable expectation still serializes, so the gap is reportable"
    );
}
