//! Issue #1138, plan 02 L2–L3 — the language-neutral intermediate representation.
//!
//! The IR is associative-stack data, not a Rust-only structure: it projects to
//! Links Notation and reads back from it, its type check names the node it could
//! not type rather than failing opaquely, and its cost is a total order so the
//! shorter derivation always wins.

use formal_ai::fragment_catalog::FragmentCatalog;
use formal_ai::program_ir::{IrNode, IrType, ProgramIr, ReuseMode};

fn parameter(name: &str, ty: IrType) -> IrNode {
    IrNode::Parameter {
        name: name.to_owned(),
        ty,
    }
}

/// `run_length(text)` as the IR sees it: fold the characters into pairs.
fn fold_ir() -> ProgramIr {
    ProgramIr {
        name: "run_length".to_owned(),
        parameters: vec![("text".to_owned(), IrType::Text)],
        result: IrType::Sequence(Box::new(IrType::Pair(
            Box::new(IrType::Text),
            Box::new(IrType::Integer),
        ))),
        body: IrNode::Fold {
            item: "character".to_owned(),
            accumulator: "runs".to_owned(),
            items: Box::new(parameter("text", IrType::Text)),
            initial: Box::new(IrNode::Literal {
                text: "[]".to_owned(),
                ty: IrType::Sequence(Box::new(IrType::Text)),
            }),
            body: Box::new(IrNode::Apply {
                fragment: "extend_run".to_owned(),
                arguments: vec![
                    parameter("runs", IrType::Sequence(Box::new(IrType::Text))),
                    parameter("character", IrType::Text),
                ],
            }),
        },
        fragments: vec!["extend_run".to_owned()],
        source_urls: vec!["https://docs.python.org/3.12/library/itertools.html".to_owned()],
        source_licenses: vec!["PSF-2.0".to_owned()],
        reuse: ReuseMode::Verbatim,
    }
}

/// The same result reached the long way round, so the two are comparable.
fn repeated_ir() -> ProgramIr {
    let inner = fold_ir();
    ProgramIr {
        body: IrNode::Bind {
            name: "runs".to_owned(),
            value: Box::new(inner.body.clone()),
            body: Box::new(IrNode::Return {
                value: Box::new(parameter(
                    "runs",
                    IrType::Sequence(Box::new(IrType::Text)),
                )),
            }),
        },
        ..inner
    }
}

#[test]
fn ir_round_trips_through_links_notation() {
    let ir = fold_ir();
    let projection = ir.to_links_notation();

    assert!(
        projection.starts_with("program_ir run_length"),
        "the projection names the program it belongs to: {projection}"
    );
    assert_eq!(
        ProgramIr::from_links_notation(&projection).as_ref(),
        Some(&ir),
        "an IR projected into the store must read back as the same IR"
    );
    assert_eq!(
        ProgramIr::from_links_notation(&projection).map(|parsed| parsed.content_id()),
        Some(ir.content_id()),
        "the content id is taken over the canonical projection"
    );
}

#[test]
fn ill_typed_composition_is_rejected_by_name() {
    let catalog = FragmentCatalog::bootstrap();
    let mut ir = fold_ir();
    ir.body = IrNode::Apply {
        fragment: "extend_run".to_owned(),
        arguments: vec![IrNode::Literal {
            text: "3".to_owned(),
            ty: IrType::Integer,
        }],
    };

    let error = ir
        .type_check(&catalog)
        .expect_err("a fragment applied to the wrong arity and type cannot be typed");
    assert!(
        error.contains("extend_run"),
        "the rejection names the offending node, not just the program: {error}"
    );
    assert!(
        fold_ir().type_check(&catalog).is_ok(),
        "a well-typed composition passes the same check"
    );
}

#[test]
fn action_cost_orders_the_shorter_derivation_first() {
    let short = fold_ir();
    let long = repeated_ir();

    assert!(
        short.action_cost() < long.action_cost(),
        "the shorter derivation costs less: {} vs {}",
        short.action_cost(),
        long.action_cost()
    );
    assert_eq!(
        short.action_cost(),
        fold_ir().action_cost(),
        "cost is a function of the IR alone, so the order is total and stable"
    );
}
