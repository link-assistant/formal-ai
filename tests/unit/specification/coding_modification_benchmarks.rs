//! Issue #362 coding-modification benchmark coverage.
//!
//! The fixture records a deterministic local ratchet plus external instructed
//! code-editing datasets that are fetched only when the ignored network test is
//! explicitly enabled.

use std::collections::{BTreeMap, BTreeSet};
use std::env;
use std::fs;
use std::io::Read;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::{Mutex, MutexGuard, OnceLock};

use formal_ai::{ConversationTurn, ExecutionSurface, SolverConfig, UniversalSolver};
use lino_objects_codec::format::parse_indented;

const BENCHMARK_FIXTURE: &str = "data/benchmarks/coding-modification-suite.lino";
const DATASET_CACHE_DIR: &str = "target/formal-ai-benchmarks";
const REQUIRED_DATASETS: [&str; 3] = ["canitedit", "humanevalfix", "editbench"];
const REQUIRED_LANGUAGES: [&str; 4] = ["en", "hi", "ru", "zh"];
const PERMISSIVE_LICENSES: [&str; 2] = ["Apache-2.0", "MIT"];

#[derive(Debug)]
struct LinoRecord {
    kind: String,
    id: String,
    fields: Vec<(String, String)>,
}

#[derive(Debug)]
struct ExternalEditDataset {
    id: String,
    title: String,
    license: String,
    license_url: String,
    source_ref: String,
    download_url: String,
    download_cache: String,
    expected_min_bytes: u64,
    expected_file_kind: String,
    integration_mode: String,
    audit_note: String,
}

#[derive(Debug)]
struct CodingModificationCase {
    id: String,
    source: String,
    language: String,
    initial_prompt: String,
    first_edit_prompt: String,
    second_edit_prompt: String,
    expected_intent: String,
    expected_answer_contains: Vec<String>,
    expected_links_contains: Vec<String>,
}

#[derive(Debug, PartialEq, Eq)]
struct CodingModificationReport {
    passed: usize,
    failed: usize,
    minimum_pass_count: usize,
    failures: Vec<String>,
}

#[derive(Debug)]
struct CodingModificationSuite {
    external_datasets: BTreeMap<String, ExternalEditDataset>,
    cases: Vec<CodingModificationCase>,
    minimum_pass_count: usize,
    ratchet_policy: String,
    download_policy: String,
    audit: String,
}

#[test]
fn issue_362_manifest_records_download_on_test_sources() {
    let suite = load_suite();

    assert_external_dataset_sources_are_recorded(&suite);
    assert_no_external_dataset_payloads_are_committed();
    assert!(
        suite.download_policy.contains(DATASET_CACHE_DIR),
        "download policy should name the ignored test cache directory: {}",
        suite.download_policy,
    );
    assert!(
        suite.audit.contains("Edit, But Verify"),
        "suite should record the instructed code-editing benchmark audit: {}",
        suite.audit,
    );
}

#[test]
fn issue_362_multilingual_multi_turn_coding_modification_ratchet() {
    let _solver_guard = benchmark_solver_lock();
    let suite = load_suite();

    assert_multilingual_cases_are_recorded(&suite);
    assert_pass_count_floor_is_recorded(&suite);

    let report = run_coding_modification_suite(&suite);
    // Promotion replay consumes this stable report rather than trusting counts
    // supplied by a proposal document.
    println!("{}", render_report(&report));
    assert_eq!(report.passed + report.failed, suite.cases.len());
    assert!(
        report.passed >= report.minimum_pass_count,
        "coding-modification pass-count floor dropped: passed={} minimum_pass_count={}\n{}",
        report.passed,
        report.minimum_pass_count,
        render_report(&report),
    );
}

#[test]
#[ignore = "network benchmark: set FORMAL_AI_BULK_BENCHMARK=1 to fetch external code-edit datasets"]
fn issue_362_external_edit_datasets_download_on_test_only() {
    if !env_flag_enabled("FORMAL_AI_BULK_BENCHMARK") {
        eprintln!(
            "skipping external code-edit dataset download; set FORMAL_AI_BULK_BENCHMARK=1 to run it"
        );
        return;
    }

    let suite = load_suite();
    for dataset in suite.external_datasets.values() {
        download_and_validate_dataset(dataset);
    }
}

fn load_suite() -> CodingModificationSuite {
    let text = fs::read_to_string(repo_root().join(BENCHMARK_FIXTURE)).expect("benchmark fixture");
    validate_lino_syntax(&text);
    parse_suite(&text)
}

fn validate_lino_syntax(text: &str) {
    for record in split_records(text) {
        parse_indented(&record).expect("benchmark record should be valid Links Notation");
    }
}

fn parse_suite(text: &str) -> CodingModificationSuite {
    let mut external_datasets = BTreeMap::new();
    let mut cases = Vec::new();
    let mut minimum_pass_count = 0;
    let mut ratchet_policy = String::new();
    let mut download_policy = String::new();
    let mut audit = String::new();

    for record in parse_records(text) {
        match record.kind.as_str() {
            "coding_modification_suite" => {
                minimum_pass_count =
                    parse_usize_field(&record.fields, "minimum_pass_count").unwrap_or(0);
                ratchet_policy = field_value(&record.fields, "ratchet_policy");
                download_policy = field_value(&record.fields, "download_policy");
                audit = field_value(&record.fields, "audit");
            }
            "external_edit_dataset" => {
                let dataset = ExternalEditDataset {
                    id: record.id,
                    title: field_value(&record.fields, "title"),
                    license: field_value(&record.fields, "license"),
                    license_url: field_value(&record.fields, "license_url"),
                    source_ref: field_value(&record.fields, "source_ref"),
                    download_url: field_value(&record.fields, "download_url"),
                    download_cache: field_value(&record.fields, "download_cache"),
                    expected_min_bytes: parse_u64_field(&record.fields, "expected_min_bytes")
                        .unwrap_or(0),
                    expected_file_kind: field_value(&record.fields, "expected_file_kind"),
                    integration_mode: field_value(&record.fields, "integration_mode"),
                    audit_note: field_value(&record.fields, "audit_note"),
                };
                external_datasets.insert(dataset.id.clone(), dataset);
            }
            "coding_modification_case" => cases.push(CodingModificationCase {
                id: record.id,
                source: field_value(&record.fields, "source"),
                language: field_value(&record.fields, "language"),
                initial_prompt: field_value(&record.fields, "initial_prompt"),
                first_edit_prompt: field_value(&record.fields, "first_edit_prompt"),
                second_edit_prompt: field_value(&record.fields, "second_edit_prompt"),
                expected_intent: field_value(&record.fields, "expected_intent"),
                expected_answer_contains: field_values(&record.fields, "expected_answer_contains"),
                expected_links_contains: field_values(&record.fields, "expected_links_contains"),
            }),
            _ => {}
        }
    }

    CodingModificationSuite {
        external_datasets,
        cases,
        minimum_pass_count,
        ratchet_policy,
        download_policy,
        audit,
    }
}

fn parse_records(text: &str) -> Vec<LinoRecord> {
    split_records(text)
        .into_iter()
        .map(|record| parse_record(&record))
        .collect()
}

fn split_records(text: &str) -> Vec<String> {
    let mut records = Vec::new();
    let mut current = Vec::new();
    for line in text.lines() {
        let line = line.trim_end();
        if line.trim().is_empty() {
            continue;
        }
        if !line.starts_with(char::is_whitespace) && !current.is_empty() {
            records.push(current.join("\n"));
            current.clear();
        }
        current.push(line.to_owned());
    }
    if !current.is_empty() {
        records.push(current.join("\n"));
    }
    records
}

fn parse_record(block: &str) -> LinoRecord {
    let mut lines = block.lines().filter(|line| !line.trim().is_empty());
    let header = lines.next().expect("record header");
    let fields = lines
        .map(parse_lino_line)
        .filter(|(name, _)| !name.is_empty())
        .collect::<Vec<_>>();
    let kind = field_value(&fields, "record_type");
    let id = field_value(&fields, "id");
    assert!(!kind.is_empty(), "record `{header}` is missing record_type");
    assert!(!id.is_empty(), "record `{header}` is missing id");
    LinoRecord { kind, id, fields }
}

fn parse_lino_line(line: &str) -> (String, String) {
    let content = line.trim();
    if let Some((name, raw_value)) = content.split_once(' ') {
        (name.to_owned(), unescape_quoted(raw_value.trim()))
    } else {
        (content.to_owned(), String::new())
    }
}

fn unescape_quoted(raw: &str) -> String {
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

fn field_value(fields: &[(String, String)], name: &str) -> String {
    fields
        .iter()
        .find_map(|(field_name, value)| (field_name == name).then(|| value.clone()))
        .unwrap_or_default()
}

fn field_values(fields: &[(String, String)], name: &str) -> Vec<String> {
    fields
        .iter()
        .filter(|(field_name, _)| field_name == name)
        .map(|(_, value)| value.clone())
        .collect()
}

fn parse_usize_field(fields: &[(String, String)], name: &str) -> Option<usize> {
    let raw = field_value(fields, name);
    (!raw.is_empty()).then(|| {
        raw.parse::<usize>()
            .unwrap_or_else(|err| panic!("invalid {name} `{raw}`: {err}"))
    })
}

fn parse_u64_field(fields: &[(String, String)], name: &str) -> Option<u64> {
    let raw = field_value(fields, name);
    (!raw.is_empty()).then(|| {
        raw.parse::<u64>()
            .unwrap_or_else(|err| panic!("invalid {name} `{raw}`: {err}"))
    })
}

fn assert_external_dataset_sources_are_recorded(suite: &CodingModificationSuite) {
    for dataset_id in REQUIRED_DATASETS {
        let dataset = suite
            .external_datasets
            .get(dataset_id)
            .unwrap_or_else(|| panic!("missing external code-edit dataset `{dataset_id}`"));
        assert!(
            PERMISSIVE_LICENSES.contains(&dataset.license.as_str()),
            "{} has non-permissive license `{}`",
            dataset.id,
            dataset.license,
        );
        assert!(
            !dataset.title.is_empty(),
            "{} must record a reviewable source title",
            dataset.id,
        );
        assert!(
            dataset.download_url.starts_with("https://"),
            "{} must use an HTTPS download URL, got {}",
            dataset.id,
            dataset.download_url,
        );
        assert!(
            dataset.license_url.starts_with("https://"),
            "{} must record an HTTPS license URL, got {}",
            dataset.id,
            dataset.license_url,
        );
        assert!(
            !dataset.source_ref.is_empty(),
            "{} must record an exact upstream revision or file hash",
            dataset.id,
        );
        assert_eq!(
            dataset.integration_mode, "download_on_test",
            "{} should never vendor the full dataset",
            dataset.id,
        );
        assert!(
            dataset.download_cache.starts_with(DATASET_CACHE_DIR),
            "{} should download under {DATASET_CACHE_DIR}, got {}",
            dataset.id,
            dataset.download_cache,
        );
        assert!(
            dataset.expected_min_bytes > 0,
            "{} must record a non-zero payload size floor",
            dataset.id,
        );
        assert_eq!(
            dataset.expected_file_kind, "parquet",
            "{} should be validated as a parquet payload",
            dataset.id,
        );
        assert!(
            !dataset.audit_note.is_empty(),
            "{} should record dataset-specific audit caveats",
            dataset.id,
        );
    }
}

fn assert_no_external_dataset_payloads_are_committed() {
    let payloads = collect_files_with_extension(&repo_root().join("data"), "parquet");
    assert!(
        payloads.is_empty(),
        "external benchmark payloads must not be committed under data/: {payloads:?}",
    );
}

fn collect_files_with_extension(root: &Path, extension: &str) -> Vec<PathBuf> {
    let mut files = Vec::new();
    collect_files_with_extension_into(root, extension, &mut files);
    files
}

fn collect_files_with_extension_into(root: &Path, extension: &str, files: &mut Vec<PathBuf>) {
    let entries = fs::read_dir(root).unwrap_or_else(|err| {
        panic!(
            "failed to read directory {} while checking benchmark payloads: {err}",
            root.display()
        )
    });
    for entry in entries {
        let entry = entry.unwrap_or_else(|err| {
            panic!(
                "failed to read directory entry under {}: {err}",
                root.display()
            )
        });
        let path = entry.path();
        if path.is_dir() {
            collect_files_with_extension_into(&path, extension, files);
        } else if path.extension().and_then(|value| value.to_str()) == Some(extension) {
            files.push(path);
        }
    }
}

fn assert_multilingual_cases_are_recorded(suite: &CodingModificationSuite) {
    let languages = suite
        .cases
        .iter()
        .map(|case| case.language.as_str())
        .collect::<BTreeSet<_>>();
    for language in REQUIRED_LANGUAGES {
        assert!(
            languages.contains(language),
            "missing multilingual coding-modification case for `{language}` in {languages:?}",
        );
    }
    assert!(
        suite
            .cases
            .iter()
            .any(|case| case.source == "issue_349_dialog"),
        "issue #349 dialog must be part of the coding-modification suite",
    );
    for case in &suite.cases {
        assert!(
            !case.initial_prompt.is_empty()
                && !case.first_edit_prompt.is_empty()
                && !case.second_edit_prompt.is_empty(),
            "{} should model initial draft -> edit -> edit",
            case.id,
        );
        assert!(
            !case.expected_answer_contains.is_empty(),
            "{} needs deterministic answer checks",
            case.id,
        );
        assert!(
            !case.expected_links_contains.is_empty(),
            "{} needs deterministic trace checks",
            case.id,
        );
    }
}

fn assert_pass_count_floor_is_recorded(suite: &CodingModificationSuite) {
    assert!(
        suite.minimum_pass_count > 0,
        "coding-modification suite must record a monotonic minimum_pass_count",
    );
    assert!(
        suite.minimum_pass_count <= suite.cases.len(),
        "minimum_pass_count={} cannot exceed suite size {}",
        suite.minimum_pass_count,
        suite.cases.len(),
    );
    assert!(
        suite.ratchet_policy.contains("minimum_pass_count"),
        "ratchet policy should name the pass-count floor: {}",
        suite.ratchet_policy,
    );
}

fn run_coding_modification_suite(suite: &CodingModificationSuite) -> CodingModificationReport {
    let solver = benchmark_solver();
    let mut passed = 0;
    let mut failures = Vec::new();

    for case in &suite.cases {
        match run_coding_modification_case(&solver, case) {
            Ok(()) => passed += 1,
            Err(error) => failures.push(error),
        }
    }

    CodingModificationReport {
        passed,
        failed: suite.cases.len() - passed,
        minimum_pass_count: suite.minimum_pass_count,
        failures,
    }
}

fn run_coding_modification_case(
    solver: &UniversalSolver,
    case: &CodingModificationCase,
) -> Result<(), String> {
    let first = solver.solve(&case.initial_prompt);
    if first.intent != case.expected_intent {
        return Err(format!(
            "{} initial prompt routed to {}, expected {}; answer: {}",
            case.id, first.intent, case.expected_intent, first.answer,
        ));
    }

    let first_history = [
        ConversationTurn::user(case.initial_prompt.clone()),
        ConversationTurn::assistant(first.answer.clone()),
    ];
    let second = solver.solve_with_history(&case.first_edit_prompt, &first_history);
    if second.intent != case.expected_intent {
        return Err(format!(
            "{} first edit routed to {}, expected {}; answer: {}",
            case.id, second.intent, case.expected_intent, second.answer,
        ));
    }

    let second_history = [
        ConversationTurn::user(case.initial_prompt.clone()),
        ConversationTurn::assistant(first.answer),
        ConversationTurn::user(case.first_edit_prompt.clone()),
        ConversationTurn::assistant(second.answer),
    ];
    let final_response = solver.solve_with_history(&case.second_edit_prompt, &second_history);
    if final_response.intent != case.expected_intent {
        return Err(format!(
            "{} second edit routed to {}, expected {}; answer: {}",
            case.id, final_response.intent, case.expected_intent, final_response.answer,
        ));
    }

    let missing_answer = case
        .expected_answer_contains
        .iter()
        .filter(|expected| !final_response.answer.contains(expected.as_str()))
        .cloned()
        .collect::<Vec<_>>();
    if !missing_answer.is_empty() {
        return Err(format!(
            "{} final answer missing {missing_answer:?}; answer: {}",
            case.id, final_response.answer,
        ));
    }

    let missing_links = case
        .expected_links_contains
        .iter()
        .filter(|expected| !final_response.links_notation.contains(expected.as_str()))
        .cloned()
        .collect::<Vec<_>>();
    if !missing_links.is_empty() {
        return Err(format!(
            "{} final trace missing {missing_links:?}; links: {}",
            case.id, final_response.links_notation,
        ));
    }

    Ok(())
}

fn benchmark_solver() -> UniversalSolver {
    UniversalSolver::new(SolverConfig {
        offline: true,
        execution_surface: ExecutionSurface::RustLibrary,
        temperature: 0.0,
        ..SolverConfig::default()
    })
}

fn benchmark_solver_lock() -> MutexGuard<'static, ()> {
    static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
    LOCK.get_or_init(|| Mutex::new(()))
        .lock()
        .expect("benchmark solver lock should not be poisoned")
}

fn env_flag_enabled(name: &str) -> bool {
    env::var(name).is_ok_and(|value| matches!(value.as_str(), "1" | "true" | "yes"))
}

fn download_and_validate_dataset(dataset: &ExternalEditDataset) {
    let destination = repo_root().join(&dataset.download_cache);
    if let Some(parent) = destination.parent() {
        fs::create_dir_all(parent).unwrap_or_else(|err| {
            panic!(
                "failed to create dataset cache directory {}: {err}",
                parent.display()
            )
        });
    }

    let status = Command::new("curl")
        .args(["-fL", "--retry", "2", "--retry-delay", "1", "-o"])
        .arg(&destination)
        .arg(&dataset.download_url)
        .status()
        .unwrap_or_else(|err| panic!("failed to start curl for {}: {err}", dataset.id));
    assert!(
        status.success(),
        "curl failed for {} from {} with status {status}",
        dataset.id,
        dataset.download_url,
    );

    let metadata = fs::metadata(&destination).unwrap_or_else(|err| {
        panic!(
            "failed to stat downloaded dataset {} at {}: {err}",
            dataset.id,
            destination.display()
        )
    });
    assert!(
        metadata.len() >= dataset.expected_min_bytes,
        "{} downloaded {} bytes, expected at least {}",
        dataset.id,
        metadata.len(),
        dataset.expected_min_bytes,
    );
    assert_parquet_signature(dataset, &destination);
}

fn assert_parquet_signature(dataset: &ExternalEditDataset, path: &Path) {
    assert_eq!(
        dataset.expected_file_kind, "parquet",
        "{} has unsupported expected_file_kind {}",
        dataset.id, dataset.expected_file_kind,
    );

    let mut file = fs::File::open(path).unwrap_or_else(|err| {
        panic!(
            "failed to open downloaded dataset {} at {}: {err}",
            dataset.id,
            path.display()
        )
    });
    let mut magic = [0_u8; 4];
    file.read_exact(&mut magic).unwrap_or_else(|err| {
        panic!(
            "failed to read parquet signature for {} at {}: {err}",
            dataset.id,
            path.display()
        )
    });
    assert_eq!(
        &magic, b"PAR1",
        "{} should start with the parquet magic bytes",
        dataset.id,
    );
}

fn repo_root() -> &'static Path {
    Path::new(env!("CARGO_MANIFEST_DIR"))
}

fn render_report(report: &CodingModificationReport) -> String {
    let mut out = format!(
        "coding-modification benchmark pass/fail counts: passed={} failed={} total={} minimum_pass_count={}\n",
        report.passed,
        report.failed,
        report.passed + report.failed,
        report.minimum_pass_count,
    );
    for failure in &report.failures {
        out.push_str("FAIL ");
        out.push_str(failure);
        out.push('\n');
    }
    out
}
