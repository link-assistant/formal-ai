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
            max_candidates: 64,
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
    let mut pool = atom_expressions(spec, &parameters, &ranked, structure_ids);
    let fold_roots = ranked
        .iter()
        .filter_map(|fragment| {
            let node = fold_candidate(fragment, &parameters)?;
            Some(Expression {
                depth: node_depth(&node),
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
        for fragment in &ranked {
            let names = fragment.argument_names(&spec.language);
            let choices = fragment
                .signature
                .iter()
                .enumerate()
                .map(|(index, expected)| {
                    argument_choices(
                        &snapshot,
                        expected,
                        names.get(index).map(String::as_str).unwrap_or(""),
                        index,
                        &parameters,
                    )
                })
                .collect::<Vec<_>>();
            if choices.iter().any(Vec::is_empty) {
                continue;
            }
            let mut combinations = Vec::new();
            enumerate_arguments(&choices, 0, &mut Vec::new(), &mut combinations, 512);
            combinations.sort_by_key(|arguments| Reverse(argument_coherence(arguments)));
            combinations.truncate(96);
            for arguments in combinations {
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
                let expression = Expression {
                    node: IrNode::Apply {
                        fragment: fragment.id.clone(),
                        arguments: arguments
                            .into_iter()
                            .map(|argument| argument.node)
                            .collect(),
                    },
                    ty: fragment.result.clone(),
                    fragments,
                    coverage,
                    depth,
                    grounding,
                };
                if node_depth(&expression.node) == depth {
                    layer.push(expression);
                }
            }
        }
        pool.extend(layer);
        normalize_pool(&mut pool, coverable.len(), 2_048);
    }
    // Root folds are complete programs, not reusable subexpressions. Preserve
    // them outside the bounded subexpression beam so a large compositional
    // neighborhood cannot evict a shallower valid reduction.
    pool.extend(fold_roots);

    let mut candidates = pool
        .into_iter()
        .filter(|expression| {
            !expression.fragments.is_empty()
                && (coverable.is_empty() || expression.coverage.is_superset(&coverable))
        })
        .map(|expression| program_from_expression(spec, parameters.clone(), expression))
        .filter(|candidate| {
            node_depth(&candidate.body) <= bounds.max_depth
                && candidate.type_check(catalog).is_ok()
                && lowered_candidate_is_closed(spec, catalog, candidate, &placeholder_names)
        })
        .collect::<Vec<_>>();
    let mut recursive = recursive_reduce_programs(
        spec,
        catalog,
        &parameters,
        structure_ids,
        bounds.max_candidates,
    );
    candidates.sort_by_key(|candidate| (candidate.action_cost(), candidate.content_id()));
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
                grounding: vec![(fragment.grounding.clone(), fragment.license.clone())],
            });
        }
        for (name, ty) in fragment
            .argument_names(&spec.language)
            .into_iter()
            .zip(&fragment.signature)
        {
            atoms.push(Expression {
                node: IrNode::Parameter {
                    name,
                    ty: ty.clone(),
                },
                ty: ty.clone(),
                fragments: Vec::new(),
                coverage: BTreeSet::new(),
                depth: 1,
                grounding: Vec::new(),
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
    for example in &spec.examples {
        literals.push(literal(
            &example.expected,
            inferred_literal_type(&example.expected),
        ));
    }
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
        grounding: Vec::new(),
    }
}

fn inferred_literal_type(value: &str) -> IrType {
    let value = value.trim();
    if matches!(value, "True" | "False" | "true" | "false") {
        IrType::Boolean
    } else if value.parse::<i128>().is_ok() {
        IrType::Integer
    } else if value.parse::<f64>().is_ok() {
        IrType::Float
    } else if value.starts_with('[') {
        IrType::Sequence(Box::new(IrType::Unknown(9_001)))
    } else {
        IrType::Text
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

fn argument_choices(
    pool: &[Expression],
    expected: &IrType,
    slot: &str,
    index: usize,
    parameters: &[(String, IrType)],
) -> Vec<Expression> {
    let mut choices = pool
        .iter()
        .filter(|expression| types_may_unify(expected, &expression.ty))
        .cloned()
        .collect::<Vec<_>>();
    choices.sort_by_key(|expression| {
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
        (
            usize::MAX - expression.coverage.len(),
            Reverse(expression.fragments.len()),
            exact_name,
            positional,
            expression.depth,
            format!("{:?}", expression.node),
        )
    });
    choices.dedup_by(|left, right| left.node == right.node);
    choices.truncate(8);
    choices
}

fn enumerate_arguments(
    choices: &[Vec<Expression>],
    index: usize,
    current: &mut Vec<Expression>,
    output: &mut Vec<Vec<Expression>>,
    limit: usize,
) {
    if output.len() >= limit {
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
            requested.saturating_sub(expression.coverage.len()),
            Reverse(expression.fragments.len()),
            expression.depth,
            format!("{:?}", expression.node),
        )
    });
    pool.dedup_by(|left, right| left.node == right.node);
    pool.truncate(limit);
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
