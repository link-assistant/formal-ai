//! Issue #1138, plan 02 L6 and L19 — IR → source, in more than one language.
//!
//! `compose` refuses every language but Python today. Once lowering is a
//! registered backend, a gap is named — "no Rust lowering for
//! `IrNode::Recurrence`" rather than "renderer unavailable" — and the same IR
//! lowers to two languages, which is the only proof that the IR is not
//! Python-shaped.

use formal_ai::fragment_catalog::FragmentCatalog;
use formal_ai::ir_lowering::{lowering_for, lowerings};
use formal_ai::program_ir::{IrNode, IrType, ProgramIr, ReuseMode};

/// One node of each variant, so the lowering is exercised exhaustively.
fn every_node() -> Vec<(&'static str, IrNode)> {
    let text = || {
        Box::new(IrNode::Parameter {
            name: "text".to_owned(),
            ty: IrType::Text,
        })
    };
    let zero = || {
        Box::new(IrNode::Literal {
            text: "0".to_owned(),
            ty: IrType::Integer,
        })
    };
    vec![
        (
            "Parameter",
            IrNode::Parameter {
                name: "text".to_owned(),
                ty: IrType::Text,
            },
        ),
        (
            "Literal",
            IrNode::Literal {
                text: "0".to_owned(),
                ty: IrType::Integer,
            },
        ),
        (
            "Apply",
            IrNode::Apply {
                fragment: "extend_run".to_owned(),
                arguments: vec![*text()],
            },
        ),
        (
            "Each",
            IrNode::Each {
                item: "character".to_owned(),
                items: text(),
                body: zero(),
                predicate: None,
            },
        ),
        (
            "Fold",
            IrNode::Fold {
                item: "character".to_owned(),
                accumulator: "runs".to_owned(),
                items: text(),
                initial: zero(),
                body: zero(),
            },
        ),
        (
            "Repeat",
            IrNode::Repeat {
                counter: "index".to_owned(),
                from: zero(),
                to: zero(),
                body: zero(),
            },
        ),
        (
            "Recurrence",
            IrNode::Recurrence {
                state: vec!["previous".to_owned()],
                base: vec![*zero()],
                transition: zero(),
                index: zero(),
            },
        ),
        (
            "RecursiveReduce",
            IrNode::RecursiveReduce {
                state: vec!["state".to_owned()],
                target: vec![*zero()],
                item: vec!["offset".to_owned()],
                items: Box::new(IrNode::Literal {
                    text: "[-1]".to_owned(),
                    ty: IrType::Sequence(Box::new(IrType::Integer)),
                }),
                next: vec![*zero()],
                admissible: None,
                base_test: Box::new(IrNode::Literal {
                    text: "true".to_owned(),
                    ty: IrType::Boolean,
                }),
                base: zero(),
                local: zero(),
                reducer: "reduce_min".to_owned(),
                combine: "integer_add".to_owned(),
            },
        ),
        (
            "Condition",
            IrNode::Condition {
                test: zero(),
                then_branch: zero(),
                else_branch: zero(),
            },
        ),
        (
            "Bind",
            IrNode::Bind {
                name: "runs".to_owned(),
                value: zero(),
                body: zero(),
            },
        ),
        ("Emit", IrNode::Emit { value: zero() }),
        ("Return", IrNode::Return { value: zero() }),
    ]
}

fn program(name: &str, body: IrNode) -> ProgramIr {
    ProgramIr {
        name: name.to_owned(),
        parameters: vec![("text".to_owned(), IrType::Text)],
        result: IrType::Sequence(Box::new(IrType::Pair(
            Box::new(IrType::Text),
            Box::new(IrType::Integer),
        ))),
        body,
        fragments: vec!["extend_run".to_owned()],
        source_urls: vec!["https://docs.python.org/3.12/library/itertools.html".to_owned()],
        source_licenses: vec!["PSF-2.0".to_owned()],
        reuse: ReuseMode::Verbatim,
    }
}

#[test]
fn every_ir_node_lowers_to_python_or_names_its_gap() {
    let catalog = FragmentCatalog::bootstrap();
    let python = lowering_for("python").expect("a Python lowering is registered");
    assert_eq!(python.language(), "python");

    for (label, node) in every_node() {
        match python.lower(&program("run_length", node), &catalog) {
            Ok(source) => assert!(
                !source.trim().is_empty(),
                "{label} lowered to nothing at all"
            ),
            Err(gap) => {
                assert_eq!(gap.language, "python");
                assert_eq!(
                    gap.node, label,
                    "an unsupported node names itself so the gap is actionable"
                );
                assert!(!gap.detail.is_empty(), "{label} gap carries no detail");
            }
        }
    }
}

#[test]
fn the_same_ir_lowers_to_python_and_rust() {
    let catalog = FragmentCatalog::bootstrap();
    let ir = program(
        "run_length",
        IrNode::Fold {
            item: "character".to_owned(),
            accumulator: "runs".to_owned(),
            items: Box::new(IrNode::Parameter {
                name: "text".to_owned(),
                ty: IrType::Text,
            }),
            initial: Box::new(IrNode::Literal {
                text: "[]".to_owned(),
                ty: run_sequence_type(),
            }),
            body: Box::new(IrNode::Apply {
                fragment: "extend_run".to_owned(),
                arguments: vec![
                    IrNode::Parameter {
                        name: "runs".to_owned(),
                        ty: run_sequence_type(),
                    },
                    IrNode::Parameter {
                        name: "character".to_owned(),
                        ty: IrType::Text,
                    },
                ],
            }),
        },
    );

    let languages: Vec<&'static str> = lowerings()
        .into_iter()
        .map(formal_ai::ir_lowering::LanguageLowering::language)
        .collect();
    assert!(
        languages.contains(&"python") && languages.contains(&"rust"),
        "two registered backends are what proves the IR is language-neutral: {languages:?}"
    );

    let python = lowering_for("python")
        .expect("python")
        .lower(&ir, &catalog)
        .expect("the case-1 program lowers to Python");
    let rust = lowering_for("rust")
        .expect("rust")
        .lower(&ir, &catalog)
        .expect("the case-4 program lowers to Rust");

    assert!(python.contains("def run_length"), "Python source: {python}");
    assert!(rust.contains("fn run_length"), "Rust source: {rust}");
    assert_ne!(python, rust, "two languages, one IR");
    assert_eq!(
        ir.content_id(),
        program(
            "run_length",
            IrNode::Fold {
                item: "character".to_owned(),
                accumulator: "runs".to_owned(),
                items: Box::new(IrNode::Parameter {
                    name: "text".to_owned(),
                    ty: IrType::Text,
                }),
                initial: Box::new(IrNode::Literal {
                    text: "[]".to_owned(),
                    ty: run_sequence_type(),
                }),
                body: Box::new(IrNode::Apply {
                    fragment: "extend_run".to_owned(),
                    arguments: vec![
                        IrNode::Parameter {
                            name: "runs".to_owned(),
                            ty: run_sequence_type(),
                        },
                        IrNode::Parameter {
                            name: "character".to_owned(),
                            ty: IrType::Text,
                        },
                    ],
                }),
            },
        )
        .content_id(),
        "the two language requests share one IR identity"
    );
}

fn run_sequence_type() -> IrType {
    IrType::Sequence(Box::new(IrType::Pair(
        Box::new(IrType::Text),
        Box::new(IrType::Integer),
    )))
}
