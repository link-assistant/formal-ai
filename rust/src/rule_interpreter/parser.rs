//! Parsing of data-owned handler rules into the interpreter's runtime model.

use super::{
    Condition, Fallback, Node, Response, RoleMode, Rule, Shape, Step, Subject, ValueRef,
    ValueSource,
};

impl Node {
    pub(super) fn first_arg(&self) -> Result<String, String> {
        self.args
            .first()
            .cloned()
            .ok_or_else(|| self.error("missing_argument"))
    }

    pub(super) fn error(&self, reason: &str) -> String {
        let line = self.line;
        let name = &self.name;
        format!("handler_rules:{line}:{reason}:{name}")
    }
}

/// Parse an indented Links Notation document into a tree of `name args…` nodes.
///
/// Values may be wrapped in double or single quotes to keep spaces; there are no
/// escape sequences.
pub(super) fn parse_tree(text: &str) -> Vec<Node> {
    let mut roots: Vec<Node> = Vec::new();
    let mut stack: Vec<(usize, Node)> = Vec::new();
    for (index, raw) in text.lines().enumerate() {
        let trimmed = raw.trim_start();
        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }
        let indent = raw.len() - trimmed.len();
        let mut tokens = tokenize(trimmed.trim_end());
        if tokens.is_empty() {
            continue;
        }
        let node = Node {
            line: index + 1,
            name: tokens.remove(0),
            args: tokens,
            children: Vec::new(),
        };
        while stack.last().is_some_and(|(depth, _)| *depth >= indent) {
            let (_, finished) = stack.pop().expect("stack_checked");
            attach(&mut roots, &mut stack, finished);
        }
        stack.push((indent, node));
    }
    while let Some((_, finished)) = stack.pop() {
        attach(&mut roots, &mut stack, finished);
    }
    roots
}

fn attach(roots: &mut Vec<Node>, stack: &mut [(usize, Node)], node: Node) {
    match stack.last_mut() {
        Some((_, parent)) => parent.children.push(node),
        None => roots.push(node),
    }
}

fn tokenize(line: &str) -> Vec<String> {
    let mut tokens = Vec::new();
    let mut current = String::new();
    let mut quote: Option<char> = None;
    for ch in line.chars() {
        match quote {
            Some(open) if ch == open => {
                quote = None;
                tokens.push(std::mem::take(&mut current));
            }
            None if ch == '"' || ch == '\'' => quote = Some(ch),
            None if ch.is_whitespace() => {
                if !current.is_empty() {
                    tokens.push(std::mem::take(&mut current));
                }
            }
            _ => current.push(ch),
        }
    }
    if !current.is_empty() {
        tokens.push(current);
    }
    tokens
}

pub(super) fn parse_rule(node: &Node) -> Result<Rule, String> {
    let name = node.first_arg()?;
    let mut when = None;
    let mut values = Vec::new();
    let mut steps = Vec::new();
    let mut response = None;
    let mut intent = None;
    let mut link = None;
    let mut confidence = 1.0;
    for child in &node.children {
        match child.name.as_str() {
            "when" => when = Some(Condition::All(parse_conditions(&child.children)?)),
            "value" => values.push(parse_value(child)?),
            "log" => {
                let kind = child.first_arg()?;
                let value = child.args.get(1).map_or_else(
                    || ValueRef::Literal(String::new()),
                    |raw| parse_value_ref(raw),
                );
                steps.push(Step::Log {
                    kind: Box::leak(kind.into_boxed_str()),
                    value,
                });
            }
            "respond" => {
                let intent_name = child.first_arg()?;
                let mut fallback = Fallback::Localized;
                let mut texts = Vec::new();
                for option in &child.children {
                    match option.name.as_str() {
                        "fallback" => {
                            fallback = match option.first_arg()?.as_str() {
                                "localized" => Fallback::Localized,
                                "exact" => Fallback::Exact,
                                language => Fallback::Language(language.to_owned()),
                            };
                        }
                        "text" => {
                            let language = option.first_arg()?;
                            let text = option
                                .args
                                .get(1)
                                .cloned()
                                .ok_or_else(|| option.error("text_without_body"))?;
                            texts.push((language, text));
                        }
                        _ => return Err(option.error("unknown_respond_option")),
                    }
                }
                response = Some(Response::Seed {
                    intent: intent_name,
                    fallback,
                    texts,
                });
            }
            "respond_unknown" => response = Some(Response::Unknown),
            "intent" => intent = Some(child.first_arg()?),
            "link" => link = Some(child.first_arg()?),
            "confidence" => {
                confidence = child
                    .first_arg()?
                    .parse::<f32>()
                    .map_err(|_| child.error("confidence_not_a_number"))?;
            }
            _ => return Err(child.error("unknown_rule_field")),
        }
    }
    Ok(Rule {
        name,
        when: when.ok_or_else(|| node.error("rule_without_when"))?,
        values,
        steps,
        response: response.ok_or_else(|| node.error("rule_without_respond"))?,
        intent,
        link,
        confidence,
    })
}

fn parse_conditions(nodes: &[Node]) -> Result<Vec<Condition>, String> {
    nodes.iter().map(parse_condition).collect()
}

fn parse_condition(node: &Node) -> Result<Condition, String> {
    let (options, subject) = split_subject(&node.args[node.args.len().min(1)..]);
    let subject = subject.map_or(Ok(Subject::Normalized), |name| parse_subject(node, name))?;
    Ok(match node.name.as_str() {
        "all" => Condition::All(parse_conditions(&node.children)?),
        "any" => Condition::Any(parse_conditions(&node.children)?),
        "none" => Condition::None(parse_conditions(&node.children)?),
        "role" => {
            let mode = match options.first().map(String::as_str) {
                None | Some("spelled") => RoleMode::Spelled,
                Some("raw") => RoleMode::Raw,
                Some("languages") => RoleMode::Languages,
                Some("forms") => RoleMode::Forms,
                Some("separated") => RoleMode::Separated,
                Some("whole") => RoleMode::Whole,
                Some(_) => return Err(node.error("unknown_role_mode")),
            };
            Condition::Role(node.first_arg()?, mode, subject)
        }
        "role_lead" => Condition::RoleLead(node.first_arg()?, subject),
        "role_prefix" => Condition::RolePrefix(node.first_arg()?, subject),
        "role_padded" => Condition::RolePadded(node.first_arg()?, subject),
        "word" => Condition::Word(node.first_arg()?, subject),
        "substring" => Condition::Substring(node.first_arg()?, subject),
        "prefix" => Condition::Prefix(node.first_arg()?, subject),
        "cue_set" => Condition::CueSet(node.first_arg()?, subject),
        "only_characters" => Condition::OnlyCharacters(node.first_arg()?),
        "unbalanced_parentheses" => Condition::UnbalancedParentheses,
        "route_exact" => Condition::RouteExact(node.first_arg()?),
        "history_role" => Condition::HistoryRole(node.first_arg()?),
        "shape" => Condition::Shape(parse_shape(node, &node.first_arg()?)?, subject),
        _ => return Err(node.error("unknown_condition")),
    })
}

fn split_subject(args: &[String]) -> (Vec<String>, Option<&str>) {
    let mut options = Vec::new();
    let mut subject = None;
    let mut iter = args.iter();
    while let Some(arg) = iter.next() {
        if arg == "of" {
            subject = iter.next().map(String::as_str);
        } else {
            options.push(arg.clone());
        }
    }
    (options, subject)
}

fn parse_shape(node: &Node, name: &str) -> Result<Shape, String> {
    Ok(match name {
        "digit" => Shape::Digit,
        "time_separator" => Shape::TimeSeparator,
        "url" => Shape::Url,
        "path" => Shape::Path,
        "quoted" => Shape::Quoted,
        _ => return Err(node.error("unknown_shape")),
    })
}

fn parse_subject(node: &Node, name: &str) -> Result<Subject, String> {
    Ok(match name {
        "normalized" => Subject::Normalized,
        "cleaned" => Subject::Cleaned,
        "lowercase" => Subject::Lowercase,
        "prompt" => Subject::Prompt,
        "trimmed" => Subject::Trimmed,
        "padded" => Subject::Padded,
        _ => return Err(node.error("unknown_subject")),
    })
}

fn parse_value(node: &Node) -> Result<(String, ValueSource), String> {
    let name = node.first_arg()?;
    let kind = node
        .args
        .get(1)
        .ok_or_else(|| node.error("value_without_source"))?;
    let source = match kind.as_str() {
        "backticks" => ValueSource::Backticks,
        "trimmed_prompt" => ValueSource::TrimmedPrompt,
        "literal" => ValueSource::Literal(
            node.args
                .get(2)
                .cloned()
                .ok_or_else(|| node.error("literal_without_text"))?,
        ),
        "agent_info" => {
            let key = node
                .args
                .get(2)
                .cloned()
                .ok_or_else(|| node.error("agent_info_without_key"))?;
            let default = match node.args.get(3).map(String::as_str) {
                Some("default") => node.args[4..].join(" "),
                _ => String::new(),
            };
            ValueSource::AgentInfo { key, default }
        }
        _ => return Err(node.error("unknown_value_source")),
    };
    Ok((name, source))
}

fn parse_value_ref(raw: &str) -> ValueRef {
    match raw {
        "$prompt" => ValueRef::Prompt,
        "$trimmed" => ValueRef::Trimmed,
        _ => raw.strip_prefix('$').map_or_else(
            || ValueRef::Literal(raw.to_owned()),
            |name| ValueRef::Capture(name.to_owned()),
        ),
    }
}
