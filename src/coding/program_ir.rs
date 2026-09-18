//! The language-neutral intermediate representation (issue #1138, plan 02
//! L2–L3, L12).
//!
//! Every node is representable as nested `.lino`, so the IR is
//! associative-stack data rather than a Rust-only structure, and its content id
//! is taken over that projection. Nothing in [`elaborate`] knows a task name:
//! a step contributes a node, consecutive nodes compose by type, and a step that
//! binds a name introduces [`IrNode::Bind`].

use std::collections::{BTreeMap, BTreeSet};

use crate::coding::fragment_catalog::{Fragment, FragmentCatalog};
use crate::coding::python_render::runtime_template;
use crate::coding::task_spec::CodingTaskSpec;
use crate::links_format::push_lino_node;
use crate::procedure_text::ProcedureStepRecord;
use crate::seed::parser::parse_lino;

mod parse;

pub(crate) use parse::parse_type_slug;
use parse::{parse_node, parse_type};

/// Whether a retrieved fragment's text may be emitted verbatim, or only its
/// abstract shape reused. Share-alike licensing is enforced structurally.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReuseMode {
    /// The fragment's text may be emitted verbatim.
    Verbatim,
    /// Only the fragment's abstract shape may be used; text must be re-derived.
    ShapeOnly,
}

/// A value shape the IR can type-check without committing to a language.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum IrType {
    Integer,
    Float,
    Boolean,
    Text,
    /// A function value (a seeded lambda such as `lambda left, right: left +
    /// right`). Distinct from text: a callable slot only accepts fragments
    /// that define a function, so an input never masquerades as an operation.
    Callable,
    Sequence(Box<Self>),
    Pair(Box<Self>, Box<Self>),
    Mapping(Box<Self>, Box<Self>),
    /// Unconstrained; unifies with anything exactly once.
    Unknown(usize),
}

/// One node of a language-neutral program.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum IrNode {
    Parameter {
        name: String,
        ty: IrType,
    },
    Literal {
        text: String,
        ty: IrType,
    },
    /// A named fragment applied to arguments. `fragment` is a
    /// [`FragmentCatalog`] id, never a language-specific symbol.
    Apply {
        fragment: String,
        arguments: Vec<Self>,
    },
    /// Comprehension over `items`, binding `item`, yielding `body`, optionally
    /// filtered by `predicate`.
    Each {
        item: String,
        items: Box<Self>,
        body: Box<Self>,
        predicate: Option<Box<Self>>,
    },
    /// Left fold with an initial value.
    Fold {
        item: String,
        accumulator: String,
        items: Box<Self>,
        initial: Box<Self>,
        body: Box<Self>,
    },
    /// Bounded iteration with an explicit termination condition.
    Repeat {
        counter: String,
        from: Box<Self>,
        to: Box<Self>,
        body: Box<Self>,
    },
    /// Named state, base cases and a transition — the recurrence shape OEIS and
    /// Wikifunctions both produce.
    Recurrence {
        state: Vec<String>,
        base: Vec<Self>,
        transition: Box<Self>,
        index: Box<Self>,
    },
    /// A well-founded recursive reduction over predecessor states. This is the
    /// language-neutral dynamic-programming shape: the catalog supplies value,
    /// predicate, reduction and combination fragments while this node supplies
    /// their scopes and recursive data flow.
    RecursiveReduce {
        state: Vec<String>,
        target: Vec<Self>,
        item: Vec<String>,
        items: Box<Self>,
        next: Vec<Self>,
        admissible: Option<Box<Self>>,
        base_test: Box<Self>,
        base: Box<Self>,
        local: Box<Self>,
        reducer: String,
        combine: String,
    },
    Condition {
        test: Box<Self>,
        then_branch: Box<Self>,
        else_branch: Box<Self>,
    },
    Bind {
        name: String,
        value: Box<Self>,
        body: Box<Self>,
    },
    Emit {
        value: Box<Self>,
    },
    Return {
        value: Box<Self>,
    },
}

/// A complete program plan: signature, body, provenance and cost.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProgramIr {
    pub name: String,
    pub parameters: Vec<(String, IrType)>,
    pub result: IrType,
    pub body: IrNode,
    pub fragments: Vec<String>,
    pub source_urls: Vec<String>,
    pub source_licenses: Vec<String>,
    pub reuse: ReuseMode,
}

impl ProgramIr {
    /// Structural cost, the least-action key: node count plus fragment depth.
    #[must_use]
    pub fn action_cost(&self) -> usize {
        node_cost(&self.body)
    }

    /// Unify parameter and result types through the fragment signatures.
    /// `Err` names the first node that cannot be typed.
    pub fn type_check(&self, catalog: &FragmentCatalog) -> Result<(), String> {
        let mut environment = self.parameters.iter().cloned().collect();
        let mut substitutions = BTreeMap::new();
        let mut fresh = FRAGMENT_TYPE_VARIABLE_OFFSET;
        infer_node(
            &self.body,
            catalog,
            &mut environment,
            &mut substitutions,
            &mut fresh,
        )?;
        Ok(())
    }

    /// The canonical `.lino` projection; also the content-hash input.
    #[must_use]
    pub fn to_links_notation(&self) -> String {
        let mut output = String::new();
        if self
            .name
            .chars()
            .all(|character| character.is_alphanumeric() || character == '_')
        {
            output.push_str("program_ir ");
            output.push_str(&self.name);
            output.push('\n');
        } else {
            push_lino_node(&mut output, 0, "program_ir", Some(&self.name));
        }
        for (name, ty) in &self.parameters {
            push_lino_node(&mut output, 2, "parameter", Some(name));
            push_type(&mut output, 4, ty);
        }
        push_lino_node(&mut output, 2, "result", None);
        push_type(&mut output, 4, &self.result);
        push_lino_node(&mut output, 2, "body", None);
        push_node(&mut output, 4, &self.body);
        for fragment in &self.fragments {
            push_lino_node(&mut output, 2, "fragment", Some(fragment));
        }
        for url in &self.source_urls {
            push_lino_node(&mut output, 2, "source_url", Some(url));
        }
        for license in &self.source_licenses {
            push_lino_node(&mut output, 2, "source_license", Some(license));
        }
        push_lino_node(&mut output, 2, "reuse", Some(reuse_slug(self.reuse)));
        output.trim_end().to_owned()
    }

    /// Parse the canonical projection back into an IR.
    #[must_use]
    pub fn from_links_notation(text: &str) -> Option<Self> {
        let tree = parse_lino(text);
        let root = tree
            .children
            .iter()
            .find(|node| node.name == "program_ir")?;
        let parameters = root
            .children
            .iter()
            .filter(|child| child.name == "parameter")
            .map(|child| Some((child.id.clone(), parse_type(child.children.first()?)?)))
            .collect::<Option<Vec<_>>>()?;
        let result = parse_type(
            root.children
                .iter()
                .find(|child| child.name == "result")?
                .children
                .first()?,
        )?;
        let body = parse_node(
            root.children
                .iter()
                .find(|child| child.name == "body")?
                .children
                .first()?,
        )?;
        let values = |name: &str| {
            root.children
                .iter()
                .filter(|child| child.name == name)
                .map(|child| child.id.clone())
                .collect::<Vec<_>>()
        };
        Some(Self {
            name: root.id.clone(),
            parameters,
            result,
            body,
            fragments: values("fragment"),
            source_urls: values("source_url"),
            source_licenses: values("source_license"),
            reuse: match root.find_child_value("reuse") {
                "verbatim" => ReuseMode::Verbatim,
                "shape_only" => ReuseMode::ShapeOnly,
                _ => return None,
            },
        })
    }

    /// Stable content id over [`Self::to_links_notation`].
    #[must_use]
    pub fn content_id(&self) -> String {
        format!(
            "program_ir_{}",
            crate::source_fetch::sha256_hex(self.to_links_notation().as_bytes())
        )
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ElaborationBounds {
    pub max_candidates: usize,
    pub max_depth: usize,
}

/// Elaborate an ordered step list into candidate IR bodies.
#[must_use]
pub fn elaborate(
    spec: &CodingTaskSpec,
    steps: &[ProcedureStepRecord],
    catalog: &FragmentCatalog,
    bounds: ElaborationBounds,
) -> Vec<ProgramIr> {
    if bounds.max_candidates == 0 || bounds.max_depth == 0 || steps.is_empty() {
        return Vec::new();
    }
    let parameters = inferred_parameters(spec);
    let mut candidates = Vec::new();
    for step in steps.iter().take(bounds.max_depth) {
        let mut ranked = catalog
            .fragments()
            .iter()
            .map(|fragment| (fragment_overlap(fragment, &step.text), fragment))
            .filter(|(overlap, _)| *overlap > 0)
            .collect::<Vec<_>>();
        ranked.sort_by(|left, right| {
            right
                .0
                .cmp(&left.0)
                .then_with(|| left.1.id.cmp(&right.1.id))
        });
        for (_, fragment) in ranked.into_iter().take(bounds.max_candidates) {
            let arguments = fragment
                .signature
                .iter()
                .enumerate()
                .map(|(index, ty)| argument_for(index, ty, &parameters))
                .collect();
            let reuse = if step.license_name.to_ascii_lowercase().contains("by-sa")
                || step.license_name.to_ascii_lowercase().contains("gfdl")
            {
                ReuseMode::ShapeOnly
            } else {
                fragment.reuse
            };
            let ir = ProgramIr {
                name: spec.name.clone(),
                parameters: parameters.clone(),
                result: fragment.result.clone(),
                body: IrNode::Return {
                    value: Box::new(IrNode::Apply {
                        fragment: fragment.id.clone(),
                        arguments,
                    }),
                },
                fragments: vec![fragment.id.clone()],
                source_urls: vec![step.source_url.clone()],
                source_licenses: vec![step.license_name.clone()],
                reuse,
            };
            if node_depth(&ir.body) <= bounds.max_depth && ir.type_check(catalog).is_ok() {
                candidates.push(ir);
            }
        }
    }
    candidates.sort_by_key(|candidate| (candidate.action_cost(), candidate.content_id()));
    candidates.dedup_by(|left, right| left.content_id() == right.content_id());
    candidates.truncate(bounds.max_candidates);
    candidates
}

fn inferred_parameters(spec: &CodingTaskSpec) -> Vec<(String, IrType)> {
    spec.parameters
        .iter()
        .enumerate()
        .map(|(index, parameter)| {
            (
                parameter.name.clone(),
                parameter
                    .annotation
                    .as_deref()
                    .and_then(type_from_annotation)
                    .unwrap_or(IrType::Unknown(index)),
            )
        })
        .collect()
}

fn type_from_annotation(annotation: &str) -> Option<IrType> {
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

fn argument_for(index: usize, ty: &IrType, parameters: &[(String, IrType)]) -> IrNode {
    parameters.get(index).map_or_else(
        || IrNode::Literal {
            text: neutral_literal(ty).to_owned(),
            ty: ty.clone(),
        },
        |(name, parameter_type)| IrNode::Parameter {
            name: name.clone(),
            ty: parameter_type.clone(),
        },
    )
}

const fn neutral_literal(ty: &IrType) -> &'static str {
    match ty {
        IrType::Integer | IrType::Float => "0",
        IrType::Boolean => "false",
        IrType::Text => "\"\"",
        // No literal denotes a function; the seed's lambda fragments do.
        IrType::Callable => "None",
        IrType::Sequence(_) => "[]",
        IrType::Pair(_, _) => "(0, 0)",
        IrType::Mapping(_, _) => "{}",
        IrType::Unknown(_) => "None",
    }
}

fn fragment_overlap(fragment: &Fragment, step: &str) -> usize {
    let words = normalized_words(step);
    normalized_words(&format!("{} {}", fragment.id, fragment.rediscovery_query))
        .intersection(&words)
        .count()
}

fn normalized_words(text: &str) -> BTreeSet<String> {
    text.split(|character: char| !character.is_alphanumeric())
        .filter(|word| word.chars().count() > 2)
        .map(str::to_lowercase)
        .collect()
}

const fn reuse_slug(reuse: ReuseMode) -> &'static str {
    match reuse {
        ReuseMode::Verbatim => "verbatim",
        ReuseMode::ShapeOnly => "shape_only",
    }
}

fn node_cost(node: &IrNode) -> usize {
    1 + match node {
        IrNode::Parameter { .. } | IrNode::Literal { .. } => 0,
        IrNode::Apply { arguments, .. } => arguments.iter().map(node_cost).sum(),
        IrNode::Each {
            items,
            body,
            predicate,
            ..
        } => node_cost(items) + node_cost(body) + predicate.as_deref().map_or(0, node_cost),
        IrNode::Fold {
            items,
            initial,
            body,
            ..
        } => node_cost(items) + node_cost(initial) + node_cost(body),
        IrNode::Repeat { from, to, body, .. } => node_cost(from) + node_cost(to) + node_cost(body),
        IrNode::Recurrence {
            base,
            transition,
            index,
            ..
        } => base.iter().map(node_cost).sum::<usize>() + node_cost(transition) + node_cost(index),
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
            target.iter().map(node_cost).sum::<usize>()
                + node_cost(items)
                + next.iter().map(node_cost).sum::<usize>()
                + admissible.as_deref().map_or(0, node_cost)
                + node_cost(base_test)
                + node_cost(base)
                + node_cost(local)
        }
        IrNode::Condition {
            test,
            then_branch,
            else_branch,
        } => node_cost(test) + node_cost(then_branch) + node_cost(else_branch),
        IrNode::Bind { value, body, .. } => node_cost(value) + node_cost(body),
        IrNode::Emit { value } | IrNode::Return { value } => node_cost(value),
    }
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

fn infer_node(
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

fn restore_binding(
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

fn iterable_element(ty: &IrType) -> IrType {
    match ty {
        IrType::Sequence(element) => (**element).clone(),
        IrType::Text => IrType::Text,
        IrType::Mapping(key, value) => IrType::Pair(key.clone(), value.clone()),
        IrType::Unknown(id) => IrType::Unknown(*id),
        _ => IrType::Unknown(usize::MAX),
    }
}

fn destructured_types(ty: &IrType, arity: usize) -> Result<Vec<IrType>, String> {
    match (arity, ty) {
        (1, ty) => Ok(vec![ty.clone()]),
        (2, IrType::Pair(left, right)) => Ok(vec![(**left).clone(), (**right).clone()]),
        _ => Err(runtime_message(
            "ir_recursive_destructure",
            &[("arity", &arity.to_string()), ("type", &format!("{ty:?}"))],
        )),
    }
}

fn check_reduction_fragment(
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

fn check_combination_fragment(
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

fn unify(
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

fn runtime_message(id: &str, values: &[(&str, &str)]) -> String {
    runtime_template(id, values).unwrap_or_else(|| id.to_owned())
}

const FRAGMENT_TYPE_VARIABLE_OFFSET: usize = usize::MAX / 4;

/// Distinct type-variable ids reserved for one fragment occurrence. Covers
/// every schematic variable a seeded fragment declares.
const FRAGMENT_TYPE_VARIABLE_STRIDE: usize = 32;

/// Instantiates a fragment's schematic type with type variables based at
/// `base`. The base is drawn fresh for every fragment occurrence: a constant
/// offset would make two applications of different fragments share the same
/// variable, letting one application's iterable binding (text iteration
/// binding an element to text) poison an unrelated fragment's element type.
fn fragment_type_variable(ty: &IrType, base: usize) -> IrType {
    match ty {
        IrType::Unknown(id) => IrType::Unknown(base.saturating_add(*id)),
        IrType::Sequence(element) => {
            IrType::Sequence(Box::new(fragment_type_variable(element, base)))
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

fn contains_unknown(ty: &IrType, sought: usize) -> bool {
    match ty {
        IrType::Unknown(id) => *id == sought,
        IrType::Sequence(element) => contains_unknown(element, sought),
        IrType::Pair(left, right) | IrType::Mapping(left, right) => {
            contains_unknown(left, sought) || contains_unknown(right, sought)
        }
        _ => false,
    }
}

fn resolve_type(ty: &IrType, substitutions: &BTreeMap<usize, IrType>) -> IrType {
    match ty {
        IrType::Unknown(id) => substitutions
            .get(id)
            .map_or_else(|| ty.clone(), |value| resolve_type(value, substitutions)),
        IrType::Sequence(element) => {
            IrType::Sequence(Box::new(resolve_type(element, substitutions)))
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

fn push_type(output: &mut String, indent: usize, ty: &IrType) {
    match ty {
        IrType::Integer => push_lino_node(output, indent, "type", Some("integer")),
        IrType::Float => push_lino_node(output, indent, "type", Some("float")),
        IrType::Boolean => push_lino_node(output, indent, "type", Some("boolean")),
        IrType::Text => push_lino_node(output, indent, "type", Some("text")),
        IrType::Callable => push_lino_node(output, indent, "type", Some("callable")),
        IrType::Unknown(id) => {
            push_lino_node(output, indent, "type", Some(&format!("unknown:{id}")));
        }
        IrType::Sequence(element) => {
            push_lino_node(output, indent, "type", Some("sequence"));
            push_lino_node(output, indent + 2, "element", None);
            push_type(output, indent + 4, element);
        }
        IrType::Pair(left, right) | IrType::Mapping(left, right) => {
            push_lino_node(
                output,
                indent,
                "type",
                Some(if matches!(ty, IrType::Pair(_, _)) {
                    "pair"
                } else {
                    "mapping"
                }),
            );
            push_lino_node(output, indent + 2, "left", None);
            push_type(output, indent + 4, left);
            push_lino_node(output, indent + 2, "right", None);
            push_type(output, indent + 4, right);
        }
    }
}

fn push_node(output: &mut String, indent: usize, node: &IrNode) {
    let variant = node_variant(node);
    push_lino_node(output, indent, "node", Some(variant));
    match node {
        IrNode::Parameter { name, ty } => {
            push_lino_node(output, indent + 2, "name", Some(name));
            push_type(output, indent + 2, ty);
        }
        IrNode::Literal { text, ty } => {
            push_lino_node(output, indent + 2, "text", Some(text));
            push_type(output, indent + 2, ty);
        }
        IrNode::Apply {
            fragment,
            arguments,
        } => {
            push_lino_node(output, indent + 2, "fragment", Some(fragment));
            for argument in arguments {
                push_lino_node(output, indent + 2, "argument", None);
                push_node(output, indent + 4, argument);
            }
        }
        IrNode::Each {
            item,
            items,
            body,
            predicate,
        } => {
            push_lino_node(output, indent + 2, "item", Some(item));
            push_child_node(output, indent, "items", items);
            push_child_node(output, indent, "body", body);
            if let Some(predicate) = predicate {
                push_child_node(output, indent, "predicate", predicate);
            }
        }
        IrNode::Fold {
            item,
            accumulator,
            items,
            initial,
            body,
        } => {
            push_lino_node(output, indent + 2, "item", Some(item));
            push_lino_node(output, indent + 2, "accumulator", Some(accumulator));
            push_child_node(output, indent, "items", items);
            push_child_node(output, indent, "initial", initial);
            push_child_node(output, indent, "body", body);
        }
        IrNode::Repeat {
            counter,
            from,
            to,
            body,
        } => {
            push_lino_node(output, indent + 2, "counter", Some(counter));
            push_child_node(output, indent, "from", from);
            push_child_node(output, indent, "to", to);
            push_child_node(output, indent, "body", body);
        }
        IrNode::Recurrence {
            state,
            base,
            transition,
            index,
        } => {
            for name in state {
                push_lino_node(output, indent + 2, "state", Some(name));
            }
            for base_case in base {
                push_child_node(output, indent, "base", base_case);
            }
            push_child_node(output, indent, "transition", transition);
            push_child_node(output, indent, "index", index);
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
            for name in state {
                push_lino_node(output, indent + 2, "state", Some(name));
            }
            for value in target {
                push_child_node(output, indent, "target", value);
            }
            for name in item {
                push_lino_node(output, indent + 2, "item", Some(name));
            }
            push_child_node(output, indent, "items", items);
            for value in next {
                push_child_node(output, indent, "next", value);
            }
            if let Some(predicate) = admissible {
                push_child_node(output, indent, "admissible", predicate);
            }
            push_child_node(output, indent, "base_test", base_test);
            push_child_node(output, indent, "base", base);
            push_child_node(output, indent, "local", local);
            push_lino_node(output, indent + 2, "reducer", Some(reducer));
            push_lino_node(output, indent + 2, "combine", Some(combine));
        }
        IrNode::Condition {
            test,
            then_branch,
            else_branch,
        } => {
            push_child_node(output, indent, "test", test);
            push_child_node(output, indent, "then", then_branch);
            push_child_node(output, indent, "else", else_branch);
        }
        IrNode::Bind { name, value, body } => {
            push_lino_node(output, indent + 2, "name", Some(name));
            push_child_node(output, indent, "value", value);
            push_child_node(output, indent, "body", body);
        }
        IrNode::Emit { value } | IrNode::Return { value } => {
            push_child_node(output, indent, "value", value);
        }
    }
}

fn push_child_node(output: &mut String, parent_indent: usize, name: &str, child: &IrNode) {
    push_lino_node(output, parent_indent + 2, name, None);
    push_node(output, parent_indent + 4, child);
}

const fn node_variant(node: &IrNode) -> &'static str {
    match node {
        IrNode::Parameter { .. } => "parameter",
        IrNode::Literal { .. } => "literal",
        IrNode::Apply { .. } => "apply",
        IrNode::Each { .. } => "each",
        IrNode::Fold { .. } => "fold",
        IrNode::Repeat { .. } => "repeat",
        IrNode::Recurrence { .. } => "recurrence",
        IrNode::RecursiveReduce { .. } => "recursive_reduce",
        IrNode::Condition { .. } => "condition",
        IrNode::Bind { .. } => "bind",
        IrNode::Emit { .. } => "emit",
        IrNode::Return { .. } => "return",
    }
}

/// Compact type syntax used by the fragment ledger and optional seed fields.
#[must_use]
pub(crate) fn type_slug(ty: &IrType) -> String {
    match ty {
        IrType::Integer => "integer".to_owned(),
        IrType::Float => "float".to_owned(),
        IrType::Boolean => "boolean".to_owned(),
        IrType::Text => "text".to_owned(),
        IrType::Callable => "callable".to_owned(),
        IrType::Sequence(element) => format!("sequence<{}>", type_slug(element)),
        IrType::Pair(left, right) => format!("pair<{},{}>", type_slug(left), type_slug(right)),
        IrType::Mapping(key, value) => {
            format!("mapping<{},{}>", type_slug(key), type_slug(value))
        }
        IrType::Unknown(id) => format!("unknown:{id}"),
    }
}
