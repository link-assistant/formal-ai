//! Typed argument selection: slot choices, coherence, diversity, and pool normalization.

use super::*;

#[allow(clippy::too_many_arguments)]
pub(super) fn argument_choices(
    pool: &[Expression],
    expected: &IrType,
    slot: &str,
    index: usize,
    parameters: &[(String, IrType)],
    extra: &[Expression],
    observation_slots: &[String],
    owned: &BTreeSet<String>,
    needed: &BTreeSet<String>,
    stated: &BTreeSet<String>,
) -> Vec<Expression> {
    // An observation literal (an atom lifted from an expected output) is only
    // the answer's vocabulary: it may name a conditional's payload and nothing
    // else. Every other slot filters these atoms out, so an expected value can
    // never become a condition operand or a standalone answer.
    let observation_allowed = observation_slots.iter().any(|name| name == slot);
    let mut choices = pool
        .iter()
        .filter(|expression| {
            types_may_unify(expected, &expression.ty)
                && (observation_allowed || !expression.observation)
        })
        .cloned()
        .collect::<Vec<_>>();
    choices.extend(
        extra
            .iter()
            .filter(|expression| {
                types_may_unify(expected, &expression.ty)
                    && (observation_allowed || !expression.observation)
            })
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
                    // A literal the task's own words state is the bound or
                    // count the request named; the pool's small default seeds
                    // are fallbacks for when the request states no number at
                    // all. Within one literal class, stated values sample
                    // first — otherwise the window keeps `0, 1, 2` and the
                    // threshold the request literally names never enters the
                    // enumeration.
                    match &expression.node {
                        IrNode::Literal { text, .. } => usize::from(!stated.contains(text)),
                        _ => 0,
                    },
                    usize::MAX
                        - expression
                            .coverage
                            .iter()
                            .filter(|id| needed.contains(*id))
                            .count(),
                    usize::MAX
                        - expression
                            .coverage
                            .iter()
                            .filter(|id| !owned.contains(*id))
                            .count(),
                    Reverse(parameter_reads(&expression.node, &input_names)),
                    literal_leaves(&expression.node),
                ),
                (
                    exact_name,
                    positional,
                    usize::MAX - expression.fragments.len(),
                    expression.depth,
                    // Between otherwise indistinguishable shapes, the
                    // per-element reading comes first: a form whose leading
                    // argument computes or reads a value (`item >= bound`,
                    // `len(item) >= bound`) is the idiom a filter or a map
                    // exists for, while its constant-anchored mirror
                    // (`bound <= item`) is the rarer direction. Without this
                    // the window's last tie is the debug spelling of the
                    // node, and alphabetical luck decides which reading a
                    // task ever gets to try.
                ),
            ),
            format!("{:?}", expression.node),
        );
        key
    });
    choices.dedup_by(|left, right| left.node == right.node);
    // The window has to stay small enough that the argument product of a
    // multi-slot fragment is enumerated in full: a three-slot fragment whose
    // slots each offer thirty-two choices produces a product far past the
    // enumeration limit, and the depth-first cut then keeps only the combos
    // of the first-listed values — the input re-read and the pool's numeric
    // literals — while the combination that names an answer's vocabulary in
    // each payload slot is never reached at all.
    diversify(&choices, 32, 12)
}

/// Counts degenerate comprehensions: a body that never reads its binder (a
/// constant map such as `[0 for item in items]`) and an iterable that reads
/// its own binder (circular, such as feeding `slice(values, item)` back as
/// the iterable of the very comprehension binding `item`). Both are well
/// typed but cannot depend on the iteration, so they are filler in any
/// window over individual items.
pub(super) fn constant_map_count(
    node: &IrNode,
    catalog: &FragmentCatalog,
    language: &str,
) -> usize {
    match node {
        IrNode::Apply {
            fragment,
            arguments,
        } => {
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

pub(super) fn references_parameter(node: &IrNode, name: &str) -> bool {
    match node {
        IrNode::Parameter {
            name: candidate, ..
        } => candidate == name,
        IrNode::Apply { arguments, .. } => arguments
            .iter()
            .any(|argument| references_parameter(argument, name)),
        _ => false,
    }
}

/// The type an argument tree presents at its root, for loose-feed counting.
pub(super) fn argument_root_type(node: &IrNode, catalog: &FragmentCatalog) -> Option<IrType> {
    match node {
        IrNode::Parameter { ty, .. } | IrNode::Literal { ty, .. } => Some(ty.clone()),
        IrNode::Apply { fragment, .. } => catalog
            .get(fragment)
            .map(|definition| definition.result.clone()),
        _ => None,
    }
}

/// Whether one slot was filled with a value that only fits through the
/// text-over-sequence leniency while the element type is concrete: a text
/// cannot promise to yield those elements, so the assembly is a guess.
pub(super) fn loose_feed(expected: &IrType, argument: &IrNode, catalog: &FragmentCatalog) -> bool {
    let Some(actual) = argument_root_type(argument, catalog) else {
        return false;
    };
    if types_fit_strictly(expected, &actual) {
        return false;
    }
    let IrType::Sequence(element) = expected else {
        return false;
    };
    matches!(actual, IrType::Text) && !matches!(element.as_ref(), IrType::Unknown(_) | IrType::Text)
}

/// Resolves a fragment's schematic result against the argument types the
/// combination actually provides. A schematic result such as
/// `sequence<unknown:1>` leaves the element open, which lets a body that
/// produces a sequence masquerade as a flat sequence in every later window;
/// binding the signature's variables to the observed argument types records
/// what the assembly really yields.
pub(super) fn inferred_expression_type(
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
pub(super) fn type_has_open_variables(ty: &IrType) -> bool {
    match ty {
        IrType::Unknown(_) => true,
        IrType::Sequence(element) => type_has_open_variables(element),
        IrType::Pair(left, right) | IrType::Mapping(left, right) => {
            type_has_open_variables(left) || type_has_open_variables(right)
        }
        _ => false,
    }
}

pub(super) fn collect_variable_bindings(
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

pub(super) fn apply_variable_bindings(
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
pub(super) fn loose_feeds(node: &IrNode, catalog: &FragmentCatalog) -> usize {
    match node {
        IrNode::Apply {
            fragment,
            arguments,
        } => {
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
pub(super) fn type_key(ty: &IrType) -> String {
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
pub(super) fn diversity_class(expression: &Expression) -> String {
    match &expression.node {
        // Literal values diversify by type, not by text: a slot choice list
        // otherwise spends its whole window on the pool's small numeric
        // literals (each its own class) and starves the text atoms that name
        // an answer's vocabulary. Same-class literals are near-duplicates for
        // sampling purposes; keeping a few of each type is the point of the
        // window.
        IrNode::Literal { .. } => format!("literal {}", type_key(&expression.ty)),
        IrNode::Parameter { .. } => format!("parameter {}", type_key(&expression.ty)),
        IrNode::Apply { fragment, .. } => format!(
            "apply {} {}{}{}",
            fragment,
            type_key(&expression.ty),
            if repeats_a_fragment(&expression.node) {
                " recursive"
            } else {
                ""
            },
            if has_computed_argument(&expression.node) {
                " computed"
            } else {
                ""
            }
        ),
        _ => format!("apply {}", type_key(&expression.ty)),
    }
}

/// True when any argument subtree applies a fragment. An apply whose
/// arguments are all leaves re-reads or renames values; an apply over a
/// computed argument is a genuinely different shape — the comprehension body
/// `values_equal(rotate(text, item), text)` versus the bare re-read
/// `values_equal(item, text)` — and must not share one diversity window with
/// it, or the leaner form crowds the composition the window exists for.
pub(super) fn has_computed_argument(node: &IrNode) -> bool {
    let IrNode::Apply { arguments, .. } = node else {
        return false;
    };
    arguments.iter().any(|argument| {
        let mut stack = vec![argument];
        while let Some(current) = stack.pop() {
            if matches!(current, IrNode::Apply { .. }) {
                return true;
            }
            if let IrNode::Apply { arguments, .. } = current {
                stack.extend(arguments);
            }
        }
        false
    })
}

/// True when a fragment is applied directly to its own output — the
/// self-nesting filler pattern (`accumulate(accumulate(values, add), add)`) —
/// or when any single fragment appears three or more times. Repetition of a
/// fragment exactly twice through *other* fragments
/// (`reverse(remove-first(reverse(...)))`, removing a trailing marker) is
/// legitimate composition, not filler.
pub(super) fn pathological_repeat(node: &IrNode) -> bool {
    fn walk(node: &IrNode, counts: &mut std::collections::BTreeMap<String, usize>) -> bool {
        match node {
            IrNode::Apply {
                fragment,
                arguments,
            } => {
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
pub(super) fn repeats_a_fragment(node: &IrNode) -> bool {
    fn count(node: &IrNode, counts: &mut std::collections::BTreeMap<String, usize>) {
        match node {
            IrNode::Apply {
                fragment,
                arguments,
            } => {
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

pub(super) fn diversify(sorted: &[Expression], limit: usize, per_type: usize) -> Vec<Expression> {
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

pub(super) fn enumerate_arguments(
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
pub(super) fn argument_grounded_structures(node: &IrNode, needed: &BTreeSet<String>) -> usize {
    let IrNode::Apply { arguments, .. } = node else {
        return 0;
    };
    let mut grounded = BTreeSet::new();
    for argument in arguments {
        let mut stack = vec![argument];
        while let Some(current) = stack.pop() {
            if let IrNode::Apply {
                fragment,
                arguments,
            } = current
            {
                if needed.contains(fragment) {
                    grounded.insert(fragment.clone());
                }
                stack.extend(arguments);
            }
        }
    }
    grounded.len()
}

pub(super) fn binder_circularity(
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

pub(super) fn argument_coherence(arguments: &[Expression]) -> usize {
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

pub(super) fn node_contains_parameter(node: &IrNode, sought: &str) -> bool {
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

pub(super) fn fragment_coverage(fragment: &Fragment, requested: &[String]) -> BTreeSet<String> {
    requested
        .iter()
        .filter(|id| {
            fragment.id == id.as_str() || fragment.supports.iter().any(|item| item == id.as_str())
        })
        .cloned()
        .collect()
}

pub(super) fn normalize_pool(pool: &mut Vec<Expression>, requested: usize, limit: usize) {
    pool.sort_by_key(|expression| {
        (
            // Atoms carry no coverage but are the inputs every composition
            // grounds in; they survive retention first.
            usize::from(!expression.fragments.is_empty()),
            requested.saturating_sub(expression.coverage.len()),
            usize::from(expression.constant_maps > 0),
            usize::from(type_has_open_variables(&expression.ty)),
            literal_leaves(&expression.node),
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
