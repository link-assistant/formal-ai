//! Hosted native receipt integration; the shared arithmetic is generated from JavaScript.
use formal_ai::agentic_coding::coding_research::verification_receipt_gap;
use std::fmt::Write as _;

#[test]
fn generated_research_receipt_kernel_through_native_adapter() {
    let vectors: serde_json::Value = serde_json::from_str(include_str!(
        "../../fixtures/coding-research/tap-kernel.json"
    ))
    .expect("valid shared TAP vectors");
    for row in vectors.as_array().expect("TAP cases") {
        let args = row["args"].as_array().expect("typed arguments");
        let fields = ["tests", "pass", "fail", "skipped", "cancelled"];
        let mut output = String::new();
        for (index, field) in fields.iter().enumerate() {
            writeln!(output, "# {field} {}", args[index + 1]).unwrap();
        }
        let receipt = serde_json::json!({
            "output": output,
            "exit_code": 0,
            "complete": args[0],
        });
        let reason = verification_receipt_gap(&receipt.to_string()).unwrap_or_default();
        assert_eq!(
            reason,
            row["expected"].as_str().expect("reason"),
            "{}",
            row["id"]
        );
    }
}
