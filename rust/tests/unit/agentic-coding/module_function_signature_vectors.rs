//! Shared declarative request vectors; execution is qualified by hosted Rust CI.
use formal_ai::agentic_coding::module_function::module_function_request;

#[test]
fn shared_module_function_signature_vectors() {
    let vectors: serde_json::Value = serde_json::from_str(include_str!(
        "../../fixtures/module-function/signature-vectors.json"
    ))
    .expect("valid shared signature vectors");
    for row in vectors["cases"].as_array().expect("cases array") {
        let request = module_function_request(row["prompt"].as_str().expect("prompt"));
        if row["expected"].is_null() {
            assert!(request.is_none(), "{}", row["id"]);
        } else {
            let request = request.unwrap_or_else(|| panic!("{}", row["id"]));
            assert_eq!(
                serde_json::json!(request.parameters),
                row["expected"],
                "{}",
                row["id"]
            );
            assert_eq!(request.name, "carried");
            assert_eq!(request.module, "carried.mjs");
        }
    }
}
