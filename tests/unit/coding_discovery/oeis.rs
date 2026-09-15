use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};

use formal_ai::coding_function_catalog::oeis::discover_programs;
use formal_ai::coding_task_spec::{ArtifactShape, CodingTaskSpec, Example, Parameter};
use formal_ai::{CachedSourceClient, FetchError, SourceTransport};

#[derive(Clone, Default)]
struct FixtureTransport {
    requests: Arc<AtomicUsize>,
}

impl SourceTransport for FixtureTransport {
    fn get(&self, url: &str) -> Result<Vec<u8>, FetchError> {
        self.requests.fetch_add(1, Ordering::SeqCst);
        let body = if url.contains("search?q=cullen") {
            r#"[{"number":2064,"data":"3,9,25,65,161,385","name":"Cullen numbers: n*2^n + 1."}]"#
        } else if url.contains("A002064") {
            r#"{"number":2064,"data":"3,9,25,65,161,385","name":"Cullen numbers: n*2^n + 1."}"#
        } else if url.contains("search?q=5%20x%20n%20tromino%20tilings") {
            r#"[{"number":999999,"data":"1","name":"Array of tromino tilings.","xref":["A123456 is a fixed-width row."]}]"#
        } else if url.contains("A999999") {
            r#"{"number":999999,"data":"1","name":"Array of tromino tilings.","xref":["A123456 is a fixed-width row."]}"#
        } else if url.contains("A123456") {
            r#"{"number":123456,"data":"1,1,2,5,13,34,89,233","name":"a(n) = 3*a(n-1) - a(n-2), with a(0) = 1, a(1) = 1."}"#
        } else {
            return Err(FetchError::Transport(format!(
                "unexpected fixture URL: {url}"
            )));
        };
        Ok(body.as_bytes().to_vec())
    }
}

fn spec(
    name: &str,
    parameter: &str,
    requirement: &str,
    examples: &[(&str, &str)],
) -> CodingTaskSpec {
    CodingTaskSpec {
        language: "python".to_owned(),
        artifact_shape: ArtifactShape::Function,
        name: name.to_owned(),
        parameters: vec![Parameter {
            name: parameter.to_owned(),
            annotation: Some("int".to_owned()),
        }],
        return_annotation: None,
        imports: Vec::new(),
        requirement_sentences: vec![requirement.to_owned()],
        examples: examples
            .iter()
            .map(|(argument, expected)| Example {
                arguments: vec![(*argument).to_owned()],
                expected: (*expected).to_owned(),
            })
            .collect(),
        expected_stdout: None,
        prose_language: "en".to_owned(),
    }
}

fn temp_cache(label: &str) -> std::path::PathBuf {
    std::env::temp_dir().join(format!("formal-ai-oeis-{label}-{}", std::process::id()))
}

#[test]
fn named_formula_membership_is_discovered_without_knowing_the_callable() {
    let cache = temp_cache("formula");
    let _ = std::fs::remove_dir_all(&cache);
    let client = CachedSourceClient::new(&cache, FixtureTransport::default())
        .with_online(true)
        .with_clock(|| 1_789_344_000);
    let discovery = discover_programs(
        &client,
        &spec(
            "is_cullen",
            "candidate_value",
            "Check whether the integer belongs to the named family.",
            &[("9", "True"), ("10", "False"), ("161", "True")],
        ),
    );
    assert!(discovery.diagnostics.is_empty(), "{discovery:?}");
    assert_eq!(discovery.programs.len(), 2);
    let program = &discovery.programs[0];
    assert!(program.source.contains("source_index*2**source_index+1"));
    assert!(program.source.contains("candidate_value"));
    assert_eq!(program.license, "CC-BY-SA-4.0");
    assert!(program.source_url.contains("A002064?fmt=json"));
    std::fs::remove_dir_all(cache).expect("remove formula cache");
}

#[test]
fn referenced_recurrence_is_formalized_and_replays_after_forgetting_transport() {
    let cache = temp_cache("recurrence");
    let _ = std::fs::remove_dir_all(&cache);
    let transport = FixtureTransport::default();
    let requests = Arc::clone(&transport.requests);
    let task = spec(
        "count_strip_coverings",
        "width",
        "Count 2 x 1 tromino coverings of a given 5 x n board.",
        &[("2", "2"), ("4", "5"), ("8", "34")],
    );
    let online = CachedSourceClient::new(&cache, transport.clone())
        .with_online(true)
        .with_clock(|| 1_789_344_000);
    let first = discover_programs(&online, &task);
    assert!(first.diagnostics.is_empty(), "{first:?}");
    let recurrence = first
        .programs
        .iter()
        .find(|program| program.composition.contains("floor(n/2)+1"))
        .expect("source terms should establish the index transform");
    assert!(recurrence.source.contains("values.append((3 * values[-1])"));
    assert!(recurrence.source.contains("width // 2 + (1)"));

    let live_requests = requests.load(Ordering::SeqCst);
    let offline = CachedSourceClient::new(&cache, transport);
    let replay = discover_programs(&offline, &task);
    assert_eq!(replay, first);
    assert_eq!(requests.load(Ordering::SeqCst), live_requests);
    std::fs::remove_dir_all(cache).expect("remove recurrence cache");
}
