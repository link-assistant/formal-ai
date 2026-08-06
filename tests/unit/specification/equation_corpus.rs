//! Issue #891 — equation-type corpus with a verified-type ratchet.
//!
//! Issue #406 asked for at least fifty verified equation-type examples; nothing
//! counted them. This harness loads `data/benchmarks/equation-type-corpus.lino`
//! — one record per distinct equation type, each carrying the exact answer the
//! production solver produced for it — and asserts:
//!
//!   1. the corpus is well formed: at least `minimum_verified_types` distinct
//!      equation types, every category and every supported language represented,
//!      and a pass-count floor that cannot exceed the number of cases;
//!   2. every case, replayed through `FormalAiEngine::answer` (the production
//!      entry point), still routes to `calculation`, still names the expected
//!      engine in its evidence links, and still produces the recorded answer;
//!   3. the observed pass count never drops below `minimum_pass_count` and the
//!      distinct verified-type count never drops below `minimum_verified_types`
//!      — the CI ratchet the issue asks for.
//!
//! `benchmark_limitation` records name equation shapes the stack does not answer
//! today (irrational and complex roots, degenerate equations, unit-carrying
//! equations, named-unknown wrappers). They are asserted to keep *failing
//! loudly*: the engine must decline rather than fabricate an answer. When a
//! limitation is lifted upstream the assertion fires, so the record gets
//! promoted into a verified case instead of silently rotting.
//!
//! Expected answers are observed, never hand-written: regenerate them with
//! `cargo run --example issue_891_equation_probe` plus
//! `experiments/issue-891-build-corpus.py`.

use std::collections::BTreeSet;
use std::fs;
use std::path::Path;

use formal_ai::language::registered_languages;
use formal_ai::FormalAiEngine;

const FIXTURE: &str = "data/benchmarks/equation-type-corpus.lino";

#[derive(Debug)]
struct Record {
    kind: String,
    fields: Vec<(String, String)>,
}

#[derive(Debug)]
struct Case {
    id: String,
    source: String,
    equation_type: String,
    category: String,
    language: String,
    prompt: String,
    expected_intent: String,
    expected_engine: String,
    expected_answer: String,
}

#[derive(Debug)]
struct Limitation {
    id: String,
    category: String,
    prompt: String,
    observed_intent: String,
    description: String,
}

#[derive(Debug)]
struct Suite {
    minimum_pass_count: usize,
    minimum_verified_types: usize,
    source_ids: BTreeSet<String>,
    cases: Vec<Case>,
    limitations: Vec<Limitation>,
}

fn load_suite() -> Suite {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(FIXTURE);
    let text = fs::read_to_string(&path)
        .unwrap_or_else(|err| panic!("missing benchmark fixture {}: {err}", path.display()));
    parse_suite(&text)
}

fn parse_suite(text: &str) -> Suite {
    let mut minimum_pass_count = 0usize;
    let mut minimum_verified_types = 0usize;
    let mut source_ids = BTreeSet::new();
    let mut cases = Vec::new();
    let mut limitations = Vec::new();

    for record in split_records(text).iter().map(|block| parse_record(block)) {
        match record.kind.as_str() {
            "benchmark_suite" => {
                minimum_pass_count = field(&record, "minimum_pass_count")
                    .parse()
                    .expect("minimum_pass_count must be a non-negative integer");
                minimum_verified_types = field(&record, "minimum_verified_types")
                    .parse()
                    .expect("minimum_verified_types must be a non-negative integer");
            }
            "benchmark_source" => {
                source_ids.insert(field(&record, "id"));
            }
            "benchmark_case" => cases.push(Case {
                id: field(&record, "id"),
                source: field(&record, "source"),
                equation_type: field(&record, "equation_type"),
                category: field(&record, "category"),
                language: field(&record, "language"),
                prompt: field(&record, "prompt"),
                expected_intent: field(&record, "expected_intent"),
                expected_engine: field(&record, "expected_engine"),
                expected_answer: field(&record, "expected_answer"),
            }),
            "benchmark_limitation" => limitations.push(Limitation {
                id: field(&record, "id"),
                category: field(&record, "category"),
                prompt: field(&record, "prompt"),
                observed_intent: field(&record, "observed_intent"),
                description: field(&record, "limitation"),
            }),
            _ => {}
        }
    }

    Suite {
        minimum_pass_count,
        minimum_verified_types,
        source_ids,
        cases,
        limitations,
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

/// Structural guarantees: at least fifty distinct equation types, a sane floor,
/// every declared category covered, and all four supported languages present.
#[test]
fn issue_891_equation_corpus_is_well_formed() {
    let suite = load_suite();

    let equation_types = suite
        .cases
        .iter()
        .map(|case| case.equation_type.clone())
        .collect::<BTreeSet<_>>();
    assert_eq!(
        equation_types.len(),
        suite.cases.len(),
        "every case must declare a distinct equation_type; found {} types across {} cases",
        equation_types.len(),
        suite.cases.len(),
    );
    assert!(
        equation_types.len() >= suite.minimum_verified_types,
        "issue #406 asks for at least {} equation types; the corpus defines {}",
        suite.minimum_verified_types,
        equation_types.len(),
    );
    assert!(
        suite.minimum_verified_types >= 50,
        "the verified-type floor must not drop below the fifty types issue #406 requires; found {}",
        suite.minimum_verified_types,
    );
    assert!(
        suite.minimum_pass_count >= suite.minimum_verified_types
            && suite.minimum_pass_count <= suite.cases.len(),
        "minimum_pass_count={} must be in {}..={}",
        suite.minimum_pass_count,
        suite.minimum_verified_types,
        suite.cases.len(),
    );

    let categories = suite
        .cases
        .iter()
        .map(|case| case.category.clone())
        .collect::<BTreeSet<_>>();
    assert_eq!(
        categories,
        [
            "evaluation_and_percent",
            "linear_multi_operation",
            "linear_one_operation",
            "natural_language_wrapper",
            "placeholder_unknown",
            "polynomial",
            "symbolic_multi_variable",
        ]
        .into_iter()
        .map(str::to_owned)
        .collect::<BTreeSet<_>>(),
        "the recorded category coverage must stay complete",
    );

    // The language coverage is read from the registry rather than spelled out,
    // so registering a new language (English, Russian, Hindi, Chinese, Spanish
    // today) fails this test until the corpus gains an equation wrapper for it.
    let languages = suite
        .cases
        .iter()
        .map(|case| case.language.clone())
        .collect::<BTreeSet<_>>();
    assert_eq!(
        languages,
        registered_languages()
            .into_iter()
            .map(|language| language.slug().to_owned())
            .collect::<BTreeSet<_>>(),
        "every registered language must be exercised by an equation wrapper",
    );

    for case in &suite.cases {
        assert!(
            suite.source_ids.contains(&case.source),
            "case {} references unknown source {}",
            case.id,
            case.source,
        );
        assert!(
            !case.prompt.is_empty() && !case.expected_answer.is_empty(),
            "case {} must record both a prompt and the answer observed for it",
            case.id,
        );
        assert_eq!(
            case.expected_intent, "calculation",
            "case {} must expect a solved calculation",
            case.id,
        );
        assert!(
            !case.expected_engine.is_empty(),
            "case {} must record which engine produced the answer",
            case.id,
        );
    }

    assert!(
        !suite.limitations.is_empty(),
        "the corpus must record the upstream calculator limitations it found",
    );
    for limitation in &suite.limitations {
        assert!(
            !limitation.description.is_empty() && !limitation.category.is_empty(),
            "limitation {} must explain the gap and name its category",
            limitation.id,
        );
        assert_ne!(
            limitation.observed_intent, "calculation",
            "limitation {} records a solved case; promote it into a benchmark_case",
            limitation.id,
        );
    }
}

/// Capability check with the ratchet: every equation type is re-solved by the
/// production engine and must still produce its recorded answer.
#[test]
fn issue_891_equation_corpus_solves_every_type() {
    let suite = load_suite();

    let mut passed = 0usize;
    let mut verified_types = BTreeSet::new();
    let mut failures = Vec::new();

    for case in &suite.cases {
        match evaluate_case(case) {
            Ok(()) => {
                passed += 1;
                verified_types.insert(case.equation_type.clone());
            }
            Err(reason) => failures.push(format!("{}: {reason}", case.id)),
        }
    }

    let report = format!(
        "issue #891 equation-type corpus: passed={passed} failed={} total={} \
         verified_types={} minimum_pass_count={} minimum_verified_types={}",
        suite.cases.len() - passed,
        suite.cases.len(),
        verified_types.len(),
        suite.minimum_pass_count,
        suite.minimum_verified_types,
    );
    println!("{report}");
    for failure in &failures {
        println!("FAIL {failure}");
    }

    assert!(
        passed >= suite.minimum_pass_count,
        "equation-corpus pass-count floor dropped: passed={passed} \
         minimum_pass_count={}\n{}",
        suite.minimum_pass_count,
        failures.join("\n"),
    );
    assert!(
        verified_types.len() >= suite.minimum_verified_types,
        "verified equation types dropped below the floor: {} < {}\n{}",
        verified_types.len(),
        suite.minimum_verified_types,
        failures.join("\n"),
    );
}

/// The recorded gaps must keep failing loudly: a documented limitation may never
/// turn into a fabricated answer. When upstream lifts one, this assertion fires
/// so the record is promoted into a verified case.
#[test]
fn issue_891_recorded_limitations_never_fabricate_answers() {
    let suite = load_suite();

    for limitation in &suite.limitations {
        let response = FormalAiEngine.answer(&limitation.prompt);
        assert_ne!(
            response.intent, "calculation",
            "limitation {} now solves ({}); promote it into a benchmark_case with its \
             verified answer: {}",
            limitation.id, limitation.description, response.answer,
        );
        assert_eq!(
            response.intent, limitation.observed_intent,
            "limitation {} changed route: recorded `{}`, observed `{}` ({})",
            limitation.id, limitation.observed_intent, response.intent, response.answer,
        );
    }
}

fn evaluate_case(case: &Case) -> Result<(), String> {
    let response = FormalAiEngine.answer(&case.prompt);
    if response.intent != case.expected_intent {
        return Err(format!(
            "prompt {:?} routed to `{}` not `{}`; answer={}",
            case.prompt, response.intent, case.expected_intent, response.answer,
        ));
    }
    let engine = response
        .evidence_links
        .iter()
        .find_map(|link| link.strip_prefix("calculation:engine:"))
        .unwrap_or_default();
    if engine != case.expected_engine {
        return Err(format!(
            "prompt {:?} was answered by `{engine}` not `{}`",
            case.prompt, case.expected_engine,
        ));
    }
    if response.answer.trim() != case.expected_answer {
        return Err(format!(
            "prompt {:?} answered `{}` not `{}`",
            case.prompt,
            response.answer.trim(),
            case.expected_answer,
        ));
    }
    Ok(())
}
