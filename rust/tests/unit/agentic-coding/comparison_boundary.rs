//! CI-only native qualification of the same predeclared JS grader vectors.
//! Intended owner: rust/tests/unit/agentic-coding/comparison_boundary.rs.
use formal_ai::agentic_coding::comparison_boundary::{
    BoundaryDefinition, native_comparison_boundary_candidate,
    native_comparison_boundary_regression, select_comparison_boundary_definition,
};
use formal_ai::comparison_boundary_generated::{
    comparison_boundary_role_kernel, comparison_boundary_witness_kernel,
};
use formal_ai::readable_source::{owned_source_links, source_to_links};
use serde_json::Value;

fn definition() -> BoundaryDefinition {
    BoundaryDefinition {
        declaration: "inside".to_owned(),
        bound: "upper".to_owned(),
        inclusive: true,
    }
}
#[test]
fn generated_native_kernel_obeys_shared_independent_vectors() {
    let vectors: Value = serde_json::from_str(include_str!(
        "../../fixtures/comparison-boundary/vectors.json"
    ))
    .unwrap();
    for row in vectors["roles"].as_array().unwrap() {
        let operands = row["operands"].as_array().unwrap();
        let operators = row["operators"].as_array().unwrap();
        let result = comparison_boundary_role_kernel(
            operands[0].as_str().unwrap().to_owned(),
            operands[1].as_str().unwrap().to_owned(),
            operands[2].as_str().unwrap().to_owned(),
            operands[3].as_str().unwrap().to_owned(),
            operators[0].as_str().unwrap().to_owned(),
            operators[1].as_str().unwrap().to_owned(),
            row["bound"].as_str().unwrap().to_owned(),
            row["inclusive"].as_bool().unwrap(),
        );
        assert_eq!(serde_json::to_value(result).unwrap(), row["expected"]);
    }
    for row in vectors["witnesses"].as_array().unwrap() {
        let args = row["args"].as_array().unwrap();
        assert_eq!(
            comparison_boundary_witness_kernel(
                args[0].as_f64().unwrap(),
                args[1].as_f64().unwrap(),
                args[2].as_f64().unwrap(),
                args[3].as_bool().unwrap(),
                args[4].as_bool().unwrap()
            ),
            row["expected"].as_bool().unwrap()
        );
    }
}
#[test]
fn native_real_source_model_preserves_comments_and_other_functions() {
    for (expression, expected) in [
        (
            "reading >= floor && reading < ceiling",
            "reading >= floor && reading <= ceiling",
        ),
        (
            "floor <= reading && ceiling > reading",
            "floor <= reading && ceiling >= reading",
        ),
        (
            "ceiling > reading && reading >= floor",
            "ceiling >= reading && reading >= floor",
        ),
    ] {
        let source = format!(
            "// 雪: preserve this comment and CRLF.\r\nexport function inside(reading, floor, ceiling) {{\r\n  return {expression};\r\n}}\r\nexport function untouched(input) {{ return input + 8; }}"
        );
        let result =
            native_comparison_boundary_candidate(&source, "src/unseen.mjs", &definition()).unwrap();
        let after = source.replace(expression, expected);
        assert_eq!(result["source"], after);
        assert_eq!(result["verification"]["exactInverse"], true);
        assert_eq!(result["verification"]["independentGrammarReparse"], true);
        let again =
            native_comparison_boundary_candidate(&after, "src/unseen.mjs", &definition()).unwrap();
        assert_eq!(again["changed"], false);
    }
}
#[test]
fn native_unique_selector_and_regression_use_actual_owned_source() {
    let source = "export function inside(reading,floor,ceiling) { return reading >= floor && reading < ceiling; }\nexport function unrelated(value) { return value + 1; }\n";
    let model = owned_source_links(
        "src/unseen.mjs",
        &source_to_links(source, "JavaScript").unwrap(),
    )
    .unwrap();
    let selected = select_comparison_boundary_definition(&model, "upper", true).unwrap();
    assert_eq!(selected.declaration, "inside");
    let regression = native_comparison_boundary_regression(
        source,
        "src/unseen.mjs",
        &selected,
        "../src/unseen.mjs",
    )
    .unwrap();
    assert!(
        regression["source"]
            .as_str()
            .unwrap()
            .contains("assert.equal(subject(2, 0, 2), true);")
    );
    assert_eq!(regression["contract"]["independentAcceptance"], false);
    let ambiguous =
        format!("{source}export function other(x,lo,hi) {{ return lo <= x && hi > x; }}\n");
    let model = owned_source_links(
        "src/unseen.mjs",
        &source_to_links(&ambiguous, "JavaScript").unwrap(),
    )
    .unwrap();
    assert!(select_comparison_boundary_definition(&model, "upper", true).is_err());
}
#[test]
fn native_structural_adapter_refuses_effects_and_ambiguous_shapes() {
    for source in [
        "export function inside(x,lo,hi) { sideEffect(); return x >= lo && x < hi; }",
        "export function inside(x,lo,hi) { return x >= lo || x < hi; }",
        "export function inside(x,lo,hi) { return x >= lo && x < hi + 1; }",
        "export function inside(x,lo,hi) { return x >= lo && x < hi && x !== 3; }",
        "export function inside(x,lo,hi) { return x >= lo && lo < hi; }",
        "export function inside(x,lo,hi) { return x >= lo && x < lo; }",
        "export async function inside(x,lo,hi) { return x >= lo && x < hi; }",
        "export function inside(x=1,lo,hi) { return x >= lo && x < hi; }",
    ] {
        assert!(
            native_comparison_boundary_candidate(source, "src/unseen.mjs", &definition()).is_err(),
            "{source}"
        );
    }
}
