//! Type inference and unification for elaboration: node typing, fragment checks, and type-variable resolution.

use super::{BTreeMap, FragmentCatalog, IrNode, IrType, runtime_template};

pub(super) fn infer_node(
    node: &IrNode,
    catalog: &FragmentCatalog,
    environment: &mut BTreeMap<String, IrType>,
    substitutions: &mut BTreeMap<usize, IrType>,
    fresh: &mut usize,
) -> Result<IrType, String> {
    match node {
        IrNode::Parameter { name, ty } => {
            if let Some(declared) = environment.get(name).cloned() {
                unify(&declared, ty, substitutions).map_err(|detail| {
                    runtime_message("ir_parameter_error", &[("name", name), ("detail", &detail)])
                })?;
            }
            Ok(resolve_type(ty, substitutions))
        }
        IrNode::Literal { ty, .. } => Ok(resolve_type(ty, substitutions)),
        IrNode::Apply {
            fragment,
            arguments,
        } => {
            let definition = catalog
                .get(fragment)
                .ok_or_else(|| runtime_message("ir_apply_missing", &[("fragment", fragment)]))?;
            if definition.signature.len() != arguments.len() {
                let expected = definition.signature.len().to_string();
                let observed = arguments.len().to_string();
                return Err(runtime_message(
                    "ir_apply_arity",
                    &[
                        ("fragment", fragment),
                        ("expected", &expected),
                        ("observed", &observed),
                    ],
                ));
            }
            let base = *fresh;
            *fresh += FRAGMENT_TYPE_VARIABLE_STRIDE;
            for (position, (argument, expected)) in
                arguments.iter().zip(&definition.signature).enumerate()
            {
                let observed = infer_node(argument, catalog, environment, substitutions, fresh)?;
                let expected = fragment_type_variable(expected, base);
                unify(&expected, &observed, substitutions).map_err(|detail| {
                    let position = (position + 1).to_string();
                    runtime_message(
                        "ir_apply_argument",
                        &[
                            ("fragment", fragment),
                            ("position", &position),
                            ("detail", &detail),
                        ],
                    )
                })?;
            }
            Ok(resolve_type(
                &fragment_type_variable(&definition.result, base),
                substitutions,
            ))
        }
        IrNode::Each {
            item,
            items,
            body,
            predicate,
        } => {
            let items_type = infer_node(items, catalog, environment, substitutions, fresh)?;
            let element = iterable_element(&items_type);
            let previous = environment.insert(item.clone(), element);
            if let Some(predicate) = predicate {
                let predicate_type =
                    infer_node(predicate, catalog, environment, substitutions, fresh)?;
                unify(&IrType::Boolean, &predicate_type, substitutions).map_err(|detail| {
                    runtime_message("ir_each_predicate", &[("detail", &detail)])
                })?;
            }
            let body_type = infer_node(body, catalog, environment, substitutions, fresh)?;
            restore_binding(environment, item, previous);
            Ok(IrType::Sequence(Box::new(body_type)))
        }
        IrNode::Fold {
            item,
            accumulator,
            items,
            initial,
            body,
        } => {
            let items_type = infer_node(items, catalog, environment, substitutions, fresh)?;
            let initial_type = infer_node(initial, catalog, environment, substitutions, fresh)?;
            let old_item = environment.insert(item.clone(), iterable_element(&items_type));
            let old_accumulator = environment.insert(accumulator.clone(), initial_type.clone());
            let body_type = infer_node(body, catalog, environment, substitutions, fresh)?;
            unify(&initial_type, &body_type, substitutions).map_err(|detail| {
                runtime_message(
                    "ir_fold_error",
                    &[("accumulator", accumulator), ("detail", &detail)],
                )
            })?;
            restore_binding(environment, item, old_item);
            restore_binding(environment, accumulator, old_accumulator);
            Ok(resolve_type(&body_type, substitutions))
        }
        IrNode::Repeat {
            counter,
            from,
            to,
            body,
        } => {
            let from_type = infer_node(from, catalog, environment, substitutions, fresh)?;
            let to_type = infer_node(to, catalog, environment, substitutions, fresh)?;
            unify(&IrType::Integer, &from_type, substitutions)
                .and_then(|()| unify(&IrType::Integer, &to_type, substitutions))
                .map_err(|detail| {
                    runtime_message(
                        "ir_repeat_error",
                        &[("counter", counter), ("detail", &detail)],
                    )
                })?;
            let previous = environment.insert(counter.clone(), IrType::Integer);
            let body_type = infer_node(body, catalog, environment, substitutions, fresh)?;
            restore_binding(environment, counter, previous);
            Ok(body_type)
        }
        IrNode::Recurrence {
            state,
            base,
            transition,
            index,
        } => {
            let index_type = infer_node(index, catalog, environment, substitutions, fresh)?;
            unify(&IrType::Integer, &index_type, substitutions)
                .map_err(|detail| runtime_message("ir_recurrence_index", &[("detail", &detail)]))?;
            let mut recurrence_type = IrType::Unknown(usize::MAX - 1);
            for base_case in base {
                let base_type = infer_node(base_case, catalog, environment, substitutions, fresh)?;
                unify(&recurrence_type, &base_type, substitutions).map_err(|detail| {
                    runtime_message("ir_recurrence_base", &[("detail", &detail)])
                })?;
                recurrence_type = resolve_type(&base_type, substitutions);
            }
            let old = state
                .iter()
                .map(|name| {
                    (
                        name.clone(),
                        environment.insert(name.clone(), recurrence_type.clone()),
                    )
                })
                .collect::<Vec<_>>();
            let transition_type =
                infer_node(transition, catalog, environment, substitutions, fresh)?;
            unify(&recurrence_type, &transition_type, substitutions).map_err(|detail| {
                runtime_message("ir_recurrence_transition", &[("detail", &detail)])
            })?;
            for (name, previous) in old {
                restore_binding(environment, &name, previous);
            }
            Ok(resolve_type(&transition_type, substitutions))
        }
        IrNode::RecursiveReduce {
            state,
            target,
            item,
            items,
            next,
            admissible,
            base_test,
            base,
            local,
            reducer,
            combine,
        } => {
            if state.is_empty() || state.len() != target.len() || state.len() != next.len() {
                return Err(runtime_message("ir_recursive_arity", &[]));
            }
            let state_types = target
                .iter()
                .map(|node| infer_node(node, catalog, environment, substitutions, fresh))
                .collect::<Result<Vec<_>, _>>()?;
            let old_state = state
                .iter()
                .zip(&state_types)
                .map(|(name, ty)| (name.clone(), environment.insert(name.clone(), ty.clone())))
                .collect::<Vec<_>>();
            let items_type = infer_node(items, catalog, environment, substitutions, fresh)?;
            let item_types = destructured_types(&iterable_element(&items_type), item.len())?;
            let old_items = item
                .iter()
                .zip(item_types)
                .map(|(name, ty)| (name.clone(), environment.insert(name.clone(), ty)))
                .collect::<Vec<_>>();
            for (position, next_node) in next.iter().enumerate() {
                let next_type = infer_node(next_node, catalog, environment, substitutions, fresh)?;
                unify(&state_types[position], &next_type, substitutions).map_err(|detail| {
                    runtime_message(
                        "ir_recursive_next",
                        &[("position", &position.to_string()), ("detail", &detail)],
                    )
                })?;
            }
            if let Some(predicate) = admissible {
                let ty = infer_node(predicate, catalog, environment, substitutions, fresh)?;
                unify(&IrType::Boolean, &ty, substitutions).map_err(|detail| {
                    runtime_message("ir_recursive_admissible", &[("detail", &detail)])
                })?;
            }
            let test_type = infer_node(base_test, catalog, environment, substitutions, fresh)?;
            unify(&IrType::Boolean, &test_type, substitutions).map_err(|detail| {
                runtime_message("ir_recursive_base_test", &[("detail", &detail)])
            })?;
            let base_type = infer_node(base, catalog, environment, substitutions, fresh)?;
            let local_type = infer_node(local, catalog, environment, substitutions, fresh)?;
            unify(&base_type, &local_type, substitutions)
                .map_err(|detail| runtime_message("ir_recursive_local", &[("detail", &detail)]))?;
            check_reduction_fragment(catalog, reducer, &base_type, substitutions, fresh)?;
            check_combination_fragment(catalog, combine, &base_type, substitutions, fresh)?;
            for (name, previous) in old_items {
                restore_binding(environment, &name, previous);
            }
            for (name, previous) in old_state {
                restore_binding(environment, &name, previous);
            }
            Ok(resolve_type(&base_type, substitutions))
        }
        IrNode::Condition {
            test,
            then_branch,
            else_branch,
        } => {
            let test_type = infer_node(test, catalog, environment, substitutions, fresh)?;
            unify(&IrType::Boolean, &test_type, substitutions)
                .map_err(|detail| runtime_message("ir_condition_test", &[("detail", &detail)]))?;
            let then_type = infer_node(then_branch, catalog, environment, substitutions, fresh)?;
            let else_type = infer_node(else_branch, catalog, environment, substitutions, fresh)?;
            unify(&then_type, &else_type, substitutions).map_err(|detail| {
                runtime_message("ir_condition_branches", &[("detail", &detail)])
            })?;
            Ok(resolve_type(&then_type, substitutions))
        }
        IrNode::Bind { name, value, body } => {
            let value_type = infer_node(value, catalog, environment, substitutions, fresh)?;
            let previous = environment.insert(name.clone(), value_type);
            let body_type = infer_node(body, catalog, environment, substitutions, fresh)?;
            restore_binding(environment, name, previous);
            Ok(body_type)
        }
        IrNode::Emit { value } | IrNode::Return { value } => {
            infer_node(value, catalog, environment, substitutions, fresh)
        }
    }
}

pub(super) fn restore_binding(
    environment: &mut BTreeMap<String, IrType>,
    name: &str,
    previous: Option<IrType>,
) {
    if let Some(previous) = previous {
        environment.insert(name.to_owned(), previous);
    } else {
        environment.remove(name);
    }
}

pub(super) fn iterable_element(ty: &IrType) -> IrType {
    match ty {
        IrType::Sequence(element) | IrType::OrderedSequence(element) => (**element).clone(),
        IrType::Text => IrType::Text,
        IrType::Mapping(key, value) => IrType::Pair(key.clone(), value.clone()),
        IrType::Unknown(id) => IrType::Unknown(*id),
        _ => IrType::Unknown(usize::MAX),
    }
}

pub(super) fn destructured_types(ty: &IrType, arity: usize) -> Result<Vec<IrType>, String> {
    match (arity, ty) {
        (1, ty) => Ok(vec![ty.clone()]),
        (2, IrType::Pair(left, right)) => Ok(vec![(**left).clone(), (**right).clone()]),
        _ => Err(runtime_message(
            "ir_recursive_destructure",
            &[("arity", &arity.to_string()), ("type", &format!("{ty:?}"))],
        )),
    }
}

pub(super) fn check_reduction_fragment(
    catalog: &FragmentCatalog,
    id: &str,
    value_type: &IrType,
    substitutions: &mut BTreeMap<usize, IrType>,
    fresh: &mut usize,
) -> Result<(), String> {
    let fragment = catalog
        .get(id)
        .ok_or_else(|| runtime_message("ir_recursive_reducer_missing", &[("fragment", id)]))?;
    if fragment.signature.len() != 1 {
        return Err(runtime_message(
            "ir_recursive_reducer_arity",
            &[("fragment", id)],
        ));
    }
    let base = *fresh;
    *fresh += FRAGMENT_TYPE_VARIABLE_STRIDE;
    let expected = fragment_type_variable(&fragment.signature[0], base);
    let result = fragment_type_variable(&fragment.result, base);
    unify(
        &expected,
        &IrType::Sequence(Box::new(value_type.clone())),
        substitutions,
    )?;
    unify(&result, value_type, substitutions)
}

pub(super) fn check_combination_fragment(
    catalog: &FragmentCatalog,
    id: &str,
    value_type: &IrType,
    substitutions: &mut BTreeMap<usize, IrType>,
    fresh: &mut usize,
) -> Result<(), String> {
    let fragment = catalog
        .get(id)
        .ok_or_else(|| runtime_message("ir_recursive_combine_missing", &[("fragment", id)]))?;
    if fragment.signature.len() != 2 {
        return Err(runtime_message(
            "ir_recursive_combine_arity",
            &[("fragment", id)],
        ));
    }
    let base = *fresh;
    *fresh += FRAGMENT_TYPE_VARIABLE_STRIDE;
    for argument in &fragment.signature {
        unify(
            &fragment_type_variable(argument, base),
            value_type,
            substitutions,
        )?;
    }
    unify(
        &fragment_type_variable(&fragment.result, base),
        value_type,
        substitutions,
    )
}

pub(super) fn unify(
    expected: &IrType,
    observed: &IrType,
    substitutions: &mut BTreeMap<usize, IrType>,
) -> Result<(), String> {
    let expected = resolve_type(expected, substitutions);
    let observed = resolve_type(observed, substitutions);
    match (&expected, &observed) {
        (IrType::Unknown(id), other) | (other, IrType::Unknown(id)) => {
            if !matches!(other, IrType::Unknown(other_id) if other_id == id)
                && !contains_unknown(other, *id)
            {
                substitutions.insert(*id, other.clone());
            }
            Ok(())
        }
        (IrType::Sequence(left), IrType::Sequence(right)) => unify(left, right, substitutions),
        (IrType::OrderedSequence(left), IrType::OrderedSequence(right)) => {
            unify(left, right, substitutions)
        }
        // An ordered sequence is a sequence: consumers indifferent to order
        // accept it. The reverse direction is refused at enumeration time by
        // `types_may_unify`, so only order-safe compositions form.
        (IrType::OrderedSequence(ordered), IrType::Sequence(unordered))
        | (IrType::Sequence(unordered), IrType::OrderedSequence(ordered)) => {
            unify(ordered, unordered, substitutions)
        }
        // Text is an iterable whose elements are text characters. Keeping this
        // relation in the type system (rather than in a vowel/count recipe)
        // lets every sequence combinator be discovered for strings too.
        (IrType::Sequence(element), IrType::Text) | (IrType::Text, IrType::Sequence(element)) => {
            unify(element, &IrType::Text, substitutions)
        }
        (IrType::Pair(left_a, left_b), IrType::Pair(right_a, right_b))
        | (IrType::Mapping(left_a, left_b), IrType::Mapping(right_a, right_b)) => {
            unify(left_a, right_a, substitutions)?;
            unify(left_b, right_b, substitutions)
        }
        _ if expected == observed => Ok(()),
        _ => Err(runtime_message(
            "ir_type_mismatch",
            &[
                ("expected", &format!("{expected:?}")),
                ("observed", &format!("{observed:?}")),
            ],
        )),
    }
}

pub(super) fn runtime_message(id: &str, values: &[(&str, &str)]) -> String {
    runtime_template(id, values).unwrap_or_else(|| id.to_owned())
}

pub(super) const FRAGMENT_TYPE_VARIABLE_OFFSET: usize = usize::MAX / 4;

/// Distinct type-variable ids reserved for one fragment occurrence. Covers
/// every schematic variable a seeded fragment declares.
pub(super) const FRAGMENT_TYPE_VARIABLE_STRIDE: usize = 32;

/// Instantiates a fragment's schematic type with type variables based at
/// `base`. The base is drawn fresh for every fragment occurrence: a constant
/// offset would make two applications of different fragments share the same
/// variable, letting one application's iterable binding (text iteration
/// binding an element to text) poison an unrelated fragment's element type.
pub(super) fn fragment_type_variable(ty: &IrType, base: usize) -> IrType {
    match ty {
        IrType::Unknown(id) => IrType::Unknown(base.saturating_add(*id)),
        IrType::Sequence(element) => {
            IrType::Sequence(Box::new(fragment_type_variable(element, base)))
        }
        IrType::OrderedSequence(element) => {
            IrType::OrderedSequence(Box::new(fragment_type_variable(element, base)))
        }
        IrType::Pair(left, right) => IrType::Pair(
            Box::new(fragment_type_variable(left, base)),
            Box::new(fragment_type_variable(right, base)),
        ),
        IrType::Mapping(key, value) => IrType::Mapping(
            Box::new(fragment_type_variable(key, base)),
            Box::new(fragment_type_variable(value, base)),
        ),
        _ => ty.clone(),
    }
}

pub(super) fn contains_unknown(ty: &IrType, sought: usize) -> bool {
    match ty {
        IrType::Unknown(id) => *id == sought,
        IrType::Sequence(element) | IrType::OrderedSequence(element) => {
            contains_unknown(element, sought)
        }
        IrType::Pair(left, right) | IrType::Mapping(left, right) => {
            contains_unknown(left, sought) || contains_unknown(right, sought)
        }
        _ => false,
    }
}

pub(super) fn resolve_type(ty: &IrType, substitutions: &BTreeMap<usize, IrType>) -> IrType {
    match ty {
        IrType::Unknown(id) => substitutions
            .get(id)
            .map_or_else(|| ty.clone(), |value| resolve_type(value, substitutions)),
        IrType::Sequence(element) => {
            IrType::Sequence(Box::new(resolve_type(element, substitutions)))
        }
        IrType::OrderedSequence(element) => {
            IrType::OrderedSequence(Box::new(resolve_type(element, substitutions)))
        }
        IrType::Pair(left, right) => IrType::Pair(
            Box::new(resolve_type(left, substitutions)),
            Box::new(resolve_type(right, substitutions)),
        ),
        IrType::Mapping(key, value) => IrType::Mapping(
            Box::new(resolve_type(key, substitutions)),
            Box::new(resolve_type(value, substitutions)),
        ),
        _ => ty.clone(),
    }
}
