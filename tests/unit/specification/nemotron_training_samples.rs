//! Issue #482 Nemotron training-data sample suite.
//!
//! The issue asks for tests built from NVIDIA Nemotron 3 Ultra training data
//! without downloading the full dataset. This harness treats the sampled rows as
//! a deterministic ingestion benchmark: the system must preserve provenance,
//! license, row offset, digest, excerpt preview, and no-full-download evidence
//! for ten randomly selected upstream rows.

use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::Path;

use lino_objects_codec::format::parse_indented;
use serde_json::Value;

const FIXTURE: &str = "data/benchmarks/nemotron-training-samples.lino";
const RAW_SAMPLES: &str = "docs/case-studies/issue-482/raw-data/nemotron-random-samples.json";
const PERMISSIVE_LICENSE: &str = "CC-BY-4.0";
const EXPECTED_DATASET: &str = "nvidia/Nemotron-Pretraining-Legal-v1";

#[derive(Debug)]
struct Record {
    kind: String,
    id: String,
    fields: Vec<(String, String)>,
}

#[derive(Debug)]
struct Source {
    id: String,
    license: String,
    dataset: String,
    revision: String,
    download_mode: String,
}

#[derive(Debug)]
struct Case {
    id: String,
    source: String,
    dataset: String,
    config: String,
    split: String,
    row_index: usize,
    num_rows_total: usize,
    row_uuid: String,
    upstream_license: String,
    source_ref: String,
    provenance_url: String,
    download_mode: String,
    text_length: usize,
    text_sha256: String,
    excerpt_sha256: String,
    test_shape: String,
    text_excerpt_preview: String,
    variant: String,
}

#[derive(Debug)]
struct Suite {
    minimum_pass_count: usize,
    sample_count: usize,
    seed: String,
    sampler: String,
    raw_sample_file: String,
    download_policy: String,
    license_policy: String,
    sources: BTreeMap<String, Source>,
    cases: Vec<Case>,
}

#[test]
fn issue_482_nemotron_training_sample_fixture_is_well_formed() {
    let suite = load_suite();

    assert_eq!(suite.seed, "issue-482");
    assert_eq!(suite.sample_count, 10);
    assert_eq!(suite.cases.len(), suite.sample_count);
    assert_eq!(suite.minimum_pass_count, 10);
    assert_eq!(suite.raw_sample_file, RAW_SAMPLES);
    assert!(suite.sampler.contains("sample-nemotron-training-data.py"));
    assert!(suite.sampler.contains("--count 10"));
    assert!(suite.sampler.contains("--seed issue-482"));
    assert!(suite.download_policy.contains("length=1"));
    assert!(suite.download_policy.contains("no parquet"));
    assert!(suite.license_policy.contains(PERMISSIVE_LICENSE));

    let source = suite
        .sources
        .get("nemotron_legal_v1")
        .expect("Nemotron legal source should be recorded");
    assert_eq!(source.license, PERMISSIVE_LICENSE);
    assert_eq!(source.dataset, EXPECTED_DATASET);
    assert_eq!(source.revision, "3d91d58a5c0c46fe9944300ec46719f97a385b13");
    assert_eq!(source.download_mode, "sample rows only");

    assert_all_cases_are_policy_compliant(&suite);
    assert_sample_diversity(&suite);
}

#[test]
fn issue_482_nemotron_training_samples_match_sampler_output() {
    let suite = load_suite();
    let raw = load_raw_samples();
    let raw_samples = raw
        .get("samples")
        .and_then(Value::as_array)
        .expect("raw sample artifact should contain samples");
    assert_eq!(raw_samples.len(), suite.cases.len());
    assert_eq!(raw.get("seed").and_then(Value::as_str), Some("issue-482"));
    assert_eq!(raw.get("count").and_then(Value::as_u64), Some(10));

    let raw_by_id = raw_samples
        .iter()
        .map(|sample| {
            let id = sample
                .get("sample_id")
                .and_then(Value::as_str)
                .expect("raw sample should have an id");
            (id.to_owned(), sample)
        })
        .collect::<BTreeMap<_, _>>();

    for case in &suite.cases {
        let sample = raw_by_id
            .get(&case.id)
            .unwrap_or_else(|| panic!("raw sample artifact missing {}", case.id));
        assert_eq!(
            sample.get("dataset").and_then(Value::as_str),
            Some(case.dataset.as_str())
        );
        assert_eq!(
            sample.get("config").and_then(Value::as_str),
            Some(case.config.as_str())
        );
        assert_eq!(
            sample.get("split").and_then(Value::as_str),
            Some(case.split.as_str())
        );
        assert_eq!(
            sample.get("row_index").and_then(Value::as_u64),
            Some(case.row_index as u64)
        );
        assert_eq!(
            sample.get("num_rows_total").and_then(Value::as_u64),
            Some(case.num_rows_total as u64)
        );
        assert_eq!(
            sample.get("row_uuid").and_then(Value::as_str),
            Some(case.row_uuid.as_str())
        );
        assert_eq!(
            sample.get("license").and_then(Value::as_str),
            Some(case.upstream_license.as_str())
        );
        assert_eq!(
            sample.get("source_ref").and_then(Value::as_str),
            Some(case.source_ref.as_str())
        );
        assert_eq!(
            sample.get("provenance_url").and_then(Value::as_str),
            Some(case.provenance_url.as_str())
        );
        assert_eq!(
            sample.get("download_mode").and_then(Value::as_str),
            Some(case.download_mode.as_str())
        );
        assert_eq!(
            sample.get("text_length").and_then(Value::as_u64),
            Some(case.text_length as u64)
        );
        assert_eq!(
            sample.get("text_sha256").and_then(Value::as_str),
            Some(case.text_sha256.as_str())
        );
        assert_eq!(
            sample.get("excerpt_sha256").and_then(Value::as_str),
            Some(case.excerpt_sha256.as_str())
        );
        assert_eq!(
            sample.get("test_shape").and_then(Value::as_str),
            Some(case.test_shape.as_str())
        );
    }
}

#[test]
fn issue_482_nemotron_training_ingestion_ratchet_passes_all_samples() {
    let suite = load_suite();
    let passed = suite
        .cases
        .iter()
        .filter(|case| sample_ingestion_check(case))
        .count();

    assert_eq!(
        passed,
        suite.cases.len(),
        "every sampled row should pass the metadata/excerpt/provenance ingestion check"
    );
    assert!(
        passed >= suite.minimum_pass_count,
        "Nemotron sample ingestion pass count {passed} fell below floor {}",
        suite.minimum_pass_count
    );
}

fn assert_all_cases_are_policy_compliant(suite: &Suite) {
    let source_ids = suite.sources.keys().collect::<BTreeSet<_>>();
    let mut sample_ids = BTreeSet::new();
    let mut sampled_rows = BTreeSet::new();
    for case in &suite.cases {
        assert!(
            sample_ids.insert(case.id.as_str()),
            "duplicate sample id {}",
            case.id
        );
        assert!(
            source_ids.contains(&case.source),
            "missing source {}",
            case.source
        );
        assert_eq!(case.dataset, EXPECTED_DATASET);
        assert_eq!(case.split, "train");
        assert!(case.row_index < case.num_rows_total);
        assert_eq!(case.upstream_license, PERMISSIVE_LICENSE);
        assert_eq!(
            suite.sources[&case.source].revision, case.source_ref,
            "{} should carry the source revision into each case",
            case.id
        );
        assert!(
            is_hex_sha256(&case.text_sha256),
            "{} text digest is invalid",
            case.id
        );
        assert!(
            is_hex_sha256(&case.excerpt_sha256),
            "{} excerpt digest is invalid",
            case.id
        );
        assert!(case.text_length > case.text_excerpt_preview.len());
        assert!(case.text_excerpt_preview.len() <= 420);
        assert_eq!(case.variant, "upstream_random_row");
        assert_eq!(
            case.download_mode,
            "datasets-server rows endpoint, length=1"
        );
        assert!(case
            .provenance_url
            .starts_with("https://datasets-server.huggingface.co/rows?"));
        assert!(case.provenance_url.contains("length=1"));
        assert!(!case.provenance_url.contains(".parquet"));
        assert!(!case.provenance_url.contains("/resolve/"));
        assert!(
            sampled_rows.insert((case.config.as_str(), case.row_index)),
            "duplicate sampled row {}:{}",
            case.config,
            case.row_index
        );
    }
}

fn assert_sample_diversity(suite: &Suite) {
    let configs = suite
        .cases
        .iter()
        .map(|case| case.config.as_str())
        .collect::<BTreeSet<_>>();
    let shapes = suite
        .cases
        .iter()
        .map(|case| case.test_shape.as_str())
        .collect::<BTreeSet<_>>();

    assert!(
        configs.len() >= 8,
        "the ten random samples should cover multiple legal configs; got {configs:?}",
    );
    assert!(
        shapes.len() >= 3,
        "the random samples should exercise several text shapes; got {shapes:?}",
    );
}

fn sample_ingestion_check(case: &Case) -> bool {
    case.dataset == EXPECTED_DATASET
        && case.split == "train"
        && case.upstream_license == PERMISSIVE_LICENSE
        && case.download_mode == "datasets-server rows endpoint, length=1"
        && case.provenance_url.contains("length=1")
        && case.text_length > 0
        && !case.text_excerpt_preview.is_empty()
        && is_hex_sha256(&case.text_sha256)
        && is_hex_sha256(&case.excerpt_sha256)
}

fn is_hex_sha256(value: &str) -> bool {
    value.len() == 64 && value.bytes().all(|byte| byte.is_ascii_hexdigit())
}

fn load_suite() -> Suite {
    let text = fs::read_to_string(repo_root().join(FIXTURE)).expect("Nemotron fixture");
    validate_lino_syntax(&text);
    parse_suite(&text)
}

fn load_raw_samples() -> Value {
    let text = fs::read_to_string(repo_root().join(RAW_SAMPLES)).expect("Nemotron raw samples");
    serde_json::from_str(&text).expect("raw sample artifact should be valid JSON")
}

fn validate_lino_syntax(text: &str) {
    for record in split_records(text) {
        parse_indented(&record).expect("Nemotron fixture record should be valid Links Notation");
    }
}

fn parse_suite(text: &str) -> Suite {
    let mut minimum_pass_count = 0;
    let mut sample_count = 0;
    let mut seed = String::new();
    let mut sampler = String::new();
    let mut raw_sample_file = String::new();
    let mut download_policy = String::new();
    let mut license_policy = String::new();
    let mut sources = BTreeMap::new();
    let mut cases = Vec::new();

    for record in parse_records(text) {
        match record.kind.as_str() {
            "benchmark_suite" => {
                minimum_pass_count = parse_usize(&record, "minimum_pass_count");
                sample_count = parse_usize(&record, "sample_count");
                seed = field_value(&record, "seed");
                sampler = field_value(&record, "sampler");
                raw_sample_file = field_value(&record, "raw_sample_file");
                download_policy = field_value(&record, "download_policy");
                license_policy = field_value(&record, "license_policy");
            }
            "benchmark_source" => {
                let source = Source {
                    id: record.id.clone(),
                    license: field_value(&record, "license"),
                    dataset: field_value(&record, "dataset"),
                    revision: field_value(&record, "source_ref"),
                    download_mode: field_value(&record, "download_mode"),
                };
                sources.insert(source.id.clone(), source);
            }
            "benchmark_case" => cases.push(Case {
                id: record.id.clone(),
                source: field_value(&record, "source"),
                dataset: field_value(&record, "dataset"),
                config: field_value(&record, "config"),
                split: field_value(&record, "split"),
                row_index: parse_usize(&record, "row_index"),
                num_rows_total: parse_usize(&record, "num_rows_total"),
                row_uuid: field_value(&record, "row_uuid"),
                upstream_license: field_value(&record, "upstream_license"),
                source_ref: field_value(&record, "source_ref"),
                provenance_url: field_value(&record, "provenance_url"),
                download_mode: field_value(&record, "download_mode"),
                text_length: parse_usize(&record, "text_length"),
                text_sha256: field_value(&record, "text_sha256"),
                excerpt_sha256: field_value(&record, "excerpt_sha256"),
                test_shape: field_value(&record, "test_shape"),
                text_excerpt_preview: field_value(&record, "text_excerpt_preview"),
                variant: field_value(&record, "variant"),
            }),
            _ => {}
        }
    }

    Suite {
        minimum_pass_count,
        sample_count,
        seed,
        sampler,
        raw_sample_file,
        download_policy,
        license_policy,
        sources,
        cases,
    }
}

fn parse_records(text: &str) -> Vec<Record> {
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
        current.push(line);
    }
    if !current.is_empty() {
        records.push(current.join("\n"));
    }
    records
}

fn parse_record(text: &str) -> Record {
    let mut lines = text.lines();
    let header = lines
        .next()
        .expect("record should have an id")
        .trim()
        .to_owned();
    let mut kind = String::new();
    let mut fields = Vec::new();
    for line in lines {
        let trimmed = line.trim();
        if let Some((name, raw_value)) = trimmed.split_once(' ') {
            let value = unquote(raw_value.trim());
            if name == "record_type" {
                kind = value;
            } else {
                fields.push((name.to_owned(), value));
            }
        }
    }
    let id = fields
        .iter()
        .find_map(|(key, value)| (key == "id").then(|| value.clone()))
        .unwrap_or_else(|| panic!("record `{header}` should have an id field"));
    Record { kind, id, fields }
}

fn field_value(record: &Record, name: &str) -> String {
    record
        .fields
        .iter()
        .find_map(|(key, value)| (key == name).then(|| value.clone()))
        .unwrap_or_default()
}

fn parse_usize(record: &Record, name: &str) -> usize {
    let raw = field_value(record, name);
    raw.parse::<usize>()
        .unwrap_or_else(|error| panic!("{} has invalid {name} `{raw}`: {error}", record.id))
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

fn repo_root() -> &'static Path {
    Path::new(env!("CARGO_MANIFEST_DIR"))
}
