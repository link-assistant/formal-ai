//! Finite source-token IR. Parsed syntax never certifies arbitrary object or module effects.
use super::source_contract::text;
use serde_json::{Value, json};
type Checked = Result<Value, &'static str>;
fn group(tree: &Value, delim: &str) -> bool {
    tree["$"] == "group" && tree["delim"] == delim
}
fn children(tree: &Value) -> &[Value] {
    tree["trees"].as_array().map_or(&[], Vec::as_slice)
}
fn binding(name: &str) -> bool {
    let mut chars = name.chars();
    let Some(first) = chars.next() else {
        return false;
    };
    (first.is_ascii_alphabetic() || matches!(first, '_' | '$'))
        && chars.all(|ch| ch.is_ascii_alphanumeric() || matches!(ch, '_' | '$'))
        && !crate::es_tokenizer::is_keyword(name)
        && !matches!(
            name,
            "eval"
                | "arguments"
                | "yield"
                | "implements"
                | "interface"
                | "package"
                | "private"
                | "protected"
                | "public"
        )
}
fn literal(value: Value) -> Value {
    json!({"op":"literal","value":value})
}
fn expression(trees: &[Value]) -> Checked {
    if trees.is_empty() {
        return Err("MissingExpression");
    }
    if let Some(question) = trees.iter().position(|tree| text(tree) == "?") {
        let mut depth = 0;
        let mut colon = None;
        for (at, tree) in trees.iter().enumerate().skip(question + 1) {
            if text(tree) == "?" {
                depth += 1;
            }
            if text(tree) == ":" {
                if depth == 0 {
                    colon = Some(at);
                    break;
                }
                depth -= 1;
            }
        }
        let colon = colon.ok_or("Conditional")?;
        return Ok(
            json!({"op":"conditional","condition":expression(&trees[..question])?,"yes":expression(&trees[question + 1..colon])?,"no":expression(&trees[colon + 1..])?}),
        );
    }
    if trees
        .iter()
        .filter(|tree| matches!(text(tree), "===" | "!=="))
        .count()
        > 1
        || trees.iter().filter(|tree| text(tree) == ">").count() > 1
    {
        return Err("UnsupportedComparisonChain");
    }
    for operator in ["??", "===", "!==", ">"] {
        if let Some(at) = trees.iter().position(|tree| text(tree) == operator) {
            return Ok(
                json!({"op":operator,"left":expression(&trees[..at])?,"right":expression(&trees[at + 1..])?}),
            );
        }
    }
    let first = &trees[0];
    let mut cursor = 1;
    let mut node = if first["$"] == "template" {
        let mut parts = Vec::new();
        for part in first["parts"].as_array().ok_or("UnsupportedExpression")? {
            if part["$"] == "chunk" {
                let chunk = part["text"].as_str().ok_or("UnsupportedExpression")?;
                if chunk.contains('\\') {
                    return Err("EscapedTemplate");
                }
                parts.push(literal(json!(chunk)));
            } else {
                parts.push(expression(children(part))?);
            }
        }
        json!({"op":"template","parts":parts})
    } else if group(first, "paren") {
        expression(children(first))?
    } else if first["kind"] == "numeric"
        && !text(first).is_empty()
        && text(first).bytes().all(|byte| byte.is_ascii_digit())
        && (text(first) == "0" || !text(first).starts_with('0'))
    {
        let value = text(first)
            .parse::<u64>()
            .map_err(|_| "UnsafeNumericLiteral")?;
        if value > 9_007_199_254_740_991 {
            return Err("UnsafeNumericLiteral");
        }
        literal(json!(value))
    } else if first["kind"] == "string" && !text(first).contains(['\\', '\r', '\n']) {
        let value = text(first);
        literal(json!(&value[1..value.len() - 1]))
    } else if text(first) == "null" {
        literal(Value::Null)
    } else if matches!(text(first), "true" | "false") {
        literal(json!(text(first) == "true"))
    } else if first["kind"] == "identifier" {
        json!({"op":"binding","name":text(first)})
    } else {
        return Err("UnsupportedExpression");
    };
    while cursor < trees.len() {
        if text(&trees[cursor]) == "."
            && trees
                .get(cursor + 1)
                .is_some_and(|tree| tree["kind"] == "identifier")
        {
            let name = text(&trees[cursor + 1]);
            cursor += 2;
            if trees.get(cursor).is_some_and(|tree| group(tree, "paren")) {
                let argument_trees = children(&trees[cursor]);
                cursor += 1;
                if name == "map" {
                    if argument_trees.len() < 3
                        || !group(&argument_trees[0], "paren")
                        || children(&argument_trees[0]).len() != 1
                        || text(&argument_trees[1]) != "=>"
                        || !binding(text(&children(&argument_trees[0])[0]))
                    {
                        return Err("UnsupportedMapper");
                    }
                    node = json!({"op":"map","receiver":node,"parameter":text(&children(&argument_trees[0])[0]),"body":expression(&argument_trees[2..])?});
                } else if matches!(name, "join" | "push") {
                    node =
                        json!({"op":name,"receiver":node,"argument":expression(argument_trees)?});
                } else {
                    return Err("UnknownMethod");
                }
            } else {
                node = json!({"op":"member","receiver":node,"name":name});
            }
        } else if group(&trees[cursor], "bracket") {
            node =
                json!({"op":"index","receiver":node,"index":expression(children(&trees[cursor]))?});
            cursor += 1;
        } else {
            return Err("UnsupportedTail");
        }
    }
    Ok(node)
}
fn statement_terms<'a>(trees: &'a [Value], cursor: &mut usize) -> &'a [Value] {
    let start = *cursor;
    while *cursor < trees.len() && text(&trees[*cursor]) != ";" {
        *cursor += 1;
    }
    let end = *cursor;
    if *cursor < trees.len() {
        *cursor += 1;
    }
    &trees[start..end]
}
fn statements(trees: &[Value], source: &str) -> Checked {
    let mut result = Vec::new();
    let mut cursor = 0;
    while cursor < trees.len() {
        match text(&trees[cursor]) {
            "const" => {
                cursor += 1;
                let name = trees.get(cursor).map_or("", text);
                cursor += 1;
                if !binding(name) {
                    return Err("UnsupportedLocalBinding");
                }
                if trees.get(cursor).map_or("", text) != "=" {
                    return Err("LocalDeclaration");
                }
                cursor += 1;
                result.push(json!({"op":"const","name":name,"value":expression(statement_terms(trees, &mut cursor))?}));
            }
            "if" => {
                cursor += 1;
                let condition = trees.get(cursor).ok_or("IfCondition")?;
                cursor += 1;
                if !group(condition, "paren") {
                    return Err("IfCondition");
                }
                result.push(json!({"op":"if","condition":expression(children(condition))?,"body":{"op":"effect","value":expression(statement_terms(trees, &mut cursor))?}}));
            }
            "return" => {
                let token = &trees[cursor];
                cursor += 1;
                if let Some(next) = trees.get(cursor) {
                    let start =
                        usize::try_from(token["span"]["end"].as_u64().ok_or("InvalidSpan")?)
                            .map_err(|_| "InvalidSpan")?;
                    let end = usize::try_from(next["span"]["start"].as_u64().ok_or("InvalidSpan")?)
                        .map_err(|_| "InvalidSpan")?;
                    let between = source.get(start..end).ok_or("InvalidSpan")?;
                    if between.contains(['\r', '\n', '\u{2028}', '\u{2029}']) {
                        return Err("ReturnLineTerminator");
                    }
                }
                result.push(
                    json!({"op":"return","value":expression(statement_terms(trees, &mut cursor))?}),
                );
                if cursor < trees.len() {
                    return Err("AfterReturn");
                }
            }
            _ => return Err("UnsupportedStatement"),
        }
    }
    if result.last().is_none_or(|node| node["op"] != "return") {
        return Err("MissingReturn");
    }
    Ok(json!(result))
}
/// Mirrors parseConditionalBody; conditions remain unproved domain obligations.
#[must_use]
pub(super) fn parse_conditional_body(parameters: &[String], body: &[Value], source: &str) -> Value {
    let result = if parameters.len() == 1 && binding(&parameters[0]) {
        statements(body, source)
    } else {
        Err("UnsupportedArity")
    };
    match result {
        Ok(ir) => {
            json!({"status":"parsed","parameter":parameters[0],"ir":ir,"sourceEffects":"unknown",
   "preconditions":["SourceOwnedPlainDataSchema","FiniteScalarCoercions","IntrinsicOwnership","FreshLocalMutationOnly"]})
        }
        Err(reason) => json!({"status":"unknown","reason":reason,"sourceEffects":"unknown"}),
    }
}
