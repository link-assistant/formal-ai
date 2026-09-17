//! Bounded typed enumeration over the available fragments (issue #1138, plan
//! 02 L7).
//!
//! The search replaces the forty hand-authored composition blocks: an unseen
//! combination of seeded meanings composes because the enumeration is over
//! fragment *type signatures*, not over authored shapes. Order is deterministic
//! — by [`crate::coding::program_ir::ProgramIr::action_cost`], then content id.

use std::cmp::Reverse;
use std::collections::BTreeSet;

use crate::coding::fragment_catalog::{Fragment, FragmentCatalog};
use crate::coding::program_ir::{IrNode, IrType, ProgramIr, parse_type_slug};
use crate::coding::task_spec::{ArtifactShape, CodingTaskSpec};

/// Declared enumeration bounds: a width and a depth, never a time budget.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SearchBounds {
    pub max_candidates: usize,
    pub max_depth: usize,
}

impl Default for SearchBounds {
    fn default() -> Self {
        Self {
            max_candidates: 256,
            max_depth: 8,
        }
    }
}

/// Enumerate typed candidate programs for `spec` from `catalog`, cheapest
/// first and deterministic across runs.
#[must_use]
pub fn search(
    spec: &CodingTaskSpec,
    catalog: &FragmentCatalog,
    bounds: SearchBounds,
) -> Vec<ProgramIr> {
    search_with_structures(spec, catalog, bounds, &[])
}

/// Enumerate with language-neutral structure ids supplied by concept discovery.
///
/// Raw prompt overlap remains useful, but cannot be the sole
/// relevance signal: equal concepts expressed in different languages must
/// expose the same typed neighborhood.
#[must_use]
pub fn search_with_structures(
    spec: &CodingTaskSpec,
    catalog: &FragmentCatalog,
    bounds: SearchBounds,
    structure_ids: &[String],
) -> Vec<ProgramIr> {
    if bounds.max_candidates == 0 || bounds.max_depth == 0 {
        return Vec::new();
    }
    if spec.artifact_shape == ArtifactShape::Program {
        return search_program(spec, catalog, bounds, structure_ids);
    }
    let requirement = spec.requirement_sentences.join(" ");
    let requirement_words = words(&requirement);
    let mut ranked = catalog
        .fragments()
        .iter()
        .map(|fragment| {
            let description = format!("{} {}", fragment.id, fragment.rediscovery_query);
            let lexical_score = if structure_ids.is_empty() {
                words(&description)
                    .iter()
                    .filter(|word| requirement_words.contains(*word))
                    .count()
            } else {
                0
            };
            let exact_score = usize::from(structure_ids.iter().any(|id| id == &fragment.id));
            let support_score = fragment
                .supports
                .iter()
                .filter(|supported| structure_ids.contains(supported))
                .count();
            let score = exact_score * 100 + support_score * 90 + lexical_score;
            (score, fragment)
        })
        .filter(|(score, _)| *score > 0)
        .collect::<Vec<_>>();
    ranked.sort_by(|left, right| {
        right
            .0
            .cmp(&left.0)
            .then_with(|| left.1.signature.len().cmp(&right.1.signature.len()))
            .then_with(|| left.1.id.cmp(&right.1.id))
    });

    let parameters = spec
        .parameters
        .iter()
        .enumerate()
        .map(|(index, parameter)| {
            (
                parameter.name.clone(),
                parameter
                    .annotation
                    .as_deref()
                    .and_then(annotation_type)
                    .or_else(|| example_parameter_type(spec, index))
                    .unwrap_or(IrType::Unknown(index)),
            )
        })
        .collect::<Vec<_>>();
    let ranked = ranked
        .into_iter()
        .take(bounds.max_candidates.min(32))
        .map(|(_, fragment)| fragment)
        .collect::<Vec<_>>();
    let coverable = structure_ids
        .iter()
        .filter(|id| {
            ranked.iter().any(|fragment| {
                fragment.id == id.as_str()
                    || fragment
                        .supports
                        .iter()
                        .any(|supported| supported == id.as_str())
            })
        })
        .cloned()
        .collect::<BTreeSet<_>>();
    let placeholder_names = ranked
        .iter()
        .flat_map(|fragment| fragment.argument_names(&spec.language))
        .collect::<BTreeSet<_>>();
    let binder_names: Vec<BTreeSet<String>> = ranked
        .iter()
        .map(|fragment| fragment_binder_slots(fragment, &spec.language))
        .collect();
    let mut pool = atom_expressions(spec, &parameters, &ranked, structure_ids);
    let fold_roots = ranked
        .iter()
        .filter_map(|fragment| {
            let node = fold_candidate(fragment, &parameters)?;
            Some(Expression {
                depth: node_depth(&node),
                constant_maps: constant_map_count(&node, catalog, &spec.language),
                loose: loose_feeds(&node, catalog),
                node,
                ty: fragment.result.clone(),
                fragments: vec![fragment.id.clone()],
                coverage: fragment_coverage(fragment, structure_ids),
                grounding: vec![(fragment.grounding.clone(), fragment.license.clone())],
            })
        })
        .collect::<Vec<_>>();
    for depth in 2..=bounds.max_depth {
        let snapshot = pool.clone();
        let mut layer = Vec::new();
        for (fragment_index, fragment) in ranked.iter().enumerate() {
            let names = fragment.argument_names(&spec.language);
            let binders = &binder_names[fragment_index];
            let binder_slot = names
                .iter()
                .enumerate()
                .find_map(|(index, name)| binders.contains(name).then_some(index));
            let (loop_variable_choices, unpacked_names) = binder_slot
                .and_then(|index| {
                    let name = names.get(index)?;
                    let element = fragment.signature.get(index)?;
                    let others = ranked
                        .iter()
                        .enumerate()
                        .filter_map(|(other, fragment)| {
                            (other != fragment_index).then_some(*fragment)
                        })
                        .collect::<Vec<_>>();
                    let mut choices = loop_variable_applies(
                        others.iter().copied(),
                        &snapshot,
                        name,
                        element,
                        &parameters,
                        structure_ids,
                        &coverable,
                        catalog,
                        &spec.language,
                    );
                    let mut element_types: Vec<IrType> = snapshot
                        .iter()
                        .map(|expression| expression.ty.clone())
                        .collect();
                    if !element_types.contains(&element) {
                        element_types.push(element.clone());
                    }
                    let (pair_applies, unpacked) = pair_loop_variable_applies(
                        others.iter().copied(),
                        &snapshot,
                        &element_types,
                        &parameters,
                        structure_ids,
                        &coverable,
                        catalog,
                        &spec.language,
                    );
                    choices.extend(pair_applies);
                    Some((choices, unpacked))
                })
                .unwrap_or_default();
            let owned = fragment_coverage(fragment, structure_ids);
            let choices = fragment
                .signature
                .iter()
                .enumerate()
                .map(|(index, expected)| {
                    if names
                        .get(index)
                        .is_some_and(|name| binders.contains(name))
                    {
                        let name = names[index].clone();
                        let mut binder_choices = vec![Expression {
                            node: IrNode::Parameter {
                                name,
                                ty: expected.clone(),
                            },
                            ty: expected.clone(),
                            fragments: Vec::new(),
                            coverage: BTreeSet::new(),
                            depth: 0,
                            constant_maps: 0,
                            loose: 0,
                            grounding: Vec::new(),
                        }];
                        // A pair element unpacks into the consuming fragment's
                        // own two slot names: `for left, right in
                        // combinations(...)`.
                        for (first, second) in &unpacked_names {
                            binder_choices.push(Expression {
                                node: IrNode::Literal {
                                    text: format!("{first}, {second}"),
                                    ty: expected.clone(),
                                },
                                ty: expected.clone(),
                                fragments: Vec::new(),
                                coverage: BTreeSet::new(),
                                depth: 0,
                                constant_maps: 0,
                                loose: 0,
                                grounding: Vec::new(),
                            });
                        }
                        return binder_choices;
                    }
                    argument_choices(
                        &snapshot,
                        expected,
                        names.get(index).map(String::as_str).unwrap_or(""),
                        index,
                        &parameters,
                        &loop_variable_choices,
                        &owned,
                        &coverable,
                    )
                })
                .collect::<Vec<_>>();
            if choices.iter().any(Vec::is_empty) {
                continue;
            }
            let mut combinations = Vec::new();
            enumerate_arguments(&choices, 0, &mut Vec::new(), &mut combinations, 2_048);
            combinations.sort_by_key(|arguments| {
                (
                    binder_circularity(binders, binder_slot, arguments),
                    Reverse(argument_coherence(arguments)),
                )
            });
            combinations.truncate(2_048);
            for arguments in combinations {
                let result_ty = inferred_expression_type(
                    &fragment.result,
                    &fragment.signature,
                    &arguments,
                );
                let mut fragments = vec![fragment.id.clone()];
                let mut coverage = fragment_coverage(fragment, structure_ids);
                let mut grounding = vec![(fragment.grounding.clone(), fragment.license.clone())];
                for argument in &arguments {
                    fragments.extend(argument.fragments.iter().cloned());
                    coverage.extend(argument.coverage.iter().cloned());
                    grounding.extend(argument.grounding.iter().cloned());
                }
                fragments.sort();
                fragments.dedup();
                grounding.sort();
                grounding.dedup();
                let node = IrNode::Apply {
                    fragment: fragment.id.clone(),
                    arguments: arguments
                        .into_iter()
                        .map(|argument| argument.node)
                        .collect(),
                };
                let actual_depth = node_depth(&node);
                if actual_depth > depth {
                    continue;
                }
                let constant_maps = constant_map_count(&node, catalog, &spec.language);
                let loose = loose_feeds(&node, catalog);
                layer.push(Expression {
                    node,
                    ty: result_ty,
                    fragments,
                    coverage,
                    depth: actual_depth,
                    constant_maps,
                    loose,
                    grounding,
                });
            }
        }
        pool.extend(layer);
        normalize_pool(&mut pool, coverable.len(), 2_048);
    }
    // Root folds are complete programs, not reusable subexpressions. Preserve
    // them outside the bounded subexpression beam so a large compositional
    // neighborhood cannot evict a shallower valid reduction.
    pool.extend(fold_roots);

    let candidates = pool
        .into_iter()
        .filter(|expression| {
            !expression.fragments.is_empty()
                && (coverable.is_empty() || expression.coverage.is_superset(&coverable))
        })
        .collect::<Vec<_>>();
    let mut candidates = candidates
        .into_iter()
        .map(|expression| program_from_expression(spec, parameters.clone(), expression))
        .filter(|candidate| {
            let depth_ok = node_depth(&candidate.body) <= bounds.max_depth;
            let type_ok = candidate.type_check(catalog).is_ok();
            let closed_ok = lowered_candidate_is_closed(spec, catalog, candidate, &placeholder_names);
            depth_ok && type_ok && closed_ok
        })
        .collect::<Vec<_>>();
    let mut recursive = recursive_reduce_programs(
        spec,
        catalog,
        &parameters,
        structure_ids,
        bounds.max_candidates,
    );
    // Truncation is not least-action: least-action selects among the drafts
    // that verify. The bounded candidate budget is spent first on programs
    // that actually read the task's inputs, then on richer compositions —
    // constant-only and literal-stuffed programs cannot generalize and only
    // pass examples by coincidence.
    let parameter_names = parameters
        .iter()
        .map(|(name, _)| name.clone())
        .collect::<BTreeSet<_>>();
    candidates.sort_by_key(|candidate| {
        let mut referenced = BTreeSet::new();
        collect_parameter_names(&candidate.body, &parameter_names, &mut referenced);
        (
            Reverse(referenced.len()),
            candidate.action_cost(),
            Reverse(candidate.fragments.len()),
            candidate.content_id(),
        )
    });
    candidates.dedup_by(|left, right| left.content_id() == right.content_id());
    // Keep a bounded share for each complete root constructor. Otherwise the
    // cheaper subexpression beam can evict every recursive program before the
    // executable examples get a chance to distinguish their semantics.
    let reserved = recursive
        .len()
        .min((bounds.max_candidates / 4).max(usize::from(!recursive.is_empty())));
    candidates.truncate(bounds.max_candidates.saturating_sub(reserved));
    recursive.truncate(reserved);
    candidates.extend(recursive);
    candidates.sort_by_key(|candidate| (candidate.action_cost(), candidate.content_id()));
    candidates
}

fn recursive_reduce_programs(
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

fn prefer_supported<'a>(
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
fn recursive_reduce_program(
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

fn checked_recursive_programs(
    mut programs: Vec<ProgramIr>,
    catalog: &FragmentCatalog,
    limit: usize,
) -> Vec<ProgramIr> {
    programs.retain(|program| program.type_check(catalog).is_ok());
    programs.sort_by_key(|program| (program.action_cost(), program.content_id()));
    programs.dedup_by(|left, right| left.content_id() == right.content_id());
    programs.truncate(limit);
    programs
}

/// Reject a candidate when catalog placeholder atoms survive lowering as free
/// target-language names. The typed frontier intentionally introduces those
/// atoms because a fragment can bind them inside a comprehension or lambda;
/// keeping a candidate where the binding never materialized would turn a
/// type-correct IR into a guaranteed `NameError` at verification time.
fn lowered_candidate_is_closed(
    spec: &CodingTaskSpec,
    catalog: &FragmentCatalog,
    candidate: &ProgramIr,
    placeholder_names: &BTreeSet<String>,
) -> bool {
    let Some(lowering) = crate::coding::ir_lowering::lowering_for(&spec.language) else {
        return false;
    };
    let Ok(source) = lowering.lower(candidate, catalog) else {
        return false;
    };
    if spec.language != "python" {
        // Other backends do not currently interpolate binder-bearing fragment
        // surfaces. Their IR-scoped binders are checked by `type_check`.
        return true;
    }
    let tokens = python_tokens(&source);
    let mut bound = spec
        .parameters
        .iter()
        .map(|parameter| parameter.name.clone())
        .collect::<BTreeSet<_>>();
    for index in 0..tokens.len() {
        match tokens[index].as_str() {
            "for" => collect_bindings_until(&tokens, index + 1, "in", &mut bound),
            "lambda" => collect_bindings_until(&tokens, index + 1, ":", &mut bound),
            _ => {}
        }
    }
    placeholder_names
        .iter()
        .all(|name| bound.contains(name) || !tokens.iter().any(|token| token == name))
}

fn collect_bindings_until(
    tokens: &[String],
    start: usize,
    delimiter: &str,
    output: &mut BTreeSet<String>,
) {
    for token in &tokens[start..] {
        if token == delimiter {
            break;
        }
        if token
            .chars()
            .next()
            .is_some_and(|character| character.is_alphabetic() || character == '_')
        {
            output.insert(token.clone());
        }
    }
}

/// Names a fragment's own surface binds through `for`/`lambda` targets.
///
/// A slot whose placeholder name the fragment itself binds takes the loop
/// variable reference as its argument: the idiom introduces the binding, so
/// the argument must lower to that name, never to a computed value. Scope
/// matters — `{item}` between `for` and `in`, or a lambda parameter
/// placeholder occurring inside the lambda body, is a binding; the same name
/// as an IIFE call argument outside the lambda is an ordinary input slot.
fn fragment_binder_slots(fragment: &Fragment, language: &str) -> BTreeSet<String> {
    let Some(surface) = fragment.realizations.get(language) else {
        return BTreeSet::new();
    };
    let characters: Vec<char> = surface.chars().collect();
    let mut depth = vec![0usize; characters.len() + 1];
    let mut current = 0usize;
    for (index, character) in characters.iter().enumerate() {
        match character {
            '(' | '[' => current += 1,
            ')' | ']' => current = current.saturating_sub(1),
            _ => {}
        }
        depth[index + 1] = current;
    }
    let mut tokens: Vec<(String, usize)> = Vec::new();
    let mut index = 0;
    while index < characters.len() {
        if characters[index].is_alphabetic() || characters[index] == '_' {
            let start = index;
            let mut word = String::new();
            while index < characters.len()
                && (characters[index].is_alphanumeric() || characters[index] == '_')
            {
                word.push(characters[index]);
                index += 1;
            }
            tokens.push((word, start));
            continue;
        }
        index += 1;
    }
    let mut placeholder_hits: Vec<(String, usize)> = Vec::new();
    let mut index = 0;
    while index < characters.len() {
        if characters[index] == '{' && characters.get(index + 1) == Some(&'{') {
            index += 2;
            continue;
        }
        if characters[index] == '{' {
            let mut name = String::new();
            let mut cursor = index + 1;
            while cursor < characters.len() && characters[cursor] != '}' {
                name.push(characters[cursor]);
                cursor += 1;
            }
            if cursor < characters.len() {
                placeholder_hits.push((name, index));
                index = cursor + 1;
                continue;
            }
        }
        index += 1;
    }
    let mut bound = BTreeSet::new();
    for (token_index, (word, start)) in tokens.iter().enumerate() {
        if word == "for" {
            let mut names: Vec<String> = Vec::new();
            let mut span_end = characters.len();
            for (inner, offset) in &tokens[token_index + 1..] {
                if inner == "in" {
                    span_end = *offset;
                    break;
                }
                names.push(inner.clone());
            }
            for (name, offset) in &placeholder_hits {
                if names.contains(name) && *offset >= *start && *offset <= span_end {
                    bound.insert(name.clone());
                }
            }
        } else if word == "lambda" {
            let lambda_depth = depth[*start];
            let mut parameters: Vec<String> = Vec::new();
            let mut body_start = None;
            for (inner, offset) in &tokens[token_index + 1..] {
                if inner == ":" && depth[*offset] == lambda_depth {
                    body_start = Some(*offset);
                    break;
                }
                parameters.push(inner.clone());
            }
            let Some(body_start) = body_start else {
                continue;
            };
            let mut body_end = characters.len();
            for index in body_start..characters.len() {
                if matches!(characters[index], ')' | ']') && depth[index + 1] < lambda_depth {
                    body_end = index;
                    break;
                }
            }
            for (name, offset) in &placeholder_hits {
                if parameters.contains(name) && *offset > body_start && *offset <= body_end {
                    bound.insert(name.clone());
                }
            }
        }
    }
    bound
}

/// One-level applications that read an enclosing comprehension's loop
/// variable.
///
/// A body like `[1 if item == '(' else -1 for item in symbols]` needs a body
/// expression referencing the binder, which no global pool expression can
/// supply — the binder only exists inside the comprehension fragment's own
/// scope. These applies are candidate choices for the comprehension's free
/// slots and never pool members: anywhere else they would lower to an
/// unbound name.
#[allow(clippy::too_many_arguments)]
fn loop_variable_applies<'a>(
    others: impl Iterator<Item = &'a Fragment>,
    pool: &[Expression],
    loop_variable: &str,
    element_type: &IrType,
    parameters: &[(String, IrType)],
    structure_ids: &[String],
    needed: &BTreeSet<String>,
    catalog: &FragmentCatalog,
    language: &str,
) -> Vec<Expression> {
    let binder = Expression {
        node: IrNode::Parameter {
            name: loop_variable.to_owned(),
            ty: element_type.clone(),
        },
        ty: element_type.clone(),
        fragments: Vec::new(),
        coverage: BTreeSet::new(),
        depth: 0,
        constant_maps: 0,
        loose: 0,
        grounding: Vec::new(),
    };
    let mut applies: Vec<Expression> = Vec::new();
    let others: Vec<&Fragment> = others.collect();
    for fragment in &others {
        if fragment.signature.is_empty() {
            continue;
        }
        let names = fragment.argument_names(language);
        let owned = fragment_coverage(fragment, structure_ids);
        for (slot, expected) in fragment.signature.iter().enumerate() {
            if !types_may_unify(expected, element_type) {
                continue;
            }
            let choices = fragment
                .signature
                .iter()
                .enumerate()
                .map(|(index, ty)| {
                    if index == slot {
                        vec![binder.clone()]
                    } else {
                        argument_choices(
                            pool,
                            ty,
                            names.get(index).map(String::as_str).unwrap_or(""),
                            index,
                            parameters,
                            &[],
                            &owned,
                            needed,
                        )
                    }
                })
                .collect::<Vec<_>>();
            if choices.iter().any(Vec::is_empty) {
                continue;
            }
            let mut combinations = Vec::new();
            enumerate_arguments(&choices, 0, &mut Vec::new(), &mut combinations, 64);
            combinations.sort_by_key(|arguments| {
                (
                    binder_circularity(
                        &std::iter::once(loop_variable.to_owned()).collect(),
                        Some(slot),
                        arguments,
                    ),
                    Reverse(argument_coherence(arguments)),
                )
            });
            for arguments in combinations {
                let result_ty = inferred_expression_type(
                    &fragment.result,
                    &fragment.signature,
                    &arguments,
                );
                let mut fragments = vec![fragment.id.clone()];
                let mut coverage = fragment_coverage(fragment, structure_ids);
                let mut grounding = vec![(fragment.grounding.clone(), fragment.license.clone())];
                for argument in &arguments {
                    fragments.extend(argument.fragments.iter().cloned());
                    coverage.extend(argument.coverage.iter().cloned());
                    grounding.extend(argument.grounding.iter().cloned());
                }
                fragments.sort();
                fragments.dedup();
                grounding.sort();
                grounding.dedup();
                let node = IrNode::Apply {
                    fragment: fragment.id.clone(),
                    arguments: arguments
                        .into_iter()
                        .map(|argument| argument.node)
                        .collect(),
                };
                applies.push(Expression {
                    depth: node_depth(&node),
                    constant_maps: constant_map_count(&node, catalog, language),
                    loose: loose_feeds(&node, catalog),
                    node,
                    ty: result_ty,
                    fragments,
                    coverage,
                    grounding,
                });
            }
        }
    }
    let input_names = parameters
        .iter()
        .map(|(name, _)| name.as_str())
        .collect::<Vec<_>>();
    let rank = |applies: &mut Vec<Expression>| {
        applies.sort_by_key(|expression| {
            (
                // Argument-grounded structures come first: a binder apply
                // whose arguments assemble a required structure is the one
                // the enclosing composition needs. A fragment's own supports
                // credit applies to every argument shape, so it cannot make
                // this distinction — and constants that ground a required
                // structure are not the constants the constant penalty is
                // for.
                usize::MAX - argument_grounded_structures(&expression.node, needed),
                usize::from(expression.constant_maps > 0),
                // A binder apply that grounds one of the required structures
                // is the one the enclosing composition needs; a leaner apply
                // merely re-reads the input.
                usize::MAX - expression
                    .coverage
                    .iter()
                    .filter(|id| needed.contains(*id))
                    .count(),
                usize::MAX - expression.coverage.len(),
                expression.fragments.len(),
                expression.depth,
                Reverse(parameter_reads(&expression.node, &input_names)),
                format!("{:?}", expression.node),
            )
        });
        applies.dedup_by(|left, right| left.node == right.node);
        let diversified = diversify(applies, 16, 3);
        *applies = diversified;
    };
    rank(&mut applies);
    // Second level: the binder may also sit one apply below the surface, as
    // in `values_equal(rotate(text, item), text)` — the comprehension body
    // compares a binder-dependent computation against an outer value. The
    // surface apply takes one binder-dependent one-level apply in a single
    // slot; its remaining slots draw plain pool choices.
    let mut two_level = Vec::new();
    for fragment in &others {
        if fragment.signature.is_empty() {
            continue;
        }
        let names = fragment.argument_names(language);
        let owned = fragment_coverage(fragment, structure_ids);
        for (slot, ty) in fragment.signature.iter().enumerate() {
            let binder_dependent = applies
                .iter()
                .filter(|expression| types_may_unify(ty, &expression.ty))
                .cloned()
                .collect::<Vec<_>>();
            if binder_dependent.is_empty() {
                continue;
            }
            let choices = fragment
                .signature
                .iter()
                .enumerate()
                .map(|(index, expected)| {
                    if index == slot {
                        binder_dependent.clone()
                    } else {
                        argument_choices(
                            pool,
                            expected,
                            names.get(index).map(String::as_str).unwrap_or(""),
                            index,
                            parameters,
                            &[],
                            &owned,
                            needed,
                        )
                    }
                })
                .collect::<Vec<_>>();
            if choices.iter().any(Vec::is_empty) {
                continue;
            }
            let mut combinations = Vec::new();
            enumerate_arguments(&choices, 0, &mut Vec::new(), &mut combinations, 8);
            combinations.sort_by_key(|arguments| {
                (
                    binder_circularity(
                        &std::iter::once(loop_variable.to_owned()).collect(),
                        Some(slot),
                        arguments,
                    ),
                    Reverse(argument_coherence(arguments)),
                )
            });
            for arguments in combinations {
                let result_ty = inferred_expression_type(
                    &fragment.result,
                    &fragment.signature,
                    &arguments,
                );
                let mut fragments = vec![fragment.id.clone()];
                let mut coverage = fragment_coverage(fragment, structure_ids);
                let mut grounding = vec![(fragment.grounding.clone(), fragment.license.clone())];
                for argument in &arguments {
                    fragments.extend(argument.fragments.iter().cloned());
                    coverage.extend(argument.coverage.iter().cloned());
                    grounding.extend(argument.grounding.iter().cloned());
                }
                fragments.sort();
                fragments.dedup();
                grounding.sort();
                grounding.dedup();
                let node = IrNode::Apply {
                    fragment: fragment.id.clone(),
                    arguments: arguments
                        .into_iter()
                        .map(|argument| argument.node)
                        .collect(),
                };
                two_level.push(Expression {
                    depth: node_depth(&node),
                    constant_maps: constant_map_count(&node, catalog, language),
                    loose: loose_feeds(&node, catalog),
                    node,
                    ty: result_ty,
                    fragments,
                    coverage,
                    grounding,
                });
            }
        }
    }
    applies.extend(two_level);
    rank(&mut applies);
    applies
}

/// Pair-destructuring loop applies.
///
/// When the iterated element is a `pair<A, B>`, a consuming fragment may take
/// the two components through two consecutive slots; the comprehension target
/// then unpacks both slot names (`for left, right in combinations(...)`).
/// Returns the applies together with the name pairs actually used, so the
/// binder slot can offer the matching unpacked target.
#[allow(clippy::too_many_arguments)]
fn pair_loop_variable_applies<'a>(
    others: impl Iterator<Item = &'a Fragment>,
    pool: &[Expression],
    element_types: &[IrType],
    parameters: &[(String, IrType)],
    structure_ids: &[String],
    needed: &BTreeSet<String>,
    catalog: &FragmentCatalog,
    language: &str,
) -> (Vec<Expression>, Vec<(String, String)>) {
    let pair_types = element_types
        .iter()
        .filter_map(|ty| match ty {
            IrType::Pair(left, right) => Some((left.as_ref().clone(), right.as_ref().clone())),
            _ => None,
        })
        .collect::<Vec<_>>();
    let mut unpacked: Vec<(String, String)> = Vec::new();
    let mut applies: Vec<Expression> = Vec::new();
    if pair_types.is_empty() {
        return (applies, unpacked);
    }
    for fragment in others {
        if fragment.signature.len() < 2 {
            continue;
        }
        let names = fragment.argument_names(language);
        let owned = fragment_coverage(fragment, structure_ids);
        for slot in 0..fragment.signature.len() - 1 {
            if !pair_types.iter().any(|(left, right)| {
                types_may_unify(&fragment.signature[slot], left)
                    && types_may_unify(&fragment.signature[slot + 1], right)
            }) {
                continue;
            };
            let Some(first) = names.get(slot) else {
                continue;
            };
            let Some(second) = names.get(slot + 1) else {
                continue;
            };
            if first.is_empty() || second.is_empty() {
                continue;
            }
            let component = |name: &str, ty: &IrType| Expression {
                node: IrNode::Parameter {
                    name: name.to_owned(),
                    ty: ty.clone(),
                },
                ty: ty.clone(),
                fragments: Vec::new(),
                coverage: BTreeSet::new(),
                depth: 0,
                constant_maps: 0,
                loose: 0,
                grounding: Vec::new(),
            };
            let choices = fragment
                .signature
                .iter()
                .enumerate()
                .map(|(index, ty)| {
                    if index == slot {
                        vec![component(first, ty)]
                    } else if index == slot + 1 {
                        vec![component(second, ty)]
                    } else {
                        argument_choices(
                            pool,
                            ty,
                            names.get(index).map(String::as_str).unwrap_or(""),
                            index,
                            parameters,
                            &[],
                            &owned,
                            needed,
                        )
                    }
                })
                .collect::<Vec<_>>();
            if choices.iter().any(Vec::is_empty) {
                continue;
            }
            let mut combinations = Vec::new();
            enumerate_arguments(&choices, 0, &mut Vec::new(), &mut combinations, 8);
            combinations.sort_by_key(|arguments| Reverse(argument_coherence(arguments)));
            let pair_names = (first.clone(), second.clone());
            for arguments in combinations {
                let result_ty = inferred_expression_type(
                    &fragment.result,
                    &fragment.signature,
                    &arguments,
                );
                let mut fragments = vec![fragment.id.clone()];
                let mut coverage = fragment_coverage(fragment, structure_ids);
                let mut grounding = vec![(fragment.grounding.clone(), fragment.license.clone())];
                for argument in &arguments {
                    fragments.extend(argument.fragments.iter().cloned());
                    coverage.extend(argument.coverage.iter().cloned());
                    grounding.extend(argument.grounding.iter().cloned());
                }
                fragments.sort();
                fragments.dedup();
                grounding.sort();
                grounding.dedup();
                let node = IrNode::Apply {
                    fragment: fragment.id.clone(),
                    arguments: arguments
                        .into_iter()
                        .map(|argument| argument.node)
                        .collect(),
                };
                applies.push(Expression {
                    depth: node_depth(&node),
                    constant_maps: constant_map_count(&node, catalog, language),
                    loose: loose_feeds(&node, catalog),
                    node,
                    ty: result_ty,
                    fragments,
                    coverage,
                    grounding,
                });
            }
            if !applies.is_empty() && !unpacked.contains(&pair_names) {
                unpacked.push(pair_names);
            }
        }
    }
    (applies, unpacked)
}

/// How many DISTINCT task inputs the expression reads anywhere in its tree.
/// A nested re-read of the same input adds no grounding beyond the first, so
/// deeply tangled assemblies gain no ranking advantage over a plain input.
fn parameter_reads(node: &IrNode, names: &[&str]) -> usize {
    let owned: BTreeSet<String> = names.iter().map(|name| (*name).to_owned()).collect();
    let mut into = BTreeSet::new();
    collect_parameter_names(node, &owned, &mut into);
    into.len()
}

fn collect_parameter_names(node: &IrNode, names: &BTreeSet<String>, into: &mut BTreeSet<String>) {
    fn walk(
        node: &IrNode,
        names: &BTreeSet<String>,
        bound: &BTreeSet<String>,
        into: &mut BTreeSet<String>,
    ) {
        match node {
            IrNode::Parameter { name, .. } => {
                if names.contains(name) && !bound.contains(name) {
                    into.insert(name.clone());
                }
            }
            IrNode::Literal { .. } => {}
            IrNode::Apply { arguments, .. } => {
                for argument in arguments {
                    walk(argument, names, bound, into);
                }
            }
            IrNode::Each {
                item,
                items,
                body,
                predicate,
            } => {
                walk(items, names, bound, into);
                let mut inner = bound.clone();
                inner.insert(item.clone());
                walk(body, names, &inner, into);
                if let Some(predicate) = predicate {
                    walk(predicate, names, &inner, into);
                }
            }
            IrNode::Fold {
                item,
                accumulator,
                items,
                initial,
                body,
            } => {
                walk(items, names, bound, into);
                walk(initial, names, bound, into);
                let mut inner = bound.clone();
                inner.insert(item.clone());
                inner.insert(accumulator.clone());
                walk(body, names, &inner, into);
            }
            IrNode::Repeat {
                counter,
                from,
                to,
                body,
            } => {
                walk(from, names, bound, into);
                walk(to, names, bound, into);
                let mut inner = bound.clone();
                inner.insert(counter.clone());
                walk(body, names, &inner, into);
            }
            IrNode::Recurrence {
                state,
                base,
                transition,
                index,
            } => {
                for value in base {
                    walk(value, names, bound, into);
                }
                let mut inner = bound.clone();
                inner.extend(state.iter().cloned());
                walk(transition, names, &inner, into);
                walk(index, names, &inner, into);
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
                ..
            } => {
                for value in target {
                    walk(value, names, bound, into);
                }
                walk(items, names, bound, into);
                let mut inner = bound.clone();
                inner.extend(state.iter().cloned());
                inner.extend(item.iter().cloned());
                for value in next {
                    walk(value, names, &inner, into);
                }
                if let Some(admissible) = admissible {
                    walk(admissible, names, &inner, into);
                }
                walk(base_test, names, &inner, into);
                walk(base, names, &inner, into);
                walk(local, names, &inner, into);
            }
            IrNode::Condition {
                test,
                then_branch,
                else_branch,
            } => {
                walk(test, names, bound, into);
                walk(then_branch, names, bound, into);
                walk(else_branch, names, bound, into);
            }
            IrNode::Bind { name, value, body } => {
                walk(value, names, bound, into);
                let mut inner = bound.clone();
                inner.insert(name.clone());
                walk(body, names, &inner, into);
            }
            IrNode::Emit { value } | IrNode::Return { value } => {
                walk(value, names, bound, into);
            }
        }
    }
    walk(node, names, &BTreeSet::new(), into);
}

/// Minimal Python lexical scan for identifier scope. String contents are
/// deliberately skipped, so a quoted example cannot accidentally bind or
/// invalidate a placeholder with the same spelling.
fn python_tokens(source: &str) -> Vec<String> {
    let mut tokens = Vec::new();
    let mut characters = source.chars().peekable();
    while let Some(character) = characters.next() {
        if matches!(character, '\'' | '"') {
            let quote = character;
            let mut escaped = false;
            for character in characters.by_ref() {
                if escaped {
                    escaped = false;
                } else if character == '\\' {
                    escaped = true;
                } else if character == quote {
                    break;
                }
            }
        } else if character.is_alphabetic() || character == '_' {
            let mut token = String::from(character);
            while characters
                .peek()
                .is_some_and(|next| next.is_alphanumeric() || *next == '_')
            {
                token.push(characters.next().expect("peeked identifier character"));
            }
            tokens.push(token);
        } else if matches!(character, ':' | ',') {
            tokens.push(character.to_string());
        }
    }
    tokens
}

#[derive(Debug, Clone)]
struct Expression {
    node: IrNode,
    ty: IrType,
    fragments: Vec<String>,
    coverage: BTreeSet<String>,
    depth: usize,
    /// How many comprehensions inside this expression map a constant: a body
    /// that never reads its binder cannot depend on the iteration, so such an
    /// expression is filler for any requirement about the individual items.
    constant_maps: usize,
    /// How many slots anywhere in this tree were filled with a text value
    /// where the fragment demands a sequence of a concrete non-text element.
    /// Iterating text as characters is free (the element is unknown or text);
    /// feeding raw text where integers are expected is a guess about what the
    /// iteration produces, and guesses rank below grounded assemblies.
    loose: usize,
    grounding: Vec<(String, String)>,
}

fn atom_expressions(
    spec: &CodingTaskSpec,
    parameters: &[(String, IrType)],
    fragments: &[&Fragment],
    structure_ids: &[String],
) -> Vec<Expression> {
    let mut atoms = parameters
        .iter()
        .map(|(name, ty)| Expression {
            node: IrNode::Parameter {
                name: name.clone(),
                ty: ty.clone(),
            },
            ty: ty.clone(),
            fragments: Vec::new(),
            coverage: BTreeSet::new(),
            depth: 1,
            constant_maps: 0,
            loose: 0,
            grounding: Vec::new(),
        })
        .collect::<Vec<_>>();
    for fragment in fragments {
        if fragment.signature.is_empty() {
            atoms.push(Expression {
                node: IrNode::Apply {
                    fragment: fragment.id.clone(),
                    arguments: Vec::new(),
                },
                ty: fragment.result.clone(),
                fragments: vec![fragment.id.clone()],
                coverage: fragment_coverage(fragment, structure_ids),
                depth: 1,
                constant_maps: 0,
                loose: 0,
                grounding: vec![(fragment.grounding.clone(), fragment.license.clone())],
            });
        }
    }
    atoms.extend(discovered_literals(spec));
    normalize_pool(&mut atoms, 0, 512);
    atoms
}

fn discovered_literals(spec: &CodingTaskSpec) -> Vec<Expression> {
    let mut literals = Vec::new();
    for value in 0..=8 {
        let text = value.to_string();
        literals.push(literal(&text, IrType::Integer));
        literals.push(literal(&text, IrType::Float));
    }
    let prose = spec.requirement_sentences.join(" ");
    for token in prose.split(|character: char| !character.is_ascii_digit() && character != '.') {
        if token.is_empty() {
            continue;
        }
        if token.parse::<u64>().is_ok() {
            literals.push(literal(token, IrType::Integer));
        } else if token.parse::<f64>().is_ok() {
            literals.push(literal(token, IrType::Float));
        }
    }
    let quoted = quoted_literals(&prose);
    for value in &quoted {
        let serialized = serde_json::to_string(value).expect("text literal serializes");
        literals.push(literal(&serialized, IrType::Text));
    }
    // Deliberately no atoms from the examples' expected outputs: an expected
    // value is the answer, not available data — composing with it would be
    // memorization of the example, not derivation from the requirement.
    if quoted.len() >= 3 {
        let fields = quoted
            .iter()
            .enumerate()
            .map(|(index, value)| {
                format!(
                    "{}: {index}",
                    serde_json::to_string(value).expect("text literal serializes")
                )
            })
            .collect::<Vec<_>>()
            .join(", ");
        literals.push(literal(
            &format!("{{{fields}}}"),
            IrType::Mapping(Box::new(IrType::Text), Box::new(IrType::Integer)),
        ));
    }
    let associations = quoted
        .iter()
        .enumerate()
        .filter_map(|(index, value)| {
            let start = prose.find(value)? + value.len();
            let end = quoted
                .get(index + 1)
                .and_then(|next| prose[start..].find(next).map(|offset| start + offset))
                .unwrap_or(prose.len());
            let number = prose[start..end]
                .split(|character: char| !character.is_ascii_digit())
                .find(|part| !part.is_empty())?;
            Some(format!(
                "{}: {number}",
                serde_json::to_string(value).expect("text literal serializes")
            ))
        })
        .collect::<Vec<_>>();
    if associations.len() >= 2 {
        literals.push(literal(
            &format!("{{{}}}", associations.join(", ")),
            IrType::Mapping(Box::new(IrType::Text), Box::new(IrType::Integer)),
        ));
    }
    literals
}

fn literal(text: &str, ty: IrType) -> Expression {
    Expression {
        node: IrNode::Literal {
            text: text.to_owned(),
            ty: ty.clone(),
        },
        ty,
        fragments: Vec::new(),
        coverage: BTreeSet::new(),
        depth: 1,
        constant_maps: 0,
        loose: 0,
        grounding: Vec::new(),
    }
}

fn quoted_literals(text: &str) -> Vec<String> {
    let mut values = Vec::new();
    let mut opening: Option<(char, usize)> = None;
    for (index, character) in text.char_indices() {
        if !matches!(character, '\'' | '"') {
            continue;
        }
        if let Some((delimiter, start)) = opening {
            if delimiter == character {
                values.push(text[start + delimiter.len_utf8()..index].to_owned());
                opening = None;
            }
        } else {
            opening = Some((character, index));
        }
    }
    values
}

#[allow(clippy::too_many_arguments)]
fn argument_choices(
    pool: &[Expression],
    expected: &IrType,
    slot: &str,
    index: usize,
    parameters: &[(String, IrType)],
    extra: &[Expression],
    owned: &BTreeSet<String>,
    needed: &BTreeSet<String>,
) -> Vec<Expression> {
    let mut choices = pool
        .iter()
        .filter(|expression| types_may_unify(expected, &expression.ty))
        .cloned()
        .collect::<Vec<_>>();
    choices.extend(
        extra
            .iter()
            .filter(|expression| types_may_unify(expected, &expression.ty))
            .cloned(),
    );
    let input_names = parameters
        .iter()
        .map(|(name, _)| name.as_str())
        .collect::<Vec<_>>();
    // Inside a comprehension, the non-container slots are the body: they can
    // only compute something general by reading the loop variable, which only
    // the binder-dependent extras provide. A pool expression there would map
    // a constant over the input. The container slot itself is not a body — it
    // keeps the plain pool ranking.
    let extras_first = !extra.is_empty() && !matches!(expected, IrType::Sequence(_));
    let extra_nodes = extra
        .iter()
        .map(|expression| format!("{:?}", expression.node))
        .collect::<std::collections::BTreeSet<_>>();
    choices.sort_by_key(|expression| {
        let is_extra = usize::from(!extra_nodes.contains(&format!("{:?}", expression.node)));
        let exact_name = match &expression.node {
            IrNode::Parameter { name, .. } => usize::from(name != slot),
            _ => 1,
        };
        let positional = match &expression.node {
            IrNode::Parameter { name, .. } => usize::from(
                parameters
                    .get(index)
                    .is_none_or(|(parameter, _)| parameter != name),
            ),
            _ => 1,
        };
        // A candidate that reads the task's inputs stays general over the
        // examples; constants can only agree with them by coincidence. A
        // choice that adds structure coverage the enclosing fragment does
        // not already provide is the one that can complete a composition:
        // required structures count first, then total structure beyond the
        // enclosing fragment's own — a grounded assembly integrates more of
        // the discovered sources than a minimal chain that merely re-wraps
        // the input.
        let novel = expression
            .coverage
            .iter()
            .filter(|id| needed.contains(*id) && !owned.contains(*id))
            .count();
        let key = (
            (
                (
                    if extras_first { is_extra } else { 0 },
                    usize::from(expression.constant_maps > 0),
                    expression.loose,
                    usize::from(pathological_repeat(&expression.node)),
                    usize::from(!expression.fragments.is_empty()),
                    usize::from(!types_fit_strictly(expected, &expression.ty)),
                    usize::MAX - novel,
                    usize::MAX - expression
                        .coverage
                        .iter()
                        .filter(|id| needed.contains(*id))
                        .count(),
                    usize::MAX - expression
                        .coverage
                        .iter()
                        .filter(|id| !owned.contains(*id))
                        .count(),
                    Reverse(parameter_reads(&expression.node, &input_names)),
                ),
                (
                    exact_name,
                    positional,
                    usize::MAX - expression.fragments.len(),
                    expression.depth,
                ),
            ),
            format!("{:?}", expression.node),
        );
        key
    });
    choices.dedup_by(|left, right| left.node == right.node);
    diversify(&choices, 32, 12)
}

/// Counts degenerate comprehensions: a body that never reads its binder (a
/// constant map such as `[0 for item in items]`) and an iterable that reads
/// its own binder (circular, such as feeding `slice(values, item)` back as
/// the iterable of the very comprehension binding `item`). Both are well
/// typed but cannot depend on the iteration, so they are filler in any
/// window over individual items.
fn constant_map_count(node: &IrNode, catalog: &FragmentCatalog, language: &str) -> usize {
    match node {
        IrNode::Apply { fragment, arguments } => {
            let mut count = 0;
            if let Some(definition) = catalog.get(fragment) {
                let names = definition.argument_names(language);
                let binders = fragment_binder_slots(definition, language);
                for (binder_index, binder) in names.iter().enumerate() {
                    if !binders.contains(binder) {
                        continue;
                    }
                    for (slot, expected) in definition.signature.iter().enumerate() {
                        if slot == binder_index {
                            continue;
                        }
                        let Some(argument) = arguments.get(slot) else {
                            continue;
                        };
                        if matches!(expected, IrType::Sequence(_)) {
                            if references_parameter(argument, binder) {
                                count += 1;
                            }
                        } else if !references_parameter(argument, binder) {
                            count += 1;
                        }
                    }
                }
            }
            count += arguments
                .iter()
                .map(|argument| constant_map_count(argument, catalog, language))
                .sum::<usize>();
            count
        }
        // The search pool only ever holds Apply trees over atoms.
        _ => 0,
    }
}

fn references_parameter(node: &IrNode, name: &str) -> bool {
    match node {
        IrNode::Parameter { name: candidate, .. } => candidate == name,
        IrNode::Apply { arguments, .. } => arguments
            .iter()
            .any(|argument| references_parameter(argument, name)),
        _ => false,
    }
}

/// The type an argument tree presents at its root, for loose-feed counting.
fn argument_root_type(node: &IrNode, catalog: &FragmentCatalog) -> Option<IrType> {
    match node {
        IrNode::Parameter { ty, .. } | IrNode::Literal { ty, .. } => Some(ty.clone()),
        IrNode::Apply { fragment, .. } => {
            catalog.get(fragment).map(|definition| definition.result.clone())
        }
        _ => None,
    }
}

/// Whether one slot was filled with a value that only fits through the
/// text-over-sequence leniency while the element type is concrete: a text
/// cannot promise to yield those elements, so the assembly is a guess.
fn loose_feed(expected: &IrType, argument: &IrNode, catalog: &FragmentCatalog) -> bool {
    let Some(actual) = argument_root_type(argument, catalog) else {
        return false;
    };
    if types_fit_strictly(expected, &actual) {
        return false;
    }
    let IrType::Sequence(element) = expected else {
        return false;
    };
    matches!(actual, IrType::Text)
        && !matches!(element.as_ref(), IrType::Unknown(_) | IrType::Text)
}

/// Resolves a fragment's schematic result against the argument types the
/// combination actually provides. A schematic result such as
/// `sequence<unknown:1>` leaves the element open, which lets a body that
/// produces a sequence masquerade as a flat sequence in every later window;
/// binding the signature's variables to the observed argument types records
/// what the assembly really yields.
fn inferred_expression_type(
    result: &IrType,
    signature: &[IrType],
    arguments: &[Expression],
) -> IrType {
    let mut bindings: std::collections::BTreeMap<usize, IrType> = std::collections::BTreeMap::new();
    for (slot, expected) in signature.iter().enumerate() {
        let Some(argument) = arguments.get(slot) else {
            continue;
        };
        collect_variable_bindings(expected, &argument.ty, &mut bindings);
    }
    if bindings.is_empty() {
        return result.clone();
    }
    apply_variable_bindings(result, &bindings)
}

/// Whether a type still names open schematic variables: an assembly whose
/// result type is unresolved promises less than one whose element types are
/// concrete, and retention keeps the concrete promise first.
fn type_has_open_variables(ty: &IrType) -> bool {
    match ty {
        IrType::Unknown(_) => true,
        IrType::Sequence(element) => type_has_open_variables(element),
        IrType::Pair(left, right) | IrType::Mapping(left, right) => {
            type_has_open_variables(left) || type_has_open_variables(right)
        }
        _ => false,
    }
}

fn collect_variable_bindings(
    expected: &IrType,
    observed: &IrType,
    bindings: &mut std::collections::BTreeMap<usize, IrType>,
) {
    match (expected, observed) {
        (IrType::Sequence(expected_element), IrType::Sequence(observed_element)) => {
            collect_variable_bindings(expected_element, observed_element, bindings);
        }
        (
            IrType::Pair(expected_left, expected_right),
            IrType::Pair(observed_left, observed_right),
        ) => {
            collect_variable_bindings(expected_left, observed_left, bindings);
            collect_variable_bindings(expected_right, observed_right, bindings);
        }
        (
            IrType::Mapping(expected_key, expected_value),
            IrType::Mapping(observed_key, observed_value),
        ) => {
            collect_variable_bindings(expected_key, observed_key, bindings);
            collect_variable_bindings(expected_value, observed_value, bindings);
        }
        (IrType::Unknown(id), observed) => {
            bindings.insert(*id, observed.clone());
        }
        _ => {}
    }
}

fn apply_variable_bindings(
    ty: &IrType,
    bindings: &std::collections::BTreeMap<usize, IrType>,
) -> IrType {
    match ty {
        IrType::Unknown(id) => bindings.get(id).cloned().unwrap_or_else(|| ty.clone()),
        IrType::Sequence(element) => {
            IrType::Sequence(Box::new(apply_variable_bindings(element, bindings)))
        }
        IrType::Pair(left, right) => IrType::Pair(
            Box::new(apply_variable_bindings(left, bindings)),
            Box::new(apply_variable_bindings(right, bindings)),
        ),
        IrType::Mapping(key, value) => IrType::Mapping(
            Box::new(apply_variable_bindings(key, bindings)),
            Box::new(apply_variable_bindings(value, bindings)),
        ),
        other => other.clone(),
    }
}

/// Counts loose feeds over a whole tree, so assemblies built on a guess carry
/// that guess with them wherever they are later slotted.
fn loose_feeds(node: &IrNode, catalog: &FragmentCatalog) -> usize {
    match node {
        IrNode::Apply { fragment, arguments } => {
            let mut count = 0;
            if let Some(definition) = catalog.get(fragment) {
                for (slot, expected) in definition.signature.iter().enumerate() {
                    let Some(argument) = arguments.get(slot) else {
                        continue;
                    };
                    if loose_feed(expected, argument, catalog) {
                        count += 1;
                    }
                }
            }
            count += arguments
                .iter()
                .map(|argument| loose_feeds(argument, catalog))
                .sum::<usize>();
            count
        }
        _ => 0,
    }
}

/// A type identity that treats unknown variables as one class, so the
/// diversity quota groups `sequence<unknown:0>` with `sequence<unknown:1>`.
fn type_key(ty: &IrType) -> String {
    match ty {
        IrType::Unknown(_) => "_".to_owned(),
        IrType::Sequence(inner) => format!("sequence<{}>", type_key(inner)),
        IrType::Pair(left, right) => format!("pair<{}, {}>", type_key(left), type_key(right)),
        IrType::Mapping(key, value) => {
            format!("mapping<{}, {}>", type_key(key), type_key(value))
        }
        other => format!("{other:?}"),
    }
}

/// Keeps a bounded choice window from being monopolized by one prolific kind
/// of expression: at most `per_type` entries per distinct class, `limit`
/// entries overall, preserving the incoming order. The class separates the
/// three ways a slot can be filled — reading an input, holding a constant,
/// applying a fragment — because input-reading applications otherwise crowd
/// every window of their result type and constants (each value a distinct
/// atom with its own meaning) become unreachable exactly when a task needs
/// one, such as a divisor or a minimum length. Self-nested chains of one
/// fragment form a class of their own: `f(f(...))` is a genuinely different
/// (and usually degenerate) shape from a single `f`, and without the split a
/// few chain levels crowd the grounded single applications out of the very
/// windows they need for their next composition step.
fn diversity_class(expression: &Expression) -> String {
    match &expression.node {
        IrNode::Literal { text, .. } => format!("literal {}", text),
        IrNode::Parameter { .. } => format!("parameter {}", type_key(&expression.ty)),
        IrNode::Apply { fragment, .. } => format!(
            "apply {} {}{}",
            fragment,
            type_key(&expression.ty),
            if repeats_a_fragment(&expression.node) {
                " recursive"
            } else {
                ""
            }
        ),
        _ => format!("apply {}", type_key(&expression.ty)),
    }
}

/// True when a fragment is applied directly to its own output — the
/// self-nesting filler pattern (`accumulate(accumulate(values, add), add)`) —
/// or when any single fragment appears three or more times. Repetition of a
/// fragment exactly twice through *other* fragments
/// (`reverse(remove-first(reverse(...)))`, removing a trailing marker) is
/// legitimate composition, not filler.
fn pathological_repeat(node: &IrNode) -> bool {
    fn walk(node: &IrNode, counts: &mut std::collections::BTreeMap<String, usize>) -> bool {
        match node {
            IrNode::Apply { fragment, arguments } => {
                let count = counts.entry(fragment.clone()).or_insert(0);
                *count += 1;
                if *count >= 3 {
                    return true;
                }
                arguments.iter().any(|argument| {
                    matches!(argument,
                        IrNode::Apply { fragment: inner, .. } if inner == fragment)
                        || walk(argument, counts)
                })
            }
            _ => false,
        }
    }
    let mut counts = std::collections::BTreeMap::new();
    walk(node, &mut counts)
}

/// Whether any fragment id is applied more than once in the tree.
fn repeats_a_fragment(node: &IrNode) -> bool {
    fn count(node: &IrNode, counts: &mut std::collections::BTreeMap<String, usize>) {
        match node {
            IrNode::Apply { fragment, arguments } => {
                *counts.entry(fragment.clone()).or_insert(0) += 1;
                for argument in arguments {
                    count(argument, counts);
                }
            }
            _ => {}
        }
    }
    let mut counts = std::collections::BTreeMap::new();
    count(node, &mut counts);
    counts.into_values().any(|count| count > 1)
}

fn diversify(sorted: &[Expression], limit: usize, per_type: usize) -> Vec<Expression> {
    let mut kept: Vec<Expression> = Vec::new();
    let mut per_result_type: std::collections::BTreeMap<String, usize> =
        std::collections::BTreeMap::new();
    for expression in sorted {
        if kept.len() >= limit {
            break;
        }
        let key = diversity_class(expression);
        let count = per_result_type.entry(key).or_insert(0);
        if *count >= per_type {
            continue;
        }
        *count += 1;
        kept.push(expression.clone());
    }
    kept
}

fn enumerate_arguments(
    choices: &[Vec<Expression>],
    index: usize,
    current: &mut Vec<Expression>,
    output: &mut Vec<Vec<Expression>>,
    limit: usize,
) {    if output.len() >= limit {
        return;
    }
    if index == choices.len() {
        output.push(current.clone());
        return;
    }
    for choice in &choices[index] {
        current.push(choice.clone());
        enumerate_arguments(choices, index + 1, current, output, limit);
        current.pop();
        if output.len() >= limit {
            break;
        }
    }
}

/// How many non-binder slots of one assembled application read a name the
/// application itself binds — an iterable such as `values[item]` feeding the
/// very comprehension that binds `item` is circular and cannot depend on any
/// real iteration. Such combos rank below every grounded one.
/// Structures the apply's own arguments ground, as opposed to the credit the
/// consuming fragment claims for itself. A fragment may declare it supports a
/// structure for any of its argument shapes (`membership` implements the vowel
/// class when the collection is the vowel set), so blanket supports credit
/// cannot tell `character in vowels` from `character in text` — only the
/// argument-side fragments can.
fn argument_grounded_structures(node: &IrNode, needed: &BTreeSet<String>) -> usize {
    let IrNode::Apply { arguments, .. } = node else {
        return 0;
    };
    let mut grounded = BTreeSet::new();
    for argument in arguments {
        let mut stack = vec![argument];
        while let Some(current) = stack.pop() {
            if let IrNode::Apply { fragment, arguments } = current {
                if needed.contains(fragment) {
                    grounded.insert(fragment.clone());
                }
                stack.extend(arguments);
            }
        }
    }
    grounded.len()
}

fn binder_circularity(
    binders: &BTreeSet<String>,
    binder_slot: Option<usize>,
    arguments: &[Expression],
) -> usize {
    arguments
        .iter()
        .enumerate()
        .filter(|(index, _)| Some(*index) != binder_slot)
        .filter(|(_, expression)| {
            binders
                .iter()
                .any(|name| node_contains_parameter(&expression.node, name))
        })
        .count()
}


fn argument_coherence(arguments: &[Expression]) -> usize {
    arguments
        .iter()
        .enumerate()
        .filter_map(|(index, expression)| match &expression.node {
            IrNode::Parameter { name, .. } => Some((index, name)),
            _ => None,
        })
        .map(|(index, name)| {
            arguments
                .iter()
                .enumerate()
                .filter(|(other, expression)| {
                    *other != index && node_contains_parameter(&expression.node, name)
                })
                .count()
        })
        .sum()
}

fn node_contains_parameter(node: &IrNode, sought: &str) -> bool {
    match node {
        IrNode::Parameter { name, .. } => name == sought,
        IrNode::Literal { .. } => false,
        IrNode::Apply { arguments, .. } => arguments
            .iter()
            .any(|argument| node_contains_parameter(argument, sought)),
        IrNode::Each {
            items,
            body,
            predicate,
            ..
        } => {
            node_contains_parameter(items, sought)
                || node_contains_parameter(body, sought)
                || predicate
                    .as_deref()
                    .is_some_and(|node| node_contains_parameter(node, sought))
        }
        IrNode::Fold {
            items,
            initial,
            body,
            ..
        } => {
            node_contains_parameter(items, sought)
                || node_contains_parameter(initial, sought)
                || node_contains_parameter(body, sought)
        }
        IrNode::Repeat { from, to, body, .. } => {
            node_contains_parameter(from, sought)
                || node_contains_parameter(to, sought)
                || node_contains_parameter(body, sought)
        }
        IrNode::Recurrence {
            base,
            transition,
            index,
            ..
        } => {
            base.iter()
                .any(|node| node_contains_parameter(node, sought))
                || node_contains_parameter(transition, sought)
                || node_contains_parameter(index, sought)
        }
        IrNode::RecursiveReduce {
            target,
            items,
            next,
            admissible,
            base_test,
            base,
            local,
            ..
        } => {
            target
                .iter()
                .any(|node| node_contains_parameter(node, sought))
                || node_contains_parameter(items, sought)
                || next
                    .iter()
                    .any(|node| node_contains_parameter(node, sought))
                || admissible
                    .as_deref()
                    .is_some_and(|node| node_contains_parameter(node, sought))
                || node_contains_parameter(base_test, sought)
                || node_contains_parameter(base, sought)
                || node_contains_parameter(local, sought)
        }
        IrNode::Condition {
            test,
            then_branch,
            else_branch,
        } => {
            node_contains_parameter(test, sought)
                || node_contains_parameter(then_branch, sought)
                || node_contains_parameter(else_branch, sought)
        }
        IrNode::Bind { value, body, .. } => {
            node_contains_parameter(value, sought) || node_contains_parameter(body, sought)
        }
        IrNode::Emit { value } | IrNode::Return { value } => node_contains_parameter(value, sought),
    }
}

fn fragment_coverage(fragment: &Fragment, requested: &[String]) -> BTreeSet<String> {
    requested
        .iter()
        .filter(|id| {
            fragment.id == id.as_str() || fragment.supports.iter().any(|item| item == id.as_str())
        })
        .cloned()
        .collect()
}

fn normalize_pool(pool: &mut Vec<Expression>, requested: usize, limit: usize) {
    pool.sort_by_key(|expression| {
        (
            // Atoms carry no coverage but are the inputs every composition
            // grounds in; they survive retention first.
            usize::from(!expression.fragments.is_empty()),
            requested.saturating_sub(expression.coverage.len()),
            usize::from(expression.constant_maps > 0),
            usize::from(type_has_open_variables(&expression.ty)),
            expression.fragments.len(),
            expression.depth,
            format!("{:?}", expression.node),
        )
    });
    pool.dedup_by(|left, right| left.node == right.node);
    // Least action alone would let one composition family fill the whole
    // pool: minimal re-wraps of one root fragment are countless, while the
    // assemblies rooted at another fragment are few but carry the structures
    // a later depth needs. Retention reserves room for every structure class
    // — the same classes the per-slot windows sample — so no family is
    // crowded out before its descendants can form.
    // Round-robin across classes in sorted order: every family keeps its
    // best entries before any family hordes the budget, so a chain rooted at
    // a rare fragment survives even when common families are countless.
    let mut buckets: std::collections::BTreeMap<String, Vec<Expression>> =
        std::collections::BTreeMap::new();
    let mut kept: Vec<Expression> = Vec::with_capacity(limit);
    for expression in pool.drain(..) {
        if kept.len() < limit && expression.fragments.is_empty() {
            kept.push(expression);
            continue;
        }
        let class = diversity_class(&expression);
        buckets.entry(class).or_default().push(expression);
    }
    for bucket in buckets.values_mut() {
        bucket.reverse();
    }
    let round: Vec<String> = buckets.keys().cloned().collect();
    'rounds: for _ in 0..limit {
        for class in &round {
            let Some(bucket) = buckets.get_mut(class) else {
                continue;
            };
            let Some(expression) = bucket.pop() else {
                continue;
            };
            kept.push(expression);
            if kept.len() >= limit {
                break 'rounds;
            }
        }
    }
    *pool = kept;
}

fn program_from_expression(
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

fn search_program(
    spec: &CodingTaskSpec,
    catalog: &FragmentCatalog,
    bounds: SearchBounds,
    structure_ids: &[String],
) -> Vec<ProgramIr> {
    let mut programs = Vec::new();
    let Some(print) = catalog.get("print_stdout") else {
        return programs;
    };
    if structure_ids.iter().any(|id| id == "print_stdout")
        && let Some(expected) = &spec.expected_stdout
        && let Ok(text) = serde_json::to_string(expected)
    {
        programs.push(ProgramIr {
            name: String::new(),
            parameters: Vec::new(),
            result: IrType::Text,
            body: IrNode::Apply {
                fragment: print.id.clone(),
                arguments: vec![IrNode::Literal {
                    text,
                    ty: IrType::Text,
                }],
            },
            fragments: vec![print.id.clone()],
            source_urls: vec![print.grounding.clone()],
            source_licenses: vec![print.license.clone()],
            reuse: print.reuse,
        });
    }
    if structure_ids.iter().any(|id| id == "range_inclusive")
        && let Some(range) = catalog.get("range_inclusive")
    {
        for bound in discovered_literals(spec)
            .into_iter()
            .filter(|literal| literal.ty == IrType::Integer)
            .take(bounds.max_candidates)
        {
            let number = IrNode::Parameter {
                name: "number".to_owned(),
                ty: IrType::Integer,
            };
            programs.push(ProgramIr {
                name: String::new(),
                parameters: Vec::new(),
                result: IrType::Sequence(Box::new(IrType::Text)),
                body: IrNode::Each {
                    item: "number".to_owned(),
                    items: Box::new(IrNode::Apply {
                        fragment: range.id.clone(),
                        arguments: vec![bound.node],
                    }),
                    body: Box::new(IrNode::Apply {
                        fragment: print.id.clone(),
                        arguments: vec![number],
                    }),
                    predicate: None,
                },
                fragments: vec![range.id.clone(), print.id.clone()],
                source_urls: vec![range.grounding.clone(), print.grounding.clone()],
                source_licenses: vec![range.license.clone(), print.license.clone()],
                reuse: crate::coding::program_ir::ReuseMode::ShapeOnly,
            });
        }
    }
    programs.sort_by_key(|program| (program.action_cost(), program.content_id()));
    programs.truncate(bounds.max_candidates);
    programs
}

fn fold_candidate(
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

const fn neutral_literal(ty: &IrType) -> &'static str {
    match ty {
        IrType::Integer | IrType::Float => "0",
        IrType::Boolean => "false",
        IrType::Text => "\"\"",
        IrType::Callable => "None",
        IrType::Sequence(_) => "[]",
        IrType::Pair(_, _) => "(0, 0)",
        IrType::Mapping(_, _) => "{}",
        IrType::Unknown(_) => "None",
    }
}

fn annotation_type(annotation: &str) -> Option<IrType> {
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
/// disagree (or are absent) stay open too.
fn example_parameter_type(spec: &CodingTaskSpec, index: usize) -> Option<IrType> {
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
        let class = if trimmed.starts_with('[') {
            let inner = trimmed.trim_start_matches('[').trim_end_matches(']').trim();
            let elements = inner
                .split(',')
                .map(|element| element.trim())
                .filter(|element| !element.is_empty())
                .collect::<Vec<_>>();
            let element = if !elements.is_empty()
                && elements.iter().all(|element| element.parse::<f64>().is_ok())
            {
                IrType::Float
            } else {
                IrType::Unknown(9_001)
            };
            IrType::Sequence(Box::new(element))
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

fn types_may_unify(left: &IrType, right: &IrType) -> bool {
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
fn types_fit_strictly(left: &IrType, right: &IrType) -> bool {
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

fn words(text: &str) -> std::collections::BTreeSet<String> {
    text.split(|character: char| !character.is_alphanumeric())
        .filter(|word| word.chars().count() > 2)
        .map(str::to_lowercase)
        .collect()
}

fn node_depth(node: &IrNode) -> usize {
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
