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
                ty: run_sequence_type(),
            }),
            body: Box::new(IrNode::Apply {
                fragment: "extend_run".to_owned(),
                arguments: vec![
                    parameter("runs", run_sequence_type()),
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
                value: Box::new(parameter("runs", run_sequence_type())),
            }),
        },
        ..inner
    }
}

fn run_sequence_type() -> IrType {
    IrType::Sequence(Box::new(IrType::Pair(
        Box::new(IrType::Text),
        Box::new(IrType::Integer),
    )))
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

#[test]
fn text_satisfies_a_sequence_of_text_fragment_input() {
    let catalog = FragmentCatalog::bootstrap();
    let ir = ProgramIr {
        name: "contains_character".to_owned(),
        parameters: vec![("text".to_owned(), IrType::Text)],
        result: IrType::Boolean,
        body: IrNode::Apply {
            fragment: "membership".to_owned(),
            arguments: vec![
                parameter("character", IrType::Text),
                parameter("text", IrType::Text),
            ],
        },
        fragments: vec!["membership".to_owned()],
        source_urls: Vec::new(),
        source_licenses: Vec::new(),
        reuse: ReuseMode::ShapeOnly,
    };

    assert!(
        ir.type_check(&catalog).is_ok(),
        "text is structurally an iterable of text characters"
    );
}

#[test]
fn recursive_reduction_round_trips_as_data_and_type_checks() {
    let integer = |name: &str| IrNode::Parameter {
        name: name.to_owned(),
        ty: IrType::Integer,
    };
    let zero = || IrNode::Literal {
        text: "0".to_owned(),
        ty: IrType::Integer,
    };
    let apply = |fragment: &str, arguments: Vec<IrNode>| IrNode::Apply {
        fragment: fragment.to_owned(),
        arguments,
    };
    let local = apply(
        "nested_sequence_value",
        vec![
            IrNode::Parameter {
                name: "weights".to_owned(),
                ty: IrType::Sequence(Box::new(IrType::Sequence(Box::new(IrType::Integer)))),
            },
            integer("state_0"),
            integer("state_1"),
        ],
    );
    let ir = ProgramIr {
        name: "minimum_cost".to_owned(),
        parameters: vec![
            (
                "weights".to_owned(),
                IrType::Sequence(Box::new(IrType::Sequence(Box::new(IrType::Integer)))),
            ),
            ("row".to_owned(), IrType::Integer),
            ("column".to_owned(), IrType::Integer),
        ],
        result: IrType::Integer,
        body: IrNode::RecursiveReduce {
            state: vec!["state_0".to_owned(), "state_1".to_owned()],
            target: vec![integer("row"), integer("column")],
            item: vec!["offset_0".to_owned(), "offset_1".to_owned()],
            items: Box::new(apply("diagonal_predecessors", Vec::new())),
            next: vec![
                apply("integer_add", vec![integer("state_0"), integer("offset_0")]),
                apply("integer_add", vec![integer("state_1"), integer("offset_1")]),
            ],
            admissible: Some(Box::new(apply(
                "boolean_and",
                vec![
                    apply("integer_nonnegative", vec![integer("state_0")]),
                    apply("integer_nonnegative", vec![integer("state_1")]),
                ],
            ))),
            base_test: Box::new(apply(
                "boolean_and",
                vec![
                    apply("values_equal", vec![integer("state_0"), zero()]),
                    apply("values_equal", vec![integer("state_1"), zero()]),
                ],
            )),
            base: Box::new(local.clone()),
            local: Box::new(local),
            reducer: "reduce_min".to_owned(),
            combine: "integer_add".to_owned(),
        },
        fragments: vec![
            "boolean_and".to_owned(),
            "diagonal_predecessors".to_owned(),
            "integer_add".to_owned(),
            "integer_nonnegative".to_owned(),
            "nested_sequence_value".to_owned(),
            "reduce_min".to_owned(),
            "values_equal".to_owned(),
        ],
        source_urls: Vec::new(),
        source_licenses: Vec::new(),
        reuse: ReuseMode::ShapeOnly,
    };
    let catalog = FragmentCatalog::bootstrap();
    assert!(ir.type_check(&catalog).is_ok());
    assert_eq!(
        ProgramIr::from_links_notation(&ir.to_links_notation()),
        Some(ir),
        "recursive control flow remains associative data, not Rust-only behavior"
    );
}
