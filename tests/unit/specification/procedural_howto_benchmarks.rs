//! Issue #444 — procedural how-to / instruction-following benchmark slice.
//!
//! The maintainer asked us to "test at least 10 test cases or tasks from most
//! popular AI benchmarks on the topic". This harness loads
//! `data/benchmarks/procedural-howto-suite.lino` — self-authored, representative
//! procedural "how to X" prompts in the style of six widely-used instruction-
//! following benchmarks (`IFEval`, `Super-NaturalInstructions`, `Self-Instruct`,
//! `OASST1`, `BIG-bench`, `MMLU`), all carrying permissive licenses — and asserts:
//!
//!   1. the suite holds at least ten cases and every source has a held-out
//!      paraphrase variant (anti-memorization, the issue #317 ratchet);
//!   2. every source carries a permissive license and a pinned `source_ref`;
//!   3. each base prompt routes to the `procedural_how_to` handler and the
//!      answer restates the requested task;
//!   4. cases carrying a `followup` exercise the issue #444 elaboration rebind
//!      end to end (the follow-up turn must re-bind to the prior procedure and
//!      emit `procedural_how_to:followup` evidence);
//!   5. the observed pass count never drops below the recorded
//!      `minimum_pass_count` floor.
//!
//! Cases are self-authored representative prompts (the local-profile convention
//! issue #408 established): we reproduce the *shape* of each benchmark's tasks
//! rather than copying upstream text, so provenance stays clean regardless of an
//! upstream item's license.

use std::collections::BTreeSet;
use std::fs;
use std::path::Path;

use formal_ai::{ConversationTurn, ExecutionSurface, SolverConfig, UniversalSolver};

const FIXTURE: &str = "data/benchmarks/procedural-howto-suite.lino";
const PERMISSIVE_LICENSES: [&str; 3] = ["Apache-2.0", "CC-BY-4.0", "MIT"];
const HELD_OUT_VARIANT: &str = "held_out";

#[derive(Debug)]
struct Record {
    kind: String,
    fields: Vec<(String, String)>,
}

#[derive(Debug)]
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
    prompt: String,
    followup: Option<String>,
    expected_intent: String,
    expected_contains: Vec<String>,
    variant: String,
}

#[derive(Debug)]
struct Suite {
    minimum_pass_count: usize,
    sources: Vec<Source>,
    cases: Vec<Case>,
}

fn solver() -> UniversalSolver {
    UniversalSolver::new(SolverConfig {
        offline: true,
        execution_surface: ExecutionSurface::RustLibrary,
        temperature: 0.0,
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
            "benchmark_case" => {
                let followup = field(&record, "followup");
                cases.push(Case {
                    id: field(&record, "id"),
                    source: field(&record, "source"),
                    prompt: field(&record, "prompt"),
                    followup: (!followup.is_empty()).then_some(followup),
                    expected_intent: field(&record, "expected_intent"),
                    expected_contains: field_values(&record, "expected_contains"),
                    variant: field(&record, "variant"),
                });
            }
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
    let inner = raw
        .strip_prefix('"')
        .and_then(|value| value.strip_suffix('"'))
        .unwrap_or(raw);
    let mut out = String::with_capacity(inner.len());
    let mut chars = inner.chars();
    while let Some(ch) = chars.next() {
        if ch == '\\' {
            match chars.next() {
                Some('n') => out.push('\n'),
                Some('"') => out.push('"'),
                Some('\\') | None => out.push('\\'),
                Some(other) => {
                    out.push('\\');
                    out.push(other);
                }
            }
        } else {
            out.push(ch);
        }
    }
    out
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

/// Structural guarantees: at least ten cases, permissive sources with pinned
/// revisions, and a held-out paraphrase for every source.
#[test]
fn issue_444_procedural_howto_suite_is_well_formed() {
    let suite = load_suite();

    assert!(
        suite.cases.len() >= 10,
        "issue #444 asks for at least ten benchmark cases on the topic; found {}",
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
            PERMISSIVE_LICENSES.contains(&source.license.as_str()),
            "source {} has non-permissive license `{}`",
            source.id,
            source.license,
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

    for case in &suite.cases {
        assert!(
            source_ids.contains(&case.source),
            "case {} references unknown source {}",
            case.id,
            case.source,
        );
        assert_eq!(
            case.expected_intent, "procedural_how_to",
            "case {} should target the procedural_how_to handler",
            case.id,
        );
        assert!(
            !case.expected_contains.is_empty(),
            "case {} needs at least one deterministic expected_contains fragment",
            case.id,
        );
    }
}

/// Capability check: each case routes to `procedural_how_to`, the answer
/// restates the task, and follow-up cases exercise the elaboration rebind. The
/// observed pass count must meet the recorded floor.
#[test]
fn issue_444_procedural_howto_suite_routes_each_case() {
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
        "issue #444 procedural how-to suite: passed={passed} failed={} total={} \
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
        "procedural how-to pass-count floor dropped: passed={passed} \
         minimum_pass_count={}\n{}",
        suite.minimum_pass_count,
        failures.join("\n"),
    );
}

fn evaluate_case(solver: &UniversalSolver, case: &Case) -> Result<(), String> {
    let plan = solver.solve(&case.prompt);
    if plan.intent != case.expected_intent {
        return Err(format!(
            "base prompt routed to `{}` not `{}`; answer={}",
            plan.intent, case.expected_intent, plan.answer,
        ));
    }

    let response = if let Some(followup) = &case.followup {
        let history = [
            ConversationTurn::user(&case.prompt),
            ConversationTurn::assistant(plan.answer),
        ];
        let follow_up = solver.solve_with_history(followup, &history);
        if follow_up.intent != case.expected_intent {
            return Err(format!(
                "follow-up {followup:?} routed to `{}` not `{}`; answer={}",
                follow_up.intent, case.expected_intent, follow_up.answer,
            ));
        }
        if !follow_up
            .evidence_links
            .iter()
            .any(|link| link.starts_with("procedural_how_to:followup"))
        {
            return Err(format!(
                "follow-up {followup:?} did not emit procedural_how_to:followup evidence: {:?}",
                follow_up.evidence_links,
            ));
        }
        follow_up
    } else {
        plan
    };

    let answer = response.answer.to_lowercase();
    let missing = case
        .expected_contains
        .iter()
        .filter(|fragment| !answer.contains(&fragment.to_lowercase()))
        .cloned()
        .collect::<Vec<_>>();
    if !missing.is_empty() {
        return Err(format!(
            "answer missing {missing:?}; answer={}",
            response.answer
        ));
    }

    Ok(())
}
