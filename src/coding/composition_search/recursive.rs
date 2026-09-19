//! Recursive reduction candidates: self-referential folds over seeded transition fragments.

use super::*;

pub(super) fn recursive_reduce_programs(
    spec: &CodingTaskSpec,
    catalog: &FragmentCatalog,
    parameters: &[(String, IrType)],
    structure_ids: &[String],
    limit: usize,
) -> Vec<ProgramIr> {
    let semantic_words = words(&format!(
        "{} {}",
        spec.requirement_sentences.join(" "),
        structure_ids.join(" ")
    ));
    let integers = parameters
        .iter()
        .filter(|(_, ty)| types_may_unify(ty, &IrType::Integer))
        .collect::<Vec<_>>();
    let predecessors = catalog
        .fragments()
        .iter()
        .filter(|fragment| {
            fragment.signature.is_empty()
                && types_may_unify(
                    &fragment.result,
                    &IrType::Sequence(Box::new(IrType::Pair(
                        Box::new(IrType::Integer),
                        Box::new(IrType::Integer),
                    ))),
                )
                && (fragment
                    .supports
                    .iter()
                    .any(|supported| structure_ids.contains(supported))
                    || !words(&format!("{} {}", fragment.id, fragment.rediscovery_query))
                        .is_disjoint(&semantic_words))
        })
        .collect::<Vec<_>>();
    let mut programs = Vec::new();
    for (grid_name, grid_type) in parameters {
        let IrType::Sequence(rows) = grid_type else {
            continue;
        };
        let IrType::Sequence(value_type) = rows.as_ref() else {
            continue;
        };
        for (row_index, (row_name, row_type)) in integers.iter().enumerate() {
            if row_name.as_str() == grid_name.as_str() {
                continue;
            }
            for (column_name, column_type) in integers.iter().skip(row_index + 1) {
                if column_name.as_str() == grid_name.as_str() {
                    continue;
                }
                let accessors = prefer_supported(
                    catalog
                        .fragments()
                        .iter()
                        .filter(|fragment| {
                            fragment.signature.len() == 3
                                && types_may_unify(&fragment.signature[0], grid_type)
                                && types_may_unify(&fragment.signature[1], &IrType::Integer)
                                && types_may_unify(&fragment.signature[2], &IrType::Integer)
                                && types_may_unify(&fragment.result, value_type)
                        })
                        .collect(),
                    structure_ids,
                );
                let reducers = prefer_supported(
                    catalog
                        .fragments()
                        .iter()
                        .filter(|fragment| {
                            fragment.signature.len() == 1
                                && types_may_unify(
                                    &fragment.signature[0],
                                    &IrType::Sequence(value_type.clone()),
                                )
                                && types_may_unify(&fragment.result, value_type)
                                && !words(&format!(
                                    "{} {}",
                                    fragment.id, fragment.rediscovery_query
                                ))
                                .is_disjoint(&semantic_words)
                        })
                        .collect(),
                    structure_ids,
                );
                let value_combiners = prefer_supported(
                    catalog
                        .fragments()
                        .iter()
                        .filter(|fragment| {
                            fragment.signature.len() == 2
                                && fragment
                                    .signature
                                    .iter()
                                    .all(|ty| types_may_unify(ty, value_type))
                                && types_may_unify(&fragment.result, value_type)
                        })
                        .collect(),
                    structure_ids,
                );
                let equalities = prefer_supported(
                    catalog
                        .fragments()
                        .iter()
                        .filter(|fragment| {
                            fragment.signature.len() == 2
                                && fragment
                                    .signature
                                    .iter()
                                    .all(|ty| types_may_unify(ty, &IrType::Integer))
                                && types_may_unify(&fragment.result, &IrType::Boolean)
                        })
                        .collect(),
                    structure_ids,
                );
                let guards = prefer_supported(
                    catalog
                        .fragments()
                        .iter()
                        .filter(|fragment| {
                            fragment.signature.len() == 1
                                && types_may_unify(&fragment.signature[0], &IrType::Integer)
                                && types_may_unify(&fragment.result, &IrType::Boolean)
                        })
                        .collect(),
                    structure_ids,
                );
                let conjunctions = prefer_supported(
                    catalog
                        .fragments()
                        .iter()
                        .filter(|fragment| {
                            fragment.signature.len() == 2
                                && fragment
                                    .signature
                                    .iter()
                                    .all(|ty| types_may_unify(ty, &IrType::Boolean))
                                && types_may_unify(&fragment.result, &IrType::Boolean)
                        })
                        .collect(),
                    structure_ids,
                );
                for predecessor in &predecessors {
                    for accessor in &accessors {
                        for reducer in &reducers {
                            for value_combiner in &value_combiners {
                                for equality in &equalities {
                                    for guard in &guards {
                                        for conjunction in &conjunctions {
                                            programs.push(recursive_reduce_program(
                                                spec,
                                                parameters,
                                                grid_name,
                                                grid_type,
                                                row_name,
                                                row_type,
                                                column_name,
                                                column_type,
                                                value_type,
                                                predecessor,
                                                accessor,
                                                reducer,
                                                value_combiner,
                                                equality,
                                                guard,
                                                conjunction,
                                            ));
                                            if programs.len() >= limit.saturating_mul(4) {
                                                return checked_recursive_programs(
                                                    programs, catalog, limit,
                                                );
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
    checked_recursive_programs(programs, catalog, limit)
}

pub(super) fn prefer_supported<'a>(
    fragments: Vec<&'a Fragment>,
    structure_ids: &[String],
) -> Vec<&'a Fragment> {
    let supported = fragments
        .iter()
        .copied()
        .filter(|fragment| {
            fragment
                .supports
                .iter()
                .any(|supported| structure_ids.contains(supported))
        })
        .collect::<Vec<_>>();
    if supported.is_empty() {
        fragments
    } else {
        supported
    }
}

#[allow(clippy::too_many_arguments)]
pub(super) fn recursive_reduce_program(
    spec: &CodingTaskSpec,
    parameters: &[(String, IrType)],
    grid_name: &str,
    grid_type: &IrType,
    row_name: &str,
    row_type: &IrType,
    column_name: &str,
    column_type: &IrType,
    value_type: &IrType,
    predecessor: &Fragment,
    accessor: &Fragment,
    reducer: &Fragment,
    value_combiner: &Fragment,
    equality: &Fragment,
    guard: &Fragment,
    conjunction: &Fragment,
) -> ProgramIr {
    let parameter = |name: &str, ty: &IrType| IrNode::Parameter {
        name: name.to_owned(),
        ty: ty.clone(),
    };
    let apply = |fragment: &Fragment, arguments: Vec<IrNode>| IrNode::Apply {
        fragment: fragment.id.clone(),
        arguments,
    };
    let state = ["state_0", "state_1"];
    let item = ["predecessor_0", "predecessor_1"];
    let state_nodes = [
        parameter(state[0], &IrType::Integer),
        parameter(state[1], &IrType::Integer),
    ];
    let next = [
        apply(
            value_combiner,
            vec![state_nodes[0].clone(), parameter(item[0], &IrType::Integer)],
        ),
        apply(
            value_combiner,
            vec![state_nodes[1].clone(), parameter(item[1], &IrType::Integer)],
        ),
    ];
    let zero = || IrNode::Literal {
        text: "0".to_owned(),
        ty: IrType::Integer,
    };
    let local = apply(
        accessor,
        vec![
            parameter(grid_name, grid_type),
            state_nodes[0].clone(),
            state_nodes[1].clone(),
        ],
    );
    let base_test = apply(
        conjunction,
        vec![
            apply(equality, vec![state_nodes[0].clone(), zero()]),
            apply(equality, vec![state_nodes[1].clone(), zero()]),
        ],
    );
    let admissible = apply(
        conjunction,
        vec![
            apply(guard, vec![next[0].clone()]),
            apply(guard, vec![next[1].clone()]),
        ],
    );
    let mut fragments = vec![
        predecessor,
        accessor,
        reducer,
        value_combiner,
        equality,
        guard,
        conjunction,
    ];
    fragments.sort_by_key(|fragment| fragment.id.as_str());
    fragments.dedup_by_key(|fragment| fragment.id.as_str());
    ProgramIr {
        name: spec.name.clone(),
        parameters: parameters.to_vec(),
        result: value_type.clone(),
        body: IrNode::RecursiveReduce {
            state: state.iter().map(ToString::to_string).collect(),
            target: vec![
                parameter(row_name, row_type),
                parameter(column_name, column_type),
            ],
            item: item.iter().map(ToString::to_string).collect(),
            items: Box::new(apply(predecessor, Vec::new())),
            next: next.into_iter().collect(),
            admissible: Some(Box::new(admissible)),
            base_test: Box::new(base_test),
            base: Box::new(local.clone()),
            local: Box::new(local),
            reducer: reducer.id.clone(),
            combine: value_combiner.id.clone(),
        },
        fragments: fragments
            .iter()
            .map(|fragment| fragment.id.clone())
            .collect(),
        source_urls: fragments
            .iter()
            .map(|fragment| fragment.grounding.clone())
            .collect(),
        source_licenses: fragments
            .iter()
            .map(|fragment| fragment.license.clone())
            .collect(),
        reuse: crate::coding::program_ir::ReuseMode::ShapeOnly,
    }
}
