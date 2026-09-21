//! Lowering a searched expression into a `ProgramIr` candidate, plus the strict type-fit helpers the enumeration gates on.

use super::{CodingTaskSpec, Expression, IrNode, IrType, ProgramIr, parse_type_slug};

pub(super) fn program_from_expression(
    spec: &CodingTaskSpec,
    parameters: Vec<(String, IrType)>,
    expression: Expression,
) -> ProgramIr {
    let (source_urls, source_licenses): (Vec<_>, Vec<_>) = expression
        .grounding
        .into_iter()
        .filter(|(url, _)| url.strip_prefix("https://").is_some())
        .unzip();
    ProgramIr {
        name: spec.name.clone(),
        parameters,
        result: expression.ty,
        body: expression.node,
        fragments: expression.fragments,
        source_urls,
        source_licenses,
        reuse: crate::coding::program_ir::ReuseMode::ShapeOnly,
    }
}

pub(super) fn fold_candidate(
    fragment: &crate::coding::fragment_catalog::Fragment,
    parameters: &[(String, IrType)],
) -> Option<IrNode> {
    let (parameter_name, parameter_type) = parameters.first()?;
    if fragment.signature.len() != 2 || !types_may_unify(&fragment.signature[0], &fragment.result) {
        return None;
    }
    let accumulator = "accumulator".to_owned();
    let item = "item".to_owned();
    Some(IrNode::Fold {
        item: item.clone(),
        accumulator: accumulator.clone(),
        items: Box::new(IrNode::Parameter {
            name: parameter_name.clone(),
            ty: parameter_type.clone(),
        }),
        initial: Box::new(IrNode::Literal {
            text: neutral_literal(&fragment.signature[0]).to_owned(),
            ty: fragment.signature[0].clone(),
        }),
        body: Box::new(IrNode::Apply {
            fragment: fragment.id.clone(),
            arguments: vec![
                IrNode::Parameter {
                    name: accumulator,
                    ty: fragment.signature[0].clone(),
                },
                IrNode::Parameter {
                    name: item,
                    ty: fragment.signature[1].clone(),
                },
            ],
        }),
    })
}

pub(super) const fn neutral_literal(ty: &IrType) -> &'static str {
    match ty {
        IrType::Integer | IrType::Float => "0",
        IrType::Boolean => "false",
        IrType::Text => "\"\"",
        IrType::Callable | IrType::Unknown(_) => "None",
        IrType::Sequence(_) | IrType::OrderedSequence(_) => "[]",
        IrType::Pair(_, _) => "(0, 0)",
        IrType::Mapping(_, _) => "{}",
    }
}

pub(super) fn annotation_type(annotation: &str) -> Option<IrType> {
    let normalized = annotation
        .to_ascii_lowercase()
        .replace([' ', '&'], "")
        .replace("list[", "sequence<")
        .replace(']', ">")
        .replace("str", "text")
        .replace("bool", "boolean")
        .replace("int", "integer");
    parse_type_slug(&normalized)
}

/// Structural parameter types discovered from the task's own examples.
///
/// The examples are trusted sources: an argument the task itself supplies
/// tells the search whether a parameter is a sequence or text, so scalar
/// slots stop accepting whole collections. Only the structural classes are
/// discovered — bare numbers stay open, and parameters whose examples
/// disagree (or are absent) stay open too. A tuple argument stays open the
/// same way: a bare `(3, 4, 5, 6)` is ambiguous between a fixed pair and an
/// ordered sequence, so it discovers no class and the parameters stay
/// generic by arity (issue #1085 upstream transfer, MBPP/2).
pub(super) fn example_parameter_type(spec: &CodingTaskSpec, index: usize) -> Option<IrType> {
    let classes = spec
        .examples
        .iter()
        .map(|example| example.arguments.get(index).map(String::as_str))
        .collect::<Option<Vec<_>>>()?;
    if classes.is_empty() {
        return None;
    }
    let same_class = |left: &IrType, right: &IrType| match (left, right) {
        (IrType::Sequence(_), IrType::Sequence(_)) => true,
        _ => left == right,
    };
    let mut discovered: Option<IrType> = None;
    for value in &classes {
        let trimmed = value.trim();
        let element_class = |inner: &str| {
            let elements = inner
                .split(',')
                .map(str::trim)
                .filter(|element| !element.is_empty())
                .collect::<Vec<_>>();
            if !elements.is_empty()
                && elements
                    .iter()
                    .all(|element| element.parse::<f64>().is_ok())
            {
                IrType::Float
            } else {
                IrType::Unknown(9_001)
            }
        };
        let class = if trimmed.starts_with('[') {
            let inner = trimmed.trim_start_matches('[').trim_end_matches(']').trim();
            IrType::Sequence(Box::new(element_class(inner)))
        } else if trimmed.starts_with('(') && trimmed.ends_with(')') {
            continue;
        } else if trimmed.starts_with('\'') || trimmed.starts_with('"') {
            IrType::Text
        } else if trimmed.parse::<f64>().is_ok() || trimmed.is_empty() {
            continue;
        } else {
            IrType::Text
        };
        match &discovered {
            None => discovered = Some(class),
            Some(seen) if same_class(seen, &class) => {}
            Some(_) => return None,
        }
    }
    discovered
}

pub(super) fn types_may_unify(left: &IrType, right: &IrType) -> bool {
    match (left, right) {
        (IrType::Unknown(_), _) | (_, IrType::Unknown(_)) => true,
        (IrType::Sequence(left), IrType::Sequence(right)) => types_may_unify(left, right),
        (IrType::Sequence(element), IrType::Text) | (IrType::Text, IrType::Sequence(element)) => {
            types_may_unify(element, &IrType::Text)
        }
        (IrType::Pair(left_a, left_b), IrType::Pair(right_a, right_b))
        | (IrType::Mapping(left_a, left_b), IrType::Mapping(right_a, right_b)) => {
            types_may_unify(left_a, right_a) && types_may_unify(left_b, right_b)
        }
        _ => left == right,
    }
}

/// Strict counterpart to [`types_may_unify`]: the same recursion without the
/// text-over-sequence leniency. A choice whose type fits a slot only through
/// that leniency is a guess about the iteration element; a choice whose type
/// fits exactly is grounded, so windows rank it first.
pub(super) fn types_fit_strictly(left: &IrType, right: &IrType) -> bool {
    match (left, right) {
        (IrType::Unknown(_), _) | (_, IrType::Unknown(_)) => true,
        (IrType::Sequence(left), IrType::Sequence(right)) => types_fit_strictly(left, right),
        (IrType::Pair(left_a, left_b), IrType::Pair(right_a, right_b))
        | (IrType::Mapping(left_a, left_b), IrType::Mapping(right_a, right_b)) => {
            types_fit_strictly(left_a, right_a) && types_fit_strictly(left_b, right_b)
        }
        _ => left == right,
    }
}

pub(super) fn words(text: &str) -> std::collections::BTreeSet<String> {
    text.split(|character: char| !character.is_alphanumeric())
        .filter(|word| word.chars().count() > 2)
        .map(str::to_lowercase)
        .collect()
}

pub(super) fn node_depth(node: &IrNode) -> usize {
    let children = match node {
        IrNode::Parameter { .. } | IrNode::Literal { .. } => 0,
        IrNode::Apply { arguments, .. } => arguments.iter().map(node_depth).max().unwrap_or(0),
        IrNode::Each {
            items,
            body,
            predicate,
            ..
        } => node_depth(items)
            .max(node_depth(body))
            .max(predicate.as_deref().map_or(0, node_depth)),
        IrNode::Fold {
            items,
            initial,
            body,
            ..
        } => node_depth(items)
            .max(node_depth(initial))
            .max(node_depth(body)),
        IrNode::Repeat { from, to, body, .. } => {
            node_depth(from).max(node_depth(to)).max(node_depth(body))
        }
        IrNode::Recurrence {
            base,
            transition,
            index,
            ..
        } => base
            .iter()
            .map(node_depth)
            .max()
            .unwrap_or(0)
            .max(node_depth(transition))
            .max(node_depth(index)),
        IrNode::RecursiveReduce {
            target,
            items,
            next,
            admissible,
            base_test,
            base,
            local,
            ..
        } => target
            .iter()
            .map(node_depth)
            .max()
            .unwrap_or(0)
            .max(node_depth(items))
            .max(next.iter().map(node_depth).max().unwrap_or(0))
            .max(admissible.as_deref().map_or(0, node_depth))
            .max(node_depth(base_test))
            .max(node_depth(base))
            .max(node_depth(local)),
        IrNode::Condition {
            test,
            then_branch,
            else_branch,
        } => node_depth(test)
            .max(node_depth(then_branch))
            .max(node_depth(else_branch)),
        IrNode::Bind { value, body, .. } => node_depth(value).max(node_depth(body)),
        IrNode::Emit { value } | IrNode::Return { value } => node_depth(value),
    };
    children + 1
}
