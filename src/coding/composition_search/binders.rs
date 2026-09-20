//! Binder-slot discovery and the loop-variable argument sets that let comprehension fragments read their own iteration variable.

use super::{
    BTreeSet, CodingTaskSpec, Expression, Fragment, FragmentCatalog, IrNode, IrType, ProgramIr,
    Reverse, argument_choices, argument_coherence, argument_grounded_structures,
    binder_circularity, constant_map_count, dedup_preserving_first, diversify, enumerate_arguments,
    fragment_citation, fragment_coverage, inferred_expression_type, literal_leaves, loose_feeds,
    node_contains_parameter, node_depth, parameter_reads, python_tokens, types_may_unify,
};

pub(super) fn checked_recursive_programs(
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
pub(super) fn lowered_candidate_is_closed(
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

pub(super) fn collect_bindings_until(
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
pub(super) fn fragment_binder_slots(fragment: &Fragment, language: &str) -> BTreeSet<String> {
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

/// The payload slots of a conditional fragment: when the realization is a
/// conditional expression (`{matched} if {condition} else {unmatched}`, with
/// both keywords at top nesting level and outside quotes), the placeholders
/// occurring inside the condition name condition slots and every other
/// placeholder is a payload slot. `None` for any non-conditional surface.
///
/// Observation-derived literals are admitted only here. A conditional's work
/// — deciding which branch applies — must be discovered from the catalog's
/// sources; the examples' contribution is limited to the answer's vocabulary,
/// the labels the branches return. Feeding an expected output into a condition
/// operand instead would bake one example's input into the program.
pub(super) fn conditional_payload_slots(
    fragment: &Fragment,
    language: &str,
) -> Option<Vec<String>> {
    let surface = fragment.realizations.get(language)?;
    let characters: Vec<char> = surface.chars().collect();
    let word_after = |position: usize, word: &str| -> bool {
        characters
            .get(position..position + word.len() + 1)
            .is_some_and(|window| {
                window[..word.len()].iter().collect::<String>() == word && window[word.len()] == ' '
            })
    };
    let mut depth = 0usize;
    let mut quote: Option<char> = None;
    let mut condition_start: Option<usize> = None;
    let mut condition_end: Option<usize> = None;
    let mut index = 0;
    while index < characters.len() {
        let character = characters[index];
        if let Some(opening) = quote {
            if character == '\\' {
                index += 2;
                continue;
            }
            if character == opening {
                quote = None;
            }
        } else {
            match character {
                '\'' | '"' => quote = Some(character),
                '(' | '[' | '{' => depth += 1,
                ')' | ']' | '}' => depth = depth.saturating_sub(1),
                ' ' if depth == 0 => {
                    let next = index + 1;
                    if condition_start.is_none() && word_after(next, "if") {
                        condition_start = Some(next);
                    } else if condition_start.is_some()
                        && condition_end.is_none()
                        && word_after(next, "else")
                    {
                        condition_end = Some(next);
                    }
                }
                _ => {}
            }
        }
        index += 1;
    }
    let (Some(condition_start), Some(condition_end)) = (condition_start, condition_end) else {
        return None;
    };
    let condition_text: String = characters[condition_start..condition_end].iter().collect();
    let names = fragment.argument_names(language);
    Some(
        names
            .into_iter()
            .filter(|name| !condition_text.contains(&format!("{{{name}}}")))
            .collect(),
    )
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
pub(super) fn loop_variable_applies<'a>(
    others: impl Iterator<Item = &'a Fragment>,
    pool: &[Expression],
    loop_variable: &str,
    element_type: &IrType,
    parameters: &[(String, IrType)],
    structure_ids: &[String],
    needed: &BTreeSet<String>,
    catalog: &FragmentCatalog,
    language: &str,
    stated: &BTreeSet<String>,
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
        observation: false,
    };
    let mut applies: Vec<Expression> = Vec::new();
    let others: Vec<&Fragment> = others.collect();
    for fragment in &others {
        if fragment.signature.is_empty() {
            continue;
        }
        let names = fragment.argument_names(language);
        let owned = fragment_coverage(fragment, structure_ids);
        let observation_payloads =
            conditional_payload_slots(fragment, language).unwrap_or_default();
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
                            names.get(index).map_or("", String::as_str),
                            index,
                            parameters,
                            &[],
                            &observation_payloads,
                            &owned,
                            needed,
                            stated,
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
                // The comprehension body should compute from the loop
                // variable: an ingredient that never reads it maps a constant
                // over the iteration, and the plain pool already offers
                // constants — at this depth or the next. The eviction is
                // measured and deliberately disabled: enabling it starves
                // compositions that legitimately bind constant ingredients
                // (specification::synthesis compound_courtesy and
                // mixed_script_definition both fail with it on), so
                // re-enabling requires widening the diversity window first.
                let _reads_binder = arguments
                    .iter()
                    .any(|argument| node_contains_parameter(&argument.node, loop_variable));
                let result_ty =
                    inferred_expression_type(&fragment.result, &fragment.signature, &arguments);
                let mut fragments = vec![fragment.id.clone()];
                let mut coverage = fragment_coverage(fragment, structure_ids);
                let mut grounding = fragment_citation(fragment);
                for argument in arguments.iter().rev() {
                    fragments.extend(argument.fragments.iter().cloned());
                    coverage.extend(argument.coverage.iter().cloned());
                    grounding.extend(argument.grounding.iter().cloned());
                }
                fragments.sort();
                fragments.dedup();
                grounding = dedup_preserving_first(grounding);
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
                    observation: false,
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
                usize::MAX
                    - expression
                        .coverage
                        .iter()
                        .filter(|id| needed.contains(*id))
                        .count(),
                usize::MAX - expression.coverage.len(),
                expression.fragments.len(),
                expression.depth,
                Reverse(parameter_reads(&expression.node, &input_names)),
                literal_leaves(&expression.node),
                format!("{:?}", expression.node),
            )
        });
        applies.dedup_by(|left, right| left.node == right.node);
        // The per-class share has to leave room for both readings of a
        // binder slot: the bare one (`bound <= item`, the fragment applied
        // to the variable directly) and the wrapped one
        // (`len(item) >= bound`, the variable computed through a
        // sub-fragment) — and for the input-grounded variants of each. A
        // tight total cut here is what starves the window downstream: the
        // consuming fragment's own slot window re-caps the product anyway,
        // so the ingredient list only needs to stay enumerable, not tiny.
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
        let observation_payloads =
            conditional_payload_slots(fragment, language).unwrap_or_default();
        for (slot, ty) in fragment.signature.iter().enumerate() {
            // The comprehension body must compute from the loop variable: an
            // ingredient that never reads it maps a constant over the
            // iteration, and the plain pool already offers constants.
            // Ingredients that also read a task input enumerate first — a
            // computation over the iteration is what a binder slot exists
            // for; re-reading the variable alone is the degenerate tail.
            let mut binder_dependent = applies
                .iter()
                .filter(|expression| {
                    types_may_unify(ty, &expression.ty)
                        && node_contains_parameter(&expression.node, loop_variable)
                })
                .cloned()
                .collect::<Vec<_>>();
            binder_dependent.sort_by_key(|expression| {
                let reads_input = input_names
                    .iter()
                    .any(|name| node_contains_parameter(&expression.node, name));
                usize::from(!reads_input)
            });
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
                            names.get(index).map_or("", String::as_str),
                            index,
                            parameters,
                            &[],
                            &observation_payloads,
                            &owned,
                            needed,
                            stated,
                        )
                    }
                })
                .collect::<Vec<_>>();
            if choices.iter().any(Vec::is_empty) {
                continue;
            }
            let mut combinations = Vec::new();
            // The cap has to cover the product, not a prefix of it: the
            // ingredient list holds several binder readings and the
            // remaining slots draw a full window each, so a cut of eight
            // keeps only the first ingredient's first window entries — the
            // literal-threshold readings (`len(item) >= 4`) the request
            // names are enumerated in full only if the cap spans the
            // product.
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
                let result_ty =
                    inferred_expression_type(&fragment.result, &fragment.signature, &arguments);
                let mut fragments = vec![fragment.id.clone()];
                let mut coverage = fragment_coverage(fragment, structure_ids);
                let mut grounding = fragment_citation(fragment);
                for argument in arguments.iter().rev() {
                    fragments.extend(argument.fragments.iter().cloned());
                    coverage.extend(argument.coverage.iter().cloned());
                    grounding.extend(argument.grounding.iter().cloned());
                }
                fragments.sort();
                fragments.dedup();
                grounding = dedup_preserving_first(grounding);
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
                    observation: false,
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
pub(super) fn pair_loop_variable_applies<'a>(
    others: impl Iterator<Item = &'a Fragment>,
    pool: &[Expression],
    element_types: &[IrType],
    parameters: &[(String, IrType)],
    structure_ids: &[String],
    needed: &BTreeSet<String>,
    catalog: &FragmentCatalog,
    language: &str,
    stated: &BTreeSet<String>,
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
        let observation_payloads =
            conditional_payload_slots(fragment, language).unwrap_or_default();
        for slot in 0..fragment.signature.len() - 1 {
            if !pair_types.iter().any(|(left, right)| {
                types_may_unify(&fragment.signature[slot], left)
                    && types_may_unify(&fragment.signature[slot + 1], right)
            }) {
                continue;
            }
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
                observation: false,
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
                            names.get(index).map_or("", String::as_str),
                            index,
                            parameters,
                            &[],
                            &observation_payloads,
                            &owned,
                            needed,
                            stated,
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
                let result_ty =
                    inferred_expression_type(&fragment.result, &fragment.signature, &arguments);
                let mut fragments = vec![fragment.id.clone()];
                let mut coverage = fragment_coverage(fragment, structure_ids);
                let mut grounding = fragment_citation(fragment);
                for argument in arguments.iter().rev() {
                    fragments.extend(argument.fragments.iter().cloned());
                    coverage.extend(argument.coverage.iter().cloned());
                    grounding.extend(argument.grounding.iter().cloned());
                }
                fragments.sort();
                fragments.dedup();
                grounding = dedup_preserving_first(grounding);
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
                    observation: false,
                });
            }
            if !applies.is_empty() && !unpacked.contains(&pair_names) {
                unpacked.push(pair_names);
            }
        }
    }
    (applies, unpacked)
}
