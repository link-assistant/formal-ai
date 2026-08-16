use std::fmt::Write as _;

use super::{SubstitutionPatternIr, SubstitutionPatternNodeIr, SubstitutionProgramIr};

pub(super) fn rules_source(ir: &SubstitutionProgramIr) -> String {
    let mut output = String::from("static RULES: &[Rule] = &[\n");
    for rule in ir.rules.iter().filter(|rule| rule.manual) {
        output.push_str("    Rule { conditions: &[\n");
        for condition in &rule.conditions {
            let _ = writeln!(output, "        {},", pattern_source(condition));
        }
        output.push_str("    ], actions: &[\n");
        for action in &rule.actions {
            let _ = writeln!(
                output,
                "        Action {{ remove: {}, add: &[",
                pattern_source(&action.remove)
            );
            for add in &action.add {
                let _ = writeln!(output, "            {},", pattern_source(add));
            }
            output.push_str("        ] },\n");
        }
        output.push_str("    ] },\n");
    }
    output.push_str("];\n");
    output
}

fn pattern_source(pattern: &SubstitutionPatternIr) -> String {
    format!(
        "Pattern {{ from: {}, to: {} }}",
        node_source(&pattern.from),
        node_source(&pattern.to)
    )
}

fn node_source(node: &SubstitutionPatternNodeIr) -> String {
    match node {
        SubstitutionPatternNodeIr::Literal { value } => {
            format!("Node::Literal({value:?})")
        }
        SubstitutionPatternNodeIr::Variable { name } => {
            format!("Node::Variable({name:?})")
        }
        SubstitutionPatternNodeIr::PrefixVariable { prefix, name } => {
            format!("Node::PrefixVariable({prefix:?}, {name:?})")
        }
    }
}

pub(super) const COMMON_RUNTIME: &str = r#"
#[allow(dead_code)]
#[derive(Clone, Copy)]
enum Node {
    Literal(&'static str),
    Variable(&'static str),
    PrefixVariable(&'static str, &'static str),
}

#[derive(Clone, Copy)]
struct Pattern {
    from: Node,
    to: Node,
}

struct Action {
    remove: Pattern,
    add: &'static [Pattern],
}

struct Rule {
    conditions: &'static [Pattern],
    actions: &'static [Action],
}

type Link = (String, String);
type Bindings = BTreeMap<String, String>;

fn bind(bindings: &mut Bindings, name: &str, value: &str) -> bool {
    if let Some(bound) = bindings.get(name) {
        bound == value
    } else {
        bindings.insert(name.to_owned(), value.to_owned());
        true
    }
}

fn match_node(pattern: Node, value: &str, bindings: &mut Bindings) -> bool {
    match pattern {
        Node::Literal(literal) => literal == value,
        Node::Variable(name) => bind(bindings, name, value),
        Node::PrefixVariable(prefix, name) => value
            .strip_prefix(prefix)
            .is_some_and(|suffix| bind(bindings, name, suffix)),
    }
}

fn match_pattern(pattern: Pattern, link: &Link, bindings: &mut Bindings) -> bool {
    match_node(pattern.from, &link.0, bindings)
        && match_node(pattern.to, &link.1, bindings)
}

fn find_bindings(
    patterns: &[Pattern],
    links: &BTreeSet<Link>,
    bindings: Bindings,
) -> Option<Bindings> {
    let Some((pattern, remaining)) = patterns.split_first() else {
        return Some(bindings);
    };
    for link in links {
        let mut candidate = bindings.clone();
        if match_pattern(*pattern, link, &mut candidate) {
            if let Some(found) = find_bindings(remaining, links, candidate) {
                return Some(found);
            }
        }
    }
    None
}

fn instantiate_node(node: Node, bindings: &Bindings) -> Option<String> {
    match node {
        Node::Literal(value) => Some(value.to_owned()),
        Node::Variable(name) => bindings.get(name).cloned(),
        Node::PrefixVariable(prefix, name) => bindings.get(name).map(|value| format!("{prefix}{value}")),
    }
}

fn instantiate(pattern: Pattern, bindings: &Bindings) -> Option<Link> {
    Some((
        instantiate_node(pattern.from, bindings)?,
        instantiate_node(pattern.to, bindings)?,
    ))
}

fn apply_rule(rule: &Rule, links: &mut BTreeSet<Link>) -> bool {
    let mut required = rule.conditions.to_vec();
    required.extend(rule.actions.iter().map(|action| action.remove));
    let Some(bindings) = find_bindings(&required, links, Bindings::new()) else {
        return false;
    };
    let before = links.clone();
    for action in rule.actions {
        let Some(remove) = instantiate(action.remove, &bindings) else {
            return false;
        };
        links.remove(&remove);
        for add in action.add {
            let Some(link) = instantiate(*add, &bindings) else {
                return false;
            };
            links.insert(link);
        }
    }
    *links != before
}

fn transform(input: &str) -> String {
    let mut links: BTreeSet<Link> = input
        .lines()
        .filter_map(|line| line.split_once('\t'))
        .map(|(from, to)| (from.to_owned(), to.to_owned()))
        .collect();
    for _ in 0..MAX_APPLICATIONS {
        let Some(rule) = RULES.iter().find(|rule| apply_rule(rule, &mut links)) else {
            break;
        };
        let _ = rule;
    }
    let mut output = String::new();
    for (from, to) in links {
        let _ = writeln!(output, "{from}\t{to}");
    }
    output
}
"#;
