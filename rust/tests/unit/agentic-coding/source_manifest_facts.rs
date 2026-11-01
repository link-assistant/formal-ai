use super::observed_manifest_facts;
use crate::source_fetch::sha256_hex;
use serde_json::Value;

#[test]
fn observed_manifest_facts_match_shared_fixtures() {
    let fixtures: Value =
        serde_json::from_str(include_str!("../../fixtures/source-manifest-facts.json"))
            .expect("SharedManifestFactsFixtureJson");
    for case in fixtures["cases"]
        .as_array()
        .expect("SharedManifestFactsCases")
    {
        let name = case["name"].as_str().expect("ManifestFactsCaseName");
        let owner = case["owner"].as_str().expect("ManifestFactsCaseOwner");
        let source = case["source"].as_str().expect("ManifestFactsCaseSource");
        let actual = observed_manifest_facts(owner, source);
        if case["rejected"] == true {
            assert!(actual.is_err(), "{name}: unsupported source was accepted");
            continue;
        }
        let actual = actual.unwrap_or_else(|error| panic!("{name}: {error}"));
        let expected_identity = sha256_hex(source.as_bytes());
        let stripped: Vec<Value> = actual
            .into_iter()
            .map(|mut row| {
                assert_eq!(row["manifest"], owner, "{name}: owner binding");
                assert_eq!(
                    row["sourceIdentity"], expected_identity,
                    "{name}: source binding"
                );
                let object = row.as_object_mut().expect("ManifestFactObject");
                object.remove("manifest");
                object.remove("sourceIdentity");
                row
            })
            .collect();
        if let Some(expected) = case.get("expected") {
            assert_eq!(
                Value::Array(stripped.clone()),
                *expected,
                "{name}: exact facts"
            );
        }
        if let Some(expected) = case.get("expectedSubset").and_then(Value::as_array) {
            for row in expected {
                assert!(
                    stripped.contains(row),
                    "{name}: missing exact declared fact {row}"
                );
            }
        }
    }
}
