use super::*;
#[test]
fn shared_import_requests_preserve_concrete_source_and_repeat_without_changes() {
    let fixture: Value =
        serde_json::from_str(include_str!("../../fixtures/source-import-edit.json"))
            .expect("ImportFixtures");
    for case in fixture["cases"].as_array().expect("ImportCases") {
        let text = |key: &str| case[key].as_str().expect("ImportCaseText");
        let contract = source_import_contract(text("task")).expect("OwnedImportRequest");
        assert_eq!(contract["owner"], case["owner"]);
        assert_eq!(contract["binding"], case["binding"]);
        assert_eq!(contract["module"], case["module"]);
        let result = derive_named_import_edit(
            text("owner"),
            text("source"),
            text("module"),
            text("binding"),
        );
        if let Some(gap) = case["gap"].as_str() {
            assert_eq!(result.expect_err("ExpectedImportGap"), gap);
            continue;
        }
        let result = result.expect("ImportDerived");
        let candidate = result["candidate"]["source"]
            .as_str()
            .expect("ImportCandidate");
        let start = result["binding"]["insertionSpan"][0]
            .as_u64()
            .expect("ImportStart") as usize;
        let end = result["binding"]["insertionSpan"][1]
            .as_u64()
            .expect("ImportEnd") as usize;
        assert_eq!(
            format!("{}{}", &candidate[..start], &candidate[end..]),
            text("source")
        );
        assert_eq!(result["candidate"]["changed"], case["noop"] != true);
        let again =
            derive_named_import_edit(text("owner"), candidate, text("module"), text("binding"))
                .expect("ImportRepeat");
        assert_eq!(again["candidate"]["changed"], false);
        assert_eq!(again["candidate"]["source"], candidate);
    }
}
