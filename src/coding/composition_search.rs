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
mod analysis;
mod arguments;
mod binders;
mod lowering;
mod pool;
mod recursive;

use analysis::{canonical_displacement, collect_parameter_names, iteration_element_types};
pub(crate) use analysis::{literal_leaves, parameter_reads, python_tokens};
use arguments::{
    argument_choices, argument_coherence, argument_grounded_structures, binder_circularity,
    constant_map_count, diversify, enumerate_arguments, fragment_coverage,
    inferred_expression_type, loose_feeds, node_contains_parameter, normalize_pool,
    parameter_displacement,
};
use binders::{
    checked_recursive_programs, conditional_payload_slots, fragment_binder_slots,
    loop_variable_applies, lowered_candidate_is_closed, pair_loop_variable_applies,
};
use lowering::{
    annotation_type, example_parameter_type, fold_candidate, node_depth, program_from_expression,
    types_fit_strictly, types_may_unify, words,
};
use pool::{Expression, atom_expressions, discovered_literals};
use recursive::recursive_reduce_programs;

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
    // A program is the same typed enumeration with two differences: no
    // callable wrapper — the candidate lowers to a bare expression, so no
    // parameters exist to bind — and the task's declared stdout joins the
    // atom pool as the task's own output slot. The print-each constructor
    // joined at the end is the one shape the enumeration cannot express:
    // `Each` introduces a binder scope, and the enumeration only assembles
    // fragment applications.
    let program_shape = spec.artifact_shape == ArtifactShape::Program;
    let requirement = spec.requirement_sentences.join(" ");
    let requirement_words = words(&requirement);
    // Numbers the task's own words state ("at least 4 characters") are the
    // bounds and counts the request named. The pool also seeds the small
    // integers so arithmetic has raw material, but in every slot window the
    // stated values must sample ahead of those defaults — otherwise the
    // threshold the request literally names never enters the enumeration.
    let stated_numbers: BTreeSet<String> = requirement
        .split(|character: char| !character.is_ascii_digit())
        .filter(|token| !token.is_empty() && token.parse::<u64>().is_ok())
        .map(std::borrow::ToOwned::to_owned)
        .collect();
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

    let parameters = if program_shape {
        Vec::new()
    } else {
        spec.parameters
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
            .collect::<Vec<_>>()
    };
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
                grounding: fragment_citation(fragment),
                observation: false,
            })
        })
        .collect::<Vec<_>>();
    for depth in 2..=bounds.max_depth {
        let snapshot = pool.clone();
        let mut layer = Vec::new();
        for (fragment_index, fragment) in ranked.iter().enumerate() {
            let names = fragment.argument_names(&spec.language);
            let observation_payloads =
                conditional_payload_slots(fragment, &spec.language).unwrap_or_default();
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
                        &stated_numbers,
                    );
                    let mut element_types: Vec<IrType> = snapshot
                        .iter()
                        .flat_map(|expression| iteration_element_types(&expression.ty))
                        .collect();
                    if !element_types.contains(element) {
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
                        &stated_numbers,
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
                    if names.get(index).is_some_and(|name| binders.contains(name)) {
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
                            observation: false,
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
                                observation: false,
                            });
                        }
                        return binder_choices;
                    }
                    argument_choices(
                        &snapshot,
                        expected,
                        names.get(index).map_or("", String::as_str),
                        index,
                        &parameters,
                        &loop_variable_choices,
                        &observation_payloads,
                        &owned,
                        &coverable,
                        &stated_numbers,
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
                    parameter_displacement(arguments, &parameters),
                )
            });
            combinations.truncate(2_048);
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
                    observation: false,
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

    // Full coverage first — as a preference, not a gate. Coverage orders the
    // drafts the oracle will see; the task's own examples decide which draft
    // is the answer. A hard cut at the observed maximum hides the honest
    // composition entirely whenever one more seeded structure exists than the
    // task needs: a stack of every discovered structure with degenerate slot
    // fills outranks it before verification, and the oracle never speaks.
    let survivors: Vec<(usize, usize, ProgramIr)> = pool
        .into_iter()
        .filter(|expression| !expression.fragments.is_empty())
        .map(|expression| {
            let covered = expression.coverage.intersection(&coverable).count();
            let constant_maps = expression.constant_maps;
            (covered, constant_maps, expression)
        })
        .filter(|(covered, _, _)| coverable.is_empty() || *covered > 0)
        .map(|(covered, constant_maps, expression)| {
            let mut candidate = program_from_expression(spec, parameters.clone(), expression);
            if program_shape {
                candidate.name = String::new();
                candidate.parameters = Vec::new();
            }
            (covered, constant_maps, candidate)
        })
        .filter(|(_, _, candidate)| {
            let depth_ok = node_depth(&candidate.body) <= bounds.max_depth;
            let type_ok = candidate.type_check(catalog).is_ok();
            let closed_ok =
                lowered_candidate_is_closed(spec, catalog, candidate, &placeholder_names);
            depth_ok && type_ok && closed_ok
        })
        .collect();
    let mut candidates: Vec<(usize, usize, ProgramIr)> = survivors;
    let mut recursive = recursive_reduce_programs(
        spec,
        catalog,
        &parameters,
        structure_ids,
        bounds.max_candidates,
    );
    // Truncation is not least-action: least-action selects among the drafts
    // that verify. The bounded candidate budget is spent first on programs
    // that cover more of the discovered structures and actually read the
    // task's inputs, then on richer compositions — constant-only and
    // literal-stuffed programs cannot generalize and only pass examples by
    // coincidence. A comprehension whose required slot ignores the very
    // binder the comprehension declares is the same filler one step more
    // subtle: `[item for item in words if len(text) >= 1]` keeps every word
    // or none regardless of which word is at hand, so among programs that
    // cover the same structures it must not crowd out the composition whose
    // predicate actually reads the element it filters — even though the
    // ignoring form is one fragment cheaper.
    let parameter_names = parameters
        .iter()
        .map(|(name, _)| name.clone())
        .collect::<BTreeSet<_>>();
    candidates.sort_by_key(|(covered, _, candidate)| {
        let mut referenced = BTreeSet::new();
        collect_parameter_names(&candidate.body, &parameter_names, &mut referenced);
        (
            Reverse(*covered),
            Reverse(referenced.len()),
            candidate.action_cost(),
            literal_leaves(&candidate.body),
            Reverse(candidate.fragments.len()),
            canonical_displacement(&candidate.body, &parameters),
            candidate.content_id(),
        )
    });
    candidates.dedup_by(|left, right| left.2.content_id() == right.2.content_id());
    let mut candidates: Vec<ProgramIr> = candidates
        .into_iter()
        .map(|(_, _, candidate)| candidate)
        .collect();
    // Keep a bounded share for each complete root constructor. Otherwise the
    // cheaper subexpression beam can evict every recursive program before the
    // executable examples get a chance to distinguish their semantics.
    let reserved = recursive
        .len()
        .min((bounds.max_candidates / 4).max(usize::from(!recursive.is_empty())));
    // Truncation is not least-action: least-action selects among the drafts
    // that verify, so the budget only bounds what reaches the examples. A
    // flat cost-ordered cut lets one prolific junk family (cheap re-wraps of
    // the input) flood every slot before the verified composition — deeper
    // because it computes rather than re-reads — is ever reached. The budget
    // is therefore spent round-robin across program families: a family is a
    // root fragment together with whether it invents constants, the same
    // division the pool beam uses, so neither the input-driven composition
    // nor the arithmetic formula whose semantics genuinely need a literal is
    // crowded out by the other's rivals.
    let budget = bounds.max_candidates.saturating_sub(reserved);
    let mut buckets: std::collections::BTreeMap<String, Vec<ProgramIr>> =
        std::collections::BTreeMap::new();
    for candidate in std::mem::take(&mut candidates) {
        let root = match &candidate.body {
            IrNode::Apply { fragment, .. } => fragment.clone(),
            _ => String::new(),
        };
        let class = format!(
            "{root}/{}",
            usize::from(literal_leaves(&candidate.body) > 0)
        );
        buckets.entry(class).or_default().push(candidate);
    }
    for bucket in buckets.values_mut() {
        bucket.reverse();
    }
    let families: Vec<String> = buckets.keys().cloned().collect();
    'family_rounds: for _ in 0..budget {
        for class in &families {
            let Some(bucket) = buckets.get_mut(class) else {
                continue;
            };
            let Some(candidate) = bucket.pop() else {
                continue;
            };
            candidates.push(candidate);
            if candidates.len() >= budget {
                break 'family_rounds;
            }
        }
    }
    recursive.truncate(reserved);
    candidates.extend(recursive);
    if program_shape {
        candidates.extend(print_each_programs(spec, catalog, bounds, structure_ids));
    }
    candidates.sort_by_key(|candidate| {
        (
            candidate.action_cost(),
            canonical_displacement(&candidate.body, &parameters),
            candidate.content_id(),
        )
    });
    candidates
}

/// Citations one fragment application contributes: its grounding and license
/// when the fragment declares `cites` (pure language-syntax idioms contribute
/// nothing). Argument subtrees were accumulated in reverse placeholder order,
/// so concatenation in application order — outermost fragment first — yields
/// the derivation's pipeline order: data enters outermost, flows inward, and
/// sources list in the order a reader meets them.
fn fragment_citation(
    fragment: &crate::coding::fragment_catalog::Fragment,
) -> Vec<(String, String)> {
    if fragment.cites {
        vec![(fragment.grounding.clone(), fragment.license.clone())]
    } else {
        Vec::new()
    }
}

/// Deduplicates citation pairs keeping the first occurrence, preserving
/// pipeline order rather than sorting.
fn dedup_preserving_first(pairs: Vec<(String, String)>) -> Vec<(String, String)> {
    let mut seen: std::collections::HashSet<String> = std::collections::HashSet::new();
    let mut kept = Vec::with_capacity(pairs.len());
    for (url, license) in pairs {
        if seen.insert(url.clone()) {
            kept.push((url, license));
        }
    }
    kept
}

/// The print-each program: `Each` introduces a binder scope, which the
/// Apply-tree enumeration does not build — this is the one constructor typed
/// enumeration genuinely cannot express, so it stays an explicit build. It
/// prints each element of a discovered integer range; the binder name is the
/// iterated element's role, not a task name.
fn print_each_programs(
    spec: &CodingTaskSpec,
    catalog: &FragmentCatalog,
    bounds: SearchBounds,
    structure_ids: &[String],
) -> Vec<ProgramIr> {
    let mut programs = Vec::new();
    let Some(print) = catalog.get("print_stdout") else {
        return programs;
    };
    let Some(range) = catalog.get("range_inclusive") else {
        return programs;
    };
    if !structure_ids.iter().any(|id| id == "range_inclusive") {
        return programs;
    }
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
    programs
}
