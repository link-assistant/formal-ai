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

#[test]
fn cargo_source_directories_match_shared_candidate_fixtures() {
    let fixture: Value =
        serde_json::from_str(include_str!("../../fixtures/cargo-source-directories.json"))
            .expect("CargoSourceDirectoryFixtures");
    for entry in fixture["cases"]
        .as_array()
        .expect("CargoSourceDirectoryCases")
    {
        let owner = entry["owner"].as_str().expect("ManifestOwner");
        let source = entry["source"].as_str().expect("ManifestSource");
        let result = super::observed_cargo_source_directories(owner, source);
        assert_eq!(result["schema"], "observed-cargo-source-directories/v1");
        assert_eq!(result["manifest"], owner);
        if let Some(reason) = entry["gap"].as_str() {
            assert_eq!(result["complete"], false, "{}", entry["name"]);
            assert_eq!(result["directories"], serde_json::json!([]));
            assert_eq!(result["facts"], serde_json::json!([]));
            assert_eq!(result["gaps"], serde_json::json!([{"reason":reason}]));
            continue;
        }
        let identity = sha256_hex(source.as_bytes());
        assert_eq!(result["sourceIdentity"], identity);
        assert_eq!(result["complete"], true);
        assert_eq!(result["directories"], entry["directories"]);
        let facts: Vec<_> = result["facts"]
            .as_array()
            .expect("DirectoryFacts")
            .iter()
            .map(|fact| {
                assert_eq!(fact["kind"], "cargo-source-directory-candidate");
                assert_eq!(fact["manifest"], owner);
                assert_eq!(fact["sourceIdentity"], identity);
                let mut value = fact.clone();
                let object = value.as_object_mut().expect("DirectoryFactObject");
                object.remove("kind");
                object.remove("manifest");
                object.remove("sourceIdentity");
                value
            })
            .collect();
        assert_eq!(serde_json::json!(facts), entry["facts"]);
    }
    let oversized = "雪".repeat(87_382);
    let result = super::observed_cargo_source_directories("rust/Cargo.toml", &oversized);
    assert_eq!(result["sourceIdentity"], Value::Null);
    assert_eq!(
        result["gaps"],
        serde_json::json!([{"reason":"CargoSourceBudgetExceeded"}])
    );
}
