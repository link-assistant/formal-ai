//! Issue #702 — bAbI-style world-state tracking benchmark slice.
//!
//! The issue's acceptance criteria ask for "a bAbI-style state-tracking
//! benchmark slice with a ratchet". This harness loads
//! `data/benchmarks/world-state-tracking-suite.lino` — short dialogues in the
//! shape of the bAbI state-tracking tasks (single supporting fact, two
//! supporting facts, object state change) plus everyday goal dialogues, in all
//! four supported languages — and asserts:
//!
//!   1. the suite holds at least ten cases, every source carries a license and a
//!      pinned `source_ref`, and every source has held-out paraphrase variants
//!      (anti-memorization, the issue #317 ratchet);
//!   2. each dialogue, replayed as conversation history, answers its state
//!      question from the current→target difference: the expected handler
//!      (`world_state_remaining` / `world_state_reached`) and, when the goal is
//!      still open, the expected remaining link named in the answer *and* in the
//!      evidence links (so the answer is links, not prose);
//!   3. the observed pass count never drops below the recorded
//!      `minimum_pass_count` floor.
//!
//! Cases are self-authored representative dialogues (the local-profile
//! convention issue #408 established): the *shape* of each upstream task is
//! reproduced rather than its text, so no upstream text is redistributed.

use std::collections::BTreeSet;
use std::fs;
use std::path::Path;

use formal_ai::solver::{ConversationTurn, ExecutionSurface, SolverConfig, UniversalSolver};
use formal_ai::world_model_dialog::WorldModelMode;

const FIXTURE: &str = "data/benchmarks/world-state-tracking-suite.lino";
const HELD_OUT_VARIANT: &str = "held_out";

#[derive(Debug)]
struct Record {
    kind: String,
    fields: Vec<(String, String)>,
}

#[derive(Debug)]
// `source_ref` is the fixture's own field name (the pinned upstream revision).
#[allow(clippy::struct_field_names)]
struct Source {
    id: String,
    license: String,
    source_ref: String,
}

#[derive(Debug)]
struct Case {
    id: String,
    source: String,
    language: String,
    turns: Vec<String>,
    query: String,
    expected_intent: String,
    expected_remaining: Vec<String>,
    variant: String,
}

#[derive(Debug)]
struct Suite {
    minimum_pass_count: usize,
    sources: Vec<Source>,
    cases: Vec<Case>,
}

/// The suite exercises the opted-in world model: with the knob off the handler
/// declines by design (R13), so the benchmark turns it on explicitly.
fn solver() -> UniversalSolver {
    UniversalSolver::new(SolverConfig {
        offline: true,
        execution_surface: ExecutionSurface::RustLibrary,
        temperature: 0.0,
        world_model_mode: WorldModelMode::Track,
        ..SolverConfig::default()
    })
}

fn load_suite() -> Suite {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(FIXTURE);
    let text = fs::read_to_string(&path)
        .unwrap_or_else(|err| panic!("missing benchmark fixture {}: {err}", path.display()));
    parse_suite(&text)
}

fn parse_suite(text: &str) -> Suite {
    let mut minimum_pass_count = 0usize;
    let mut sources = Vec::new();
    let mut cases = Vec::new();

    for record in split_records(text).iter().map(|block| parse_record(block)) {
        match record.kind.as_str() {
            "benchmark_suite" => {
                minimum_pass_count = field(&record, "minimum_pass_count")
                    .parse()
                    .expect("minimum_pass_count must be a non-negative integer");
            }
            "benchmark_source" => sources.push(Source {
                id: field(&record, "id"),
                license: field(&record, "license"),
                source_ref: field(&record, "source_ref"),
            }),
            "benchmark_case" => cases.push(Case {
                id: field(&record, "id"),
                source: field(&record, "source"),
                language: field(&record, "language"),
                turns: field_values(&record, "turn"),
                query: field(&record, "query"),
                expected_intent: field(&record, "expected_intent"),
                expected_remaining: field_values(&record, "expected_remaining"),
                variant: field(&record, "variant"),
            }),
            _ => {}
        }
    }

    Suite {
        minimum_pass_count,
        sources,
        cases,
    }
}

fn split_records(text: &str) -> Vec<String> {
    let mut records = Vec::new();
    let mut current: Vec<&str> = Vec::new();
    for line in text.lines() {
        let line = line.trim_end();
        if line.trim().is_empty() {
            continue;
        }
        if !line.starts_with(char::is_whitespace) && !current.is_empty() {
            records.push(current.join("\n"));
            current.clear();
        }
        current.push(line);
    }
    if !current.is_empty() {
        records.push(current.join("\n"));
    }
    records
}

fn parse_record(block: &str) -> Record {
    let mut lines = block.lines();
    let _header = lines.next();
    let mut kind = String::new();
    let mut fields = Vec::new();
    for line in lines {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }
        if let Some((name, raw)) = trimmed.split_once(' ') {
            let value = unquote(raw.trim());
            if name == "record_type" {
                kind = value;
            } else {
                fields.push((name.to_owned(), value));
            }
        }
    }
    Record { kind, fields }
}

fn unquote(raw: &str) -> String {
    raw.strip_prefix('"')
        .and_then(|value| value.strip_suffix('"'))
        .unwrap_or(raw)
        .to_owned()
}

fn field(record: &Record, name: &str) -> String {
    record
        .fields
        .iter()
        .find_map(|(k, v)| (k == name).then(|| v.clone()))
        .unwrap_or_default()
}

fn field_values(record: &Record, name: &str) -> Vec<String> {
    record
        .fields
        .iter()
        .filter(|(k, _)| k == name)
        .map(|(_, v)| v.clone())
        .collect()
}

/// Structural guarantees: enough cases, licensed and pinned sources, held-out
/// variants everywhere, and every supported language represented.
#[test]
fn issue_702_world_state_suite_is_well_formed() {
    let suite = load_suite();

    assert!(
        suite.cases.len() >= 10,
        "the state-tracking slice needs at least ten cases; found {}",
        suite.cases.len(),
    );
    assert!(
        suite.minimum_pass_count > 0 && suite.minimum_pass_count <= suite.cases.len(),
        "minimum_pass_count={} must be in 1..={}",
        suite.minimum_pass_count,
        suite.cases.len(),
    );

    for source in &suite.sources {
        assert!(
            !source.license.is_empty(),
            "source {} must name the license its shape is reproduced under",
            source.id,
        );
        assert!(
            !source.source_ref.is_empty(),
            "source {} must pin an upstream revision via source_ref",
            source.id,
        );
    }

    let source_ids = suite
        .sources
        .iter()
        .map(|s| s.id.clone())
        .collect::<BTreeSet<_>>();
    let held_out_sources = suite
        .cases
        .iter()
        .filter(|case| case.variant == HELD_OUT_VARIANT)
        .map(|case| case.source.clone())
        .collect::<BTreeSet<_>>();
    assert_eq!(
        held_out_sources, source_ids,
        "every benchmark source needs a held-out/paraphrased anti-memorization variant",
    );

    let languages = suite
        .cases
        .iter()
        .map(|case| case.language.clone())
        .collect::<BTreeSet<_>>();
    assert_eq!(
        languages,
        ["en", "hi", "ru", "zh"]
            .into_iter()
            .map(str::to_owned)
            .collect::<BTreeSet<_>>(),
        "all four supported languages must be tracked",
    );

    for case in &suite.cases {
        assert!(
            source_ids.contains(&case.source),
            "case {} references unknown source {}",
            case.id,
            case.source,
        );
        assert!(
            case.turns.len() >= 2 && !case.query.is_empty(),
            "case {} must state at least a fact, a goal, and the question",
            case.id,
        );
        assert!(
            matches!(
                case.expected_intent.as_str(),
                "world_state_remaining" | "world_state_reached"
            ),
            "case {} must expect a world-state answer, got `{}`",
            case.id,
            case.expected_intent,
        );
        assert_eq!(
            case.expected_remaining.is_empty(),
            case.expected_intent == "world_state_reached",
            "case {} must list its open target links unless the goal is reached",
            case.id,
        );
    }
}

/// Capability check with the ratchet: every dialogue must answer its state
/// question from the difference, and the pass count must meet the floor.
#[test]
fn issue_702_world_state_suite_tracks_each_case() {
    let suite = load_suite();
    let solver = solver();

    let mut passed = 0usize;
    let mut failures = Vec::new();

    for case in &suite.cases {
        match evaluate_case(&solver, case) {
            Ok(()) => passed += 1,
            Err(reason) => failures.push(format!("{}: {reason}", case.id)),
        }
    }

    let report = format!(
        "issue #702 world-state tracking suite: passed={passed} failed={} total={} \
         minimum_pass_count={}",
        suite.cases.len() - passed,
        suite.cases.len(),
        suite.minimum_pass_count,
    );
    println!("{report}");
    for failure in &failures {
        println!("FAIL {failure}");
    }

    assert!(
        passed >= suite.minimum_pass_count,
        "world-state tracking pass-count floor dropped: passed={passed} \
         minimum_pass_count={}\n{}",
        suite.minimum_pass_count,
        failures.join("\n"),
    );
}

fn evaluate_case(solver: &UniversalSolver, case: &Case) -> Result<(), String> {
    // The dialogue is replayed as history; the assistant's own turns never write
    // state, so a neutral acknowledgement between user turns is enough.
    let mut history = Vec::new();
    for turn in &case.turns {
        history.push(ConversationTurn::user(turn));
        history.push(ConversationTurn::assistant("noted"));
    }

    let answer = solver.solve_with_history(&case.query, &history);
    if answer.intent != case.expected_intent {
        return Err(format!(
            "query routed to `{}` not `{}`; answer={}",
            answer.intent, case.expected_intent, answer.answer,
        ));
    }
    for expected in &case.expected_remaining {
        if !answer.answer.contains(expected) {
            return Err(format!(
                "answer does not name the open target `{expected}`; answer={}",
                answer.answer,
            ));
        }
    }
    // Each open target is also its own evidence link, so the answer is backed by
    // the difference network rather than by prose alone. (Evidence values are
    // content-addressed, so the count is what is checkable here.)
    let recorded = answer
        .evidence_links
        .iter()
        .filter(|link| link.starts_with("world_state:remaining"))
        .count();
    if recorded != case.expected_remaining.len() {
        return Err(format!(
            "expected {} world_state:remaining evidence link(s), found {recorded}: {:?}",
            case.expected_remaining.len(),
            answer.evidence_links,
        ));
    }
    Ok(())
}
