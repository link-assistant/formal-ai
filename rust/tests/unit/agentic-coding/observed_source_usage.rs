//! Native grammar qualification belongs exclusively to CI.
use super::observed_source_usage;
use serde_json::Value;
#[test]
fn actual_native_grammars_obey_shared_source_usage_witness_vectors() {
    let vectors: Value = serde_json::from_str(include_str!(
        "../../../../data/benchmarks/fixtures/source-usage/vectors.json"
    ))
    .unwrap();
    for row in vectors["cases"].as_array().unwrap() {
        let source = row["source"].as_str().unwrap();
        let name = row["name"].as_str().unwrap();
        let result = observed_source_usage(
            row["owner"].as_str().unwrap(),
            source,
            row["language"].as_str().unwrap(),
        )
        .unwrap();
        let imports = result["imports"].as_array().unwrap();
        let calls = result["calls"].as_array().unwrap();
        assert_eq!(
            imports
                .iter()
                .map(|item| item["module"].clone())
                .collect::<Vec<_>>(),
            row["modules"].as_array().unwrap().clone(),
            "{name}"
        );
        let bindings = imports
            .iter()
            .flat_map(|item| {
                item["bindings"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .map(|binding| binding["local"].clone())
            })
            .collect::<Vec<_>>();
        assert_eq!(
            bindings,
            row["bindings"].as_array().unwrap().clone(),
            "{name}"
        );
        assert_eq!(
            calls
                .iter()
                .map(|item| item["binding"].clone())
                .collect::<Vec<_>>(),
            row["calls"].as_array().unwrap().clone(),
            "{name}"
        );
        let options = calls
            .iter()
            .flat_map(|item| {
                item["options"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .map(|option| option["name"].clone())
            })
            .collect::<Vec<_>>();
        assert_eq!(
            options,
            row["options"].as_array().unwrap().clone(),
            "{name}"
        );
        for kind in row["gapKinds"].as_array().unwrap() {
            assert!(
                result["gaps"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .any(|gap| &gap["kind"] == kind),
                "{name}: {kind}"
            );
        }
        for item in imports
            .iter()
            .chain(calls)
            .chain(result["references"].as_array().unwrap())
            .chain(result["gaps"].as_array().unwrap())
        {
            let Some(span) = item["span"].as_array() else {
                continue;
            };
            let start = usize::try_from(span[0].as_u64().unwrap()).unwrap();
            let end = usize::try_from(span[1].as_u64().unwrap()).unwrap();
            let snippet = item["source"].as_str().unwrap();
            assert!(source[start..end].starts_with(snippet));
            assert!(snippet.len() <= 512);
        }
    }
}
#[test]
fn original_rust_consumers_emit_qualified_syntax_references_without_external_root_claims() {
    for (owner, source, root) in [
        (
            "rust/src/web_search_core.rs",
            include_str!("../../../src/web_search_core.rs"),
            "web_search",
        ),
        (
            "rust/src/orchestration/runner.rs",
            include_str!("../../../src/orchestration/runner.rs"),
            "command_stream",
        ),
    ] {
        let result = observed_source_usage(owner, source, "Rust").unwrap();
        assert!(
            result["references"]
                .as_array()
                .unwrap()
                .iter()
                .any(|item| item["root"] == root)
        );
        assert_eq!(
            result["scope"],
            "static-source-observations-only; no execution or external API semantics"
        );
    }
}
