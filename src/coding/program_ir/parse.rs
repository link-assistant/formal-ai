//! Readers for the canonical program-IR Links projection and compact type slugs.

use super::{IrNode, IrType};
use crate::seed::parser::LinoNode;

pub(super) fn parse_type(node: &LinoNode) -> Option<IrType> {
    if node.name != "type" {
        return None;
    }
    match node.id.as_str() {
        "integer" => Some(IrType::Integer),
        "float" => Some(IrType::Float),
        "boolean" => Some(IrType::Boolean),
        "text" => Some(IrType::Text),
        "sequence" => Some(IrType::Sequence(Box::new(parse_nested_type(
            node, "element",
        )?))),
        "pair" => Some(IrType::Pair(
            Box::new(parse_nested_type(node, "left")?),
            Box::new(parse_nested_type(node, "right")?),
        )),
        "mapping" => Some(IrType::Mapping(
            Box::new(parse_nested_type(node, "left")?),
            Box::new(parse_nested_type(node, "right")?),
        )),
        value => value
            .strip_prefix("unknown:")
            .and_then(|id| id.parse().ok())
            .map(IrType::Unknown),
    }
}

fn parse_nested_type(node: &LinoNode, name: &str) -> Option<IrType> {
    parse_type(
        node.children
            .iter()
            .find(|child| child.name == name)?
            .children
            .first()?,
    )
}

pub(super) fn parse_node(node: &LinoNode) -> Option<IrNode> {
    if node.name != "node" {
        return None;
    }
    let child_node = |name: &str| {
        parse_node(
            node.children
                .iter()
                .find(|child| child.name == name)?
                .children
                .first()?,
        )
    };
    match node.id.as_str() {
        "parameter" => Some(IrNode::Parameter {
            name: node.find_child_value("name").to_owned(),
            ty: parse_type(node.children.iter().find(|child| child.name == "type")?)?,
        }),
        "literal" => Some(IrNode::Literal {
            text: node.find_child_value("text").to_owned(),
            ty: parse_type(node.children.iter().find(|child| child.name == "type")?)?,
        }),
        "apply" => Some(IrNode::Apply {
            fragment: node.find_child_value("fragment").to_owned(),
            arguments: node
                .children
                .iter()
                .filter(|child| child.name == "argument")
                .map(|argument| parse_node(argument.children.first()?))
                .collect::<Option<Vec<_>>>()?,
        }),
        "each" => Some(IrNode::Each {
            item: node.find_child_value("item").to_owned(),
            items: Box::new(child_node("items")?),
            body: Box::new(child_node("body")?),
            predicate: if node.children.iter().any(|child| child.name == "predicate") {
                Some(Box::new(child_node("predicate")?))
            } else {
                None
            },
        }),
        "fold" => Some(IrNode::Fold {
            item: node.find_child_value("item").to_owned(),
            accumulator: node.find_child_value("accumulator").to_owned(),
            items: Box::new(child_node("items")?),
            initial: Box::new(child_node("initial")?),
            body: Box::new(child_node("body")?),
        }),
        "repeat" => Some(IrNode::Repeat {
            counter: node.find_child_value("counter").to_owned(),
            from: Box::new(child_node("from")?),
            to: Box::new(child_node("to")?),
            body: Box::new(child_node("body")?),
        }),
        "recurrence" => Some(IrNode::Recurrence {
            state: node
                .children
                .iter()
                .filter(|child| child.name == "state")
                .map(|child| child.id.clone())
                .collect(),
            base: node
                .children
                .iter()
                .filter(|child| child.name == "base")
                .map(|base| parse_node(base.children.first()?))
                .collect::<Option<Vec<_>>>()?,
            transition: Box::new(child_node("transition")?),
            index: Box::new(child_node("index")?),
        }),
        "recursive_reduce" => Some(IrNode::RecursiveReduce {
            state: node
                .children
                .iter()
                .filter(|child| child.name == "state")
                .map(|child| child.id.clone())
                .collect(),
            target: node
                .children
                .iter()
                .filter(|child| child.name == "target")
                .map(|target| parse_node(target.children.first()?))
                .collect::<Option<Vec<_>>>()?,
            item: node
                .children
                .iter()
                .filter(|child| child.name == "item")
                .map(|child| child.id.clone())
                .collect(),
            items: Box::new(child_node("items")?),
            next: node
                .children
                .iter()
                .filter(|child| child.name == "next")
                .map(|next| parse_node(next.children.first()?))
                .collect::<Option<Vec<_>>>()?,
            admissible: if node.children.iter().any(|child| child.name == "admissible") {
                Some(Box::new(child_node("admissible")?))
            } else {
                None
            },
            base_test: Box::new(child_node("base_test")?),
            base: Box::new(child_node("base")?),
            local: Box::new(child_node("local")?),
            reducer: node.find_child_value("reducer").to_owned(),
            combine: node.find_child_value("combine").to_owned(),
        }),
        "condition" => Some(IrNode::Condition {
            test: Box::new(child_node("test")?),
            then_branch: Box::new(child_node("then")?),
            else_branch: Box::new(child_node("else")?),
        }),
        "bind" => Some(IrNode::Bind {
            name: node.find_child_value("name").to_owned(),
            value: Box::new(child_node("value")?),
            body: Box::new(child_node("body")?),
        }),
        "emit" => Some(IrNode::Emit {
            value: Box::new(child_node("value")?),
        }),
        "return" => Some(IrNode::Return {
            value: Box::new(child_node("value")?),
        }),
        _ => None,
    }
}

pub fn parse_type_slug(value: &str) -> Option<IrType> {
    let value = value.trim();
    match value {
        "integer" => Some(IrType::Integer),
        "float" => Some(IrType::Float),
        "boolean" => Some(IrType::Boolean),
        "text" => Some(IrType::Text),
        "callable" => Some(IrType::Callable),
        _ => {
            if let Some(id) = value.strip_prefix("unknown:") {
                return id.parse().ok().map(IrType::Unknown);
            }
            if let Some(inner) = value
                .strip_prefix("sequence<")
                .and_then(|inner| inner.strip_suffix('>'))
            {
                return Some(IrType::Sequence(Box::new(parse_type_slug(inner)?)));
            }
            let (kind, inner) = if let Some(inner) = value
                .strip_prefix("pair<")
                .and_then(|inner| inner.strip_suffix('>'))
            {
                ("pair", inner)
            } else {
                (
                    "mapping",
                    value
                        .strip_prefix("mapping<")
                        .and_then(|inner| inner.strip_suffix('>'))?,
                )
            };
            let (left, right) = split_type_pair(inner)?;
            let left = Box::new(parse_type_slug(left)?);
            let right = Box::new(parse_type_slug(right)?);
            Some(if kind == "pair" {
                IrType::Pair(left, right)
            } else {
                IrType::Mapping(left, right)
            })
        }
    }
}

fn split_type_pair(value: &str) -> Option<(&str, &str)> {
    let mut depth = 0_usize;
    for (index, character) in value.char_indices() {
        match character {
            '<' => depth += 1,
            '>' => depth = depth.saturating_sub(1),
            ',' if depth == 0 => return Some((&value[..index], &value[index + 1..])),
            _ => {}
        }
    }
    None
}
