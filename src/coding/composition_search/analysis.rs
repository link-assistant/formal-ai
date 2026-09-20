//! Ranking signals computed over a lowered candidate tree.

use super::{BTreeSet, IrNode, IrType};

/// How many DISTINCT task inputs the expression reads anywhere in its tree.
/// A nested re-read of the same input adds no grounding beyond the first, so
/// deeply tangled assemblies gain no ranking advantage over a plain input.
/// Number of literal leaves in a tree. At an otherwise full rank tie, the
/// fill that reads the task's input beats the fill that invents a constant:
/// a literal is an assumption the examples never justified, and preferring
/// it is how a search drifts toward memorizing its examples.
pub fn literal_leaves(node: &IrNode) -> usize {
    match node {
        IrNode::Literal { .. } => 1,
        IrNode::Apply { arguments, .. } => arguments.iter().map(literal_leaves).sum(),
        _ => 0,
    }
}

/// The types a comprehension's loop variable can take while iterating a pool
/// value. Iterating a sequence yields its elements, so a value of type
/// `sequence<pair<A, B>>` makes `pair<A, B>` itself a binder element type: a
/// consuming fragment may then take the two components through consecutive
/// slots (`for left, right in combinations(values, 2)`). The value's own type
/// is kept — a declared binder element may already be the whole value.
pub(super) fn iteration_element_types(ty: &IrType) -> Vec<IrType> {
    match ty {
        IrType::Sequence(element) => vec![ty.clone(), element.as_ref().clone()],
        _ => vec![ty.clone()],
    }
}

pub(super) fn parameter_reads(node: &IrNode, names: &[&str]) -> usize {
    let owned: BTreeSet<String> = names.iter().map(|name| (*name).to_owned()).collect();
    let mut into = BTreeSet::new();
    collect_parameter_names(node, &owned, &mut into);
    into.len()
}

pub(super) fn collect_parameter_names(
    node: &IrNode,
    names: &BTreeSet<String>,
    into: &mut BTreeSet<String>,
) {
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
pub(super) fn python_tokens(source: &str) -> Vec<String> {
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
