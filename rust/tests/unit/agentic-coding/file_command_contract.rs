use super::*;
#[test]
fn shared_operation_arguments_and_refusals_match() {
    let fixture: Value =
        serde_json::from_str(include_str!("../../fixtures/file-command-cases.json"))
            .expect("FileCommandFixtures");
    for case in fixture["cases"].as_array().expect("FileCommandCases") {
        let result = file_command_contract(case["task"].as_str().expect("FileCommandTask"));
        if case["unclaimed"] == true {
            assert!(result.is_none(), "{case}");
            continue;
        }
        let result = result.expect("FileCommandContract");
        assert_eq!(result["operation"], case["operation"], "{case}");
        if !case["reason"].is_null() {
            assert_eq!(result["reason"], case["reason"]);
        }
        if !case["arguments"].is_null() {
            assert_eq!(result["arguments"], case["arguments"]);
        }
        if let Some(path) = case["countPath"].as_str() {
            let arguments = result["arguments"].as_array().expect("FileCountArguments");
            assert_eq!(arguments[0], "node");
            assert_eq!(arguments[1], "-e");
            assert_eq!(
                arguments[2],
                include_str!("../../../embedded/data/meta/file-count-program.cjs")
            );
            assert_eq!(arguments[4], path);
        }
    }
}
