//! PR #1188 dogfooding, T1: `Add a function multiply(a, b) to math.mjs that
//! returns a times b, add a test for it to math.test.mjs, and run node --test`.
//! The function is lowered from the same IR as the Python `def`, through the
//! seeded `javascript` realization of `integer_multiply` and the seeded
//! `javascript_ir_function` template; a node with no JavaScript realization is
//! a named gap. The agentic arm is JavaScript-first
//! (`js/agentic/module_function.mjs`, pinned in
//! `rust/tests/web/pull-request-1188-dogfood.test.mjs`).

use formal_ai::fragment_catalog::FragmentCatalog;
use formal_ai::ir_lowering::lowering_for;
use formal_ai::program_ir::{IrNode, IrType, ProgramIr, ReuseMode};

fn multiply(body: IrNode) -> ProgramIr {
    ProgramIr {
        name: "multiply".to_owned(),
        parameters: vec![
            ("a".to_owned(), IrType::Integer),
            ("b".to_owned(), IrType::Integer),
        ],
        result: IrType::Integer,
        body,
        fragments: vec!["integer_multiply".to_owned()],
        source_urls: Vec::new(),
        source_licenses: Vec::new(),
        reuse: ReuseMode::Verbatim,
    }
}

fn parameter(name: &str) -> IrNode {
    IrNode::Parameter {
        name: name.to_owned(),
        ty: IrType::Integer,
    }
}

#[test]
fn the_multiplication_ir_lowers_to_an_es_module_function() {
    let catalog = FragmentCatalog::bootstrap();
    let ir = multiply(IrNode::Apply {
        fragment: "integer_multiply".to_owned(),
        arguments: vec![parameter("a"), parameter("b")],
    });
    let javascript = lowering_for("javascript")
        .expect("a javascript lowering is registered")
        .lower(&ir, &catalog)
        .expect("integer_multiply has a seeded javascript realization");
    assert_eq!(
        javascript,
        "export function multiply(a, b) {\n  return a * b;\n}\n"
    );
}

#[test]
fn a_node_with_no_javascript_realization_is_a_named_gap() {
    let catalog = FragmentCatalog::bootstrap();
    let ir = multiply(IrNode::Apply {
        fragment: "drop_whitespace".to_owned(),
        arguments: vec![parameter("a")],
    });
    let gap = lowering_for("javascript")
        .expect("a javascript lowering is registered")
        .lower(&ir, &catalog)
        .expect_err("drop_whitespace is realized in Python only");
    assert_eq!(gap.language, "javascript");
    assert_eq!(gap.node, "Apply");
    assert_eq!(
        gap.detail,
        "`drop_whitespace` has no seeded javascript realization"
    );
}
